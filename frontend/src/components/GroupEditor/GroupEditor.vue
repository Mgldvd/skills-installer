<template>
  <dialog ref="dialogEl" class="group-editor" @close="emit('update:open', false)">
    <form class="group-editor__form" @submit.prevent="handleSubmit">
      <h2 class="group-editor__title">
        {{ mode === "create" ? "Create Group" : "Edit Group" }}
      </h2>

      <label class="group-editor__field">
        <span>Group name</span>
        <input v-model="name" type="text" required class="group-editor__input" placeholder="e.g. Frontend" />
      </label>

      <div class="group-editor__field">
        <span>Color</span>
        <div class="group-editor__swatches" role="radiogroup" aria-label="Group color">
          <button
            v-for="swatch in palette"
            :key="swatch.value"
            type="button"
            class="group-editor__swatch"
            :class="{
              'is-selected': isSameColor(color, swatch.value),
            }"
            :style="{ backgroundColor: swatch.value }"
            :aria-label="swatch.label"
            role="radio"
            :aria-checked="isSameColor(color, swatch.value)"
            @click="color = swatch.value"
          >
            <svg
              v-if="isSameColor(color, swatch.value)"
              viewBox="0 0 16 16"
              class="group-editor__swatch-check"
              aria-hidden="true"
            >
              <path
                d="M3 8.5l3 3 7-7"
                fill="none"
                stroke="white"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>
        </div>
        <input
          v-model="color"
          type="text"
          class="group-editor__input group-editor__hex-input"
          placeholder="#RRGGBB"
          pattern="^#[0-9A-Fa-f]{6}$"
        />
      </div>

      <label v-if="mode === 'edit'" class="group-editor__checkbox">
        <input v-model="enabled" type="checkbox" />
        <span>Enabled</span>
      </label>

      <p v-if="errorMessage" class="group-editor__error" role="alert">
        {{ errorMessage }}
      </p>

      <div class="group-editor__actions">
        <button
          v-if="mode === 'edit' && canDelete"
          type="button"
          class="group-editor__btn group-editor__btn--danger group-editor__btn--delete"
          @click="handleDelete"
        >
          Delete Group
        </button>
        <button type="button" class="group-editor__btn" @click="close">Cancel</button>
        <button type="submit" class="group-editor__btn group-editor__btn--primary" :disabled="!isValid">
          {{ mode === "create" ? "Create Group" : "Save Changes" }}
        </button>
      </div>
    </form>
  </dialog>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";
import type { SkillGroup } from "../../types";

const props = withDefaults(
  defineProps<{
    open: boolean;
    mode: "create" | "edit";
    group?: SkillGroup | null;
    errorMessage?: string | null;
    canDelete?: boolean;
  }>(),
  { group: null, errorMessage: null, canDelete: true },
);

const emit = defineEmits<{
  "update:open": [value: boolean];
  submit: [payload: { name: string; color: string; enabled: boolean }];
  delete: [groupId: string];
}>();

function handleDelete() {
  if (props.group) emit("delete", props.group.id);
}

// Mirrors `config::color::CURATED_PALETTE` in the Rust backend.
const palette = [
  { label: "Pink", value: "#F43F75" },
  { label: "Red / Coral", value: "#F05252" },
  { label: "Orange", value: "#F97316" },
  { label: "Amber", value: "#F59E0B" },
  { label: "Green", value: "#22C55E" },
  { label: "Teal", value: "#14B8A6" },
  { label: "Cyan", value: "#06B6D4" },
  { label: "Blue", value: "#3B82F6" },
  { label: "Indigo", value: "#6366F1" },
  { label: "Violet", value: "#A855F7" },
];

const dialogEl = ref<HTMLDialogElement | null>(null);
const name = ref("");
const color = ref(palette[0].value);
const enabled = ref(true);

const isValid = computed(() => name.value.trim().length > 0 && /^#[0-9A-Fa-f]{6}$/.test(color.value));

function isSameColor(a: string, b: string) {
  return a.toLowerCase() === b.toLowerCase();
}

function resetForm() {
  name.value = props.group?.name ?? "";
  color.value = props.group?.color ?? palette[0].value;
  enabled.value = props.group?.enabled ?? true;
}

useNativeDialog(dialogEl, () => props.open, resetForm);

function close() {
  emit("update:open", false);
}

function handleSubmit() {
  if (!isValid.value) return;
  emit("submit", {
    name: name.value.trim(),
    color: color.value,
    enabled: enabled.value,
  });
}
</script>

<style scoped lang="scss" src="./GroupEditor.scss"></style>
