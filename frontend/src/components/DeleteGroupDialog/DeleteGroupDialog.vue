<template>
    <dialog ref="dialogEl" class="delete-group-dialog" @close="emit('update:open', false)">
        <div class="delete-group-dialog__body">
            <h2 class="delete-group-dialog__title">Delete "{{ group?.name }}"?</h2>
            <p v-if="skillCount === 0" class="delete-group-dialog__message">
                This group is empty and can be deleted safely.
            </p>
            <template v-else>
                <p class="delete-group-dialog__message">
                    This group still has {{ skillCount }} skill{{ skillCount === 1 ? "" : "s" }}. Choose what happens to
                    them — skills are never deleted along with their group.
                </p>
                <label class="delete-group-dialog__option">
                    <input v-model="strategy" type="radio" value="move-to-other" />
                    <span>Move skills to "Other"</span>
                </label>
                <label class="delete-group-dialog__option">
                    <input v-model="strategy" type="radio" value="move-to" :disabled="otherGroups.length === 0" />
                    <span>Move skills to another group</span>
                </label>
                <select v-if="strategy === 'move-to'" v-model="targetGroupId" class="delete-group-dialog__select">
                    <option v-for="g in otherGroups" :key="g.id" :value="g.id">
                        {{ g.name }}
                    </option>
                </select>
            </template>
            <div class="delete-group-dialog__actions">
                <button type="button" class="delete-group-dialog__btn" @click="close">Cancel</button>
                <button
                    type="button"
                    class="delete-group-dialog__btn delete-group-dialog__btn--danger"
                    :disabled="!canConfirm"
                    @click="handleConfirm">
                    Delete Group
                </button>
            </div>
        </div>
    </dialog>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";
import type { SkillGroup } from "../../types";

export type DeleteGroupStrategyResult =
    | { kind: "require-empty" }
    | { kind: "move-to-other" }
    | { kind: "move-to"; targetGroupId: string };

const props = defineProps<{
    open: boolean;
    group: SkillGroup | null;
    skillCount: number;
    otherGroups: SkillGroup[];
}>();

const emit = defineEmits<{
    "update:open": [value: boolean];
    confirm: [strategy: DeleteGroupStrategyResult];
}>();

const dialogEl = ref<HTMLDialogElement | null>(null);
const strategy = ref<"move-to-other" | "move-to">("move-to-other");
const targetGroupId = ref<string | null>(null);

const canConfirm = computed(() => {
    if (props.skillCount === 0) return true;
    if (strategy.value === "move-to-other") return true;
    return Boolean(targetGroupId.value);
});

function resetForm() {
    strategy.value = "move-to-other";
    targetGroupId.value = props.otherGroups[0]?.id ?? null;
}

useNativeDialog(dialogEl, () => props.open, resetForm);

function close() {
    emit("update:open", false);
}

function handleConfirm() {
    if (!canConfirm.value) return;
    if (props.skillCount === 0) {
        emit("confirm", { kind: "require-empty" });
    } else if (strategy.value === "move-to-other") {
        emit("confirm", { kind: "move-to-other" });
    } else if (targetGroupId.value) {
        emit("confirm", {
            kind: "move-to",
            targetGroupId: targetGroupId.value,
        });
    }
    emit("update:open", false);
}
</script>

<style scoped lang="scss" src="./DeleteGroupDialog.scss"></style>
