import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../src/services/backend", () => ({ selectLocalCatalogDirectory: vi.fn() }));

import PreferencesDialog from "../src/components/PreferencesDialog/PreferencesDialog.vue";
import * as backend from "../src/services/backend";
import { defaultPreferences } from "../src/types";

function sourceButton(wrapper: ReturnType<typeof mount>, label: string) {
  return wrapper.findAll(".preferences-dialog__source-actions button").find((b) => b.text() === label)!;
}

describe("PreferencesDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("saves the Local Skill Source by pressing Enter, with no separate save button", async () => {
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: { ...defaultPreferences(), localSourcePath: "/old/source" } },
    });
    await nextTick();

    expect(wrapper.find(".preferences-dialog__source-actions").text()).not.toContain("Change Folder");

    const input = wrapper.find("#local-skill-source");
    expect(input.element).toHaveProperty("value", "/old/source");
    await input.setValue("/new/source");
    await input.trigger("keydown.enter");

    expect(wrapper.emitted("updateLocalSource")).toEqual([["/new/source"]]);
  });

  it("also saves the Local Skill Source on blur", async () => {
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: { ...defaultPreferences(), localSourcePath: "/old/source" } },
    });
    await nextTick();

    const input = wrapper.find("#local-skill-source");
    await input.setValue("/new/source");
    await input.trigger("blur");

    expect(wrapper.emitted("updateLocalSource")).toEqual([["/new/source"]]);
  });

  it("does not save on Enter or blur when the source is unchanged or empty", async () => {
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: { ...defaultPreferences(), localSourcePath: "/old/source" } },
    });
    await nextTick();

    const input = wrapper.find("#local-skill-source");
    await input.trigger("keydown.enter");
    await input.trigger("blur");
    expect(wrapper.emitted("updateLocalSource")).toBeUndefined();

    await input.setValue("");
    await input.trigger("keydown.enter");
    expect(wrapper.emitted("updateLocalSource")).toBeUndefined();
  });

  it("offers a refresh action for the configured local source", async () => {
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: defaultPreferences() },
    });

    await sourceButton(wrapper, "Refresh Catalog").trigger("click");
    expect(wrapper.emitted("refreshLocalSource")).toHaveLength(1);
  });

  it("picks a folder from the native dialog and saves it immediately, no confirmation step", async () => {
    vi.mocked(backend.selectLocalCatalogDirectory).mockResolvedValue("/picked/folder");
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: { ...defaultPreferences(), localSourcePath: "/old/source" } },
    });
    await nextTick();

    await sourceButton(wrapper, "Browse…").trigger("click");
    await nextTick();

    expect(backend.selectLocalCatalogDirectory).toHaveBeenCalledWith("/old/source");
    expect((wrapper.find("#local-skill-source").element as HTMLInputElement).value).toBe("/picked/folder");
    expect(wrapper.emitted("updateLocalSource")).toEqual([["/picked/folder"]]);
  });

  it("leaves the field untouched when the folder picker is dismissed", async () => {
    vi.mocked(backend.selectLocalCatalogDirectory).mockResolvedValue(null);
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: { ...defaultPreferences(), localSourcePath: "/old/source" } },
    });
    await nextTick();

    await sourceButton(wrapper, "Browse…").trigger("click");
    await nextTick();

    expect((wrapper.find("#local-skill-source").element as HTMLInputElement).value).toBe("/old/source");
    expect(wrapper.emitted("updateLocalSource")).toBeUndefined();
  });

  it("offers Linux CLI installation", async () => {
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: defaultPreferences() },
    });

    await wrapper.get(".preferences-dialog__cli-button").trigger("click");
    expect(wrapper.emitted("installCli")).toHaveLength(1);
  });

  it("no longer offers Compact cards here — it's the toolbar's Compact grid view option now", () => {
    const wrapper = mount(PreferencesDialog, { props: { open: true, preferences: defaultPreferences() } });
    expect(wrapper.text()).not.toContain("Compact cards");
  });

  it("offers the same 16 colors as the Pack picker for the Accent color", async () => {
    const wrapper = mount(PreferencesDialog, { props: { open: true, preferences: defaultPreferences() } });

    expect(wrapper.findAll(".color-palette-picker__swatch")).toHaveLength(16);
    expect(wrapper.find(".color-palette-picker__swatch.is-selected").attributes("aria-label")).toBe(
      `Use color ${defaultPreferences().accent}`,
    );

    await wrapper.findAll(".color-palette-picker__swatch")[3].trigger("click");
    expect(wrapper.emitted("update")).toContainEqual([{ accent: "#F59E0B" }]);
  });

  it("offers quick links to manage Packs and Agents from an Organize section", async () => {
    const wrapper = mount(PreferencesDialog, { props: { open: true, preferences: defaultPreferences() } });

    const packsButton = wrapper.findAll("button").find((b) => b.text() === "Manage Packs")!;
    const agentsButton = wrapper.findAll("button").find((b) => b.text() === "Manage Agents")!;
    expect(packsButton.classes()).toContain("preferences-dialog__tool-button");
    expect(agentsButton.classes()).toContain("preferences-dialog__tool-button");

    await packsButton.trigger("click");
    expect(wrapper.emitted("openPacks")).toHaveLength(1);

    await agentsButton.trigger("click");
    expect(wrapper.emitted("openAgents")).toHaveLength(1);
  });
});
