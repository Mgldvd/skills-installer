<template>
  <dialog ref="dialogEl" class="install-confirm-dialog" @close="emit('update:open', false)">
    <div class="install-confirm-dialog__body">
      <h2 class="install-confirm-dialog__title">
        Install {{ skills.length }} skill{{ skills.length === 1 ? "" : "s" }}?
      </h2>
      <dl class="install-confirm-dialog__summary">
        <div>
          <dt>Destination</dt>
          <dd>
            {{ options.scope === "global" ? "Global installation" : projectPath }}
          </dd>
        </div>
        <div>
          <dt>Agents</dt>
          <dd>{{ agentLabels }}</dd>
        </div>
        <div>
          <dt>Skills</dt>
          <dd>{{ skills.length }} selected</dd>
        </div>
        <div>
          <dt>Method</dt>
          <dd>{{ options.copy ? "Copy" : "Link" }}</dd>
        </div>
      </dl>
      <div class="install-confirm-dialog__actions">
        <button type="button" class="install-confirm-dialog__btn" @click="close">Cancel</button>
        <button
          type="button"
          class="install-confirm-dialog__btn install-confirm-dialog__btn--primary"
          @click="handleConfirm">
          Install Selected
        </button>
      </div>
    </div>
  </dialog>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";

import { SUPPORTED_AGENTS, type InstallOptions, type Skill } from "../../types";

const props = defineProps<{
  open: boolean;
  skills: Skill[];
  options: InstallOptions;
  projectPath: string;
}>();
const agentLabels = computed(() =>
  props.options.agents.map((id) => SUPPORTED_AGENTS.find((a) => a.id === id)?.label ?? id).join(", "),
);

const emit = defineEmits<{
  "update:open": [value: boolean];
  confirm: [];
}>();

const dialogEl = ref<HTMLDialogElement | null>(null);

useNativeDialog(dialogEl, () => props.open);

function close() {
  emit("update:open", false);
}

function handleConfirm() {
  emit("confirm");
  emit("update:open", false);
}
</script>

<style scoped lang="scss" src="./InstallConfirmDialog.scss"></style>
