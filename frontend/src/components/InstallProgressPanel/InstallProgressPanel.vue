<template>
  <section class="install-progress-panel" aria-live="polite">
    <header class="install-progress-panel__header">
      <button
        type="button"
        class="install-progress-panel__toggle"
        :aria-expanded="expanded"
        @click="expanded = !expanded"
      >
        <svg
          class="install-progress-panel__chevron"
          :class="{ 'is-expanded': expanded }"
          viewBox="0 0 16 16"
          aria-hidden="true"
        >
          <path
            d="M6 4l4 4-4 4"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>{{ headline }}</span>
      </button>
      <div class="install-progress-panel__actions">
        <button
          v-if="installation.isInstalling"
          type="button"
          class="install-progress-panel__cancel"
          @click="emit('cancel')"
        >
          Cancel
        </button>
        <CloseButton v-else aria-label="Dismiss installation output" @click="emit('dismiss')" />
      </div>
    </header>

    <div
      v-if="expanded"
      ref="outputElement"
      class="install-progress-panel__body scroll-x"
      aria-label="Installation command output"
    >
      <template v-for="skillId in orderedSkillIds" :key="skillId">
        <p class="install-progress-panel__line install-progress-panel__line--heading">
          <span class="install-progress-panel__status-icon" :class="statusClass(skillId)">
            {{ statusIcon(skillId) }}
          </span>
          {{ installation.displayNames[skillId] ?? skillId }}
        </p>
        <p
          v-for="(line, i) in outputFor(skillId)"
          :key="i"
          class="install-progress-panel__line install-progress-panel__line--output"
          :class="`is-${line.stream}`"
        >
          {{ line.line }}
        </p>
      </template>
    </div>

    <ul v-if="installation.result" class="install-progress-panel__results">
      <li v-for="skillId in orderedSkillIds" :key="skillId" class="install-progress-panel__result-row">
        <span class="install-progress-panel__status-icon" :class="statusClass(skillId)">
          {{ statusIcon(skillId) }}
        </span>
        <span class="install-progress-panel__result-name">{{ installation.displayNames[skillId] ?? skillId }}</span>
        <span
          v-if="installedAgentsFor(skillId).length"
          class="install-progress-panel__result-agents"
          :title="`Installed for: ${formatAgents(installedAgentsFor(skillId))}`"
        >
          <AgentIcon v-for="id in installedAgentsFor(skillId)" :key="id" :agent-id="id" />
        </span>
      </li>
    </ul>

    <p v-if="installation.result" class="install-progress-panel__summary">
      {{ installation.result.requested }} requested, {{ installation.result.installed }} installed,
      {{ installation.result.alreadyInstalled }} already installed, {{ installation.result.failed }} failed
    </p>
    <p
      v-if="installation.error"
      class="install-progress-panel__summary install-progress-panel__summary--error"
      role="alert"
    >
      {{ installation.error }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";

import type { InstallationState } from "../../composables/useAppState";
import type { Skill } from "../../types";
import { formatAgents } from "../../utils/agents";
import AgentIcon from "../AgentIcon/AgentIcon.vue";
import CloseButton from "../CloseButton/CloseButton.vue";

const props = withDefaults(
  defineProps<{
    installation: InstallationState;
    skills?: Skill[];
  }>(),
  { skills: () => [] },
);

const emit = defineEmits<{
  cancel: [];
  dismiss: [];
}>();

const expanded = ref(true);
const outputElement = ref<HTMLElement | null>(null);

const headline = computed(() => {
  if (props.installation.isInstalling) {
    return `Installing ${props.installation.currentIndex}/${props.installation.total}...`;
  }
  if (props.installation.result) {
    return `Installed ${props.installation.result.installed}/${props.installation.result.requested}`;
  }
  return "Installation";
});

const orderedSkillIds = computed(() => Object.keys(props.installation.perSkillStatus));

function statusIcon(skillId: string) {
  const status = props.installation.perSkillStatus[skillId];
  if (status === "installed" || status === "alreadyInstalled") return "✓";
  if (status === "failed") return "✗";
  return "…";
}

function statusClass(skillId: string) {
  const status = props.installation.perSkillStatus[skillId];
  if (status === "installed" || status === "alreadyInstalled") return "is-success";
  if (status === "failed") return "is-failed";
  return "is-pending";
}

function outputFor(skillId: string) {
  return props.installation.outputLines.filter((line) => line.skillId === skillId);
}

// `refresh()` (called right after the install finishes, before this result
// is dismissed — see App.vue's runInstall) has already brought `skills` up
// to date, so this is the skill's *current* installed-for list, not a
// snapshot of what the request asked for.
function installedAgentsFor(skillId: string): string[] {
  return props.skills.find((s) => s.id === skillId)?.installedAgents ?? [];
}

watch(
  () => [props.installation.outputLines.length, expanded.value],
  async () => {
    await nextTick();
    if (expanded.value && outputElement.value) {
      outputElement.value.scrollTop = outputElement.value.scrollHeight;
    }
  },
);
</script>

<style scoped lang="scss" src="./InstallProgressPanel.scss"></style>
