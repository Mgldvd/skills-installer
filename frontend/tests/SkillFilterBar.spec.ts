import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import SkillFilterBar from "../src/components/SkillFilterBar/SkillFilterBar.vue";
import type { SkillTag } from "../src/types";

const tags: SkillTag[] = [
  { id: "recommended", name: "Recommended", color: "#E75480", order: 10, enabled: true },
  { id: "testing", name: "Testing", color: "#56A37B", order: 20, enabled: true },
];

describe("SkillFilterBar", () => {
  it("puts Sort controls in the left group and Filter controls in the right group", () => {
    const wrapper = mount(SkillFilterBar, { props: { query: "", sortBy: "name" } });
    const groups = wrapper.findAll(".skill-filter-bar__group");
    expect(groups[0].classes()).toContain("skill-filter-bar__group--sort");
    expect(groups[0].text()).toContain("Sort");
    expect(groups[0].find("select[aria-label='Sort Skills']").exists()).toBe(true);

    expect(groups[1].classes()).toContain("skill-filter-bar__group--filter");
    expect(groups[1].text()).toContain("Filter");
    expect(groups[1].find('input[type="search"]').exists()).toBe(true);
  });

  it("emits filter and sort updates", async () => {
    const wrapper = mount(SkillFilterBar, { props: { query: "", sortBy: "name" } });
    const search = wrapper.get('input[type="search"]');
    await search.setValue("triage");
    expect(wrapper.emitted("update:query")).toEqual([["triage"]]);
    const sort = wrapper.get('select[aria-label="Sort Skills"]');
    expect(sort.findAll("option").map((option) => option.text())).toEqual(["Name", "Pack", "Local", "Remote"]);
    await sort.setValue("pack");
    expect(wrapper.emitted("update:sortBy")).toEqual([["pack"]]);
  });

  it("toggles between Grid and List view, reflecting the active one via aria-pressed", async () => {
    const wrapper = mount(SkillFilterBar, { props: { view: "grid" } });
    const [gridBtn, listBtn] = wrapper.findAll('[aria-label="Grid view"], [aria-label="List view"]');
    expect(gridBtn.attributes("aria-pressed")).toBe("true");
    expect(listBtn.attributes("aria-pressed")).toBe("false");

    await listBtn.trigger("click");
    expect(wrapper.emitted("update:view")).toEqual([["list"]]);
  });

  it("toggles Local/Remote quick filters, deactivating on a second click", async () => {
    const wrapper = mount(SkillFilterBar, { props: { sourceFilter: "all" } });
    const [local, remote] = wrapper.findAll(".skill-filter-bar__source button");
    expect(local.attributes("aria-pressed")).toBe("false");
    expect(remote.attributes("aria-pressed")).toBe("false");

    await local.trigger("click");
    expect(wrapper.emitted("update:sourceFilter")).toEqual([["local"]]);

    await wrapper.setProps({ sourceFilter: "local" });
    await local.trigger("click");
    expect(wrapper.emitted("update:sourceFilter")).toEqual([["local"], ["all"]]);
  });

  it("offers an All Packs option plus one option per Pack, and emits null when reset", async () => {
    const wrapper = mount(SkillFilterBar, { props: { tags, packFilter: "recommended" } });
    const select = wrapper.get('select[aria-label="Filter by Pack"]');
    expect(select.findAll("option").map((option) => option.text())).toEqual([
      "All Packs",
      "Recommended",
      "Testing",
    ]);
    expect((select.element as HTMLSelectElement).value).toBe("recommended");

    await select.setValue("");
    expect(wrapper.emitted("update:packFilter")).toEqual([[null]]);
  });
});
