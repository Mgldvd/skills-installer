<template>
  <dialog ref="dialogEl" class="add-skill-dialog" @close="emit('update:open', false)">
    <section class="add-skill-dialog__panel">
      <header class="add-skill-dialog__header">
        <div>
          <h2>Add Skill</h2>
          <p>Add a Skill or import a Pack from Skills.sh into the catalog.</p>
        </div>
        <div class="add-skill-dialog__header-actions">
          <button type="button" class="button button--primary" @click="close">Done</button>
          <CloseButton aria-label="Close Add Skill" @click="close" />
        </div>
      </header>

      <div class="add-skill-dialog__body">
        <form class="add-skill-dialog__form" @submit.prevent="handleSubmit">
          <div class="add-skill-dialog__form-primary">
            <label class="add-skill-dialog__field">
              <span class="add-skill-dialog__field-heading">
                Skills.sh URL
                <a
                  class="add-skill-dialog__browse"
                  href="https://skills.sh"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Browse Skills.sh
                </a>
              </span>
              <input
                v-model="url"
                type="url"
                required
                class="add-skill-dialog__input"
                placeholder="https://skills.sh/owner/repository/skill or https://skills.sh/p/..."
                @input="handleUrlInput"
              />
            </label>

            <p v-if="urlError" class="add-skill-dialog__error" role="alert">
              {{ urlError }}
            </p>
            <div v-else-if="packPreview" class="add-skill-dialog__preview">
              <p>
                Detected Pack:
                <strong>{{ packPreview.skills.length }} Skills</strong>
              </p>
              <div class="add-skill-dialog__pack-skills">
                <span v-for="skill in packPreview.skills" :key="skill.name">{{ skill.name }}</span>
              </div>
              <p class="add-skill-dialog__catalog-note">
                This imports catalog entries only. Nothing is installed into the current project.
              </p>
            </div>
            <div v-else-if="preview" class="add-skill-dialog__preview">
              <p>
                Detected skill:
                <strong>{{ preview.skillName }}</strong>
              </p>
              <p>
                Repository:
                <strong>{{ preview.owner }}/{{ preview.repository }}</strong>
              </p>
            </div>
            <p v-if="duplicateSkill && !isPackUrl" class="add-skill-dialog__duplicate" role="alert">
              This Skill is already added as
              <strong>{{ duplicateSkill.displayName }}</strong>
              .
            </p>

            <label class="add-skill-dialog__field">
              <span>{{ isPackUrl ? "Pack Name" : "Display Name" }}</span>
              <input
                v-model="displayName"
                type="text"
                class="add-skill-dialog__input"
                :placeholder="packPreview?.suggestedName ?? suggestedDisplayName ?? 'Display name'"
                @input="displayNameEdited = true"
              />
            </label>

            <p v-if="submitError" class="add-skill-dialog__error" role="alert">
              {{ submitError }}
            </p>

            <div class="add-skill-dialog__form-actions">
              <div v-if="!isPackUrl" class="add-skill-dialog__checkboxes">
                <label class="add-skill-dialog__checkbox">
                  <input v-model="preselected" type="checkbox" />
                  <span>Preselected by default</span>
                </label>
                <label class="add-skill-dialog__checkbox">
                  <input v-model="enabled" type="checkbox" />
                  <span>Enabled</span>
                </label>
              </div>
              <button type="submit" class="button button--primary" :disabled="!canSubmit">
                {{ isPackUrl ? "Import Pack" : "Add Skill" }}
              </button>
            </div>
          </div>

          <label v-if="!isPackUrl" class="add-skill-dialog__field add-skill-dialog__form-secondary">
            <span>Description</span>
            <textarea
              v-model="description"
              class="add-skill-dialog__input add-skill-dialog__textarea"
              @input="descriptionEdited = true"
            />
          </label>
          <div v-else class="add-skill-dialog__field add-skill-dialog__form-secondary">
            <span>Pack Color</span>
            <ColorPalettePicker v-model="packColor" aria-label="Pack color" />
          </div>

          <fieldset v-if="!isPackUrl" class="add-skill-dialog__packs">
            <legend>Packs</legend>
            <p>Assign this Skill to one or more installation packs.</p>
            <div v-if="enabledTags.length" class="add-skill-dialog__pack-list">
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
                @click="toggleTag(tag.id)"
              />
            </div>
            <p v-else class="add-skill-dialog__packs-empty">No enabled packs are available.</p>
          </fieldset>
        </form>

        <section class="add-skill-dialog__catalog">
          <div class="add-skill-dialog__catalog-heading">
            <h3>Skills from Skills.sh</h3>
            <button
              type="button"
              class="add-skill-dialog__catalog-toggle"
              :aria-expanded="showCatalog"
              @click="showCatalog = !showCatalog"
            >
              {{ showCatalog ? "Hide" : "Show" }} ({{ remoteSkills.length }})
            </button>
          </div>

          <template v-if="showCatalog">
            <p class="add-skill-dialog__catalog-warning">
              Danger zone: deleting a Skill here removes it from the catalog entirely.
            </p>
            <p v-if="deleteError" class="add-skill-dialog__error" role="alert">{{ deleteError }}</p>

            <div v-if="remoteSkills.length" class="add-skill-dialog__table" role="table" aria-label="Skills from Skills.sh">
              <div class="add-skill-dialog__row add-skill-dialog__row--header" role="row">
                <input
                  type="checkbox"
                  :checked="allSelected"
                  aria-label="Select all Skills from Skills.sh"
                  @change="toggleSelectAll"
                />
                <span role="columnheader">Name</span>
                <span role="columnheader">URL</span>
                <span role="columnheader" class="add-skill-dialog__row-actions-header">
                  <button
                    v-if="selectedIds.length"
                    type="button"
                    class="add-skill-dialog__bulk-delete"
                    :disabled="deleting"
                    :aria-label="`Delete ${selectedIds.length} selected Skills`"
                    :title="`Delete ${selectedIds.length} selected Skills`"
                    @click="requestBulkDelete"
                  >
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                      <path
                        d="M3 4.5h10M6.5 4.5V3a1 1 0 0 1 1-1h1a1 1 0 0 1 1 1v1.5M4.5 4.5v8.5a1 1 0 0 0 1 1h5a1 1 0 0 0 1-1V4.5M6.5 7.5v4M9.5 7.5v4"
                      />
                    </svg>
                  </button>
                </span>
              </div>
              <div v-for="skill in remoteSkills" :key="skill.id" class="add-skill-dialog__row" role="row">
                <input
                  type="checkbox"
                  :checked="selected.has(skill.id)"
                  :disabled="deleting"
                  :aria-label="`Select ${skill.displayName}`"
                  @change="toggleSelect(skill.id)"
                />
                <span class="add-skill-dialog__skill-name">{{ skill.displayName }}</span>
                <a
                  class="add-skill-dialog__skill-link"
                  :href="skill.skillsUrl"
                  :title="skill.skillsUrl"
                  @click.prevent="handleOpenSkillUrl(skill.skillsUrl)"
                >
                  {{ skill.skillsUrl }}
                </a>
                <button
                  type="button"
                  class="add-skill-dialog__row-edit"
                  :aria-label="`Edit ${skill.displayName}`"
                  :title="`Edit ${skill.displayName}`"
                  @click="emit('edit', skill.id)"
                >
                  <svg viewBox="0 0 14 14" aria-hidden="true">
                    <path
                      d="M2.5 10.4V12h1.6l6.6-6.6-1.6-1.6-6.6 6.6Zm7.4-7.4 1-1 1.6 1.6-1 1L9.9 3Z"
                      fill="currentColor"
                    />
                  </svg>
                </button>
                <button
                  type="button"
                  class="add-skill-dialog__row-delete"
                  :disabled="deleting"
                  :aria-label="`Delete ${skill.displayName}`"
                  :title="`Delete ${skill.displayName}`"
                  @click="requestDelete(skill)"
                >
                  <svg viewBox="0 0 16 16" aria-hidden="true">
                    <path
                      d="M3 4.5h10M6.5 4.5V3a1 1 0 0 1 1-1h1a1 1 0 0 1 1 1v1.5M4.5 4.5v8.5a1 1 0 0 0 1 1h5a1 1 0 0 0 1-1V4.5M6.5 7.5v4M9.5 7.5v4"
                    />
                  </svg>
                </button>
              </div>
            </div>
            <p v-else class="add-skill-dialog__catalog-empty">No Skills added from Skills.sh yet.</p>
          </template>
        </section>
      </div>
    </section>

    <ConfirmDialog
      v-model:open="confirmDeleteOpen"
      title="Delete Skill"
      :message="confirmDeleteMessage"
      confirm-label="Delete"
      destructive
      @confirm="handleConfirmDelete"
    />
  </dialog>
