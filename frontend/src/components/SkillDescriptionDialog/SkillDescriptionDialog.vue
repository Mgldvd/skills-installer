<template>
  <dialog
    ref="dialogElement"
    class="skill-description-dialog"
    @close="emit('update:open', false)"
    @click="handleBackdropClick">
    <article v-if="skill" class="skill-description-dialog__content">
      <header class="skill-description-dialog__header">
        <div>
          <span class="skill-description-dialog__eyebrow">Skill description</span>
          <h2>{{ skill.displayName }}</h2>
        </div>
        <CloseButton aria-label="Close description" @click="close" />
      </header>

      <p class="skill-description-dialog__description">{{ skill.description || "No description provided." }}</p>

      <footer class="skill-description-dialog__footer">
        <a
          v-if="skill.skillsUrl"
          class="skill-description-dialog__link"
          :href="skill.skillsUrl"
          target="_blank"
          rel="noopener noreferrer">
          View on skills.sh
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              d="M6 3h7v7M13 3 5 11M11 9v4H3V5h4"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round" />
          </svg>
        </a>
        <span v-else class="skill-description-dialog__local">Local Skill — no skills.sh page available</span>
      </footer>
    </article>
  </dialog>
</template>

<script setup lang="ts">
import { ref } from "vue";

import { useNativeDialog } from "../../composables/useNativeDialog";
import type { Skill } from "../../types";
import CloseButton from "../CloseButton/CloseButton.vue";

const props = defineProps<{ open: boolean; skill: Skill | null }>();
const emit = defineEmits<{ "update:open": [value: boolean] }>();
const dialogElement = ref<HTMLDialogElement | null>(null);

useNativeDialog(dialogElement, () => props.open);

function close() {
  dialogElement.value?.close();
}
function handleBackdropClick(event: MouseEvent) {
  if (event.target === dialogElement.value) close();
}
</script>

<style scoped lang="scss" src="./SkillDescriptionDialog.scss"></style>
