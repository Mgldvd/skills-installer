<template>
  <TransitionGroup
    name="skill-grid"
    tag="div"
    class="skill-grid"
    :class="{ 'skill-grid--compact': compact, 'skill-grid--list': view === 'list' }"
    role="list">
    <p v-if="skills.length === 0" key="empty" class="skill-grid__empty">
      No skills match the current search and filter.
    </p>
    <div v-for="skill in skills" :key="skill.id" class="skill-grid__item" role="listitem">
      <SkillCard
        :skill="skill"
        :selected="selectedIds.has(skill.id)"
        :tags="tags"
        :compact="compact"
        :list="view === 'list'"
        :installing="skill.id === installingSkillId"
        @toggle="(id) => emit('toggle', id)"
        @edit="(id) => emit('edit', id)" />
    </div>
  </TransitionGroup>
</template>

<script setup lang="ts">
import type { Skill, SkillTag } from "../../types";
import SkillCard from "../SkillCard/SkillCard.vue";

withDefaults(
  defineProps<{
    skills: Skill[];
    tags: SkillTag[];
    selectedIds: Set<string>;
    compact?: boolean;
    installingSkillId?: string | null;
    view?: "grid" | "list";
  }>(),
  { installingSkillId: null, view: "grid" },
);

const emit = defineEmits<{
  toggle: [skillId: string];
  edit: [skillId: string];
}>();
</script>

<style scoped lang="scss" src="./SkillGrid.scss"></style>
