import * as backend from '../services/backend'
import { useAppState } from './useAppState'

export function useTags() {
  const state = useAppState()
  const replaceSkill = (updated: Awaited<ReturnType<typeof backend.assignTagToSkill>>) => {
    state.skills = state.skills.map((skill) => skill.id === updated.id ? updated : skill)
  }
  async function assign(skillId: string, tagId: string) { replaceSkill(await backend.assignTagToSkill(skillId, tagId)) }
  async function unassign(skillId: string, tagId: string) { replaceSkill(await backend.unassignTagFromSkill(skillId, tagId)) }
  async function create(name: string, color: string) {
    const tag = await backend.createTag(name, color)
    state.tags = [...state.tags, tag]
    return tag
  }
  async function update(tagId: string, name: string, color: string) {
    const tag = await backend.updateTag(tagId, name, color)
    state.tags = state.tags.map((item) => item.id === tag.id ? tag : item)
  }
  async function remove(tagId: string) {
    await backend.deleteTag(tagId)
    state.tags = state.tags.filter((tag) => tag.id !== tagId)
    state.skills = state.skills.map((skill) => ({ ...skill, tags: skill.tags.filter((id) => id !== tagId) }))
  }
  return { assign, unassign, create, update, remove }
}
