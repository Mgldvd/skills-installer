<template>
  <dialog ref="dialogEl" class="presets-dialog" @close="emit('update:open', false)" @click="onBackdrop">
    <section class="presets-dialog__panel">
      <header class="presets-dialog__header">
        <div>
          <h2>Presets</h2>
          <p>Save which Skills are installed here, so you can re-select them after a fresh clone.</p>
        </div>
        <CloseButton aria-label="Close Presets" @click="close" />
      </header>

      <div class="presets-dialog__body">
        <form class="presets-dialog__form" @submit.prevent="handleSave">
          <label class="presets-dialog__field">
            <span>Name</span>
            <input v-model="name" type="text" maxlength="64" required placeholder="e.g. Marketing Site" />
          </label>
          <label class="presets-dialog__field">
            <span>Git URL <em>(optional)</em></span>
            <input v-model="gitUrl" type="url" placeholder="https://github.com/owner/repo" />
          </label>
          <button type="submit" class="button button--primary presets-dialog__submit" :disabled="!canSave">
            Save {{ installedSkillNames.length }} installed Skill{{ installedSkillNames.length === 1 ? "" : "s" }}
          </button>
        </form>
        <p v-if="installedSkillNames.length === 0" class="presets-dialog__hint">
          Nothing installed yet in the current destination — install some Skills first.
        </p>
        <p v-if="error" class="presets-dialog__error" role="alert">{{ error }}</p>

        <ul v-if="presets.length" class="presets-dialog__list" aria-label="Saved Presets">
          <li v-for="preset in presets" :key="preset.id" class="presets-dialog__row">
            <div class="presets-dialog__info">
              <strong class="presets-dialog__name">{{ preset.name }}</strong>
              <span class="presets-dialog__count">
                {{ preset.skillNames.length }} Skill{{ preset.skillNames.length === 1 ? "" : "s" }}
              </span>
              <a
                v-if="preset.gitUrl"
                class="presets-dialog__git-url"
                :href="preset.gitUrl"
                :title="preset.gitUrl"
                @click.prevent="handleOpenGitUrl(preset.gitUrl)"
              >
                <svg
                  v-if="gitHostFromUrl(preset.gitUrl) === 'github'"
                  class="presets-dialog__git-icon"
                  viewBox="0 0 16 16"
                  aria-hidden="true"
                >
                  <path
                    d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8Z"
                  />
                </svg>
                <svg
                  v-else-if="gitHostFromUrl(preset.gitUrl) === 'gitlab'"
                  class="presets-dialog__git-icon"
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <path
                    d="M23.6 9.59v-.09L20.33.9a.85.85 0 0 0-1.61.16L16.7 7.16H7.37L5.37 1.03a.85.85 0 0 0-1.61-.15L.5 9.51v.09a10.03 10.03 0 0 0 3.15 11.24l.02.02.09.07 5.33 3.93 2.64 1.97 1.62 1.2a1 1 0 0 0 1.2 0l1.62-1.2 2.64-1.97 5.36-3.94.02-.01a10.03 10.03 0 0 0 3.13-11.25Z"
                  />
                </svg>
                <svg v-else class="presets-dialog__git-icon" viewBox="0 0 16 16" aria-hidden="true">
                  <path
                    d="M6.5 4H4.5A1.5 1.5 0 0 0 3 5.5v6A1.5 1.5 0 0 0 4.5 13h6a1.5 1.5 0 0 0 1.5-1.5V9.5M9 3h4v4M7 9l6-6"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.3"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
                {{ preset.gitUrl }}
              </a>
            </div>
            <div class="presets-dialog__actions">
              <button type="button" class="button button--primary" @click="emit('load', preset.id)">Load</button>
              <button
                type="button"
                class="button"
                :disabled="installedSkillNames.length === 0"
                :title="
                  installedSkillNames.length === 0
                    ? 'Nothing installed here to update this Preset with'
                    : `Replace this Preset's Skills with the ${installedSkillNames.length} currently installed`
                "
                @click="requestUpdate(preset)"
              >
                Update
              </button>
              <button
                type="button"
                class="presets-dialog__delete"
                :aria-label="`Delete ${preset.name}`"
                :title="`Delete ${preset.name}`"
                @click="requestDelete(preset)"
              >
                <svg viewBox="0 0 16 16" aria-hidden="true">
                  <path
                    d="M3 4.5h10M6.5 4.5V3a1 1 0 0 1 1-1h1a1 1 0 0 1 1 1v1.5M4.5 4.5v8.5a1 1 0 0 0 1 1h5a1 1 0 0 0 1-1V4.5M6.5 7.5v4M9.5 7.5v4"
                  />
                </svg>
              </button>
            </div>
          </li>
        </ul>
        <p v-else class="presets-dialog__empty">No Presets saved yet.</p>
      </div>
    </section>
  </dialog>
</template>

<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed, ref, watch } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";
import type { Preset } from "../../types";
import { gitHostFromUrl, repoNameFromGitUrl } from "../../utils/gitUrl";
import CloseButton from "../CloseButton/CloseButton.vue";

const props = withDefaults(
  defineProps<{
    open: boolean;
    presets: Preset[];
    installedSkillNames: string[];
    error?: string | null;
  }>(),
  { error: null },
);

const emit = defineEmits<{
  "update:open": [boolean];
  save: [name: string, gitUrl: string | null];
  load: [presetId: string];
  update: [presetId: string];
  delete: [presetId: string];
}>();

const dialogEl = ref<HTMLDialogElement | null>(null);
const name = ref("");
const gitUrl = ref("");

useNativeDialog(dialogEl, () => props.open);

// A light autocomplete, not a binding: only fills the Name field while it's
// still empty, and only from what's actually typed so far — the moment the
// user types anything into Name themselves, this stops touching it.
watch(gitUrl, (url) => {
  if (name.value.trim()) return;
  const derived = repoNameFromGitUrl(url);
  if (derived) name.value = derived;
});

const canSave = computed(() => name.value.trim().length > 0 && props.installedSkillNames.length > 0);

function handleSave() {
  if (!canSave.value) return;
  emit("save", name.value.trim(), gitUrl.value.trim() || null);
  name.value = "";
  gitUrl.value = "";
}

// Opens in the system's default browser, not the app's own webview — a
// plain `<a href>` would otherwise just try to navigate this window itself.
// Best-effort: the URL is still shown as text either way, so a failure here
// (no browser configured, sandboxed environment) isn't worth surfacing.
function handleOpenGitUrl(url: string) {
  openUrl(url).catch(() => undefined);
}

function requestUpdate(preset: Preset) {
  const count = props.installedSkillNames.length;
  if (
    window.confirm(
      `Replace “${preset.name}”'s ${preset.skillNames.length} saved Skill${preset.skillNames.length === 1 ? "" : "s"} with the ${count} currently installed?`,
    )
  ) {
    emit("update", preset.id);
  }
}

function requestDelete(preset: Preset) {
  if (window.confirm(`Delete “${preset.name}”? This only removes the saved Preset, not any installed Skills.`)) {
    emit("delete", preset.id);
  }
}

function close() {
  dialogEl.value?.close();
}
function onBackdrop(event: MouseEvent) {
  if (event.target === dialogEl.value) close();
}
</script>

<style scoped lang="scss" src="./PresetsDialog.scss"></style>
