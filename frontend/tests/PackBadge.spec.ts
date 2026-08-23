import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import PackBadge from "../src/components/PackBadge/PackBadge.vue";

describe("PackBadge", () => {
  it("uses one static structure with the configured Pack color", () => {
    const wrapper = mount(PackBadge, { props: { name: "Frontend", color: "#3B82F6", compact: true } });

    expect(wrapper.element.tagName).toBe("SPAN");
    expect(wrapper.get(".pack-badge__label").text()).toBe("Frontend");
    expect(wrapper.get(".pack-badge__dot").attributes("style")).toBeUndefined();
    expect(wrapper.attributes("style")).toContain("--pack-color: #3B82F6");
    expect(wrapper.attributes("aria-pressed")).toBeUndefined();
  });

  it("uses button semantics and aria-pressed when selected, without a checkmark icon", async () => {
    const wrapper = mount(PackBadge, {
      props: { name: "Frontend", color: "#3B82F6", interactive: true, selected: true },
    });

    expect(wrapper.element.tagName).toBe("BUTTON");
    expect(wrapper.attributes("type")).toBe("button");
    expect(wrapper.attributes("aria-pressed")).toBe("true");
    expect(wrapper.classes()).toContain("pack-badge--selected");
    expect(wrapper.find(".pack-badge__state").exists()).toBe(false);
    await wrapper.trigger("click");
    expect(wrapper.emitted("click")).toHaveLength(1);
  });

  it("supports a pale, tinted-by-its-own-color muted appearance for inactive assignment controls", () => {
    const wrapper = mount(PackBadge, { props: { name: "Frontend", color: "#3B82F6", interactive: true, muted: true } });

    expect(wrapper.classes()).toContain("pack-badge--muted");
    expect(wrapper.classes()).not.toContain("pack-badge--selected");
    expect(wrapper.attributes("aria-pressed")).toBe("false");
  });
});
