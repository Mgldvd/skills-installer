import { beforeEach, describe, expect, it, vi } from "vitest";

import { useAppState } from "../src/composables/useAppState";
import { useSkills } from "../src/composables/useSkills";
import { makeGroup, makeSkill } from "./fixtures";
import { resetState } from "./test-utils";

vi.mock("../src/services/backend", () => ({
  getApplicationState: vi.fn(),
  refresh: vi.fn(),
  addSkill: vi.fn(),
  updateSkill: vi.fn(),
  deleteSkill: vi.fn(),
  createGroup: vi.fn(),
  updateGroup: vi.fn(),
  deleteGroup: vi.fn(),
  checkLocalSkillUpdates: vi.fn(),
}));

import * as backend from "../src/services/backend";

describe("useSkills", () => {
  const state = useAppState();

  beforeEach(() => {
    resetState(state);
    vi.clearAllMocks();
  });

  it("loadAll selects exactly the configured preselected skills by default", async () => {
    vi.mocked(backend.getApplicationState).mockResolvedValue({
      version: 1,
      defaults: { agent: null, copy: true, scope: "project" },
      groups: [makeGroup()],
      skills: [
        makeSkill({ id: "a", preselected: true }),
        makeSkill({ id: "b", preselected: false }),
        makeSkill({ id: "c", preselected: true, enabled: false }),
      ],
      sourcePath: null,
      isEmbeddedDefault: true,
    });

    const { loadAll } = useSkills();
    await loadAll();

    expect(state.selectedSkillIds).toEqual(new Set(["a"]));
  });

  it("loadAll excludes already-installed skills from the default selection even if preselected", async () => {
    vi.mocked(backend.getApplicationState).mockResolvedValue({
      version: 1,
      defaults: { agent: null, copy: true, scope: "project" },
      groups: [makeGroup()],
      skills: [
        makeSkill({ id: "a", preselected: true, installed: false }),
        makeSkill({ id: "b", preselected: true, installed: true, installedAgents: ["universal"] }),
      ],
      sourcePath: null,
      isEmbeddedDefault: true,
    });

    const { loadAll } = useSkills();
    await loadAll();

    expect(state.selectedSkillIds).toEqual(new Set(["a"]));
  });

  it("refresh drops any skill that is now installed out of the current selection", async () => {
    state.skills = [makeSkill({ id: "a", installed: false }), makeSkill({ id: "b", installed: false })];
    state.selectedSkillIds = new Set(["a", "b"]);
    vi.mocked(backend.refresh).mockResolvedValue({
      version: 1,
      defaults: { agent: null, copy: true, scope: "project" },
      groups: [makeGroup()],
      skills: [
        makeSkill({ id: "a", installed: true, installedAgents: ["universal"] }),
        makeSkill({ id: "b", installed: false }),
      ],
      sourcePath: null,
      isEmbeddedDefault: true,
    });

    const { refresh } = useSkills();
    await refresh();

    expect(state.selectedSkillIds).toEqual(new Set(["b"]));
  });

  it("toggleSelected refuses to newly select an already-installed skill, but still allows deselecting one", () => {
    state.skills = [makeSkill({ id: "a", installed: true, installedAgents: ["universal"] })];

    const { toggleSelected } = useSkills();
    toggleSelected("a");
    expect(state.selectedSkillIds.has("a")).toBe(false);

    state.selectedSkillIds = new Set(["a"]);
    toggleSelected("a");
    expect(state.selectedSkillIds.has("a")).toBe(false);
  });

  it("toggleSelected still allows selecting a skill installed for only some of the targeted agents", () => {
    // Default target agents is ["universal"] (see defaultPreferences) — this
    // skill is only installed for claude-code, so there's still a gap.
    state.skills = [makeSkill({ id: "a", installed: true, installedAgents: ["claude-code"] })];

    const { toggleSelected } = useSkills();
    toggleSelected("a");
    expect(state.selectedSkillIds.has("a")).toBe(true);
  });

  it("restoreDefaultSelection resets selection to exactly the current preselected set", () => {
    state.skills = [makeSkill({ id: "a", preselected: true }), makeSkill({ id: "b", preselected: false })];
    state.selectedSkillIds = new Set(["a", "b"]);

    const { restoreDefaultSelection } = useSkills();
    restoreDefaultSelection();

    expect(state.selectedSkillIds).toEqual(new Set(["a"]));
  });

  it("restoreDefaultSelection still includes a preselected skill that's only partially installed", () => {
    state.skills = [
      makeSkill({ id: "a", preselected: true, installed: true, installedAgents: ["claude-code"] }),
      makeSkill({ id: "b", preselected: true, installed: true, installedAgents: ["universal"] }),
    ];

    const { restoreDefaultSelection } = useSkills();
    restoreDefaultSelection();

    // Default target agents is ["universal"]: "a" is missing it, "b" already has it.
    expect(state.selectedSkillIds).toEqual(new Set(["a"]));
  });

  it("clearSelection empties the selection without touching configuration", () => {
    state.skills = [makeSkill({ id: "a" })];
    state.selectedSkillIds = new Set(["a"]);

    const { clearSelection } = useSkills();
    clearSelection();

    expect(state.selectedSkillIds.size).toBe(0);
    expect(state.skills).toHaveLength(1);
  });

  it("toggleSelected adds and removes a skill id", () => {
    const { toggleSelected } = useSkills();
    toggleSelected("a");
    expect(state.selectedSkillIds.has("a")).toBe(true);
    toggleSelected("a");
    expect(state.selectedSkillIds.has("a")).toBe(false);
  });

  it("addSkill persists through the backend and appends to local state", async () => {
    const created = makeSkill({ id: "new-skill" });
    vi.mocked(backend.addSkill).mockResolvedValue(created);

    const { addSkill } = useSkills();
    await addSkill({ url: created.skillsUrl, groupId: "testing" });

    expect(state.skills.map((s) => s.id)).toContain("new-skill");
  });

  it("deleteSkill removes it from local state and from selection", async () => {
    state.skills = [makeSkill({ id: "a" })];
    state.selectedSkillIds = new Set(["a"]);
    vi.mocked(backend.deleteSkill).mockResolvedValue(undefined);

    const { deleteSkill } = useSkills();
    await deleteSkill("a");

    expect(state.skills).toHaveLength(0);
    expect(state.selectedSkillIds.has("a")).toBe(false);
  });

  it("updateSkill replaces the skill in place, preserving array position", async () => {
    state.skills = [makeSkill({ id: "a", displayName: "Old Name" }), makeSkill({ id: "b" })];
    const updated = makeSkill({ id: "a", displayName: "New Name" });
    vi.mocked(backend.updateSkill).mockResolvedValue(updated);

    const { updateSkill } = useSkills();
    await updateSkill({ skillId: "a", displayName: "New Name" });

    expect(state.skills[0].displayName).toBe("New Name");
    expect(state.skills[1].id).toBe("b");
  });

  it("checkForUpdates populates skillsWithUpdates from the backend and returns the ids", async () => {
    vi.mocked(backend.checkLocalSkillUpdates).mockResolvedValue(["taskfile", "bash-scripting"]);

    const { checkForUpdates } = useSkills();
    const result = await checkForUpdates();

    expect(result).toEqual(["taskfile", "bash-scripting"]);
    expect(state.skillsWithUpdates).toEqual(new Set(["taskfile", "bash-scripting"]));
  });

  it("checkForUpdates replaces, rather than merges into, any previous result", async () => {
    state.skillsWithUpdates = new Set(["stale-id"]);
    vi.mocked(backend.checkLocalSkillUpdates).mockResolvedValue([]);

    const { checkForUpdates } = useSkills();
    await checkForUpdates();

    expect(state.skillsWithUpdates.size).toBe(0);
  });
});
