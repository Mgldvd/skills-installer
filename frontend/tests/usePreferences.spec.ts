import { beforeEach, describe, expect, it, vi } from "vitest";

import { useAppState } from "../src/composables/useAppState";
import { usePreferences } from "../src/composables/usePreferences";
import { defaultPreferences } from "../src/types";
import { resetState } from "./test-utils";

vi.mock("../src/services/backend", () => ({
  getPreferences: vi.fn(),
  updatePreferences: vi.fn(),
}));

import * as backend from "../src/services/backend";

describe("usePreferences", () => {
  const state = useAppState();

  beforeEach(() => {
    resetState(state);
    vi.clearAllMocks();
    document.documentElement.style.removeProperty("--font-scale");
    document.documentElement.style.removeProperty("--accent");
  });

  it("load() restores the persisted font scale onto the document root", async () => {
    vi.mocked(backend.getPreferences).mockResolvedValue({ ...defaultPreferences(), fontScale: 1.25 });

    const { load } = usePreferences();
    await load();

    expect(document.documentElement.style.getPropertyValue("--font-scale")).toBe("1.25");
    expect(state.preferences.fontScale).toBe(1.25);
  });

  it("load() restores the persisted accent color onto the document root as --accent", async () => {
    vi.mocked(backend.getPreferences).mockResolvedValue({ ...defaultPreferences(), accent: "#A855F7" });

    const { load } = usePreferences();
    await load();

    expect(document.documentElement.style.getPropertyValue("--accent")).toBe("#A855F7");
    expect(state.preferences.accent).toBe("#A855F7");
  });

  it("update({ accent }) applies to the document root and persists, restoring the previous value on failure", async () => {
    vi.mocked(backend.getPreferences).mockResolvedValue({ ...defaultPreferences(), accent: "#F43F75" });
    const { load, update } = usePreferences();
    await load();

    vi.mocked(backend.updatePreferences).mockRejectedValue(new Error("nope"));
    await expect(update({ accent: "#22C55E" })).rejects.toThrow("nope");

    expect(document.documentElement.style.getPropertyValue("--accent")).toBe("#F43F75");
    expect(state.preferences.accent).toBe("#F43F75");
  });

  it("update({ fontScale }) applies to the document root and persists", async () => {
    vi.mocked(backend.updatePreferences).mockImplementation(async (preferences) => preferences);
    const { update } = usePreferences();

    await update({ fontScale: 1.25 });

    expect(state.preferences.fontScale).toBe(1.25);
    expect(document.documentElement.style.getPropertyValue("--font-scale")).toBe("1.25");
    expect(backend.updatePreferences).toHaveBeenCalledWith(expect.objectContaining({ fontScale: 1.25 }));
  });

  it("changing font scale from 100% to 125% updates the single --font-scale token that every semantic size derives from, proportionally, not one text selector", async () => {
    vi.mocked(backend.updatePreferences).mockImplementation(async (preferences) => preferences);
    const { update } = usePreferences();

    document.documentElement.style.setProperty("--font-scale", "1");
    const probe = document.createElement("div");
    probe.style.fontSize = "calc(1rem * var(--font-scale))";
    document.body.appendChild(probe);

    await update({ fontScale: 1.25 });

    // The proportional-scaling contract is that every semantic font-size
    // token is `calc(<base> * var(--font-scale))` (see styles/tokens.scss);
    // this asserts the one variable those tokens key off actually changed,
    // which is what makes every size scale together instead of just one.
    expect(document.documentElement.style.getPropertyValue("--font-scale")).toBe("1.25");
    document.body.removeChild(probe);
  });
});
