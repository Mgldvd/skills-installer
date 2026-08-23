import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import SourceIcon from "../src/components/SourceIcon/SourceIcon.vue";

describe("SourceIcon", () => {
  it("renders a triangle with an L for a local Skill", () => {
    const wrapper = mount(SourceIcon, { props: { local: true } });
    expect(wrapper.classes()).toContain("source-icon--local");
    expect(wrapper.find("path").exists()).toBe(true);
    expect(wrapper.find("rect").exists()).toBe(false);
    expect(wrapper.get("text").text()).toBe("L");
    expect(wrapper.attributes("aria-label")).toBe("Local Skill");
  });

  it("renders a square with an R for a remote Skill", () => {
    const wrapper = mount(SourceIcon, { props: { local: false } });
    expect(wrapper.classes()).toContain("source-icon--remote");
    expect(wrapper.find("rect").exists()).toBe(true);
    expect(wrapper.find("path").exists()).toBe(false);
    expect(wrapper.get("text").text()).toBe("R");
    expect(wrapper.attributes("aria-label")).toBe("Remote Skill");
  });

  it("hides its own accessible name when used decoratively beside an already-labeled control", () => {
    const wrapper = mount(SourceIcon, { props: { local: true, decorative: true } });
    expect(wrapper.attributes("aria-label")).toBeUndefined();
    expect(wrapper.attributes("title")).toBeUndefined();
    expect(wrapper.attributes("aria-hidden")).toBe("true");
  });
});
