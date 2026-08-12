<template>
  <article class="skill-card" :class="{ 'is-selected': selected, 'is-disabled': !skill.enabled }">
    <button
      type="button"
      class="skill-card__selection-surface"
      :aria-pressed="selected"
      :aria-label="ariaLabel"
      :disabled="!skill.enabled"
      @click="emit('toggle', skill.id)"
    />

    <span class="skill-card__selection-ribbon" aria-hidden="true" />

    <header class="skill-card__top">
      <h3 class="skill-card__title">{{ skill.displayName }}</h3>
      <span v-if="skill.installed" class="skill-card__installed">Installed</span>
    </header>

    <button type="button" class="skill-card__description" @click="emit('description', skill.id)">
      <span class="skill-card__description-label">Description</span>
      <span class="skill-card__description-text">{{ skill.description || 'No description provided.' }}</span>
      <span class="skill-card__description-more">View details</span>
    </button>

    <footer class="skill-card__footer">
      <div class="skill-card__meta">
        <span v-if="skill.local" class="skill-card__local">Local</span>
        <div v-if="assignedTags.length" class="skill-card__packs" aria-label="Assigned packs">
          <PackBadge v-for="tag in assignedTags" :key="tag.id" :name="tag.name" :color="tag.color" compact />
        </div>
      </div>
      <button type="button" class="skill-card__edit" @click="emit('edit', skill.id)">
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M3 11.8V13h1.2l7.1-7.1-1.2-1.2L3 11.8Zm8-8 1.2-1.2 1.2 1.2L12.2 5 11 3.8Z" fill="currentColor" />
        </svg>
        Edit
      </button>
    </footer>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { Skill, SkillTag } from '../../types'
import PackBadge from '../PackBadge/PackBadge.vue'

const props = withDefaults(defineProps<{
  skill: Skill
  selected: boolean
  tags?: SkillTag[]
}>(), { tags: () => [] })

const emit = defineEmits<{
  toggle: [skillId: string]
  edit: [skillId: string]
  description: [skillId: string]
}>()

const ariaLabel = computed(() => `${props.skill.displayName}${props.selected ? ', selected' : ', not selected'}`)
const assignedTags = computed(() => props.tags.filter((tag) => props.skill.tags.includes(tag.id)))
</script>

<style scoped lang="scss" src="./SkillCard.scss"></style>
