import { computed } from "vue";

import * as backend from "../services/backend";
import { useAppState } from "./useAppState";

/**
 * Owns skill/group data loading and every CRUD action. Filtering/search
 * live in `useFilters` instead, so this composable stays focused on "what
 * is the current data" rather than "what subset is currently visible."
 */
export function useSkills() {
  const state = useAppState();

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
   * used both on first load and by the toolbar's "Defaults" action. Already-
   * installed skills are never selectable (see toggleSelected), so they're
   * excluded here too even if preselected. */
  function restoreDefaultSelection() {
    const next = new Set<string>();
    for (const skill of state.skills) {
      if (skill.preselected && skill.enabled && !skill.installed) next.add(skill.id);
    }
    state.selectedSkillIds = next;
  }

  function pruneInstalledFromSelection() {
    if (!state.selectedSkillIds.size) return;
    const installedIds = new Set(state.skills.filter((s) => s.installed).map((s) => s.id));
    if (![...state.selectedSkillIds].some((id) => installedIds.has(id))) return;
    state.selectedSkillIds = new Set([...state.selectedSkillIds].filter((id) => !installedIds.has(id)));
  }

  function clearSelection() {
    state.selectedSkillIds = new Set();
  }

  function toggleSelected(skillId: string) {
    const alreadySelected = state.selectedSkillIds.has(skillId);
    // Already-installed skills can only be removed from selection, never
    // added — there's nothing left to install, so selecting one would just
    // let a batch install silently re-request something already done.
    const skill = state.skills.find((s) => s.id === skillId);
    if (!alreadySelected && skill?.installed) return;
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
  };
}
