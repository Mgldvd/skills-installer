import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAppState } from "../src/composables/useAppState";
import { useProjects } from "../src/composables/useProjects";
import { resetState } from "./test-utils";

vi.mock("../src/services/backend", () => ({
  getProjects: vi.fn(),
  saveProject: vi.fn(),
  deleteProject: vi.fn(),
}));
import * as backend from "../src/services/backend";

describe("useProjects", () => {
  const state = useAppState();
  beforeEach(() => {
    resetState(state);
    vi.clearAllMocks();
  });

  it("loads Projects from the backend into state", async () => {
    const zephyr = { id: "zephyr", name: "Zephyr", gitUrl: null, skillNames: ["triage"] };
    vi.mocked(backend.getProjects).mockResolvedValue([zephyr]);

    await useProjects().loadAll();

    expect(state.projects).toEqual([zephyr]);
  });

  it("saves a Project through the backend and keeps the list sorted by name", async () => {
    state.projects = [{ id: "zephyr", name: "Zephyr", gitUrl: null, skillNames: ["triage"] }];
    const atlas = { id: "atlas", name: "Atlas", gitUrl: "https://github.com/me/atlas", skillNames: ["docs"] };
    vi.mocked(backend.saveProject).mockResolvedValue(atlas);

    const result = await useProjects().save("Atlas", "https://github.com/me/atlas", ["docs"]);

    expect(backend.saveProject).toHaveBeenCalledWith({
      name: "Atlas",
      gitUrl: "https://github.com/me/atlas",
      skillNames: ["docs"],
    });
    expect(result).toEqual(atlas);
    expect(state.projects.map((p) => p.name)).toEqual(["Atlas", "Zephyr"]);
  });

  it("removes a Project through the backend and drops it from state", async () => {
    state.projects = [
      { id: "atlas", name: "Atlas", gitUrl: null, skillNames: ["docs"] },
      { id: "zephyr", name: "Zephyr", gitUrl: null, skillNames: ["triage"] },
    ];
    vi.mocked(backend.deleteProject).mockResolvedValue(undefined);

    await useProjects().remove("atlas");

    expect(backend.deleteProject).toHaveBeenCalledWith("atlas");
    expect(state.projects.map((p) => p.id)).toEqual(["zephyr"]);
  });

  it("does not mutate local state when saving fails", async () => {
    state.projects = [];
    vi.mocked(backend.saveProject).mockRejectedValue(new Error("disk full"));

    await expect(useProjects().save("Atlas", null, ["docs"])).rejects.toThrow("disk full");

    expect(state.projects).toEqual([]);
  });
});