</template>

<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed, ref } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";
import * as backend from "../../services/backend";
import type { ParsedSkillSource, Skill, SkillGroup, SkillTag } from "../../types";
import ColorPalettePicker from "../ColorPalettePicker/ColorPalettePicker.vue";
import CloseButton from "../CloseButton/CloseButton.vue";
import ConfirmDialog from "../ConfirmDialog/ConfirmDialog.vue";
import PackBadge from "../PackBadge/PackBadge.vue";

const props = withDefaults(
  defineProps<{
    open: boolean;
    groups?: SkillGroup[];
    skills?: Skill[];
    tags?: SkillTag[];
    defaultGroupId?: string | null;
    submitError?: string | null;
    deleting?: boolean;
    deleteError?: string | null;
  }>(),
  {
    groups: () => [],
    skills: () => [],
    tags: () => [],
    defaultGroupId: null,
    submitError: null,
    deleting: false,
    deleteError: null,
  },
);

const emit = defineEmits<{
  "update:open": [value: boolean];
  submit: [payload: backend.AddSkillArgs];
  importPack: [payload: backend.ImportPackArgs];
  deleteSkills: [skillIds: string[]];
  edit: [skillId: string];
}>();

const dialogEl = ref<HTMLDialogElement | null>(null);
const url = ref("");
const displayName = ref("");
const description = ref("");
const packColor = ref("#6366F1");
const groupId = ref<string | null>(null);
const preselected = ref(false);
const enabled = ref(true);
const preview = ref<ParsedSkillSource | null>(null);
const packPreview = ref<backend.PackPreview | null>(null);
const urlError = ref<string | null>(null);
const displayNameEdited = ref(false);
const descriptionEdited = ref(false);
const selectedTagIds = ref<string[]>([]);
const selected = ref(new Set<string>());
// Deleting here removes a catalog entry outright, so the list stays folded
// away behind an explicit click every time the dialog opens, rather than
// sitting exposed next to the "Add Skill" form where a stray click can hit it.
const showCatalog = ref(false);

