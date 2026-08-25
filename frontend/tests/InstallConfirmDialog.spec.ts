import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import InstallConfirmDialog from "../src/components/InstallConfirmDialog/InstallConfirmDialog.vue";
import { defaultInstallOptions } from "../src/types";
import { makeSkill } from "./fixtures";

describe("InstallConfirmDialog", () => {
  it("shows an agent chip and a row per selected skill, all icons highlighted for a fresh install", () => {
    const wrapper = mount(InstallConfirmDialog, {
      props: {
        open: true,
        skills: [makeSkill({ id: "a", displayName: "Alpha" }), makeSkill({ id: "b", displayName: "Beta" })],
        options: { ...defaultInstallOptions(), agents: ["universal"] },
        projectPath: "/tmp/project",
      },
    });

    const chips = wrapper.findAll(".install-confirm-dialog__agent-chip");
    expect(chips).toHaveLength(1);
    expect(chips[0].text()).toContain("Universal (.agents)");

    const rows = wrapper.findAll(".install-confirm-dialog__skill-row");
    expect(rows).toHaveLength(2);
    expect(rows[0].get(".install-confirm-dialog__skill-icon").classes()).toContain("is-new");
    expect(wrapper.find(".install-confirm-dialog__meta").exists()).toBe(false);
  });

  it("shows the project folder in Project scope, and a Global note instead in Global scope", () => {
    const projectWrapper = mount(InstallConfirmDialog, {
      props: {
        open: true,
        skills: [makeSkill({ id: "a" })],
        options: { ...defaultInstallOptions(), scope: "project" },
        projectPath: "/tmp/project",
      },
    });
    expect(projectWrapper.text()).toContain("Project folder");
    expect(projectWrapper.text()).toContain("/tmp/project");
    expect(projectWrapper.text()).not.toContain("Scope");

    const globalWrapper = mount(InstallConfirmDialog, {
      props: {
        open: true,
        skills: [makeSkill({ id: "a" })],
        options: { ...defaultInstallOptions(), scope: "global" },
        projectPath: "/tmp/project",
      },
    });
    expect(globalWrapper.text()).toContain("Scope");
    expect(globalWrapper.text()).toContain("Global");
    expect(globalWrapper.text()).not.toContain("Project folder");
  });

  it("only highlights the agent icons a partially-installed skill is actually gaining", () => {
    const wrapper = mount(InstallConfirmDialog, {
      props: {
        open: true,
        skills: [makeSkill({ id: "gap", displayName: "Gap Fill", installed: true, installedAgents: ["windsurf"] })],
        options: { ...defaultInstallOptions(), agents: ["windsurf", "claude-code"] },
        projectPath: "/tmp/project",
      },
    });

    const icons = wrapper.get(".install-confirm-dialog__skill-row").findAll(".install-confirm-dialog__skill-icon");
    expect(icons).toHaveLength(2);
    expect(icons[0].classes()).not.toContain("is-new"); // windsurf: already installed
    expect(icons[1].classes()).toContain("is-new"); // claude-code: being added
    expect(wrapper.get(".install-confirm-dialog__meta").text()).toContain("1 of these are already installed");
  });

  it("emits confirm with the full default agent list and closes when Install Selected is clicked", async () => {
    const wrapper = mount(InstallConfirmDialog, {
      props: {
        open: true,
        skills: [makeSkill({ id: "a" })],
        options: { ...defaultInstallOptions(), agents: ["universal", "claude-code"] },
        projectPath: "/tmp/project",
      },
    });

    await wrapper.get(".install-confirm-dialog__btn--primary").trigger("click");

    expect(wrapper.emitted("confirm")).toEqual([[["universal", "claude-code"]]]);
    expect(wrapper.emitted("update:open")).toEqual([[false]]);
  });

  it("lets an agent be excluded from just this install without touching the configured defaults", async () => {
    const wrapper = mount(InstallConfirmDialog, {
      props: {
        open: true,
        skills: [makeSkill({ id: "a", displayName: "Alpha" })],
        options: { ...defaultInstallOptions(), agents: ["universal", "claude-code"] },
        projectPath: "/tmp/project",
      },
    });

    const chips = wrapper.findAll(".install-confirm-dialog__agent-chip");
    const claudeChip = chips.find((c) => c.text().includes("Claude Code"))!;
    await claudeChip.trigger("click");

    expect(claudeChip.classes()).toContain("is-off");
    expect(claudeChip.attributes("aria-pressed")).toBe("false");
    // Excluding an agent drops its icon from every skill row too, since it
    // won't actually be installed for any of them this time.
    expect(wrapper.get(".install-confirm-dialog__skill-row").findAll(".install-confirm-dialog__skill-icon")).toHaveLength(
      1,
    );

    await wrapper.get(".install-confirm-dialog__btn--primary").trigger("click");
    expect(wrapper.emitted("confirm")).toEqual([[["universal"]]]);
    // The dialog's own options prop — standing in for the configured
    // defaults — is untouched; only the emitted override changed.
    expect(wrapper.props("options").agents).toEqual(["universal", "claude-code"]);
  });

  it("blocks confirmation once every agent has been excluded", async () => {
    const wrapper = mount(InstallConfirmDialog, {
      props: {
        open: true,
        skills: [makeSkill({ id: "a" })],
        options: { ...defaultInstallOptions(), agents: ["universal"] },
        projectPath: "/tmp/project",
      },
    });

    await wrapper.get(".install-confirm-dialog__agent-chip").trigger("click");

    expect(wrapper.get(".install-confirm-dialog__error").text()).toContain("Select at least one agent");
    expect(wrapper.get(".install-confirm-dialog__btn--primary").attributes("disabled")).toBeDefined();

    await wrapper.get(".install-confirm-dialog__btn--primary").trigger("click");
    expect(wrapper.emitted("confirm")).toBeUndefined();
  });

  it("resets excluded agents back to the defaults each time the dialog reopens", async () => {
    const wrapper = mount(InstallConfirmDialog, {
      props: {
        open: true,
        skills: [makeSkill({ id: "a" })],
        options: { ...defaultInstallOptions(), agents: ["universal", "claude-code"] },
        projectPath: "/tmp/project",
      },
    });

    await wrapper.get(".install-confirm-dialog__agent-chip").trigger("click");
    expect(wrapper.findAll(".install-confirm-dialog__agent-chip.is-off")).toHaveLength(1);

    await wrapper.setProps({ open: false });
    await wrapper.setProps({ open: true });

    expect(wrapper.findAll(".install-confirm-dialog__agent-chip.is-off")).toHaveLength(0);
  });
});
