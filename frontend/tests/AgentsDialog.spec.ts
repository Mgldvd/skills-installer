import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import AgentsDialog from "../src/components/AgentsDialog/AgentsDialog.vue";
describe("AgentsDialog", () => {
  it("supports visible one-click multi-agent toggles", async () => {
    const wrapper = mount(AgentsDialog, {
      props: { open: true, modelValue: ["universal"], agentOrder: [], scope: "project" },
    });
    const buttons = wrapper.findAll(".agents-dialog__toggle");
    expect(buttons[0].attributes("aria-pressed")).toBe("true");
    await buttons.find((b) => b.text().includes("Claude Code"))!.trigger("click");
    await buttons.find((b) => b.text().includes("Codex"))!.trigger("click");
    await wrapper.get(".agents-dialog__done").trigger("click");
    expect(wrapper.emitted("update:modelValue")).toEqual([[["universal", "claude-code", "codex"]]]);
  });

  it("shows Universal as indeterminate once a sibling agent sharing its folder is selected", async () => {
    const wrapper = mount(AgentsDialog, { props: { open: true, modelValue: [], agentOrder: [], scope: "project" } });
    const buttons = wrapper.findAll(".agents-dialog__toggle");
    const universalButton = buttons.find((b) => b.text().includes("Universal"))!;
    const codexButton = buttons.find((b) => b.text().includes("Codex"))!;

    expect(universalButton.get(".agents-dialog__check").classes()).not.toContain("is-indeterminate");

    await codexButton.trigger("click");

    expect(universalButton.get(".agents-dialog__check").classes()).toContain("is-indeterminate");
    expect(universalButton.attributes("aria-pressed")).toBe("false");

    // Explicitly selecting Universal itself clears the indeterminate hint —
    // it's no longer merely implied, it's chosen.
    await universalButton.trigger("click");
    expect(universalButton.get(".agents-dialog__check").classes()).not.toContain("is-indeterminate");
  });

  it("highlights an agent's folder path only when it's exclusively its own, not shared with Universal", () => {
    const wrapper = mount(AgentsDialog, { props: { open: true, modelValue: [], agentOrder: [], scope: "project" } });
    const buttons = wrapper.findAll(".agents-dialog__toggle");

    const claudePath = buttons.find((b) => b.text().includes("Claude Code"))!.get(".agents-dialog__path");
    expect(claudePath.classes()).toContain("agents-dialog__path--own");

    const windsurfPath = buttons.find((b) => b.text().includes("Windsurf"))!.get(".agents-dialog__path");
    expect(windsurfPath.classes()).toContain("agents-dialog__path--own");

    const codexPath = buttons.find((b) => b.text().includes("Codex"))!.get(".agents-dialog__path");
    expect(codexPath.classes()).not.toContain("agents-dialog__path--own");

    const universalPath = buttons.find((b) => b.text().includes("Universal"))!.get(".agents-dialog__path");
    expect(universalPath.classes()).not.toContain("agents-dialog__path--own");
  });

  // Scope is a single app-wide switch now, not a per-agent toggle — this
  // dialog just shows both destinations as reference info, always, for
  // every agent.
  it("always shows both the Project and Global path for every agent", () => {
    const wrapper = mount(AgentsDialog, { props: { open: true, modelValue: [], agentOrder: [], scope: "project" } });
    const claudeRow = wrapper.findAll(".agents-dialog__toggle").find((b) => b.text().includes("Claude Code"))!;
    const paths = claudeRow.findAll(".agents-dialog__path").map((p) => p.text());
    expect(paths).toEqual(["Project: .claude/skills", "Global: ~/.claude/skills"]);
    expect(wrapper.find(".agents-dialog__scope").exists()).toBe(false);
    expect(wrapper.find(".agents-dialog__bulk-scope").exists()).toBe(false);
  });

  // Same treatment as AppHeader's Global mode, so opening Agents while
  // Global is active reads as "these agents install into your home
  // directory" instead of looking identical to Project mode.
  it("tints the header and shows a Global badge when the active scope is global", () => {
    const projectWrapper = mount(AgentsDialog, {
      props: { open: true, modelValue: [], agentOrder: [], scope: "project" },
    });
    expect(projectWrapper.find(".agents-dialog__header--global").exists()).toBe(false);
    expect(projectWrapper.find(".agents-dialog__global-badge").exists()).toBe(false);

    const globalWrapper = mount(AgentsDialog, {
      props: { open: true, modelValue: [], agentOrder: [], scope: "global" },
    });
    expect(globalWrapper.get(".agents-dialog__header").classes()).toContain("agents-dialog__header--global");
    expect(globalWrapper.get(".agents-dialog__global-badge").text()).toBe("Global");
  });
});
