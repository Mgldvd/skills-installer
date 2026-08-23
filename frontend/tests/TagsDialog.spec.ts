import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import TagsDialog from "../src/components/TagsDialog/TagsDialog.vue";
import { makeSkill } from "./fixtures";

const tags = [
  { id: "recommended", name: "Recommended", color: "#E75480", order: 10, enabled: true },
  { id: "frontend", name: "Frontend", color: "#5F82C9", order: 20, enabled: true },
  { id: "testing", name: "Testing", color: "#56A37B", order: 30, enabled: true },
  { id: "hidden", name: "Hidden", color: "#808089", order: 40, enabled: false },
];

function matrix() {
  return mount(TagsDialog, {
    props: {
      open: true,
      tags,
      skills: [
        makeSkill({ id: "a", displayName: "Skill A", description: "Must not render", tags: ["recommended"] }),
        makeSkill({ id: "b", displayName: "Skill B", description: "Also hidden", tags: ["frontend", "testing"] }),
      ],
    },
  });
}

describe("Packs assignment matrix", () => {
  it("renders only skill names and every enabled Pack in consistent order", () => {
    const wrapper = matrix();
    const rows = wrapper.findAll(".skill-row");
    expect(rows).toHaveLength(2);
    expect(wrapper.text()).not.toContain("Must not render");
    expect(wrapper.text()).not.toContain("Also hidden");
    expect(
      rows.map((row) => row.findAll(".tag-toggle").map((toggle) => toggle.find(".pack-badge__label").text())),
    ).toEqual([
      ["Recommended", "Frontend", "Testing"],
      ["Recommended", "Frontend", "Testing"],
    ]);
    expect(rows.every((row) => !row.text().includes("Hidden"))).toBe(true);
  });

  it("renders the requested active/inactive assignment states", () => {
    const [skillA, skillB] = matrix().findAll(".skill-row");
    expect(skillA.findAll(".tag-toggle").map((toggle) => toggle.attributes("aria-pressed"))).toEqual([
      "true",
      "false",
      "false",
    ]);
    expect(skillB.findAll(".tag-toggle").map((toggle) => toggle.attributes("aria-pressed"))).toEqual([
      "false",
      "true",
      "true",
    ]);
    expect(skillA.findAll(".pack-badge--selected")).toHaveLength(1);
    expect(skillB.findAll(".pack-badge--selected")).toHaveLength(2);
    expect(skillA.findAll(".pack-badge--muted")).toHaveLength(2);
    expect(skillB.findAll(".pack-badge--muted")).toHaveLength(1);
    expect(skillA.find(".pack-badge--selected").classes()).not.toContain("pack-badge--muted");
  });

  it("assigns an inactive Pack with one click and opens no assignment surface", async () => {
    const wrapper = matrix();
    const testing = wrapper.findAll(".skill-row")[0].findAll(".tag-toggle")[2];
    await testing.trigger("click");
    expect(wrapper.emitted("assign")).toEqual([["a", "testing"]]);
    expect(wrapper.find(".assign-popover").exists()).toBe(false);
    expect(wrapper.find("select").exists()).toBe(false);
  });

  it("unassigns an active Pack with one click and no confirmation", async () => {
    const wrapper = matrix();
    const frontend = wrapper.findAll(".skill-row")[1].findAll(".tag-toggle")[1];
    await frontend.trigger("click");
    expect(wrapper.emitted("unassign")).toEqual([["b", "frontend"]]);
  });

  it("shows the create-Pack form as part of the layout, with no +New Pack toggle or per-Skill Add controls", () => {
    const wrapper = matrix();
    expect(wrapper.find(".tags-dialog__editor").exists()).toBe(true);
    expect(wrapper.text()).not.toContain("+ New Pack");
    expect(wrapper.text()).not.toContain("+ Add Tag");
    expect(wrapper.text()).not.toContain("+ Create New Tag");
  });

  it("starts blank with no Cancel/Delete, ready to type a new Pack name", () => {
    const wrapper = matrix();
    const editor = wrapper.get(".tags-dialog__editor");
    expect((editor.get("input").element as HTMLInputElement).value).toBe("");
    expect(editor.find(".link-button--danger").exists()).toBe(false);
    expect(editor.find(".link-button").exists()).toBe(false);
    expect(editor.get("button[type=submit]").text()).toBe("Create");
  });

  it("switches the same form into editing an existing Pack, with Cancel returning it to blank", async () => {
    const wrapper = matrix();
    await wrapper.get(".catalog-tag").trigger("click");

    const editor = wrapper.get(".tags-dialog__editor");
    expect((editor.get("input").element as HTMLInputElement).value).toBe("Recommended");
    expect(editor.get("button[type=submit]").text()).toBe("Save");
    expect(editor.get(".link-button--danger").text()).toBe("Delete");

    await editor.get(".link-button:not(.link-button--danger)").trigger("click");
    expect((editor.get("input").element as HTMLInputElement).value).toBe("");
    expect(editor.get("button[type=submit]").text()).toBe("Create");
  });

  it("shows a newly supplied global Pack inactive on every row", async () => {
    const wrapper = matrix();
    await wrapper.setProps({
      tags: [...tags, { id: "security", name: "Security", color: "#6870C4", order: 50, enabled: true }],
    });
    const rows = wrapper.findAll(".skill-row");
    expect(rows.every((row) => row.findAll(".tag-toggle").at(-1)?.text() === "Security")).toBe(true);
    expect(rows.every((row) => row.findAll(".tag-toggle").at(-1)?.attributes("aria-pressed") === "false")).toBe(true);
  });

  it("searches by skill name only", async () => {
    const wrapper = matrix();
    await wrapper.get('[aria-label="Search skills"]').setValue("skill b");
    expect(wrapper.findAll(".skill-row")).toHaveLength(1);
    expect(wrapper.text()).toContain("Skill B");
    await wrapper.get('[aria-label="Search skills"]').setValue("hidden");
    expect(wrapper.findAll(".skill-row")).toHaveLength(0);
  });

  it("offers 15 curated colors plus a Custom color picker in the Pack editor", async () => {
    const wrapper = matrix();
    await wrapper.get(".tags-dialog__header .button").trigger("click");

    const swatches = wrapper.findAll(".color-palette-picker__swatch");
    expect(swatches).toHaveLength(16);
    expect(wrapper.find(".color-palette-picker__custom").exists()).toBe(true);
    // The very first curated swatch is the default for a brand-new Pack.
    expect(wrapper.find(".color-palette-picker__swatch.is-selected").exists()).toBe(true);
    expect(wrapper.find(".color-palette-picker__custom.is-selected").exists()).toBe(false);
  });

  it("picking a Custom color selects the Custom swatch and saves that exact color", async () => {
    const wrapper = matrix();
    await wrapper.get(".tags-dialog__header .button").trigger("click");
    await wrapper.get(".tags-dialog__editor-field input").setValue("New Pack");

    await wrapper.get(".color-palette-picker__custom input[type='color']").setValue("#abcdef");

    expect(wrapper.get(".color-palette-picker__custom").attributes("aria-pressed")).toBe("true");
    expect(wrapper.findAll(".color-palette-picker__swatch.is-selected")).toHaveLength(1);

    await wrapper.get(".tags-dialog__editor").trigger("submit");
    expect(wrapper.emitted("create")).toEqual([["New Pack", "#ABCDEF"]]);
  });

  it("picking a curated swatch again deselects the Custom color", async () => {
    const wrapper = matrix();
    await wrapper.get(".tags-dialog__header .button").trigger("click");
    await wrapper.get(".color-palette-picker__custom input[type='color']").setValue("#abcdef");
    expect(wrapper.get(".color-palette-picker__custom").attributes("aria-pressed")).toBe("true");

    await wrapper.findAll(".color-palette-picker__swatch")[2].trigger("click");
    expect(wrapper.get(".color-palette-picker__custom").attributes("aria-pressed")).toBe("false");
  });

  it("disables only the pending relationship without moving layout", () => {
    const wrapper = mount(TagsDialog, {
      props: {
        open: true,
        tags,
        pendingKeys: ["a:frontend"],
        skills: [makeSkill({ id: "a", displayName: "Skill A", tags: [] })],
      },
    });
    const controls = wrapper.findAll(".tag-toggle");
    expect(controls[1].attributes("disabled")).toBeDefined();
    expect(controls[1].classes()).toContain("is-pending");
    expect(controls[0].attributes("disabled")).toBeUndefined();
  });
});
