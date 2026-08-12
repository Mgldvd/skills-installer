<template>
  <div
    class="skill-card"
    :class="{ 'is-selected': selected, 'is-disabled': !skill.enabled }"
    role="button"
    tabindex="0"
    :aria-pressed="selected"
    :aria-label="ariaLabel"
    @click="handleActivate"
    @keydown.enter.prevent="handleActivate"
    @keydown.space.prevent="handleActivate"
  >
    <div class="skill-card__top">
      <h3 class="skill-card__title">{{ skill.displayName }}</h3>
      <span class="skill-card__selection-indicator" aria-hidden="true">
        <svg v-if="selected" viewBox="0 0 16 16" class="skill-card__check">
          <path
            d="M3 8.5l3 3 7-7"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </span>
    </div>

    <p class="skill-card__description">{{ skill.description || 'No description provided.' }}</p>

    <div v-if="skill.local || skill.installed" class="skill-card__meta">
      <span v-if="skill.local" class="skill-card__pill skill-card__pill--local">Local</span>
      <span v-if="skill.installed" class="skill-card__pill skill-card__pill--installed">✓ Installed</span>
    </div>

    <div v-if="editMode" class="skill-card__actions" @click.stop @keydown.stop>
      <button type="button" class="skill-card__action-btn" @click="emit('edit', skill.id)">Edit</button>
      <button type="button" class="skill-card__action-btn" @click="emit('more', skill.id)">More</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { Skill } from '../../types'

const props = defineProps<{
  skill: Skill
  selected: boolean
  editMode: boolean
}>()

const emit = defineEmits<{
  toggle: [skillId: string]
  edit: [skillId: string]
  more: [skillId: string]
}>()

const ariaLabel = computed(() => `${props.skill.displayName}${props.selected ? ', selected' : ''}`)

function handleActivate() {
  emit('toggle', props.skill.id)
}
</script>

<style scoped lang="scss" src="./SkillCard.scss"></style>
