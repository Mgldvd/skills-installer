<template>
    <dialog ref="dialogEl" class="confirm-dialog" @close="emit('update:open', false)">
        <form method="dialog" class="confirm-dialog__form" @submit.prevent>
            <h2 class="confirm-dialog__title">{{ title }}</h2>
            <p class="confirm-dialog__message">{{ message }}</p>
            <div class="confirm-dialog__actions">
                <button type="button" class="confirm-dialog__btn" @click="handleCancel">
                    {{ cancelLabel }}
                </button>
                <button
                    type="button"
                    class="confirm-dialog__btn confirm-dialog__btn--primary"
                    :class="{ 'confirm-dialog__btn--danger': destructive }"
                    @click="handleConfirm">
                    {{ confirmLabel }}
                </button>
            </div>
        </form>
    </dialog>
</template>

<script setup lang="ts">
import { ref } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";

const props = withDefaults(
    defineProps<{
        open: boolean;
        title: string;
        message: string;
        confirmLabel?: string;
        cancelLabel?: string;
        destructive?: boolean;
    }>(),
    { confirmLabel: "Confirm", cancelLabel: "Cancel", destructive: false },
);

const emit = defineEmits<{
    "update:open": [value: boolean];
    confirm: [];
    cancel: [];
}>();

const dialogEl = ref<HTMLDialogElement | null>(null);

// Escape always cancels, never confirms — the native <dialog> `cancel`
// event already does this before falling through to `close`, so a
// destructive action can never be triggered by accident via the keyboard.
useNativeDialog(dialogEl, () => props.open);

function handleCancel() {
    emit("cancel");
    emit("update:open", false);
}

function handleConfirm() {
    emit("confirm");
    emit("update:open", false);
}
</script>

<style scoped lang="scss" src="./ConfirmDialog.scss"></style>
