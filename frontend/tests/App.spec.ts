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
      makeSkill({ id: "b", displayName: "Beta", tags: ["frontend"], installed: true }),
    ];
    state.selectedSkillIds = new Set();
    await nextTick();

    const packBadge = wrapper.findAll(".skill-toolbar__tag").find((el) => el.text().includes("Frontend"))!;
    await packBadge.trigger("click");

    expect(state.selectedSkillIds).toEqual(new Set(["a"]));
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

  it("renders a non-color-only Skills CLI status indicator reflecting dependency availability", async () => {
    mockLoadedApp();
    const wrapper = mount(App);
    await flushPromises();

    expect(wrapper.text()).toContain("Skills CLI");
    expect(wrapper.text()).toContain("Ready");
  });

  it("surfaces a load error as a toast rather than a silent failure", async () => {
    vi.mocked(backend.getApplicationState).mockRejectedValue(new Error("disk on fire"));
    vi.mocked(backend.getPreferences).mockResolvedValue(defaultPreferences());
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
});
