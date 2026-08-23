import { flushPromises, mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../src/services/backend", () => ({
  getApplicationState: vi.fn(),
  refresh: vi.fn(),
  addSkill: vi.fn(),
  updateSkill: vi.fn(),
  deleteSkill: vi.fn(),
  createGroup: vi.fn(),
  updateGroup: vi.fn(),
  deleteGroup: vi.fn(),
  getPreferences: vi.fn(),
  updatePreferences: vi.fn(),
  getDependencyStatus: vi.fn(),
  installSkills: vi.fn(),
  cancelInstallation: vi.fn(),
  previewSkillUrl: vi.fn(),
  getProjects: vi.fn(),
  saveProject: vi.fn(),
  deleteProject: vi.fn(),
}));

import App from "../src/App.vue";
import AppHeader from "../src/components/AppHeader/AppHeader.vue";
import { useAppState } from "../src/composables/useAppState";
import * as backend from "../src/services/backend";
import { defaultPreferences } from "../src/types";
import { makeGroup, makeSkill } from "./fixtures";
import { resetState } from "./test-utils";

function mockLoadedApp() {
  vi.mocked(backend.getApplicationState).mockResolvedValue({
    version: 1,
    defaults: { agent: null, copy: true, scope: "project" },
    groups: [makeGroup({ id: "other", name: "Other" }), makeGroup({ id: "testing", name: "Testing" })],
    skills: [
      makeSkill({ id: "a", displayName: "Alpha", groupId: "testing", preselected: true }),
      makeSkill({ id: "b", displayName: "Beta", groupId: "other", preselected: false }),
    ],
    sourcePath: null,
    isEmbeddedDefault: true,
    projectRoot: "/home/user/project",
  });
  vi.mocked(backend.getPreferences).mockResolvedValue(defaultPreferences());
  vi.mocked(backend.getDependencyStatus).mockResolvedValue({
    available: true,
    source: "installedExecutable",
    executablePath: "/usr/bin/skills",
    version: "1.0.0",
    detail: null,
  });
  vi.mocked(backend.getProjects).mockResolvedValue([]);
}

