import { computed } from 'vue'

import * as backend from '../services/backend'
import { useAppState } from './useAppState'

/**
 * Owns skill/group data loading and every CRUD action. Filtering/search
 * live in `useFilters` instead, so this composable stays focused on "what
 * is the current data" rather than "what subset is currently visible."
 */
export function useSkills() {
  const state = useAppState()

  async function loadAll() {
    const config = await backend.getApplicationState()
    state.skills = config.skills
    state.groups = config.groups
    state.tags = config.tags ?? []
    state.sourcePath = config.sourcePath
    state.isEmbeddedDefault = config.isEmbeddedDefault
    state.projectRoot = config.projectRoot
    state.loaded = true
    if (state.selectedSkillIds.size === 0) {
      restoreDefaultSelection()
    }
  }

  async function refresh() {
    const config = await backend.refresh()
    state.skills = config.skills
    state.groups = config.groups
    state.tags = config.tags ?? []
  }

  /** Resets selection to exactly the currently configured preselected set —
   * used both on first load and by the toolbar's "Defaults" action. */
  function restoreDefaultSelection() {
    const next = new Set<string>()
    for (const skill of state.skills) {
      if (skill.preselected && skill.enabled) next.add(skill.id)
    }
    state.selectedSkillIds = next
  }

  function clearSelection() {
    state.selectedSkillIds = new Set()
  }

  function toggleSelected(skillId: string) {
    const next = new Set(state.selectedSkillIds)
    if (next.has(skillId)) next.delete(skillId)
    else next.add(skillId)
    state.selectedSkillIds = next
  }

  const selectedSkills = computed(() => state.skills.filter((s) => state.selectedSkillIds.has(s.id)))
  const installedSkills = computed(() => state.skills.filter((s) => s.installed))

  async function addSkill(args: backend.AddSkillArgs) {
    const skill = await backend.addSkill(args)
    state.skills = [...state.skills, skill]
    return skill
  }

  async function updateSkill(args: backend.UpdateSkillArgs) {
    const updated = await backend.updateSkill(args)
    state.skills = state.skills.map((s) => (s.id === updated.id ? updated : s))
    return updated
  }

  async function deleteSkill(skillId: string) {
    await backend.deleteSkill(skillId)
    state.skills = state.skills.filter((s) => s.id !== skillId)
    if (state.selectedSkillIds.has(skillId)) {
      const next = new Set(state.selectedSkillIds)
      next.delete(skillId)
      state.selectedSkillIds = next
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
  }
}
