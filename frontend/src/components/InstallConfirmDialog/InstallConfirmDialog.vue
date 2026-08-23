<template>
  <dialog ref="dialogEl" class="install-confirm-dialog" @close="emit('update:open', false)">
    <div class="install-confirm-dialog__body">
      <h2 class="install-confirm-dialog__title">
        Install {{ skills.length }} skill{{ skills.length === 1 ? "" : "s" }}?
      </h2>

      <dl class="install-confirm-dialog__summary">
        <div>
          <dt>Destination</dt>
          <dd>{{ options.scope === "global" ? "Global installation" : projectPath }}</dd>
        </div>
        <div>
          <dt>Method</dt>
          <dd>{{ options.copy ? "Copy" : "Link" }}</dd>
        </div>
      </dl>

      <div class="install-confirm-dialog__agents">
        <span class="install-confirm-dialog__agents-label">Installing for</span>
        <button
          v-for="id in options.agents"
          :key="id"
          type="button"
          class="install-confirm-dialog__agent-chip"
          :class="{ 'is-off': !activeAgents.includes(id) }"
          :aria-pressed="activeAgents.includes(id)"
          :title="activeAgents.includes(id) ? `Skip ${agentLabel(id)} for this install` : `Include ${agentLabel(id)} again`"
          @click="toggleAgent(id)">
          <AgentIcon :agent-id="id" />
          {{ agentLabel(id) }}
        </button>
      </div>
      <p v-if="activeAgents.length === 0" class="install-confirm-dialog__error" role="alert">
        Select at least one agent to install for.
      </p>

      <ul class="install-confirm-dialog__skills">
        <li v-for="skill in skills" :key="skill.id" class="install-confirm-dialog__skill-row">
          <span class="install-confirm-dialog__skill-name">{{ skill.displayName }}</span>
          <span class="install-confirm-dialog__skill-icons">
            <span
              v-for="id in activeAgents"
              :key="id"
              class="install-confirm-dialog__skill-icon"
              :class="{ 'is-new': !skill.installedAgents.includes(id) }"
              :title="`${agentLabel(id)}${skill.installedAgents.includes(id) ? ' — already installed' : ' — new'}`">
              <AgentIcon :agent-id="id" />
            </span>
          </span>
        </li>
      </ul>
      <p v-if="gapFillCount" class="install-confirm-dialog__meta">
        {{ gapFillCount }} of these are already installed for some agents — highlighted icons show what's being added.
      </p>

      <div class="install-confirm-dialog__actions">
        <button type="button" class="install-confirm-dialog__btn" @click="close">Cancel</button>
        <button
          type="button"
          class="install-confirm-dialog__btn install-confirm-dialog__btn--primary"
          :disabled="activeAgents.length === 0"
          @click="handleConfirm">
          Install Selected
        </button>
      </div>
    </div>
  </dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";

import type { InstallOptions, Skill } from "../../types";
import { agentLabel } from "../../utils/agents";
import AgentIcon from "../AgentIcon/AgentIcon.vue";

const props = defineProps<{
  open: boolean;
  skills: Skill[];
  options: InstallOptions;
  projectPath: string;
}>();

// How many selected skills already have at least one of the targeted agents
// — the "is-new" highlight on each row's icons is what actually shows what
// changes for them; this line just explains that highlight exists.
const gapFillCount = computed(() => props.skills.filter((skill) => skill.installedAgents.length > 0).length);

const emit = defineEmits<{
  "update:open": [value: boolean];
  confirm: [agents: string[]];
}>();

const dialogEl = ref<HTMLDialogElement | null>(null);

useNativeDialog(dialogEl, () => props.open);

// A per-install override of `options.agents`, not a change to the user's
// configured defaults — deselecting an agent chip here only skips it for
// this one confirmation. Reset from the current defaults every time the
// dialog opens, so a previous run's exclusions don't linger silently.
const activeAgents = ref<string[]>([...props.options.agents]);
watch(
  () => props.open,
  (open) => {
    if (open) activeAgents.value = [...props.options.agents];
  },
);

function toggleAgent(id: string) {
  activeAgents.value = activeAgents.value.includes(id)
    ? activeAgents.value.filter((agentId) => agentId !== id)
    : [...activeAgents.value, id];
}

function close() {
  emit("update:open", false);
}

function handleConfirm() {
  if (activeAgents.value.length === 0) return;
  emit("confirm", [...activeAgents.value]);
  emit("update:open", false);
}
</script>

<style scoped lang="scss" src="./InstallConfirmDialog.scss"></style>
