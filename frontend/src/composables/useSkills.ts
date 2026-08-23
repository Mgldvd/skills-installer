import { computed } from "vue";

import * as backend from "../services/backend";
import type { Skill } from "../types";
import { agentsNeedingInstall } from "../utils/skillInstall";
import { useAppState } from "./useAppState";

/**
 * Owns skill/group data loading and every CRUD action. Filtering/search
 * live in `useFilters` instead, so this composable stays focused on "what
 * is the current data" rather than "what subset is currently visible."
 */
export function useSkills() {
  const state = useAppState();

  /** A skill can't be (re-)selected once it's installed for every agent
   * currently targeted for installation — there's nothing left to install.
   * If it's only installed for some of them, selecting it again just closes
   * the gap for the rest (see `agentsNeedingInstall`). */
  function isFullyInstalledForTargets(skill: Skill) {
    return agentsNeedingInstall(skill, state.preferences.defaultAgents).length === 0;
  }

  async function loadAll() {
    const config = await backend.getApplicationState();
    state.skills = config.skills;
    state.groups = config.groups;
    state.tags = config.tags ?? [];
    state.sourcePath = config.sourcePath;
    state.isEmbeddedDefault = config.isEmbeddedDefault;
    state.projectRoot = config.projectRoot;
    state.loaded = true;
    if (state.selectedSkillIds.size === 0) {
      restoreDefaultSelection();
    }
    pruneInstalledFromSelection();
  }

  async function refresh() {
    const config = await backend.refresh(state.projectRoot);
    state.skills = config.skills;
    state.groups = config.groups;
    state.tags = config.tags ?? [];
    // An installed skill can't be selected (see toggleSelected) — this is
    // what actually drops selection the moment an install finishes: every
    // caller that changes what's installed (a completed install, switching
    // the destination folder) already calls refresh() afterward.
    pruneInstalledFromSelection();
  }

  /** Resets selection to exactly the currently configured preselected set —
   * used both on first load and by the toolbar's "Defaults" action. A skill
   * already installed for every currently targeted agent is never
   * selectable (see toggleSelected), so it's excluded here too even if
   * preselected. */
  function restoreDefaultSelection() {
    const next = new Set<string>();
    for (const skill of state.skills) {
      if (skill.preselected && skill.enabled && !isFullyInstalledForTargets(skill)) next.add(skill.id);
    }
    state.selectedSkillIds = next;
  }

  function pruneInstalledFromSelection() {
    if (!state.selectedSkillIds.size) return;
    const doneIds = new Set(
      state.skills.filter((s) => isFullyInstalledForTargets(s)).map((s) => s.id),
    );
    if (![...state.selectedSkillIds].some((id) => doneIds.has(id))) return;
    state.selectedSkillIds = new Set([...state.selectedSkillIds].filter((id) => !doneIds.has(id)));
  }

  function clearSelection() {
    state.selectedSkillIds = new Set();
  }

  function toggleSelected(skillId: string) {
    const alreadySelected = state.selectedSkillIds.has(skillId);
    // A skill fully installed for every targeted agent can only be removed
    // from selection, never added — there's nothing left to install, so
    // selecting it would just let a batch install silently re-request
    // something already done. One only installed for *some* targeted
    // agents stays selectable, so re-installing can close the gap.
    const skill = state.skills.find((s) => s.id === skillId);
    if (!alreadySelected && skill && isFullyInstalledForTargets(skill)) return;
    const next = new Set(state.selectedSkillIds);
    if (alreadySelected) next.delete(skillId);
    else next.add(skillId);
    state.selectedSkillIds = next;
  }

  const selectedSkills = computed(() => state.skills.filter((s) => state.selectedSkillIds.has(s.id)));
  const installedSkills = computed(() => state.skills.filter((s) => s.installed));

  async function addSkill(args: backend.AddSkillArgs) {
    const skill = await backend.addSkill(args);
    state.skills = [...state.skills, skill];
    return skill;
  }

  async function updateSkill(args: backend.UpdateSkillArgs) {
    const updated = await backend.updateSkill(args);
    state.skills = state.skills.map((s) => (s.id === updated.id ? updated : s));
    return updated;
  }

  /** Local-only: explicit, on-demand check — populates `skillsWithUpdates`
   * from the backend's `.signature` comparison. Never runs automatically
   * (see `AppState.skillsWithUpdates`); the GUI's "Check for Updates"
   * button is the only caller. */
  async function checkForUpdates() {
    const outdated = await backend.checkLocalSkillUpdates(state.projectRoot);
    state.skillsWithUpdates = new Set(outdated);
    return outdated;
  }

  async function deleteSkill(skillId: string) {
    await backend.deleteSkill(skillId);
    state.skills = state.skills.filter((s) => s.id !== skillId);
    if (state.selectedSkillIds.has(skillId)) {
      const next = new Set(state.selectedSkillIds);
      next.delete(skillId);
      state.selectedSkillIds = next;
    }
  }

  return {
    state,
    selectedSkills,
    installedSkills,
    loadAll,
    refresh,
    restoreDefaultSelection,
    clearSelection,
    toggleSelected,
    addSkill,
    updateSkill,
    deleteSkill,
    checkForUpdates,
  };
}
