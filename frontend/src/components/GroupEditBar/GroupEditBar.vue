<template>
    <div v-if="groups.length" class="group-edit-bar scroll-x" role="list" aria-label="Manage groups">
        <span v-for="group in groups" :key="group.id" class="group-edit-bar__item" role="listitem">
            <span class="group-edit-bar__chip">
                <span class="group-edit-bar__dot" :style="{ backgroundColor: group.color }" aria-hidden="true" />
                <span class="group-edit-bar__label">{{ group.name }}</span>
                <span class="group-edit-bar__count">{{ group.count }}</span>
            </span>
            <button
                type="button"
                class="group-edit-bar__edit-btn"
                :aria-label="`Edit ${group.name} group`"
                @click="emit('editGroup', group.id)">
                ✎
            </button>
        </span>
    </div>
</template>

<script setup lang="ts">
export interface GroupChip {
    id: string;
    name: string;
    color: string;
    count: number;
}

defineProps<{
    groups: GroupChip[];
}>();

const emit = defineEmits<{
    editGroup: [groupId: string];
}>();
</script>

<style scoped lang="scss" src="./GroupEditBar.scss"></style>
