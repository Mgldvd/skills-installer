<template>
  <dialog ref="dialogEl" class="agents-dialog" @close="emit('update:open', false)" @click="handleBackdropClick">
    <section class="agents-dialog__panel">
      <header class="agents-dialog__header" :class="{ 'agents-dialog__header--global': scope === 'global' }">
        <div>
          <h2>
            Agents
            <span v-if="scope === 'global'" class="agents-dialog__global-badge">Global</span>
          </h2>
          <p>Choose the agents that receive every installation.</p>
        </div>
        <div class="agents-dialog__header-actions">
          <button type="button" class="agents-dialog__done" :disabled="selected.length === 0" @click="save">
            Done
          </button>
          <CloseButton aria-label="Close Agents" @click="close" />
        </div>
      </header>
      <div class="agents-dialog__body">
        <p class="agents-dialog__hint">Drag by the handle to reorder — the same order shows on every Skill card.</p>
        <p v-if="universalGroupAgents.length" class="agents-dialog__legend">
          <span class="agents-dialog__legend-icon">
            <AgentIcon agent-id="universal" />
          </span>
          <span class="agents-dialog__legend-equals">=</span>
          <span
            v-for="agent in universalGroupAgents"
            :key="agent.id"
            class="agents-dialog__legend-icon"
            :title="agent.label"
          >
            <AgentIcon :agent-id="agent.id" />
          </span>
          <span class="agents-dialog__legend-text">
            share Universal's folder — a Skill card collapses all of these into one Universal icon.
          </span>
        </p>
        <div class="agents-dialog__list">
          <div
            v-for="agent in orderedAgents"
            :key="agent.id"
            class="agents-dialog__toggle"
            :class="{ 'is-active': selected.includes(agent.id), 'is-dragging': draggedId === agent.id }"
            role="button"
            tabindex="0"
            :aria-pressed="selected.includes(agent.id)"
            :title="agent.id === 'universal' && isUniversalIndeterminate ? 'Already covered by another selected agent that shares this folder' : undefined"
            @click="toggle(agent.id)"
            @keydown.enter.prevent="toggle(agent.id)"
            @keydown.space.prevent="toggle(agent.id)"
            @dragover.prevent="handleDragOver(agent.id)"
            @drop.prevent
          >
            <span
              class="agents-dialog__grip"
              aria-hidden="true"
              draggable="true"
              :title="`Drag to reorder ${agent.label}`"
              @click.stop
              @dragstart="handleDragStart(agent.id, $event)"
              @dragend="handleDragEnd"
            >
              ⠿
            </span>
            <span
              class="agents-dialog__check"
              :class="{ 'is-indeterminate': agent.id === 'universal' && isUniversalIndeterminate }"
              aria-hidden="true"
            >
              {{ checkMark(agent.id) }}
            </span>
            <span class="agents-dialog__icon">
              <AgentIcon :agent-id="agent.id" />
            </span>
            <span class="agents-dialog__info">
              <strong>{{ agent.label }}</strong>
              <span class="agents-dialog__paths">
                <span
                  class="agents-dialog__path"
                  :class="{ 'agents-dialog__path--own': !isUniversalGroup(agent.id) }"
                  :title="`Project: ${agent.projectPath}`"
                >
                  Project: {{ agent.projectPath }}
                </span>
                <span
                  class="agents-dialog__path"
                  :class="{ 'agents-dialog__path--own': !isUniversalGroup(agent.id) }"
                  :title="`Global: ${agent.globalPath}`"
                >
                  Global: {{ agent.globalPath }}
                </span>
              </span>
            </span>
          </div>
        </div>
        <p v-if="selected.length === 0" class="agents-dialog__error" role="alert">Select at least one agent.</p>
      </div>
    </section>
  </dialog>
</template>
<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useNativeDialog } from "../../composables/useNativeDialog";
import { isUniversalGroup, resolveAgentOrder } from "../../utils/agents";
import AgentIcon from "../AgentIcon/AgentIcon.vue";
import CloseButton from "../CloseButton/CloseButton.vue";
import { SUPPORTED_AGENTS, type InstallScope } from "../../types";
const props = defineProps<{
  open: boolean;
  modelValue: string[];
  agentOrder: string[];
  scope: InstallScope;
}>();
const emit = defineEmits<{
  "update:open": [boolean];
  "update:modelValue": [string[]];
  "update:agentOrder": [string[]];
}>();
const dialogEl = ref<HTMLDialogElement | null>(null),
  selected = ref<string[]>([...props.modelValue]),
  order = ref<string[]>(resolveAgentOrder(props.agentOrder));
watch(
  () => props.open,
  (open) => {
    if (open) {
      selected.value = [...props.modelValue];
      order.value = resolveAgentOrder(props.agentOrder);
    }
  },
);
const orderedAgents = computed(() => order.value.map((id) => SUPPORTED_AGENTS.find((agent) => agent.id === id)!));
useNativeDialog(dialogEl, () => props.open);
function toggle(id: string) {
  selected.value = selected.value.includes(id) ? selected.value.filter((item) => item !== id) : [...selected.value, id];
}
// Live-reorders while dragging (standard drag-and-drop feel); the actual
// preference write only happens once, on drop — see `handleDragEnd`.
const draggedId = ref<string | null>(null);
function handleDragStart(id: string, event: DragEvent) {
  draggedId.value = id;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", id);
  }
}
function handleDragOver(overId: string) {
  const draggedAgentId = draggedId.value;
  if (!draggedAgentId || draggedAgentId === overId) return;
  const from = order.value.indexOf(draggedAgentId);
  const to = order.value.indexOf(overId);
  if (from === -1 || to === -1 || from === to) return;
  const next = [...order.value];
  next.splice(from, 1);
  next.splice(to, 0, draggedAgentId);
  order.value = next;
}
function handleDragEnd() {
  if (draggedId.value) emit("update:agentOrder", order.value);
  draggedId.value = null;
}
// Agents (other than Universal itself) that share Universal's folder — a
// static fact about each agent's own documented convention, independent of
// which scopes are currently enabled — see `isUniversalGroup`. Selecting
// one effectively installs for Universal too, even when Universal isn't
// explicitly checked.
const universalGroupAgents = computed(() =>
  SUPPORTED_AGENTS.filter((agent) => agent.id !== "universal" && isUniversalGroup(agent.id)),
);
const isUniversalIndeterminate = computed(
  () =>
    !selected.value.includes("universal") &&
    selected.value.some((id) => id !== "universal" && isUniversalGroup(id)),
);
function checkMark(agentId: string): string {
  if (selected.value.includes(agentId)) return "✓";
  if (agentId === "universal" && isUniversalIndeterminate.value) return "–";
  return "";
}
function close() {
  emit("update:open", false);
}
function handleBackdropClick(event: MouseEvent) {
  if (event.target === dialogEl.value) close();
}
function save() {
  if (selected.value.length) {
    emit("update:modelValue", selected.value);
    close();
  }
}
</script>
<style scoped lang="scss" src="./AgentsDialog.scss"></style>