let debounceHandle: ReturnType<typeof setTimeout> | null = null;
let previewRequest = 0;

function resetForm() {
  previewRequest += 1;
  if (debounceHandle) clearTimeout(debounceHandle);
  url.value = "";
  displayName.value = "";
  description.value = "";
  packColor.value = "#6366F1";
  groupId.value = props.defaultGroupId ?? props.groups?.[0]?.id ?? null;
  preselected.value = false;
  enabled.value = true;
  preview.value = null;
  packPreview.value = null;
  urlError.value = null;
  displayNameEdited.value = false;
  descriptionEdited.value = false;
  selectedTagIds.value = [];
  selected.value = new Set();
  showCatalog.value = false;
}

useNativeDialog(dialogEl, () => props.open, resetForm);

function handleUrlInput() {
  const request = ++previewRequest;
  if (debounceHandle) clearTimeout(debounceHandle);
  debounceHandle = setTimeout(async () => {
    if (!url.value.trim()) {
      preview.value = null;
      packPreview.value = null;
      urlError.value = null;
      if (!displayNameEdited.value) displayName.value = "";
      if (!descriptionEdited.value) description.value = "";
      return;
    }
    try {
      const pack = isPackUrl.value;
      const detected = pack
        ? await backend.previewPackUrl(url.value.trim())
        : await backend.previewSkillUrl(url.value.trim());
      if (request !== previewRequest) return;
      if (pack) {
        packPreview.value = detected as backend.PackPreview;
        preview.value = null;
        if (!displayNameEdited.value) displayName.value = (detected as backend.PackPreview).suggestedName;
      } else {
        preview.value = detected as ParsedSkillSource;
        packPreview.value = null;
        const skill = detected as ParsedSkillSource;
        if (!displayNameEdited.value) displayName.value = `${skill.owner} - ${skill.skillName}`;
        if (!descriptionEdited.value) description.value = `${skill.owner}/${skill.repository}`;
      }
      urlError.value = null;
    } catch (error) {
      if (request !== previewRequest) return;
      preview.value = null;
      packPreview.value = null;
      urlError.value = error instanceof Error ? error.message : String(error);
    }
  }, 300);
}