describe("App", () => {
  const state = useAppState();

  beforeEach(() => {
    resetState(state);
    vi.clearAllMocks();
  });

  it("selects exactly the preselected skills on load", async () => {
    mockLoadedApp();
    mount(App);
    await flushPromises();

    expect(state.selectedSkillIds).toEqual(new Set(["a"]));
  });

  it("keeps selected cards first, ordered by Pack and then alphabetically", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();
    state.tags = [
      { id: "frontend", name: "Frontend", color: "#3B82F6", order: 10, enabled: true },
      { id: "testing", name: "Testing", color: "#22C55E", order: 20, enabled: true },
    ];
    state.skills = [
      makeSkill({ id: "z", displayName: "Zulu", tags: ["testing"] }),
      makeSkill({ id: "b", displayName: "Beta", tags: ["frontend"] }),
      makeSkill({ id: "a", displayName: "Alpha", tags: ["frontend"] }),
      makeSkill({ id: "u", displayName: "Unselected", tags: ["frontend"] }),
    ];
    state.selectedSkillIds = new Set(["z", "b", "a"]);
    await nextTick();

    expect(wrapper.findAll(".skill-card__title").map((title) => title.text())).toEqual([
      "Alpha",
      "Beta",
      "Zulu",
      "Unselected",
    ]);
  });

  it("Pack toggle only selects the not-yet-installed skills in that Pack", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();
    state.tags = [{ id: "frontend", name: "Frontend", color: "#3B82F6", order: 10, enabled: true }];
    state.skills = [
      makeSkill({ id: "a", displayName: "Alpha", tags: ["frontend"], installed: false }),
      makeSkill({ id: "b", displayName: "Beta", tags: ["frontend"], installed: true, installedAgents: ["universal"] }),
    ];
    state.selectedSkillIds = new Set();
    await nextTick();

    const packBadge = wrapper.findAll(".skill-toolbar__tag").find((el) => el.text().includes("Frontend"))!;
    await packBadge.trigger("click");

    expect(state.selectedSkillIds).toEqual(new Set(["a"]));
  });

  it("Needs agents only counts Skills already installed but missing one of the targeted agents, not fresh installs", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();
    // Default target agent is ["universal"] (see defaultPreferences).
    state.skills = [
      makeSkill({ id: "done", displayName: "Fully Done", installed: true, installedAgents: ["universal"] }),
      makeSkill({ id: "gap", displayName: "Needs Universal", installed: true, installedAgents: ["claude-code"] }),
      makeSkill({ id: "fresh", displayName: "Never Installed", installed: false }),
    ];
    state.selectedSkillIds = new Set();
    await nextTick();

    const needsAgentsButton = wrapper
      .findAll(".skill-filter-bar__source button")
      .find((b) => b.text().startsWith("Needs agents"))!;
    expect(needsAgentsButton.text()).toBe("Needs agents (1)");
    await needsAgentsButton.trigger("click");

    expect(wrapper.findAll(".skill-card__title").map((t) => t.text())).toEqual(["Needs Universal"]);
  });

  it("Select missing lives in the selection toolbar and acts on the whole catalog, independent of the active filter", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();
    state.skills = [
      makeSkill({ id: "done", displayName: "Fully Done", installed: true, installedAgents: ["universal"] }),
      makeSkill({ id: "gap", displayName: "Needs Universal", installed: true, installedAgents: ["claude-code"] }),
      makeSkill({ id: "fresh", displayName: "Never Installed", installed: false }),
    ];
    state.selectedSkillIds = new Set();
    await nextTick();
    // A search query that hides "gap" from the grid entirely.
    await wrapper.get('input[type="search"]').setValue("Fully Done");
    await nextTick();
    expect(wrapper.findAll(".skill-card__title").map((t) => t.text())).toEqual(["Fully Done"]);

    const selectMissing = wrapper.get(".skill-toolbar__select-missing");
    expect(selectMissing.text()).toBe("Select missing (1)");
    await selectMissing.trigger("click");

    expect(state.selectedSkillIds).toEqual(new Set(["gap"]));
  });

  it("Sort by Local/Remote groups skills by source before falling back to name", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();
    state.skills = [
      makeSkill({ id: "z", displayName: "Zulu", local: false }),
      makeSkill({ id: "b", displayName: "Beta", local: true }),
      makeSkill({ id: "a", displayName: "Alpha", local: false }),
      makeSkill({ id: "u", displayName: "Unrelated", local: true }),
    ];
    state.selectedSkillIds = new Set();
    await nextTick();

    const sortSelect = wrapper.get('select[aria-label="Sort Skills"]');

    await sortSelect.setValue("local");
    expect(wrapper.findAll(".skill-card__title").map((t) => t.text())).toEqual([
      "Beta",
      "Unrelated",
      "Alpha",
      "Zulu",
    ]);

    await sortSelect.setValue("remote");
    expect(wrapper.findAll(".skill-card__title").map((t) => t.text())).toEqual([
      "Alpha",
      "Zulu",
      "Beta",
      "Unrelated",
    ]);
  });

  it("switches the grid to list view from the toolbar toggle", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    expect(wrapper.find(".skill-grid").classes()).not.toContain("skill-grid--list");
    expect(wrapper.find(".skill-card").classes()).not.toContain("is-list");

    await wrapper.get('[aria-label="List view"]').trigger("click");

    expect(wrapper.find(".skill-grid").classes()).toContain("skill-grid--list");
    expect(wrapper.find(".skill-card").classes()).toContain("is-list");
  });

  it("Compact grid view saves the compactCards preference and List view clears it again", async () => {
    mockLoadedApp();
    vi.mocked(backend.updatePreferences).mockImplementation(async (preferences) => preferences);
    const wrapper = mount(App);
    await flushPromises();

    await wrapper.get('[aria-label="Compact grid view"]').trigger("click");
    await flushPromises();

    expect(backend.updatePreferences).toHaveBeenCalledWith(expect.objectContaining({ compactCards: true }));
    expect(wrapper.get('[aria-label="Compact grid view"]').attributes("aria-pressed")).toBe("true");
    expect(wrapper.get('[aria-label="Grid view"]').attributes("aria-pressed")).toBe("false");
    expect(wrapper.find(".skill-grid").classes()).toContain("skill-grid--compact");
    expect(wrapper.find(".skill-grid").classes()).not.toContain("skill-grid--list");

    await wrapper.get('[aria-label="List view"]').trigger("click");
    await flushPromises();

    expect(backend.updatePreferences).toHaveBeenLastCalledWith(expect.objectContaining({ compactCards: false }));
    expect(wrapper.find(".skill-grid").classes()).toContain("skill-grid--list");
    expect(wrapper.find(".skill-grid").classes()).not.toContain("skill-grid--compact");
  });

  it("leads with selected cards, then installed-but-unselected ones, then the rest", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();
    state.skills = [
      makeSkill({ id: "z", displayName: "Zulu" }),
      makeSkill({ id: "b", displayName: "Beta", installed: true }),
      makeSkill({ id: "a", displayName: "Alpha" }),
      makeSkill({ id: "u", displayName: "Unrelated" }),
    ];
    state.selectedSkillIds = new Set(["a"]);
    await nextTick();

    expect(wrapper.findAll(".skill-card__title").map((title) => title.text())).toEqual([
      "Alpha",
      "Beta",
      "Unrelated",
      "Zulu",
    ]);
  });

  it("moves a newly selected card into the leading selected block", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();
    state.skills = state.skills.map((skill) => (skill.id === "b" ? { ...skill, displayName: "Aardvark" } : skill));
    await nextTick();
    const cardsBefore = wrapper.findAll(".skill-card__title").map((title) => title.text());
    expect(cardsBefore).toEqual(["Alpha", "Aardvark"]);

    await wrapper.findAll(".skill-card__selection-surface")[1].trigger("click");
    await nextTick();
    expect(wrapper.findAll(".skill-card__title").map((title) => title.text())).toEqual(["Aardvark", "Alpha"]);
    expect(wrapper.findAll(".skill-card")[0].classes()).toContain("is-selected");
  });

  it("opens Add Skill directly from the green footer action", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    const addButton = wrapper.findAll(".app-shell__footer-btn").find((button) => button.text() === "Add Skill")!;
    expect(addButton.classes()).toContain("app-shell__footer-btn--add");
    await addButton.trigger("click");
    await flushPromises();
    expect(wrapper.find(".add-skill-dialog").attributes("open")).toBeDefined();
  });

  it("bulk-deletes selected Skills.sh catalog Skills sequentially, one backend call per Skill", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    mockLoadedApp();
    vi.mocked(backend.deleteSkill).mockResolvedValue(undefined);
    const wrapper = mount(App);
    await flushPromises();

    const addButton = wrapper.findAll(".app-shell__footer-btn").find((button) => button.text() === "Add Skill")!;
    await addButton.trigger("click");
    await flushPromises();

    await wrapper.get('[aria-label="Select all Skills from Skills.sh"]').setValue(true);
    await wrapper.get(".add-skill-dialog__bulk-delete").trigger("click");
    await flushPromises();

    expect(backend.deleteSkill).toHaveBeenCalledTimes(2);
    expect(backend.deleteSkill).toHaveBeenNthCalledWith(1, "a");
    expect(backend.deleteSkill).toHaveBeenNthCalledWith(2, "b");
    expect(wrapper.find(".skill-card").exists()).toBe(false);

    confirmSpy.mockRestore();
  });

  it("edits a catalog Skill from the Add Skill table, closing Add Skill behind it", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    const addButton = wrapper.findAll(".app-shell__footer-btn").find((button) => button.text() === "Add Skill")!;
    await addButton.trigger("click");
    await flushPromises();

    await wrapper.get(".add-skill-dialog__row-edit").trigger("click");
    await flushPromises();

    expect((wrapper.find(".add-skill-dialog").element as HTMLDialogElement).open).toBe(false);
    expect((wrapper.find(".edit-skill-dialog").element as HTMLDialogElement).open).toBe(true);
  });

  it("returns to Add Skill after cancelling an edit that was opened from its catalog table", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    const addButton = wrapper.findAll(".app-shell__footer-btn").find((button) => button.text() === "Add Skill")!;
    await addButton.trigger("click");
    await wrapper.get(".add-skill-dialog__row-edit").trigger("click");
    await flushPromises();

    const cancelButton = wrapper.findAll(".edit-skill-dialog__btn").find((b) => b.text() === "Cancel")!;
    await cancelButton.trigger("click");
    await flushPromises();

    expect((wrapper.find(".edit-skill-dialog").element as HTMLDialogElement).open).toBe(false);
    expect((wrapper.find(".add-skill-dialog").element as HTMLDialogElement).open).toBe(true);
  });

  it("does not return to Add Skill after saving an edit that was opened from its catalog table", async () => {
    mockLoadedApp();
    vi.mocked(backend.updateSkill).mockImplementation(async (args) => ({
      ...makeSkill({ id: args.skillId, displayName: args.displayName }),
    }));
    const wrapper = mount(App);
    await flushPromises();

    const addButton = wrapper.findAll(".app-shell__footer-btn").find((button) => button.text() === "Add Skill")!;
    await addButton.trigger("click");
    await wrapper.get(".add-skill-dialog__row-edit").trigger("click");
    await flushPromises();

    await wrapper.get(".edit-skill-dialog__form").trigger("submit");
    await flushPromises();

    expect((wrapper.find(".edit-skill-dialog").element as HTMLDialogElement).open).toBe(false);
    expect((wrapper.find(".add-skill-dialog").element as HTMLDialogElement).open).toBe(false);
  });

  it("does not reopen Add Skill after cancelling an edit opened from a Skill card", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    await wrapper.get(".skill-card__edit").trigger("click");
    await flushPromises();
    expect((wrapper.find(".edit-skill-dialog").element as HTMLDialogElement).open).toBe(true);

    const cancelButton = wrapper.findAll(".edit-skill-dialog__btn").find((b) => b.text() === "Cancel")!;
    await cancelButton.trigger("click");
    await flushPromises();

    expect((wrapper.find(".edit-skill-dialog").element as HTMLDialogElement).open).toBe(false);
    expect((wrapper.find(".add-skill-dialog").element as HTMLDialogElement).open).toBe(false);
  });

  it("selecting a new destination folder re-scans Installed status for that folder", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();
    expect(state.selectedSkillIds).toEqual(new Set(["a"]));

    // The just-selected folder has nothing installed in it — the catalog
    // must reflect that, not whatever was installed at the app's launch dir.
    vi.mocked(backend.refresh).mockResolvedValue({
      version: 1,
      defaults: { agent: null, copy: true, scope: "project" },
      groups: [makeGroup({ id: "other", name: "Other" })],
      skills: [makeSkill({ id: "a", displayName: "Alpha", groupId: "other", installed: false })],
      tags: [],
      sourcePath: null,
      isEmbeddedDefault: true,
      projectRoot: "/home/user/empty-folder",
    });

    wrapper.findComponent(AppHeader).vm.$emit("update:projectPath", "/home/user/empty-folder");
    await flushPromises();

    expect(backend.refresh).toHaveBeenCalledWith("/home/user/empty-folder");
    expect(state.projectRoot).toBe("/home/user/empty-folder");
    expect(state.skills.find((s) => s.id === "a")?.installed).toBe(false);
  });

  it('"Clear" empties the current selection', async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    const clearButton = wrapper.get(".skill-toolbar__clear");
    await clearButton.trigger("click");

    expect(state.selectedSkillIds.size).toBe(0);
    expect(wrapper.findAll(".app-shell__footer-btn").some((button) => button.text() === "Clear")).toBe(false);
  });

  it("clicking Install Selected opens the install confirmation dialog when confirmBeforeInstall is enabled", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    const installButton = wrapper.findAll(".app-shell__footer-btn").find((b) => b.text() === "Install Selected");
    expect(wrapper.get(".app-shell__footer-actions .app-shell__selected-count").text()).toBe("1 selected");
    await installButton!.trigger("click");
    await flushPromises();

    const dialog = wrapper.find(".install-confirm-dialog");
    expect((dialog.element as HTMLDialogElement).open).toBe(true);
  });

  it("Skip confirmation toggles confirmBeforeInstall and installing then runs without opening the dialog", async () => {
    mockLoadedApp();
    vi.mocked(backend.updatePreferences).mockImplementation(async (preferences) => preferences);
    vi.mocked(backend.installSkills).mockResolvedValue({
      requested: 1,
      installed: 1,
      alreadyInstalled: 0,
      failed: 0,
      cancelled: false,
      perSkill: [],
    });
    const wrapper = mount(App);
    await flushPromises();

    const skipToggle = wrapper.get(".app-shell__toggle-input");
    expect((skipToggle.element as HTMLInputElement).checked).toBe(false);
    await skipToggle.setValue(true);
    await flushPromises();
    expect((skipToggle.element as HTMLInputElement).checked).toBe(true);
    expect(state.preferences.confirmBeforeInstall).toBe(false);

    const installButton = wrapper.findAll(".app-shell__footer-btn").find((b) => b.text() === "Install Selected");
    await installButton!.trigger("click");
    await flushPromises();

    expect(backend.installSkills).toHaveBeenCalled();
    const dialog = wrapper.find(".install-confirm-dialog");
    expect((dialog.element as HTMLDialogElement).open).toBe(false);
  });

  it("excluding an agent chip in the confirm dialog only trims that one install, not the configured defaults", async () => {
    mockLoadedApp();
    vi.mocked(backend.getPreferences).mockResolvedValue({
      ...defaultPreferences(),
      defaultAgents: ["universal", "claude-code"],
    });
    vi.mocked(backend.installSkills).mockResolvedValue({
      requested: 1,
      installed: 1,
      alreadyInstalled: 0,
      failed: 0,
      cancelled: false,
      perSkill: [],
    });
    const wrapper = mount(App);
    await flushPromises();

    const installButton = wrapper.findAll(".app-shell__footer-btn").find((b) => b.text() === "Install Selected");
    await installButton!.trigger("click");
    await flushPromises();

    const claudeChip = wrapper
      .findAll(".install-confirm-dialog__agent-chip")
      .find((c) => c.text().includes("Claude Code"))!;
    await claudeChip.trigger("click");
    await wrapper.get(".install-confirm-dialog__btn--primary").trigger("click");
    await flushPromises();

    expect(backend.installSkills).toHaveBeenCalledWith(
      expect.objectContaining({ options: expect.objectContaining({ agents: ["universal"] }) }),
      expect.anything(),
    );
    expect(state.preferences.defaultAgents).toEqual(["universal", "claude-code"]);
  });

  it("shows the spinner only on the card currently being installed", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    state.installation.isInstalling = true;
    state.installation.currentSkillId = "b";
    await nextTick();

    const cards = wrapper.findAll(".skill-card");
    const alphaCard = cards.find((card) => card.find(".skill-card__title").text() === "Alpha")!;
    const betaCard = cards.find((card) => card.find(".skill-card__title").text() === "Beta")!;
    expect(betaCard.classes()).toContain("is-installing");
    expect(betaCard.find(".skill-card__spinner").exists()).toBe(true);
    expect(alphaCard.classes()).not.toContain("is-installing");
    expect(alphaCard.find(".skill-card__spinner").exists()).toBe(false);
  });

  it("shows the Skills CLI status and the Agents icon summary together as the footer's status row", async () => {
    mockLoadedApp();
    vi.mocked(backend.getPreferences).mockResolvedValue({
      ...defaultPreferences(),
      defaultAgents: ["universal", "claude-code"],
    });
    const wrapper = mount(App);
    await flushPromises();

    const status = wrapper.get(".app-shell__footer-status");
    expect(status.text()).toContain("Skills CLI");
    expect(status.text()).toContain("Ready");
    expect(status.get(".app-shell__dependency").classes()).toContain("is-ready");

    const agentsSummary = status.get(".app-shell__agents-summary");
    expect(agentsSummary.findAll(".agent-icon")).toHaveLength(2);
    expect(agentsSummary.attributes("title")).toBe("Universal (.agents), Claude Code");

    // Neither the header nor the footer's action row own this anymore.
    expect(wrapper.find(".app-header").text()).not.toContain("Skills CLI");
    expect(wrapper.find(".app-shell__footer-row").text()).not.toContain("Universal");
  });

  it("keeps the header down to just the brand and Install to, on the right", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    const items = wrapper.findAll(".app-header > *");
    const classNames = items.map(
      (item) => item.classes().find((c) => c !== "app-header__item" && c.startsWith("app-header__")),
    );
    expect(classNames).toEqual(["app-header__brand", "app-header__destination"]);
  });

  it("opens the Agents dialog by clicking anywhere on the footer's icon summary", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    await wrapper.get(".app-shell__agents-summary").trigger("click");
    expect((wrapper.find(".agents-dialog").element as HTMLDialogElement).open).toBe(true);
  });

  it("does not show a separate 'Agents' text button in the footer's action row anymore", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    expect(wrapper.findAll(".app-shell__footer-btn").some((b) => b.text() === "Agents")).toBe(false);
  });

  it("opens Packs and Agents from Preferences' Organize section, closing Preferences behind it", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    const preferencesButton = wrapper.findAll(".app-shell__footer-btn").find((b) => b.text() === "Preferences")!;
    await preferencesButton.trigger("click");
    expect((wrapper.find(".preferences-dialog").element as HTMLDialogElement).open).toBe(true);

    const packsButton = wrapper.findAll(".preferences-dialog button").find((b) => b.text() === "Manage Packs")!;
    await packsButton.trigger("click");
    expect((wrapper.find(".preferences-dialog").element as HTMLDialogElement).open).toBe(false);
    expect((wrapper.find(".tags-dialog").element as HTMLDialogElement).open).toBe(true);

    await preferencesButton.trigger("click");
    const agentsButton = wrapper.findAll(".preferences-dialog button").find((b) => b.text() === "Manage Agents")!;
    await agentsButton.trigger("click");
    expect((wrapper.find(".preferences-dialog").element as HTMLDialogElement).open).toBe(false);
    expect((wrapper.find(".agents-dialog").element as HTMLDialogElement).open).toBe(true);
  });

  it("manages Packs from the toolbar's + button, not a separate footer button", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    expect(wrapper.findAll(".app-shell__footer-btn").some((b) => b.text() === "Packs")).toBe(false);

    await wrapper.get(".skill-toolbar__manage-packs").trigger("click");
    expect((wrapper.find(".tags-dialog").element as HTMLDialogElement).open).toBe(true);
  });

  it("surfaces a load error as a toast rather than a silent failure", async () => {
    vi.mocked(backend.getApplicationState).mockRejectedValue(new Error("disk on fire"));
    vi.mocked(backend.getPreferences).mockResolvedValue(defaultPreferences());
    vi.mocked(backend.getProjects).mockResolvedValue([]);
    vi.mocked(backend.getDependencyStatus).mockResolvedValue({
      available: false,
      source: "unavailable",
      executablePath: null,
      version: null,
      detail: "not found",
    });

    const wrapper = mount(App);
    await flushPromises();

    expect(wrapper.text()).toContain("disk on fire");
  });

  it("Save Project captures the skillName of every currently installed Skill, not the checkbox selection", async () => {
    mockLoadedApp();
    vi.mocked(backend.getApplicationState).mockResolvedValue({
      version: 1,
      defaults: { agent: null, copy: true, scope: "project" },
      groups: [makeGroup({ id: "other", name: "Other" })],
      skills: [
        makeSkill({ id: "a", skillName: "triage", installed: true, installedAgents: ["universal"] }),
        makeSkill({ id: "b", skillName: "docs", installed: false }),
      ],
      sourcePath: null,
      isEmbeddedDefault: true,
      projectRoot: "/home/user/project",
    });
    const saved = { id: "atlas", name: "Atlas", gitUrl: "https://github.com/me/atlas", skillNames: ["triage"] };
    vi.mocked(backend.saveProject).mockResolvedValue(saved);
    const wrapper = mount(App);
    await flushPromises();
    state.selectedSkillIds = new Set(["b"]); // deliberately not what gets saved

    const projectsButton = wrapper.findAll(".app-shell__footer-btn").find((b) => b.text() === "Projects")!;
    await projectsButton.trigger("click");
    await wrapper.get(".projects-dialog__form input[type=text]").setValue("Atlas");
    await wrapper.get(".projects-dialog__form input[type=url]").setValue("https://github.com/me/atlas");
    await wrapper.get(".projects-dialog__form").trigger("submit");
    await flushPromises();

    expect(backend.saveProject).toHaveBeenCalledWith({
      name: "Atlas",
      gitUrl: "https://github.com/me/atlas",
      skillNames: ["triage"],
    });
    expect(state.projects).toContainEqual(saved);
  });

  it("Load Project replaces the selection with matching Skills and skips any not found here", async () => {
    mockLoadedApp();
    vi.mocked(backend.getProjects).mockResolvedValue([
      { id: "atlas", name: "Atlas", gitUrl: null, skillNames: ["alpha-skill", "ghost-skill"] },
    ]);
    const wrapper = mount(App);
    await flushPromises();
    state.skills = [makeSkill({ id: "a", skillName: "alpha-skill" }), makeSkill({ id: "b", skillName: "beta-skill" })];
    state.selectedSkillIds = new Set(["b"]);
    await nextTick();

    const projectsButton = wrapper.findAll(".app-shell__footer-btn").find((b) => b.text() === "Projects")!;
    await projectsButton.trigger("click");
    await wrapper.get(".projects-dialog__row .button").trigger("click");

    expect(state.selectedSkillIds).toEqual(new Set(["a"]));
    expect((wrapper.get(".projects-dialog").element as HTMLDialogElement).open).toBe(false);
  });

  it("Delete Project removes it from the saved list via the backend", async () => {
    mockLoadedApp();
    vi.mocked(backend.getProjects).mockResolvedValue([{ id: "atlas", name: "Atlas", gitUrl: null, skillNames: ["a"] }]);
    vi.mocked(backend.deleteProject).mockResolvedValue(undefined);
    vi.spyOn(window, "confirm").mockReturnValue(true);
    const wrapper = mount(App);
    await flushPromises();

    const projectsButton = wrapper.findAll(".app-shell__footer-btn").find((b) => b.text() === "Projects")!;
    await projectsButton.trigger("click");
    await wrapper.get(".projects-dialog__row .projects-dialog__delete").trigger("click");
    await flushPromises();

    expect(backend.deleteProject).toHaveBeenCalledWith("atlas");
    expect(state.projects).toEqual([]);
  });
});
