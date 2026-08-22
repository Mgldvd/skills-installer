import * as backend from "../services/backend";
import { useAppState } from "./useAppState";

export function useTags() {
  const state = useAppState();
  const replaceSkill = (updated: Awaited<ReturnType<typeof backend.assignTagToSkill>>) => {
    state.skills = state.skills.map((skill) => (skill.id === updated.id ? updated : skill));
  };
  async function assign(skillId: string, tagId: string) {
    replaceSkill(await backend.assignTagToSkill(skillId, tagId));
  }
  async function unassign(skillId: string, tagId: string) {
    replaceSkill(await backend.unassignTagFromSkill(skillId, tagId));
  }
  async function create(name: string, color: string) {
    const tag = await backend.createTag(name, color);
    state.tags = [...state.tags, tag];
    return tag;
  }
  async function update(tagId: string, name: string, color: string) {
    const tag = await backend.updateTag(tagId, name, color);
    state.tags = state.tags.map((item) => (item.id === tag.id ? tag : item));
  }
  async function remove(tagId: string) {
    await backend.deleteTag(tagId);
    state.tags = state.tags.filter((tag) => tag.id !== tagId);
    state.skills = state.skills.map((skill) => ({ ...skill, tags: skill.tags.filter((id) => id !== tagId) }));
  }
  async function reorder(enabledTagIds: string[]) {
    const previous = state.tags;
    const enabled = new Set(enabledTagIds);
    const allIds = [
      ...enabledTagIds,
      ...previous
        .filter((tag) => !enabled.has(tag.id))
        .sort((a, b) => a.order - b.order)
        .map((tag) => tag.id),
    ];
    const order = new Map(allIds.map((id, index) => [id, (index + 1) * 10]));
    state.tags = previous.map((tag) => ({ ...tag, order: order.get(tag.id) ?? tag.order }));
    try {
      state.tags = await backend.reorderTags(allIds);
    } catch (error) {
      state.tags = previous;
      throw error;
    }
  }
  return { assign, unassign, create, update, remove, reorder };
}
