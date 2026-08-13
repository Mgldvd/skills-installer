<template>
    <TransitionGroup
        name="skill-grid"
        tag="div"
        class="skill-grid"
        :class="{ 'skill-grid--compact': compact }"
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
                @toggle="(id) => emit('toggle', id)"
                @edit="(id) => emit('edit', id)" />
        </div>
    </TransitionGroup>
</template>

<script setup lang="ts">
import type { Skill, SkillTag } from "../../types";
import SkillCard from "../SkillCard/SkillCard.vue";

defineProps<{
    skills: Skill[];
    tags: SkillTag[];
    selectedIds: Set<string>;
    compact?: boolean;
}>();

const emit = defineEmits<{
    toggle: [skillId: string];
    edit: [skillId: string];
}>();
</script>

<style scoped lang="scss" src="./SkillGrid.scss"></style>
