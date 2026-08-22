<template>
  <dialog ref="dialogEl" class="tags-dialog" @close="$emit('update:open', false)" @click="onBackdrop">
    <section class="tags-dialog__panel">
      <header class="tags-dialog__header">
        <div>
          <h2>Packs</h2>
          <p>Create reusable groups of Skills for quick installation selection.</p>
        </div>
        <div class="tags-dialog__header-actions">
          <button type="button" class="button" @click="openCreate()">+ New Pack</button>
          <button type="button" class="button button--primary" @click="close">Done</button>
          <CloseButton aria-label="Close Packs" @click="close" />
        </div>
      </header>

      <div class="tags-dialog__body">
        <form v-if="editor" class="tags-dialog__editor" @submit.prevent="saveEditor">
          <label class="tags-dialog__editor-field">
            Name
            <input ref="nameInput" v-model="editor.name" maxlength="64" required />
          </label>
          <fieldset class="tags-dialog__palette">
            <legend>Color</legend>
            <button
              v-for="color in palette"
              :key="color"
              type="button"
              class="color"
              :class="{ selected: editor.color === color }"
              :style="{ backgroundColor: color }"
              :aria-pressed="editor.color === color"
              :aria-label="`Use color ${color}`"
              @click="editor.color = color" />
          </fieldset>
          <div class="tags-dialog__editor-actions">
            <button
              v-if="editor.id"
              type="button"
              class="link-button link-button--danger"
              @click="requestDelete(editor.id, editor.name)">
              Delete
            </button>
            <button type="button" class="link-button" @click="editor = null">Cancel</button>
            <button type="submit" class="button button--primary" :disabled="busy">
              {{ editor.id ? "Save" : "Create" }}
            </button>
          </div>
        </form>
        <p v-if="error" class="tags-dialog__error" role="alert">
          {{ error }}
        </p>

        <div v-if="tags.length" class="tags-dialog__catalog" aria-label="Available Packs">
          <PackBadge
            v-for="tag in managedTags"
            :key="tag.id"
            class="catalog-tag"
            :name="tag.name"
            :color="tag.color"
            interactive
            :aria-label="`Edit ${tag.name} Pack`"
            @click="openEdit(tag)">
            <template #trailing>
              <svg class="catalog-tag__edit" viewBox="0 0 14 14" aria-hidden="true">
                <path
                  d="M2.5 10.4V12h1.6l6.6-6.6-1.6-1.6-6.6 6.6Zm7.4-7.4 1-1 1.6 1.6-1 1L9.9 3Z"
                  fill="currentColor" />
              </svg>
            </template>
          </PackBadge>
        </div>
        <input
          v-model="query"
          class="tags-dialog__search"
          type="search"
          placeholder="Search skills..."
          aria-label="Search skills" />

        <div class="tags-dialog__skills" role="list" aria-label="Skills">
          <div class="tags-dialog__columns" aria-hidden="true">
            <span>Skill</span>
            <span>Packs</span>
          </div>
          <article v-for="skill in filteredSkills" :key="skill.id" class="skill-row" role="listitem">
            <h3 class="skill-row__name">{{ skill.displayName }}</h3>
            <div class="skill-row__tags">
              <PackBadge
                v-for="tag in enabledTags"
                :key="tag.id"
                class="tag-toggle"
                :class="{
                  'is-pending': isPending(skill.id, tag.id),
                }"
                :name="tag.name"
                :color="tag.color"
                interactive
                compact
                :selected="hasTag(skill, tag)"
                :muted="!hasTag(skill, tag)"
                :aria-label="`${hasTag(skill, tag) ? 'Unassign' : 'Assign'} ${tag.name} ${hasTag(skill, tag) ? 'from' : 'to'} ${skill.displayName}`"
                :disabled="isPending(skill.id, tag.id)"
                @click="toggleTag(skill, tag)" />
              <span v-if="enabledTags.length === 0" class="skill-row__empty">No enabled Packs</span>
            </div>
          </article>
          <p v-if="filteredSkills.length === 0" class="tags-dialog__empty">No skills match “{{ query }}”.</p>
        </div>
      </div>
    </section>
  </dialog>
</template>
<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import type { Skill, SkillTag } from "../../types";
import CloseButton from "../CloseButton/CloseButton.vue";
import PackBadge from "../PackBadge/PackBadge.vue";
const props = withDefaults(
  defineProps<{
    open: boolean;
    skills?: Skill[];
    tags?: SkillTag[];
    pendingKeys?: string[];
    busy?: boolean;
    error?: string | null;
  }>(),
  {
    skills: () => [],
    tags: () => [],
    pendingKeys: () => [],
    busy: false,
    error: null,
  },
);
const emit = defineEmits<{
  "update:open": [boolean];
  assign: [string, string];
  unassign: [string, string];
  create: [string, string];
  update: [string, string, string];
  delete: [string];
}>();
const palette = [
  "#F43F75",
  "#F05252",
  "#F97316",
  "#F59E0B",
  "#22C55E",
  "#14B8A6",
  "#06B6D4",
  "#3B82F6",
  "#6366F1",
  "#A855F7",
];
const dialogEl = ref<HTMLDialogElement | null>(null),
  nameInput = ref<HTMLInputElement | null>(null),
  query = ref("");
const editor = ref<{ id: string | null; name: string; color: string } | null>(null);
const managedTags = computed(() => [...props.tags].sort((a, b) => a.order - b.order || a.name.localeCompare(b.name)));
const enabledTags = computed(() => managedTags.value.filter((t) => t.enabled));
const filteredSkills = computed(() => {
  const q = query.value.trim().toLowerCase();
  return props.skills.filter((s) => !q || s.displayName.toLowerCase().includes(q) || s.name.toLowerCase().includes(q));
});
watch(
  () => props.open,
  (open) => {
    if (open && !dialogEl.value?.open) dialogEl.value?.showModal();
    else if (!open && dialogEl.value?.open) dialogEl.value.close();
  },
);
function close() {
  dialogEl.value?.close();
}
function onBackdrop(e: MouseEvent) {
  if (e.target === dialogEl.value) close();
}
function hasTag(skill: Skill, tag: SkillTag) {
  return skill.tags.includes(tag.id);
}
function operationKey(skillId: string, tagId: string) {
  return `${skillId}:${tagId}`;
}
function isPending(skillId: string, tagId: string) {
  return props.pendingKeys.includes(operationKey(skillId, tagId));
}
function toggleTag(skill: Skill, tag: SkillTag) {
  if (isPending(skill.id, tag.id)) return;
  if (hasTag(skill, tag)) emit("unassign", skill.id, tag.id);
  else emit("assign", skill.id, tag.id);
}
function openCreate() {
  editor.value = { id: null, name: "", color: palette[0] };
  nextTick(() => nameInput.value?.focus());
}
function openEdit(tag: SkillTag) {
  editor.value = { id: tag.id, name: tag.name, color: tag.color };
  nextTick(() => nameInput.value?.focus());
}
function saveEditor() {
  if (!editor.value) return;
  const v = editor.value;
  if (v.id) emit("update", v.id, v.name, v.color);
  else emit("create", v.name, v.color);
  editor.value = null;
}
function requestDelete(id: string, name: string) {
  const count = props.skills.filter((s) => s.tags.includes(id)).length;
  if (
    window.confirm(
      `Delete “${name}”?\n\nThis Pack contains ${count} skill${count === 1 ? "" : "s"}. Those skills will remain available and will not be uninstalled.`,
    )
  ) {
    emit("delete", id);
    editor.value = null;
  }
}
</script>
<style scoped lang="scss" src="./TagsDialog.scss"></style>
