<template>
  <div class="skill-grid-sections">
    <p v-if="skills.length === 0" class="skill-grid__empty">No skills match the current search and filter.</p>
    <template v-else>
      <section
        v-for="group in groups"
        v-show="group.skills.length"
        :key="group.key"
        class="skill-grid__section"
        :aria-labelledby="`skill-grid-section-${group.key}`"
      >
        <h2 :id="`skill-grid-section-${group.key}`" class="skill-grid__section-header">
          <span class="skill-grid__section-label">{{ group.label }}</span>
          <span class="skill-grid__section-count">{{ group.skills.length }}</span>
        </h2>
        <TransitionGroup
          name="skill-grid"
          tag="div"
          class="skill-grid"
          :class="{ 'skill-grid--compact': compact, 'skill-grid--list': view === 'list' }"
          role="list"
        >
          <div v-for="skill in group.skills" :key="skill.id" class="skill-grid__item" role="listitem">
            <SkillCard
              :skill="skill"
              :selected="selectedIds.has(skill.id)"
              :tags="tags"
              :compact="compact"
              :list="view === 'list'"
              :installing="skill.id === installingSkillId"
              :has-update="skillsWithUpdates.has(skill.id)"
              :target-agents="targetAgents"
              :agent-order="agentOrder"
              :delete-mode="deleteMode"
              :delete-selected="deleteSelectedIds.has(skill.id)"
              @toggle="(id) => emit('toggle', id)"
              @edit="(id) => emit('edit', id)"
              @update="(id) => emit('update', id)"
              @toggle-delete="(id) => emit('toggle-delete', id)"
            />
          </div>
        </TransitionGroup>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

import type { Skill, SkillTag } from "../../types";
import SkillCard from "../SkillCard/SkillCard.vue";

const props = withDefaults(
  defineProps<{
    skills: Skill[];
    tags: SkillTag[];
    selectedIds: Set<string>;
    compact?: boolean;
    installingSkillId?: string | null;
    skillsWithUpdates?: Set<string>;
    targetAgents: string[];
    agentOrder?: string[];
    view?: "grid" | "list";
    deleteMode?: boolean;
    deleteSelectedIds?: Set<string>;
  }>(),
  {
    installingSkillId: null,
    skillsWithUpdates: () => new Set<string>(),
    agentOrder: () => [],
    view: "grid",
    deleteMode: false,
    deleteSelectedIds: () => new Set<string>(),
  },
);

const emit = defineEmits<{
  toggle: [skillId: string];
  edit: [skillId: string];
  update: [skillId: string];
  "toggle-delete": [skillId: string];
}>();

// `props.skills` arrives already sorted (see App.vue's `displayedSkills`,
// which already leads with selected, then installed, then the rest, each
// block sorted the same way this split now visualizes) — filtering it into
// three buckets is a stable partition, so each section's relative order
// falls straight out of that existing sort instead of needing its own.
const forInstallSkills = computed(() => props.skills.filter((skill) => props.selectedIds.has(skill.id)));
const installedSkills = computed(
  () => props.skills.filter((skill) => !props.selectedIds.has(skill.id) && skill.installed),
);
const notInstalledSkills = computed(
  () => props.skills.filter((skill) => !props.selectedIds.has(skill.id) && !skill.installed),
);

const groups = computed(() => [
  { key: "for-install", label: "For install", skills: forInstallSkills.value },
  { key: "installed", label: "Installed", skills: installedSkills.value },
  { key: "not-installed", label: "Not installed", skills: notInstalledSkills.value },
]);
</script>

<style scoped lang="scss" src="./SkillGrid.scss"></style>
