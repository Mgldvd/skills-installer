<template>
    <dialog ref="dialogEl" class="agents-dialog" @close="emit('update:open', false)" @click="handleBackdropClick">
        <section class="agents-dialog__panel">
            <header class="agents-dialog__header">
                <div>
                    <h2>Agents</h2>
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
                <div class="agents-dialog__list">
                    <button
                        v-for="agent in SUPPORTED_AGENTS"
                        :key="agent.id"
                        type="button"
                        class="agents-dialog__toggle"
                        :class="{ 'is-active': selected.includes(agent.id) }"
                        :aria-pressed="selected.includes(agent.id)"
                        @click="toggle(agent.id)">
                        <span class="agents-dialog__check" aria-hidden="true">
                            {{ selected.includes(agent.id) ? "✓" : "" }}
                        </span>
                        <span>
                            <strong>{{ agent.label }}</strong>
                            <small>{{ scope === "global" ? agent.globalPath : agent.projectPath }}</small>
                        </span>
                    </button>
                </div>
                <p v-if="selected.length === 0" class="agents-dialog__error" role="alert">Select at least one agent.</p>
            </div>
        </section>
    </dialog>
</template>
<script setup lang="ts">
import { ref, watch } from "vue";
import { useNativeDialog } from "../../composables/useNativeDialog";
import CloseButton from "../CloseButton/CloseButton.vue";
import { SUPPORTED_AGENTS, type InstallScope } from "../../types";
const props = defineProps<{
    open: boolean;
    modelValue: string[];
    scope: InstallScope;
}>();
const emit = defineEmits<{
    "update:open": [boolean];
    "update:modelValue": [string[]];
}>();
const dialogEl = ref<HTMLDialogElement | null>(null),
    selected = ref<string[]>([...props.modelValue]);
watch(
    () => props.open,
    (open) => {
        if (open) selected.value = [...props.modelValue];
    },
);
useNativeDialog(dialogEl, () => props.open);
function toggle(id: string) {
    selected.value = selected.value.includes(id)
        ? selected.value.filter((item) => item !== id)
        : [...selected.value, id];
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
