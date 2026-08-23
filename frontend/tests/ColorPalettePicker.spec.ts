import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import ColorPalettePicker from "../src/components/ColorPalettePicker/ColorPalettePicker.vue";

describe("ColorPalettePicker", () => {
  it("marks the swatch matching modelValue as selected and emits the clicked color", async () => {
    const wrapper = mount(ColorPalettePicker, { props: { modelValue: "#3B82F6" } });

    const swatches = wrapper.findAll(".color-palette-picker__swatch");
    expect(swatches).toHaveLength(16);
    expect(wrapper.find(".color-palette-picker__swatch.is-selected").attributes("aria-label")).toBe(
      "Use color #3B82F6",
    );

    await swatches[0].trigger("click");
    expect(wrapper.emitted("update:modelValue")).toEqual([["#F43F75"]]);
  });

  it("treats a modelValue outside the curated palette as the Custom color", () => {
    const wrapper = mount(ColorPalettePicker, { props: { modelValue: "#123456" } });

    expect(wrapper.find(".color-palette-picker__custom").attributes("aria-pressed")).toBe("true");
    expect(wrapper.find(".color-palette-picker__custom.is-selected").exists()).toBe(true);
    expect(wrapper.findAll(".color-palette-picker__swatch.is-selected")).toHaveLength(1);
  });

  it("picking a custom color emits the uppercased hex", async () => {
    const wrapper = mount(ColorPalettePicker, { props: { modelValue: "#F43F75" } });

    await wrapper.get(".color-palette-picker__custom input[type='color']").setValue("#abcdef");

    expect(wrapper.emitted("update:modelValue")).toEqual([["#ABCDEF"]]);
  });
});
