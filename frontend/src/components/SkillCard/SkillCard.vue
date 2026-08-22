<template>
  <article
    class="skill-card"
    :class="{
      'is-selected': selected,
      'is-disabled': !skill.enabled,
      'is-compact': compact,
      'is-list': list,
      'is-installed': skill.installed,
      'is-installing': installing,
    }">
    <button
      type="button"
      class="skill-card__selection-surface"
      :aria-pressed="selected"
      :aria-label="ariaLabel"
      :disabled="!skill.enabled || skill.installed"
      @click="emit('toggle', skill.id)" />

    <span class="skill-card__selection-ribbon" aria-hidden="true" />

    <header class="skill-card__top">
      <h3 class="skill-card__title">{{ skill.displayName }}</h3>
      <span v-if="skill.installed" class="skill-card__installed">Installed</span>
      <span v-else-if="installing" class="skill-card__installing">
        <span class="skill-card__spinner" aria-hidden="true" />
        Installing…
      </span>
    </header>

    <div v-if="!compact && !list" class="skill-card__description">
      <span class="skill-card__description-text">{{ skill.description || "No description provided." }}</span>
    </div>

    <footer class="skill-card__footer">
      <div class="skill-card__meta">
        <span v-if="skill.local" class="skill-card__local">Local</span>
        <span v-else class="skill-card__remote">Remote</span>
        <div v-if="assignedTags.length" class="skill-card__packs" aria-label="Assigned packs">
          <PackBadge v-for="tag in assignedTags" :key="tag.id" :name="tag.name" :color="tag.color" compact />
        </div>
      </div>
      <button
        type="button"
        class="skill-card__edit"
        :aria-label="`View and edit ${skill.displayName}`"
        title="View and edit Skill"
        @click="emit('edit', skill.id)">
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M1.5 8s2.2-4 6.5-4 6.5 4 6.5 4-2.2 4-6.5 4-6.5-4-6.5-4Z" />
          <circle cx="8" cy="8" r="1.8" />
        </svg>
      </button>
    </footer>
  </article>
</template>

<script setup lang="ts">
import { computed } from "vue";

import type { Skill, SkillTag } from "../../types";
import PackBadge from "../PackBadge/PackBadge.vue";

const props = withDefaults(
  defineProps<{
    skill: Skill;
    selected: boolean;
    tags?: SkillTag[];
    compact?: boolean;
    list?: boolean;
    installing?: boolean;
  }>(),
  { tags: () => [], compact: false, list: false, installing: false },
);

const emit = defineEmits<{
  toggle: [skillId: string];
  edit: [skillId: string];
}>();

const ariaLabel = computed(() => {
  if (props.skill.installed) return `${props.skill.displayName}, already installed`;
  if (props.installing) return `${props.skill.displayName}, installing`;
  return `${props.skill.displayName}${props.selected ? ", selected" : ", not selected"}`;
});
const assignedTags = computed(() => props.tags.filter((tag) => props.skill.tags.includes(tag.id)));
</script>

<style scoped lang="scss" src="./SkillCard.scss"></style>
