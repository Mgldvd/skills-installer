<template>
  <div ref="rootEl" class="group-selector" @keydown.escape="isOpen = false">
    <button
      type="button"
      class="group-selector__trigger"
      :aria-expanded="isOpen"
      aria-haspopup="listbox"
      @click="toggle"
    >
      <GroupBadge v-if="selectedGroup" :name="selectedGroup.name" :color="selectedGroup.color" size="sm" />
      <span v-else class="group-selector__placeholder">Select a group</span>
      <svg class="group-selector__chevron" viewBox="0 0 16 16" aria-hidden="true">
        <path
          d="M4 6l4 4 4-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <ul v-if="isOpen" class="group-selector__list" role="listbox">
      <li v-for="group in groups" :key="group.id" role="option" :aria-selected="group.id === modelValue">
        <button type="button" class="group-selector__option" @click="select(group.id)">
          <GroupBadge :name="group.name" :color="group.color" size="sm" />
        </button>
      </li>
      <li class="group-selector__divider" role="separator"></li>
      <li role="option">
        <button type="button" class="group-selector__option group-selector__option--create" @click="handleCreateNew">
          + Create new group
        </button>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";

import type { SkillGroup } from "../../types";
import GroupBadge from "../GroupBadge/GroupBadge.vue";

const props = defineProps<{
  groups: SkillGroup[];
  modelValue: string | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  createNew: [];
}>();

const isOpen = ref(false);
const rootEl = ref<HTMLElement | null>(null);

const selectedGroup = computed(() => props.groups.find((g) => g.id === props.modelValue) ?? null);

function toggle() {
  isOpen.value = !isOpen.value;
}

function select(groupId: string) {
  emit("update:modelValue", groupId);
  isOpen.value = false;
}

function handleCreateNew() {
  isOpen.value = false;
  emit("createNew");
}

function handleDocumentClick(event: MouseEvent) {
  if (isOpen.value && rootEl.value && !rootEl.value.contains(event.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => document.addEventListener("click", handleDocumentClick));
onUnmounted(() => document.removeEventListener("click", handleDocumentClick));
</script>

<style scoped lang="scss" src="./GroupSelector.scss"></style>
