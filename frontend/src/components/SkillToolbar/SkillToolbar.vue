<template>
  <section class="skill-toolbar" aria-label="Skill preselection Packs">
    <span class="skill-toolbar__label">Pack select</span>
    <button
      type="button"
      class="skill-toolbar__manage-packs"
      aria-label="Manage Packs"
      title="Manage Packs"
      @click="emit('openPacks')"
    >
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <path d="M8 2v12M2 8h12" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" />
      </svg>
    </button>
    <!-- Every Pack must stay on screen with no scrolling of any kind — this
         is a desktop app, not a page — so `.skill-toolbar` is a plain
         `flex-wrap: wrap` group and every tag slot below is a genuine
         direct child of it (a bare `<template>` for the `v-for`, no
         wrapper div — `display: contents` looked equivalent but is exactly
         what broke this: WebKitGTK, the engine this app's window actually
         renders with, does not reliably run flex-wrap layout on children
         collapsed that way, which is what produced the overlapping,
         out-of-order rendering seen in the field). Nothing is pinned to
         the right with `margin-left: auto` either — that was what
         stranded the buttons far from the last Pack with a large empty gap
         whenever the wrapped Packs happened to end partway across a line.
         Left to flow naturally in real DOM order, the buttons just sit
         right after the last Pack, wrapping down with it if there isn't
         room. -->
    <template v-if="orderedTags.length">
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
        @dragend="endDrag"
      >
        <PackBadge
          class="skill-toolbar__tag"
          :name="tag.name"
          :color="tag.color"
          interactive
          :selected="tagState(tag.id) === 'all'"
          :partial="tagState(tag.id) === 'some'"
          :disabled="tagSkillCount(tag.id) === 0 || tagAllInstalled(tag.id)"
          :title="`${tagTitle(tag.id, tag.name)}. Drag to reorder; Alt+Left or Alt+Right also moves it.`"
          @click="emit('toggleTag', tag.id)"
          @keydown.alt.left.prevent="moveTag(tag.id, -1)"
          @keydown.alt.right.prevent="moveTag(tag.id, 1)"
        >
          <template #trailing>
            <span class="skill-toolbar__count" aria-hidden="true">{{ tagSkillCount(tag.id) }}</span>
          </template>
        </PackBadge>
      </div>
    </template>
    <p v-else class="skill-toolbar__empty">Create Packs from the Packs menu to build reusable selections.</p>
    <div class="skill-toolbar__actions">
      <button
        type="button"
        class="skill-toolbar__select-missing"
        :disabled="needsAgentsCount === 0"
        :title="
          needsAgentsCount
            ? `Add the ${needsAgentsCount} Skill${needsAgentsCount === 1 ? '' : 's'} still missing an agent to the selection`
            : 'Every installed Skill already covers every targeted agent'
        "
        @click="emit('selectMissing')"
      >
        Select missing{{ needsAgentsCount ? ` (${needsAgentsCount})` : "" }}
      </button>
      <!-- Selects every outdated local Skill so the same Install Selected
           button below re-installs (= updates) them in one batch, instead
           of clicking each card's own Update button one at a time. -->
      <button
        type="button"
        class="skill-toolbar__update-all"
        :disabled="needsUpdateCount === 0"
        :title="
          needsUpdateCount
            ? `Add the ${needsUpdateCount} outdated Skill${needsUpdateCount === 1 ? '' : 's'} to the selection`
            : 'No installed Skill has an update available'
        "
        @click="emit('selectUpdates')"
      >
        Update all{{ needsUpdateCount ? ` (${needsUpdateCount})` : "" }}
      </button>
      <button
        type="button"
        class="skill-toolbar__clear"
        :disabled="selectedIds.length === 0"
        aria-label="Clear all selected Skills"
        @click="emit('clearSelection')"
      >
        Clear
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import type { Skill, SkillTag } from "../../types";
import PackBadge from "../PackBadge/PackBadge.vue";

const props = withDefaults(
  defineProps<{
    tags?: SkillTag[];
    skills?: Skill[];
    selectedIds?: string[];
    needsAgentsCount?: number;
    needsUpdateCount?: number;
  }>(),
  {
    tags: () => [],
    skills: () => [],
    selectedIds: () => [],
    needsAgentsCount: 0,
    needsUpdateCount: 0,
  },
);

const emit = defineEmits<{
  toggleTag: [tagId: string];
  reorderTags: [tagIds: string[]];
  clearSelection: [];
  selectMissing: [];
  selectUpdates: [];
  openPacks: [];
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
// Nothing left to select for install when every Skill this Pack covers is
// already installed — greyed out and unclickable rather than toggling a
// selection that would install nothing.
function tagAllInstalled(tagId: string): boolean {
  const skills = taggedSkills(tagId);
  return skills.length > 0 && skills.every((skill) => skill.installed);
}
function tagTitle(tagId: string, name: string) {
  const count = tagSkillCount(tagId);
  if (!count) return `${name} has no Skills assigned`;
  if (tagAllInstalled(tagId)) return `${name}: all ${count} Skill${count === 1 ? "" : "s"} already installed`;
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
