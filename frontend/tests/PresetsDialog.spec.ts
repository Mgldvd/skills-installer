import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn().mockResolvedValue(undefined) }));
import { openUrl } from "@tauri-apps/plugin-opener";

import PresetsDialog from "../src/components/PresetsDialog/PresetsDialog.vue";
import type { Preset } from "../src/types";

const presets: Preset[] = [
  { id: "atlas", name: "Atlas", gitUrl: "https://github.com/me/atlas", skillNames: ["triage", "docs"] },
  { id: "zephyr", name: "Zephyr", gitUrl: null, skillNames: ["triage"] },
];

describe("PresetsDialog", () => {
  it("lists saved Presets with their git URL and Skill count", () => {
    const wrapper = mount(PresetsDialog, {
      props: { open: true, presets, installedSkillNames: [] },
    });

    const rows = wrapper.findAll(".presets-dialog__row");
    expect(rows).toHaveLength(2);
    expect(rows[0].get(".presets-dialog__name").text()).toBe("Atlas");
    expect(rows[0].get(".presets-dialog__git-url").text()).toBe("https://github.com/me/atlas");
    expect(rows[0].get(".presets-dialog__count").text()).toBe("2 Skills");
    expect(rows[1].find(".presets-dialog__git-url").exists()).toBe(false);
    expect(rows[1].get(".presets-dialog__count").text()).toBe("1 Skill");
  });

  it("shows the GitHub or GitLab mark for a recognized host, and a generic link icon otherwise", () => {
    const wrapper = mount(PresetsDialog, {
      props: {
        open: true,
        presets: [
          { id: "gh", name: "GH", gitUrl: "https://github.com/me/gh", skillNames: ["a"] },
          { id: "gl", name: "GL", gitUrl: "https://gitlab.com/me/gl", skillNames: ["a"] },
          { id: "other", name: "Other", gitUrl: "https://example.com/me/other", skillNames: ["a"] },
        ],
        installedSkillNames: [],
      },
    });
    const rows = wrapper.findAll(".presets-dialog__row");
    expect(rows[0].get(".presets-dialog__git-icon").html()).toContain("viewBox=\"0 0 16 16\"");
    expect(rows[1].get(".presets-dialog__git-icon").html()).toContain("viewBox=\"0 0 24 24\"");
    expect(rows[2].get(".presets-dialog__git-icon").exists()).toBe(true);
  });

  it("opens the git URL in the system browser instead of navigating the app's own window", async () => {
    const wrapper = mount(PresetsDialog, { props: { open: true, presets, installedSkillNames: [] } });
    const link = wrapper.get(".presets-dialog__git-url");

    await link.trigger("click");

    expect(openUrl).toHaveBeenCalledWith("https://github.com/me/atlas");
  });

  it("shows the empty state when nothing has been saved yet", () => {
    const wrapper = mount(PresetsDialog, { props: { open: true, presets: [], installedSkillNames: [] } });
    expect(wrapper.get(".presets-dialog__empty").text()).toBe("No Presets saved yet.");
  });

  it("fills the Name field from the git URL's repo name while Name is still empty", async () => {
    const wrapper = mount(PresetsDialog, { props: { open: true, presets: [], installedSkillNames: ["a"] } });

    await wrapper.get("input[type=url]").setValue("https://github.com/Mgldvd/skills-installer");

    expect((wrapper.get("input[type=text]").element as HTMLInputElement).value).toBe("skills-installer");
  });

  it("never overwrites a Name the user already typed themselves", async () => {
    const wrapper = mount(PresetsDialog, { props: { open: true, presets: [], installedSkillNames: ["a"] } });

    await wrapper.get("input[type=text]").setValue("Custom Name");
    await wrapper.get("input[type=url]").setValue("https://github.com/Mgldvd/skills-installer");

    expect((wrapper.get("input[type=text]").element as HTMLInputElement).value).toBe("Custom Name");
  });

  it("disables Save until there's a name and at least one installed Skill", async () => {
    const wrapper = mount(PresetsDialog, {
      props: { open: true, presets: [], installedSkillNames: [] },
    });
    const submit = wrapper.get("button[type=submit]");
    expect(submit.attributes("disabled")).toBeDefined();
    expect(submit.text()).toBe("Save 0 installed Skills");

    await wrapper.setProps({ installedSkillNames: ["triage"] });
    expect(wrapper.get("button[type=submit]").attributes("disabled")).toBeDefined();
    expect(wrapper.get("button[type=submit]").text()).toBe("Save 1 installed Skill");

    await wrapper.get("input[type=text]").setValue("My Preset");
    expect(wrapper.get("button[type=submit]").attributes("disabled")).toBeUndefined();
  });

  it("emits save with the trimmed name and a null git URL when left blank, then clears the form", async () => {
    const wrapper = mount(PresetsDialog, {
      props: { open: true, presets: [], installedSkillNames: ["triage"] },
    });

    await wrapper.get("input[type=text]").setValue("  My Preset  ");
    await wrapper.get(".presets-dialog__form").trigger("submit");

    expect(wrapper.emitted("save")).toEqual([["My Preset", null]]);
    expect((wrapper.get("input[type=text]").element as HTMLInputElement).value).toBe("");
  });

  it("emits save with a trimmed git URL when provided", async () => {
    const wrapper = mount(PresetsDialog, {
      props: { open: true, presets: [], installedSkillNames: ["triage"] },
    });

    await wrapper.get("input[type=text]").setValue("My Preset");
    await wrapper.get("input[type=url]").setValue("  https://github.com/me/repo  ");
    await wrapper.get(".presets-dialog__form").trigger("submit");

    expect(wrapper.emitted("save")).toEqual([["My Preset", "https://github.com/me/repo"]]);
  });

  it("emits load with the preset id", async () => {
    const wrapper = mount(PresetsDialog, { props: { open: true, presets, installedSkillNames: [] } });
    await wrapper.findAll(".presets-dialog__row")[0].get(".button").trigger("click");
    expect(wrapper.emitted("load")).toEqual([["atlas"]]);
  });

  it("asks for confirmation before emitting delete, and does nothing if declined", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    const wrapper = mount(PresetsDialog, { props: { open: true, presets, installedSkillNames: [] } });

    await wrapper.findAll(".presets-dialog__row")[0].get(".presets-dialog__delete").trigger("click");
    expect(confirmSpy).toHaveBeenCalled();
    expect(wrapper.emitted("delete")).toBeUndefined();

    confirmSpy.mockReturnValue(true);
    await wrapper.findAll(".presets-dialog__row")[0].get(".presets-dialog__delete").trigger("click");
    expect(wrapper.emitted("delete")).toEqual([["atlas"]]);

    confirmSpy.mockRestore();
  });

  it("shows a save error when the backend rejects it", () => {
    const wrapper = mount(PresetsDialog, {
      props: { open: true, presets: [], installedSkillNames: [], error: "Preset name cannot be empty" },
    });
    expect(wrapper.get(".presets-dialog__error").text()).toBe("Preset name cannot be empty");
  });
});
