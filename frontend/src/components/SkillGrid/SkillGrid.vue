<template>
  <div class="skill-grid" role="list">
    <p v-if="skills.length === 0" class="skill-grid__empty">No skills match the current search and filter.</p>
    <div v-for="skill in skills" :key="skill.id" role="listitem">
      <SkillCard
        :skill="skill"
        :selected="selectedIds.has(skill.id)"
        :edit-mode="editMode"
        @toggle="(id) => emit('toggle', id)"
        @edit="(id) => emit('edit', id)"
        @more="(id) => emit('more', id)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Skill } from '../../types'
import SkillCard from '../SkillCard/SkillCard.vue'

defineProps<{
  skills: Skill[]
  selectedIds: Set<string>
  editMode: boolean
}>()

const emit = defineEmits<{
  toggle: [skillId: string]
  edit: [skillId: string]
  more: [skillId: string]
}>()
</script>

<style scoped lang="scss" src="./SkillGrid.scss"></style>
