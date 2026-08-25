import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import SkillCard from "../src/components/SkillCard/SkillCard.vue";
import { makeSkill } from "./fixtures";

describe("SkillCard", () => {
  it("renders the skill name and description in dedicated sections", () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ displayName: "Issue Triage", description: "Helps triage issues." }),
        selected: false,
      },
    });

    expect(wrapper.find(".skill-card__title").text()).toBe("Issue Triage");
    expect(wrapper.find(".skill-card__description-text").text()).toBe("Helps triage issues.");
    expect(wrapper.find(".skill-card__description-label").exists()).toBe(false);
    expect(wrapper.find(".skill-card__description").element.tagName).toBe("DIV");
    expect(wrapper.text()).not.toContain("View details");
  });

  it("hides description details in compact mode", () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ description: "Hidden details" }), selected: false, compact: true },
    });
    expect(wrapper.classes()).toContain("is-compact");
    expect(wrapper.find(".skill-card__description").exists()).toBe(false);
    expect(wrapper.text()).not.toContain("Hidden details");
  });

  it("hides description details in list mode, still selectable and editable", async () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ id: "triage", description: "Hidden in list" }), selected: false, list: true },
    });
    expect(wrapper.classes()).toContain("is-list");
    expect(wrapper.find(".skill-card__description").exists()).toBe(false);
    expect(wrapper.text()).not.toContain("Hidden in list");

    await wrapper.find(".skill-card__selection-surface").trigger("click");
    expect(wrapper.emitted("toggle")).toEqual([["triage"]]);
  });

  it("selects from the title and dead card surface without hijacking description", async () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill({ id: "triage" }), selected: false } });

    await wrapper.find(".skill-card__description").trigger("click");
    expect(wrapper.emitted("toggle")).toBeUndefined();
    expect(wrapper.emitted("edit")).toBeUndefined();

    await wrapper.find(".skill-card__selection-surface").trigger("click");
    expect(wrapper.emitted("toggle")).toEqual([["triage"]]);
  });

  it("exposes selection through aria-pressed and a clean top-right ribbon", () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill(), selected: true } });
    const selection = wrapper.find(".skill-card__selection-surface");

    expect(wrapper.classes()).toContain("is-selected");
    expect(selection.attributes("aria-pressed")).toBe("true");
    expect(wrapper.find(".skill-card__selection-ribbon").exists()).toBe(true);
    expect(wrapper.find(".skill-card__selection-checkbox").exists()).toBe(false);
    expect(wrapper.text()).not.toContain("Selected for installation");
  });

  it("marks a fully installed skill with is-installed and stays quiet up top — no badge, no spinner", () => {
    const installed = mount(SkillCard, { props: { skill: makeSkill({ installed: true }), selected: false } });
    expect(installed.classes()).toContain("is-installed");
    expect(installed.classes()).not.toContain("is-partially-installed");
    expect(installed.get(".skill-card__top").text()).toBe(installed.get(".skill-card__title").text());

    const notInstalled = mount(SkillCard, { props: { skill: makeSkill({ installed: false }), selected: false } });
    expect(notInstalled.classes()).not.toContain("is-installed");
  });

  it("marks a skill as Local or Remote with the source icon, never both", () => {
    const local = mount(SkillCard, { props: { skill: makeSkill({ local: true }), selected: false } });
    const localIcon = local.get(".skill-card__source");
    expect(localIcon.classes()).toContain("source-icon--local");
    expect(localIcon.attributes("aria-label")).toBe("Local Skill");

    const remote = mount(SkillCard, { props: { skill: makeSkill({ local: false }), selected: false } });
    const remoteIcon = remote.get(".skill-card__source");
    expect(remoteIcon.classes()).toContain("source-icon--remote");
    expect(remoteIcon.attributes("aria-label")).toBe("Remote Skill");
  });

  it("shows a spinner while a skill is installing", () => {
    const installing = mount(SkillCard, {
      props: { skill: makeSkill({ installed: false }), selected: false, installing: true },
    });
    expect(installing.classes()).toContain("is-installing");
    expect(installing.find(".skill-card__spinner").exists()).toBe(true);
    expect(installing.find(".skill-card__installing").text()).toContain("Installing");
    expect(installing.find(".skill-card__selection-surface").attributes("aria-label")).toContain("installing");

    const notInstalling = mount(SkillCard, {
      props: { skill: makeSkill({ installed: false }), selected: false, installing: false },
    });
    expect(notInstalling.classes()).not.toContain("is-installing");
    expect(notInstalling.find(".skill-card__spinner").exists()).toBe(false);
  });

  it("never shows the spinner for a fully-installed skill even if `installing` is stale", () => {
    const alreadyInstalled = mount(SkillCard, {
      props: { skill: makeSkill({ installed: true }), selected: false, installing: true },
    });
    expect(alreadyInstalled.find(".skill-card__spinner").exists()).toBe(false);
    expect(alreadyInstalled.find(".skill-card__installing").exists()).toBe(false);
  });

  it("disables selection for an already-installed skill without hiding its edit action", async () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill({ installed: true }), selected: false } });

    expect(wrapper.find(".skill-card__selection-surface").attributes("disabled")).toBeDefined();
    expect(wrapper.find(".skill-card__selection-surface").attributes("aria-label")).toContain("already installed");
    expect(wrapper.find(".skill-card__edit").attributes("disabled")).toBeUndefined();

    await wrapper.find(".skill-card__selection-surface").trigger("click");
    expect(wrapper.emitted("toggle")).toBeUndefined();
  });

  it("disables selection for disabled skills while keeping details and editing available", () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill({ enabled: false }), selected: false } });

    expect(wrapper.find(".skill-card__selection-surface").attributes("disabled")).toBeDefined();
    expect(wrapper.find(".skill-card__description").attributes("disabled")).toBeUndefined();
    expect(wrapper.find(".skill-card__edit").attributes("disabled")).toBeUndefined();
  });

  it("delete mode only allows selecting installed skills, and emits toggle-delete instead of toggle", async () => {
    const installed = mount(SkillCard, {
      props: { skill: makeSkill({ id: "triage", installed: true }), selected: false, deleteMode: true },
    });
    expect(installed.find(".skill-card__selection-surface").attributes("disabled")).toBeUndefined();
    await installed.find(".skill-card__selection-surface").trigger("click");
    expect(installed.emitted("toggle-delete")).toEqual([["triage"]]);
    expect(installed.emitted("toggle")).toBeUndefined();

    const notInstalled = mount(SkillCard, {
      props: { skill: makeSkill({ installed: false }), selected: false, deleteMode: true },
    });
    expect(notInstalled.find(".skill-card__selection-surface").attributes("disabled")).toBeDefined();
    expect(notInstalled.find(".skill-card__selection-surface").attributes("aria-label")).toContain(
      "nothing to uninstall",
    );
  });

  it("marks a delete-mode-selected card with is-delete-selected, never is-selected", () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ installed: true }), selected: false, deleteMode: true, deleteSelected: true },
    });
    expect(wrapper.classes()).toContain("is-delete-selected");
    expect(wrapper.classes()).not.toContain("is-selected");
  });

  it("always exposes the per-skill edit action", async () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill({ id: "triage" }), selected: false } });
    await wrapper.find(".skill-card__edit").trigger("click");
    expect(wrapper.emitted("edit")).toEqual([["triage"]]);
    expect(wrapper.find(".skill-card__edit").attributes("aria-label")).toBe("View and edit Issue Triage");
    expect(wrapper.find(".skill-card__edit circle").exists()).toBe(true);
  });

  it("renders assigned pack names and local status, with is-installed carrying the install state", () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ local: true, installed: true, tags: ["workflow"] }),
        selected: false,
        tags: [{ id: "workflow", name: "Workflow", color: "#E75480", order: 1, enabled: true }],
      },
    });

    expect(wrapper.get(".skill-card__source").classes()).toContain("source-icon--local");
    expect(wrapper.text()).toContain("Workflow");
    expect(wrapper.classes()).toContain("is-installed");
  });

  it("does not render raw pack ids when pack metadata is unavailable", () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ tags: ["secret-tag", "#E75480"] }), selected: false },
    });

    expect(wrapper.text()).not.toContain("secret-tag");
    expect(wrapper.attributes("style") ?? "").not.toContain("#E75480");
    expect(wrapper.find(".pack-badge").exists()).toBe(false);
    expect(wrapper.text()).not.toContain("No packs");
  });

  it("renders Pack markers as static compact badges with their configured color", () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ tags: ["workflow"] }),
        selected: false,
        tags: [{ id: "workflow", name: "Workflow", color: "#14B8A6", order: 1, enabled: true }],
      },
    });
    const badge = wrapper.get(".pack-badge");
    expect(badge.element.tagName).toBe("SPAN");
    expect(badge.classes()).toContain("pack-badge--compact");
    expect(badge.attributes("style")).toContain("--pack-color: #14B8A6");
  });

  it("shows the Update button only for an installed Local skill flagged with hasUpdate", () => {
    const notFlagged = mount(SkillCard, {
      props: { skill: makeSkill({ local: true, installed: true }), selected: false, hasUpdate: false },
    });
    expect(notFlagged.find(".skill-card__update").exists()).toBe(false);

    const remoteFlagged = mount(SkillCard, {
      props: { skill: makeSkill({ local: false, installed: true }), selected: false, hasUpdate: true },
    });
    expect(remoteFlagged.find(".skill-card__update").exists()).toBe(false);

    const notInstalledFlagged = mount(SkillCard, {
      props: { skill: makeSkill({ local: true, installed: false }), selected: false, hasUpdate: true },
    });
    expect(notInstalledFlagged.find(".skill-card__update").exists()).toBe(false);

    const flagged = mount(SkillCard, {
      props: { skill: makeSkill({ id: "triage", local: true, installed: true }), selected: false, hasUpdate: true },
    });
    expect(flagged.find(".skill-card__update").exists()).toBe(true);
  });

  it("emits update, not toggle, when the Update button is clicked", async () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ id: "triage", local: true, installed: true }), selected: false, hasUpdate: true },
    });

    await wrapper.find(".skill-card__update").trigger("click");

    expect(wrapper.emitted("update")).toEqual([["triage"]]);
    expect(wrapper.emitted("toggle")).toBeUndefined();
  });

  it("shows one icon per agent the skill is installed for, with no overlap", () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ installed: true, installedAgents: ["universal", "claude-code"] }),
        selected: false,
      },
    });

    const agents = wrapper.get(".skill-card__agents");
    const icons = agents.findAll(".skill-card__agent-icon");
    expect(icons).toHaveLength(2);
    expect(icons.map((icon) => icon.attributes("title"))).toEqual([
      "Installed for: Universal (.agents)",
      "Installed for Claude Code",
    ]);
  });

  it("shows every distinguishable installed agent without truncating", () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ installed: true, installedAgents: ["universal", "claude-code", "windsurf"] }),
        selected: false,
      },
    });

    expect(wrapper.get(".skill-card__agents").findAll(".skill-card__agent-icon")).toHaveLength(3);
  });

  it("shows no agent icons for a skill that isn't installed anywhere", () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ installed: false, installedAgents: [] }), selected: false },
    });
    expect(wrapper.find(".skill-card__agents").exists()).toBe(false);
  });

  it("stays selectable and shows a Missing-agents badge when installed for only some of the targeted agents", async () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ installed: true, installedAgents: ["windsurf"] }),
        selected: false,
        targetAgents: ["windsurf", "claude-code"],
      },
    });

    expect(wrapper.classes()).toContain("is-partially-installed");
    expect(wrapper.get(".skill-card__partial").text()).toBe("Missing 1 agent");
    expect(wrapper.find(".skill-card__selection-surface").attributes("disabled")).toBeUndefined();
    expect(wrapper.find(".skill-card__selection-surface").attributes("aria-label")).toContain(
      "not yet installed for Claude Code",
    );

    await wrapper.find(".skill-card__selection-surface").trigger("click");
    expect(wrapper.emitted("toggle")).toEqual([[wrapper.props("skill").id]]);
  });

  it("disables selection once installed for every targeted agent, even if installed for extra ones too", () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ installed: true, installedAgents: ["windsurf", "claude-code", "universal"] }),
        selected: false,
        targetAgents: ["windsurf"],
      },
    });

    expect(wrapper.classes()).not.toContain("is-partially-installed");
    expect(wrapper.find(".skill-card__partial").exists()).toBe(false);
    expect(wrapper.find(".skill-card__selection-surface").attributes("disabled")).toBeDefined();
  });
});
