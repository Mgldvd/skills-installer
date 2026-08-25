<template>
  <header class="app-header" :class="{ 'app-header--global': scope === 'global' }">
    <div class="app-header__brand">
      <img class="app-header__logo" :src="appLogo" alt="" aria-hidden="true" />
      <h1 class="app-header__title">Skills Installer</h1>
      <span v-if="scope === 'global'" class="app-header__global-badge">Global</span>
    </div>

    <div class="app-header__item app-header__scope-switch" role="group" aria-label="Installation scope">
      <button
        type="button"
        class="app-header__scope-btn"
        :class="{ 'is-active': scope === 'project' }"
        :aria-pressed="scope === 'project'"
        title="Install into the selected project folder"
        @click="emit('update:scope', 'project')"
      >
        Project
      </button>
      <button
        type="button"
        class="app-header__scope-btn"
        :class="{ 'is-active': scope === 'global' }"
        :aria-pressed="scope === 'global'"
        title="Install into your home directory, for every project"
        @click="emit('update:scope', 'global')"
      >
        Global
      </button>
    </div>

    <div v-if="scope === 'project'" class="app-header__item app-header__destination">
      <span class="app-header__label">Project folder</span>
      <button
        type="button"
        class="app-header__value app-header__path"
        :disabled="selectingPath"
        title="Select installation folder"
        aria-label="Select project installation folder"
        @click="selectPath"
      >
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M1.8 4.5h4l1.3 1.6h7.1v6.7H1.8V4.5Zm0 1.6V3.2h4.7l1.3 1.3" />
        </svg>
        {{ projectPath || "(select a project)" }}
      </button>
    </div>
    <div v-else class="app-header__item app-header__destination app-header__destination--global">
      <span class="app-header__label">Destination</span>
      <span class="app-header__value">Your home directory — every project</span>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref } from "vue";

import appLogo from "../../../../icon.png";
import * as backend from "../../services/backend";
import type { InstallScope } from "../../types";

const props = defineProps<{
  projectPath: string;
  scope: InstallScope;
}>();
const emit = defineEmits<{
  "update:projectPath": [path: string];
  "update:scope": [scope: InstallScope];
}>();
const selectingPath = ref(false);
async function selectPath() {
  if (selectingPath.value) return;
  selectingPath.value = true;
  try {
    const path = await backend.selectInstallationDirectory(props.projectPath);
    if (path) emit("update:projectPath", path);
  } finally {
    selectingPath.value = false;
  }
}
</script>

<style scoped lang="scss" src="./AppHeader.scss"></style>