const duplicateSkill = computed(() => {
  if (!preview.value) return null;
  return props.skills?.find((skill) => skill.skillsUrl === preview.value?.canonicalUrl) ?? null;
});
const isPackUrl = computed(() => /^https:\/\/(?:www\.)?skills\.sh\/p\//i.test(url.value.trim()));
const suggestedDisplayName = computed(() =>
  preview.value ? `${preview.value.owner} - ${preview.value.skillName}` : null,
);
const enabledTags = computed(() => props.tags.filter((tag) => tag.enabled).sort((a, b) => a.order - b.order));

function toggleTag(tagId: string) {
  selectedTagIds.value = selectedTagIds.value.includes(tagId)
    ? selectedTagIds.value.filter((id) => id !== tagId)
    : [...selectedTagIds.value, tagId];
}
const canSubmit = computed(() =>
  Boolean(
    (preview.value || packPreview.value) &&
    displayName.value.trim() &&
    groupId.value &&
    !urlError.value &&
    (!preview.value || !duplicateSkill.value),
  ),
);

// The catalog-management table below the form: every Skill added via a
// Skills.sh URL (as opposed to one discovered locally on disk), so users can
// see and prune what they've added without leaving this dialog.
const remoteSkills = computed(() => props.skills.filter((skill) => skill.source.kind === "remote"));
const selectedIds = computed(() => [...selected.value]);
const allSelected = computed(
  () => remoteSkills.value.length > 0 && remoteSkills.value.every((skill) => selected.value.has(skill.id)),
);

function toggleSelect(skillId: string) {
  const next = new Set(selected.value);
  if (next.has(skillId)) next.delete(skillId);
  else next.add(skillId);
  selected.value = next;
}

function toggleSelectAll() {
  selected.value = allSelected.value ? new Set() : new Set(remoteSkills.value.map((skill) => skill.id));
}

// A native `window.confirm` doesn't reliably show anything in every Tauri
// webview (WebKitGTK on Linux in particular can auto-resolve it without ever
// rendering a dialog) — the app's own `ConfirmDialog` is a real DOM/`<dialog>`
// element, so it always shows up regardless of platform.
const confirmDeleteOpen = ref(false);
const pendingDeleteIds = ref<string[]>([]);
const confirmDeleteMessage = computed(() => {
  if (pendingDeleteIds.value.length === 1) {
    const skill = remoteSkills.value.find((candidate) => candidate.id === pendingDeleteIds.value[0]);
    return `Delete “${skill?.displayName ?? "this Skill"}” from the catalog?`;
  }
  return `Delete ${pendingDeleteIds.value.length} Skills from the catalog?`;
});

function requestDelete(skill: Skill) {
  pendingDeleteIds.value = [skill.id];
  confirmDeleteOpen.value = true;
}

function requestBulkDelete() {
  pendingDeleteIds.value = [...selectedIds.value];
  confirmDeleteOpen.value = true;
}

function handleConfirmDelete() {
  const ids = pendingDeleteIds.value;
  emit("deleteSkills", ids);
  const next = new Set(selected.value);
  for (const id of ids) next.delete(id);
  selected.value = next;
  pendingDeleteIds.value = [];
}

// Opens in the system's default browser, not the app's own webview — a
// plain `<a href>` would otherwise just try to navigate this window itself.
function handleOpenSkillUrl(skillsUrl: string) {
  openUrl(skillsUrl).catch(() => undefined);
}

function close() {
  emit("update:open", false);
}

function handleSubmit() {
  if (!canSubmit.value || !groupId.value) return;
  if (packPreview.value) {
    emit("importPack", {
      url: url.value.trim(),
      packName: displayName.value.trim(),
      color: packColor.value,
      groupId: groupId.value,
    });
    return;
  }
  emit("submit", {
    url: url.value.trim(),
    displayName: displayName.value.trim() || undefined,
    description: description.value.trim() || undefined,
    groupId: groupId.value,
    tags: selectedTagIds.value,
    preselected: preselected.value,
    enabled: enabled.value,
  });
}
</script>

<style scoped lang="scss" src="./AddSkillDialog.scss"></style>
