<template>
  <dialog ref="dialogEl" class="skills-dialog" @close="emit('update:open', false)">
    <section class="skills-dialog__panel">
      <header>
        <div>
          <h2>Skills</h2>
          <p>Maintain the curated catalog available for installation.</p>
        </div>
        <CloseButton aria-label="Close Skills" @click="close" />
      </header>
      <div class="skills-dialog__heading">
        <h3>Curated Skills</h3>
        <button type="button" class="primary" @click="emit('add')">+ Add Skill</button>
      </div>
      <div class="skills-dialog__list">
        <div v-for="skill in skills" :key="skill.id">
          <span>
            <strong>{{ skill.displayName }}</strong>
            <small>{{ skill.local ? "Local source" : skill.skillsUrl }}</small>
          </span>
          <span v-if="!skill.local" class="skills-dialog__actions">
            <button type="button" @click="emit('edit', skill.id)">Edit</button>
            <button type="button" @click="emit('remove', skill.id)">Remove</button>
          </span>
        </div>
      </div>
      <div class="skills-dialog__source">
        <h3>Local Skill Source</h3>
        <p>This folder is a catalog of installable Skills, not an installation destination.</p>
        <label>
          <span>Source folder</span>
          <input v-model="source" type="text" placeholder="~/.control/skill" />
        </label>
        <div>
          <button type="button" @click="saveSource">Change Folder</button>
          <button type="button" @click="emit('refresh')">Refresh</button>
        </div>
      </div>
    </section>
  </dialog>
</template>
<script setup lang="ts">
import { ref, watch } from "vue";
import { useNativeDialog } from "../../composables/useNativeDialog";
import CloseButton from "../CloseButton/CloseButton.vue";
import type { Skill } from "../../types";
const props = defineProps<{
  open: boolean;
  skills: Skill[];
  localSourcePath: string | null;
}>();
const emit = defineEmits<{
  "update:open": [boolean];
  add: [];
  edit: [string];
  remove: [string];
  refresh: [];
  updateLocalSource: [string];
}>();
const dialogEl = ref<HTMLDialogElement | null>(null),
  source = ref("");
watch(
  () => props.open,
  (open) => {
    if (open) source.value = props.localSourcePath ?? "~/.control/skill";
  },
);
useNativeDialog(dialogEl, () => props.open);
function close() {
  emit("update:open", false);
}
function saveSource() {
  if (source.value.trim()) emit("updateLocalSource", source.value.trim());
}
</script>
<style scoped lang="scss" src="./SkillsDialog.scss"></style>
