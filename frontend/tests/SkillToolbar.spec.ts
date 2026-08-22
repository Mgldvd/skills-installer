import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import SkillToolbar from "../src/components/SkillToolbar/SkillToolbar.vue";
import type { SkillTag } from "../src/types";
import { makeSkill } from "./fixtures";

const tags: SkillTag[] = [
  { id: "recommended", name: "Recommended", color: "#E75480", order: 10, enabled: true },
  { id: "testing", name: "Testing", color: "#56A37B", order: 20, enabled: true },
];

describe("SkillToolbar Pack preselection", () => {
  it("renders globally ordered Pack controls", () => {
    const wrapper = mount(SkillToolbar, {
      props: { tags, skills: [makeSkill({ tags: ["recommended"] })], selectedIds: [] },
    });
    expect(wrapper.findAll(".skill-toolbar__tag").map((item) => item.text())).toEqual(["Recommended1", "Testing0"]);
  });

  it("shows the active state when every Skill belonging to a Pack is selected", () => {
    const wrapper = mount(SkillToolbar, {
      props: { tags, skills: [makeSkill({ id: "a", tags: ["recommended"] })], selectedIds: ["a"] },
    });
    expect(wrapper.find(".skill-toolbar__tag").attributes("aria-pressed")).toBe("true");
    expect(wrapper.find(".skill-toolbar__tag").classes()).toContain("pack-badge--selected");
  });

  it("exposes partial Pack selection as a non-color mixed state", () => {
    const skills = [makeSkill({ id: "a", tags: ["recommended"] }), makeSkill({ id: "b", tags: ["recommended"] })];
    const wrapper = mount(SkillToolbar, { props: { tags, skills, selectedIds: ["a"] } });
    const badge = wrapper.find(".skill-toolbar__tag");
    expect(badge.attributes("aria-pressed")).toBe("mixed");
    expect(badge.classes()).toContain("pack-badge--partial");
    expect(badge.find(".pack-badge__state").exists()).toBe(true);
  });

  it("emits one Pack toggle without opening another menu", async () => {
    const wrapper = mount(SkillToolbar, {
      props: { tags, skills: [makeSkill({ tags: ["recommended"] })], selectedIds: [] },
    });
    await wrapper.find(".skill-toolbar__tag").trigger("click");
    expect(wrapper.emitted("toggleTag")).toEqual([["recommended"]]);
    expect(wrapper.find("dialog").exists()).toBe(false);
  });

  it("reorders Packs with drag and drop", async () => {
    const wrapper = mount(SkillToolbar, {
      props: { tags, skills: [makeSkill({ tags: ["recommended"] })], selectedIds: [] },
    });
    const slots = wrapper.findAll(".skill-toolbar__tag-slot");
    const dataTransfer = { setData: vi.fn(), effectAllowed: "" };

    await slots[0].trigger("dragstart", { dataTransfer });
    await slots[1].trigger("dragover");
    expect(slots[1].classes()).toContain("is-drag-over");
    expect(slots[1].classes()).toContain("is-insert-after");
    expect(slots[1].classes()).not.toContain("is-insert-before");
    await slots[1].trigger("drop");

    expect(wrapper.emitted("reorderTags")).toEqual([[["testing", "recommended"]]]);
  });

  it("supports keyboard Pack reordering with Alt and arrow keys", async () => {
    const wrapper = mount(SkillToolbar, {
      props: { tags, skills: [makeSkill({ tags: ["recommended", "testing"] })], selectedIds: [] },
    });
    await wrapper.findAll(".skill-toolbar__tag")[0].trigger("keydown", { key: "ArrowRight", altKey: true });
    expect(wrapper.emitted("reorderTags")).toEqual([[["testing", "recommended"]]]);
  });

  it("keeps Clear with Preselect and disables it when nothing is selected", async () => {
    const wrapper = mount(SkillToolbar, { props: { tags, skills: [makeSkill()], selectedIds: [] } });
    const clear = wrapper.get(".skill-toolbar__clear");
    expect(clear.attributes("disabled")).toBeDefined();

    await wrapper.setProps({ selectedIds: ["triage"] });
    expect(clear.attributes("disabled")).toBeUndefined();
    await clear.trigger("click");
    expect(wrapper.emitted("clearSelection")).toHaveLength(1);
  });
});
