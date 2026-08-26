import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAppState } from "../src/composables/useAppState";
import { usePresets } from "../src/composables/usePresets";
import { resetState } from "./test-utils";

vi.mock("../src/services/backend", () => ({
  getPresets: vi.fn(),
  savePreset: vi.fn(),
  deletePreset: vi.fn(),
}));
import * as backend from "../src/services/backend";

describe("usePresets", () => {
  const state = useAppState();
  beforeEach(() => {
    resetState(state);
    vi.clearAllMocks();
  });

  it("loads Presets from the backend into state", async () => {
    const zephyr = { id: "zephyr", name: "Zephyr", gitUrl: null, skillNames: ["triage"] };
    vi.mocked(backend.getPresets).mockResolvedValue([zephyr]);

    await usePresets().loadAll();

    expect(state.presets).toEqual([zephyr]);
  });

  it("saves a Preset through the backend and keeps the list sorted by name", async () => {
    state.presets = [{ id: "zephyr", name: "Zephyr", gitUrl: null, skillNames: ["triage"] }];
    const atlas = { id: "atlas", name: "Atlas", gitUrl: "https://github.com/me/atlas", skillNames: ["docs"] };
    vi.mocked(backend.savePreset).mockResolvedValue(atlas);

    const result = await usePresets().save("Atlas", "https://github.com/me/atlas", ["docs"]);

    expect(backend.savePreset).toHaveBeenCalledWith({
      name: "Atlas",
      gitUrl: "https://github.com/me/atlas",
      skillNames: ["docs"],
    });
    expect(result).toEqual(atlas);
    expect(state.presets.map((p) => p.name)).toEqual(["Atlas", "Zephyr"]);
  });

  it("removes a Preset through the backend and drops it from state", async () => {
    state.presets = [
      { id: "atlas", name: "Atlas", gitUrl: null, skillNames: ["docs"] },
      { id: "zephyr", name: "Zephyr", gitUrl: null, skillNames: ["triage"] },
    ];
    vi.mocked(backend.deletePreset).mockResolvedValue(undefined);

    await usePresets().remove("atlas");

    expect(backend.deletePreset).toHaveBeenCalledWith("atlas");
    expect(state.presets.map((p) => p.id)).toEqual(["zephyr"]);
  });

  it("does not mutate local state when saving fails", async () => {
    state.presets = [];
    vi.mocked(backend.savePreset).mockRejectedValue(new Error("disk full"));

    await expect(usePresets().save("Atlas", null, ["docs"])).rejects.toThrow("disk full");

    expect(state.presets).toEqual([]);
  });
});
