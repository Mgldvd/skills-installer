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

  it("toggles between Grid, Compact grid, and List view, reflecting the active one via aria-pressed", async () => {
    const wrapper = mount(SkillFilterBar, { props: { view: "grid" } });
    const gridBtn = wrapper.get('[aria-label="Grid view"]');
    const listBtn = wrapper.get('[aria-label="List view"]');
    const compactBtn = wrapper.get('[aria-label="Compact grid view"]');
    expect(gridBtn.attributes("aria-pressed")).toBe("true");
    expect(listBtn.attributes("aria-pressed")).toBe("false");
    expect(compactBtn.attributes("aria-pressed")).toBe("false");

    await listBtn.trigger("click");
    expect(wrapper.emitted("update:view")).toEqual([["list"]]);

    await compactBtn.trigger("click");
    expect(wrapper.emitted("update:view")).toEqual([["list"], ["compact"]]);
  });

  it("prefixes the Local/Remote filters with a decorative source icon, not a second accessible name", () => {
    const wrapper = mount(SkillFilterBar, { props: { sourceFilter: "all" } });
    const [local, remote] = wrapper.findAll(".skill-filter-bar__source button");
    expect(local.get(".source-icon").classes()).toContain("source-icon--local");
    expect(local.get(".source-icon").attributes("aria-hidden")).toBe("true");
    expect(remote.get(".source-icon").classes()).toContain("source-icon--remote");
    expect(remote.get(".source-icon").attributes("aria-hidden")).toBe("true");
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

  it("shows how many Skills need agents, disables the button when there are none, and toggles the filter", async () => {
    const empty = mount(SkillFilterBar, { props: { needsAgentsCount: 0 } });
    const emptyButton = empty.get(".skill-filter-bar__source button:nth-child(3)");
    expect(emptyButton.text()).toBe("Needs agents");
    expect(emptyButton.attributes("disabled")).toBeDefined();

    const wrapper = mount(SkillFilterBar, { props: { needsAgentsCount: 3, needsAgentsOnly: false } });
    const button = wrapper.get(".skill-filter-bar__source button:nth-child(3)");
    expect(button.text()).toBe("Needs agents (3)");
    expect(button.attributes("disabled")).toBeUndefined();
    expect(button.attributes("aria-pressed")).toBe("false");

    await button.trigger("click");
    expect(wrapper.emitted("update:needsAgentsOnly")).toEqual([[true]]);
  });

  it("never renders a selection action — filtering and selecting are separate concerns", () => {
    const wrapper = mount(SkillFilterBar, { props: { needsAgentsOnly: true, needsAgentsCount: 3 } });
    expect(wrapper.find(".skill-filter-bar__select-all").exists()).toBe(false);
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
