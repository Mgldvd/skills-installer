import { beforeEach, describe, expect, it, vi } from "vitest";

import { useAppState } from "../src/composables/useAppState";
import { useInstallation } from "../src/composables/useInstallation";
import { resetState } from "./test-utils";
import type { InstallProgressEvent, InstallRequest, InstallResult } from "../src/types";

vi.mock("../src/services/backend", () => ({
  installSkills: vi.fn(),
  cancelInstallation: vi.fn(),
  getDependencyStatus: vi.fn(),
}));

import * as backend from "../src/services/backend";

function emptyResult(): InstallResult {
  return { requested: 0, installed: 0, alreadyInstalled: 0, failed: 0, cancelled: false, perSkill: [] };
}

describe("useInstallation", () => {
  const state = useAppState();

  beforeEach(() => {
    resetState(state);
    vi.clearAllMocks();
  });

  // The whole point of moving the callback into `install()`: a batch of
  // several Skills must let the caller react to each one finishing, not
  // only find out once the entire batch is done — otherwise the GUI grid
  // has no way to move a Skill into "Installed" until every other Skill in
  // the selection has also finished installing.
  it("calls onSkillSuccess once per skill-success event, in order, before the batch resolves", async () => {
    const events: InstallProgressEvent[] = [
      { event: "start", data: { total: 2 } },
      { event: "skill-success", data: { skillId: "a", displayName: "A" } },
      { event: "skill-success", data: { skillId: "b", displayName: "B" } },
      { event: "complete", data: { result: emptyResult() } },
    ];
    vi.mocked(backend.installSkills).mockImplementation(async (_request, onEvent) => {
      for (const event of events) onEvent(event);
      return emptyResult();
    });

    const seen: string[] = [];
    const { install } = useInstallation();
    const request: InstallRequest = { selection: { skillIds: ["a", "b"] }, options: {} as InstallRequest["options"] };
    await install(request, (skillId) => seen.push(skillId));

    expect(seen).toEqual(["a", "b"]);
  });

  it("does not require an onSkillSuccess callback", async () => {
    vi.mocked(backend.installSkills).mockImplementation(async (_request, onEvent) => {
      onEvent({ event: "skill-success", data: { skillId: "a", displayName: "A" } });
      return emptyResult();
    });

    const { install } = useInstallation();
    const request: InstallRequest = { selection: { skillIds: ["a"] }, options: {} as InstallRequest["options"] };
    await expect(install(request)).resolves.toEqual(emptyResult());
    expect(state.installation.perSkillStatus.a).toBe("installed");
  });

  it("a skill-error event does not trigger onSkillSuccess", async () => {
    vi.mocked(backend.installSkills).mockImplementation(async (_request, onEvent) => {
      onEvent({ event: "skill-error", data: { skillId: "a", displayName: "A", message: "boom" } });
      return emptyResult();
    });

    const seen: string[] = [];
    const { install } = useInstallation();
    const request: InstallRequest = { selection: { skillIds: ["a"] }, options: {} as InstallRequest["options"] };
    await install(request, (skillId) => seen.push(skillId));

    expect(seen).toEqual([]);
    expect(state.installation.perSkillStatus.a).toBe("failed");
  });
});
