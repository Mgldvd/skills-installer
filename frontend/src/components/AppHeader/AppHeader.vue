<template>
  <header class="app-header">
    <div class="app-header__brand">
      <img class="app-header__logo" :src="appLogo" alt="" aria-hidden="true" />
      <h1 class="app-header__title">Skills Installer</h1>
    </div>
    <div class="app-header__item app-header__destination">
      <span class="app-header__label">Install to</span>
      <button
        type="button"
        class="app-header__value app-header__path"
        :disabled="scope === 'global' || selectingPath"
        :title="scope === 'global' ? 'Change to Project scope to select a folder' : 'Select installation folder'"
        aria-label="Select project installation folder"
        @click="selectPath">
        {{ scope === "global" ? "Global installation" : projectPath || "(select a project)" }}
        <svg v-if="scope === 'project'" viewBox="0 0 16 16" aria-hidden="true">
          <path d="M1.8 4.5h4l1.3 1.6h7.1v6.7H1.8V4.5Zm0 1.6V3.2h4.7l1.3 1.3" />
        </svg>
      </button>
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
}>();
const selectingPath = ref(false);
async function selectPath() {
  if (props.scope === "global" || selectingPath.value) return;
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
