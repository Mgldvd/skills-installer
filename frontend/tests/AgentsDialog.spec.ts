import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import AgentsDialog from "../src/components/AgentsDialog/AgentsDialog.vue";
describe("AgentsDialog", () => {
  it("supports visible one-click multi-agent toggles", async () => {
    const wrapper = mount(AgentsDialog, { props: { open: true, modelValue: ["universal"], scope: "project" } });
    const buttons = wrapper.findAll(".agents-dialog__toggle");
    expect(buttons[0].attributes("aria-pressed")).toBe("true");
    await buttons.find((b) => b.text().includes("Claude Code"))!.trigger("click");
    await buttons.find((b) => b.text().includes("Codex"))!.trigger("click");
    await wrapper.get(".agents-dialog__done").trigger("click");
    expect(wrapper.emitted("update:modelValue")).toEqual([[["universal", "claude-code", "codex"]]]);
  });
});
