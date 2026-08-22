<template>
  <section class="skill-toolbar" aria-label="Skill preselection Packs">
    <span class="skill-toolbar__label">Preselect</span>
    <div v-if="orderedTags.length" class="skill-toolbar__tags">
      <div
        v-for="tag in orderedTags"
        :key="tag.id"
        class="skill-toolbar__tag-slot"
        :class="{
          'is-dragging': draggedTagId === tag.id,
          'is-drag-over': dragOverTagId === tag.id,
          'is-insert-before': insertionSide(tag.id) === 'before',
          'is-insert-after': insertionSide(tag.id) === 'after',
        }"
        draggable="true"
        @dragstart="startDrag($event, tag.id)"
        @dragover.prevent="dragOverTagId = tag.id"
        @drop.prevent="dropTag(tag.id)"
        @dragend="endDrag">
        <PackBadge
          class="skill-toolbar__tag"
          :name="tag.name"
          :color="tag.color"
          interactive
          :selected="tagState(tag.id) === 'all'"
          :partial="tagState(tag.id) === 'some'"
          :disabled="tagSkillCount(tag.id) === 0"
          :title="`${tagTitle(tag.id, tag.name)}. Drag to reorder; Alt+Left or Alt+Right also moves it.`"
          @click="emit('toggleTag', tag.id)"
          @keydown.alt.left.prevent="moveTag(tag.id, -1)"
          @keydown.alt.right.prevent="moveTag(tag.id, 1)">
          <template #trailing>
            <span class="skill-toolbar__count" aria-hidden="true">{{ tagSkillCount(tag.id) }}</span>
          </template>
        </PackBadge>
      </div>
    </div>
    <p v-else class="skill-toolbar__empty">Create Packs from the Packs menu to build reusable selections.</p>
    <button
      type="button"
      class="skill-toolbar__clear"
      :disabled="selectedIds.length === 0"
      aria-label="Clear all selected Skills"
      @click="emit('clearSelection')">
      Clear
    </button>
    <div class="skill-toolbar__controls">
      <label class="skill-toolbar__search">
        <span class="sr-only">Filter Skills</span>
        <svg viewBox="0 0 20 20" aria-hidden="true">
          <circle cx="8.5" cy="8.5" r="5.5" />
          <path d="m13 13 4 4" />
        </svg>
        <input
          type="search"
          :value="query"
          placeholder="Filter skills..."
          @input="emit('update:query', ($event.target as HTMLInputElement).value)" />
      </label>
      <label class="skill-toolbar__sort">
        <span>Sort by</span>
        <select
          :value="sortBy"
          aria-label="Sort Skills"
          @change="emit('update:sortBy', ($event.target as HTMLSelectElement).value as SortMode)">
          <option value="name">Name</option>
          <option value="pack">Pack</option>
          <option value="local">Local</option>
          <option value="remote">Remote</option>
        </select>
      </label>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import type { Skill, SkillTag } from "../../types";
import PackBadge from "../PackBadge/PackBadge.vue";

type SortMode = "name" | "pack" | "local" | "remote";

const props = withDefaults(
  defineProps<{
    tags?: SkillTag[];
    skills?: Skill[];
    selectedIds?: string[];
    query?: string;
    sortBy?: SortMode;
  }>(),
  {
    tags: () => [],
    skills: () => [],
    selectedIds: () => [],
    query: "",
    sortBy: "name",
  },
);

const emit = defineEmits<{
  toggleTag: [tagId: string];
  reorderTags: [tagIds: string[]];
  clearSelection: [];
  "update:query": [value: string];
  "update:sortBy": [value: SortMode];
}>();
const draggedTagId = ref<string | null>(null);
const dragOverTagId = ref<string | null>(null);
const selected = computed(() => new Set(props.selectedIds));
const orderedTags = computed(() =>
  props.tags.filter((tag) => tag.enabled).sort((a, b) => a.order - b.order || a.name.localeCompare(b.name)),
);
const taggedSkills = (tagId: string) => props.skills.filter((skill) => skill.enabled && skill.tags.includes(tagId));
const tagSkillCount = (tagId: string) => taggedSkills(tagId).length;
function tagState(tagId: string): "none" | "some" | "all" {
  const skills = taggedSkills(tagId);
  const count = skills.filter((skill) => selected.value.has(skill.id)).length;
  return count === 0 ? "none" : count === skills.length ? "all" : "some";
}
function tagTitle(tagId: string, name: string) {
  const count = tagSkillCount(tagId);
  if (!count) return `${name} has no Skills assigned`;
  return tagState(tagId) === "all" ? `Remove ${count} ${name} Skills from selection` : `Select ${count} ${name} Skills`;
}
function insertionSide(targetId: string): "before" | "after" | null {
  if (!draggedTagId.value || dragOverTagId.value !== targetId || draggedTagId.value === targetId) return null;
  const ids = orderedTags.value.map((tag) => tag.id);
  return ids.indexOf(draggedTagId.value) < ids.indexOf(targetId) ? "after" : "before";
}
function reorderedIds(tagId: string, targetId: string) {
  const ids = orderedTags.value.map((tag) => tag.id);
  const from = ids.indexOf(tagId);
  const to = ids.indexOf(targetId);
  if (from < 0 || to < 0 || from === to) return null;
  ids.splice(to, 0, ids.splice(from, 1)[0]);
  return ids;
}
function startDrag(event: globalThis.DragEvent, tagId: string) {
  draggedTagId.value = tagId;
  event.dataTransfer?.setData("text/plain", tagId);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}
function dropTag(targetId: string) {
  if (draggedTagId.value) {
    const ids = reorderedIds(draggedTagId.value, targetId);
    if (ids) emit("reorderTags", ids);
  }
  endDrag();
}
function endDrag() {
  draggedTagId.value = null;
  dragOverTagId.value = null;
}
function moveTag(tagId: string, offset: -1 | 1) {
  const ids = orderedTags.value.map((tag) => tag.id);
  const index = ids.indexOf(tagId);
  const target = ids[index + offset];
  if (target) {
    const reordered = reorderedIds(tagId, target);
    if (reordered) emit("reorderTags", reordered);
  }
}
</script>

<style scoped lang="scss" src="./SkillToolbar.scss"></style>
