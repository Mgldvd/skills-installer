<template>
  <dialog ref="dialogEl" class="edit-skill-dialog" @close="emit('update:open', false)" @click="handleBackdropClick">
    <form v-if="skill" class="edit-skill-dialog__form" @submit.prevent="handleSubmit">
      <section class="edit-skill-dialog__hero">
        <div class="edit-skill-dialog__hero-heading">
          <div>
            <span class="edit-skill-dialog__eyebrow">Skill information</span>
            <h2 class="edit-skill-dialog__title">
              {{ displayName || skill.displayName }}
            </h2>
          </div>
          <a v-if="skill.skillsUrl" :href="skill.skillsUrl" target="_blank" rel="noopener noreferrer">
            View on skills.sh
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path d="M6 3h7v7M13 3 5 11M11 9v4H3V5h4" />
            </svg>
          </a>
        </div>
        <p class="edit-skill-dialog__description">
          {{ description || "No description provided." }}
        </p>
        <div class="edit-skill-dialog__meta">
          <span>
            {{ skill.local ? "Local Skill" : `${skill.repository || "Remote repository"} / ${skill.skillName}` }}
          </span>
          <span>{{ selectedTagIds.length }} {{ selectedTagIds.length === 1 ? "Pack" : "Packs" }}</span>
        </div>
      </section>

      <section class="edit-skill-dialog__details">
        <div class="edit-skill-dialog__fields">
          <label v-if="!skill.local" class="edit-skill-dialog__field edit-skill-dialog__field--wide">
            <span>Skills.sh URL</span>
            <input v-model="url" type="url" class="edit-skill-dialog__input" @input="handleUrlInput" />
          </label>
          <p v-if="urlError" class="edit-skill-dialog__error edit-skill-dialog__field--wide" role="alert">
            {{ urlError }}
          </p>
          <div v-else-if="preview" class="edit-skill-dialog__preview edit-skill-dialog__field--wide">
            <p>
              Detected skill:
              <strong>{{ preview.skillName }}</strong>
            </p>
            <p>
              Repository:
              <strong>{{ preview.owner }}/{{ preview.repository }}</strong>
            </p>
          </div>

          <label class="edit-skill-dialog__field">
            <span>Display Name</span>
            <input v-model="displayName" type="text" required class="edit-skill-dialog__input" />
          </label>

          <label class="edit-skill-dialog__field">
            <span>Description</span>
            <textarea
              v-model="description"
              class="edit-skill-dialog__input edit-skill-dialog__textarea"
              rows="3"
              maxlength="320" />
            <small>Short summary displayed on the Skill card.</small>
          </label>

          <div class="edit-skill-dialog__checkboxes edit-skill-dialog__field--wide">
            <label class="edit-skill-dialog__checkbox">
              <input v-model="preselected" type="checkbox" />
              <span>Preselected by default</span>
            </label>
            <label class="edit-skill-dialog__checkbox">
              <input v-model="enabled" type="checkbox" />
              <span>Enabled</span>
            </label>
          </div>

          <fieldset class="edit-skill-dialog__packs edit-skill-dialog__field--wide">
            <legend>Packs</legend>
            <p>Assign this Skill to one or more installation packs.</p>
            <div v-if="enabledTags.length" class="edit-skill-dialog__pack-list">
              <PackBadge
                v-for="tag in enabledTags"
                :key="tag.id"
                :name="tag.name"
                :color="tag.color"
                interactive
                compact
                :selected="selectedTagIds.includes(tag.id)"
                :muted="!selectedTagIds.includes(tag.id)"
                :aria-label="`${selectedTagIds.includes(tag.id) ? 'Unassign' : 'Assign'} ${tag.name}`"
                @click="toggleTag(tag.id)" />
            </div>
            <p v-else class="edit-skill-dialog__packs-empty">No enabled packs are available.</p>
          </fieldset>
        </div>
      </section>

      <p v-if="submitError" class="edit-skill-dialog__error" role="alert">
        {{ submitError }}
      </p>

      <div v-if="deleteRequested" class="edit-skill-dialog__delete-confirm" role="alert">
        <div>
          <strong>Delete “{{ skill.displayName }}”?</strong>
          <span>This removes the Skill from the catalog. This action cannot be undone.</span>
        </div>
      </div>

      <div class="edit-skill-dialog__actions">
        <template v-if="deleteRequested">
          <button type="button" class="edit-skill-dialog__btn" @click="deleteRequested = false">Keep Skill</button>
          <button type="button" class="edit-skill-dialog__btn edit-skill-dialog__btn--danger" @click="confirmDelete">
            Delete Skill
          </button>
        </template>
        <template v-else>
          <button
            type="button"
            class="edit-skill-dialog__btn edit-skill-dialog__btn--delete"
            @click="deleteRequested = true">
            Delete Skill
          </button>
          <span class="edit-skill-dialog__actions-spacer" />
          <button type="button" class="edit-skill-dialog__btn" @click="close">Cancel</button>
          <button type="submit" class="edit-skill-dialog__btn edit-skill-dialog__btn--primary" :disabled="!canSubmit">
            Save Changes
          </button>
        </template>
      </div>
    </form>
  </dialog>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";
