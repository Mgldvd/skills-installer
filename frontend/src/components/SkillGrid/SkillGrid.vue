<template>
  <div class="skill-grid" role="list">
    <p v-if="skills.length === 0" class="skill-grid__empty">No skills match the current search and filter.</p>
    <div v-for="skill in skills" :key="skill.id" class="skill-grid__item" role="listitem">
      <SkillCard
        :skill="skill"
        :selected="selectedIds.has(skill.id)"
        :tags="tags"
        @toggle="(id) => emit('toggle', id)"
        @edit="(id) => emit('edit', id)"
        @description="(id) => emit('description', id)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Skill, SkillTag } from '../../types'
import SkillCard from '../SkillCard/SkillCard.vue'

defineProps<{
  skills: Skill[]
  tags: SkillTag[]
  selectedIds: Set<string>
}>()

const emit = defineEmits<{
  toggle: [skillId: string]
  edit: [skillId: string]
  description: [skillId: string]
}>()
</script>

<style scoped lang="scss" src="./SkillGrid.scss"></style>
