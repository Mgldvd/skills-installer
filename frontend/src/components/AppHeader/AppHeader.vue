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
    <button
      type="button"
      class="app-header__item app-header__agents"
      aria-label="Open Agents settings"
      title="Configure Agents"
      @click="emit('open-agents')">
      <span class="app-header__label">Agents</span>
      <strong class="app-header__value">{{ agentLabels }}</strong>
    </button>
    <div class="app-header__status-row">
      <span class="app-header__dependency" :class="dependencyClass">
        <span class="app-header__dependency-dot" aria-hidden="true" />
        Skills CLI {{ dependencyLabel }}
      </span>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import appLogo from "../../../../icon.png";
import * as backend from "../../services/backend";
import { SUPPORTED_AGENTS, type DependencyStatus, type InstallScope } from "../../types";

const props = defineProps<{
  projectPath: string;
  dependencyStatus: DependencyStatus | null;
  scope: InstallScope;
  agents: string[];
}>();
const emit = defineEmits<{
  "update:projectPath": [path: string];
  "open-agents": [];
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
const agentLabels = computed(
  () =>
    props.agents.map((id) => SUPPORTED_AGENTS.find((agent) => agent.id === id)?.label ?? id).join(", ") ||
    "None selected",
);

const dependencyClass = computed(() => {
  if (!props.dependencyStatus) return "is-unknown";
  return props.dependencyStatus.available ? "is-ready" : "is-missing";
});

const dependencyLabel = computed(() => {
  if (!props.dependencyStatus) return "checking…";
  return props.dependencyStatus.available ? "● Ready" : "● Not found";
});
</script>

<style scoped lang="scss" src="./AppHeader.scss"></style>