import * as backend from "../../services/backend";
import type { ParsedSkillSource, Skill, SkillTag } from "../../types";
import PackBadge from "../PackBadge/PackBadge.vue";

const props = withDefaults(
  defineProps<{
    open: boolean;
    skill: Skill | null;
    tags?: SkillTag[];
    submitError?: string | null;
  }>(),
  { tags: () => [], submitError: null },
);

const emit = defineEmits<{
  "update:open": [value: boolean];
  submit: [payload: backend.UpdateSkillArgs];
  delete: [skillId: string];
}>();

const dialogEl = ref<HTMLDialogElement | null>(null);
const url = ref("");
const displayName = ref("");
const description = ref("");
const groupId = ref<string | null>(null);
const preselected = ref(false);
const enabled = ref(true);
const selectedTagIds = ref<string[]>([]);
const preview = ref<ParsedSkillSource | null>(null);
const urlError = ref<string | null>(null);
const urlChanged = ref(false);
const deleteRequested = ref(false);

let debounceHandle: ReturnType<typeof setTimeout> | null = null;

function loadFromSkill() {
  const skill = props.skill;
  if (!skill) return;
  url.value = skill.skillsUrl;
  displayName.value = skill.displayName;
  description.value = skill.description;
  groupId.value = skill.groupId;
  preselected.value = skill.preselected;
  enabled.value = skill.enabled;
  selectedTagIds.value = [...skill.tags];
  preview.value = null;
  urlError.value = null;
  urlChanged.value = false;
  deleteRequested.value = false;
}

useNativeDialog(dialogEl, () => props.open, loadFromSkill);

function handleUrlInput() {
  urlChanged.value = true;
  if (debounceHandle) clearTimeout(debounceHandle);
  debounceHandle = setTimeout(async () => {
    if (!url.value.trim()) {
      preview.value = null;
      urlError.value = null;
      return;
    }
    try {
      preview.value = await backend.previewSkillUrl(url.value.trim());
      urlError.value = null;
    } catch (error) {
      preview.value = null;
      urlError.value = error instanceof Error ? error.message : String(error);
    }
  }, 300);
}

const canSubmit = computed(() => displayName.value.trim().length > 0 && !urlError.value && Boolean(groupId.value));
const enabledTags = computed(() => props.tags.filter((tag) => tag.enabled).sort((a, b) => a.order - b.order));

function close() {
  emit("update:open", false);
}

function handleBackdropClick(event: MouseEvent) {
  if (event.target === dialogEl.value) close();
}

function toggleTag(tagId: string) {
  selectedTagIds.value = selectedTagIds.value.includes(tagId)
    ? selectedTagIds.value.filter((id) => id !== tagId)
    : [...selectedTagIds.value, tagId];
}

function confirmDelete() {
  if (props.skill) emit("delete", props.skill.id);
}

function handleSubmit() {
  if (!canSubmit.value || !props.skill || !groupId.value) return;
  emit("submit", {
    skillId: props.skill.id,
    url: urlChanged.value ? url.value.trim() : undefined,
    displayName: displayName.value.trim(),
    description: description.value.trim(),
    groupId: groupId.value,
    tags: selectedTagIds.value,
    preselected: preselected.value,
    enabled: enabled.value,
  });
}
</script>

<style scoped lang="scss" src="./EditSkillDialog.scss"></style>
