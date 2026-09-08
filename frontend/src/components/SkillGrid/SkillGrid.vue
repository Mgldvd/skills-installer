<template>
  <div class="skill-grid-sections">
    <p
      v-if="skills.length === 0 && unrecognizedSkills.length === 0"
      class="skill-grid__empty"
    >
      No skills match the current search and filter.
    </p>
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

      <section
        v-show="unrecognizedSkills.length"
        class="skill-grid__section"
        aria-labelledby="skill-grid-section-unrecognized"
      >
        <h2 id="skill-grid-section-unrecognized" class="skill-grid__section-header">
          <span class="skill-grid__section-label">Installed but not in your catalog</span>
          <span class="skill-grid__section-count">{{ unrecognizedSkills.length }}</span>
        </h2>
        <ul class="skill-grid__unrecognized-list">
          <li
            v-for="skill in unrecognizedSkills"
            :key="skill.path"
            class="skill-grid__unrecognized-item"
          >
            <span class="skill-grid__unrecognized-name">{{ skill.displayName }}</span>
            <button
              type="button"
              class="skill-grid__unrecognized-copy"
              :disabled="copyingPath === skill.path"
              :aria-label="`Copy ${skill.displayName} into your catalog`"
              title="Copy into your catalog"
              @click="emit('copy-to-catalog', skill)"
            >
              <svg viewBox="0 0 16 16" aria-hidden="true">
                <rect x="6" y="6" width="8" height="8" rx="1" />
                <path d="M4 10V3a1 1 0 0 1 1-1h7" />
              </svg>
            </button>
          </li>
        </ul>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

import type { Skill, SkillTag, UnrecognizedSkill } from "../../types";
import SkillCard from "../SkillCard/SkillCard.vue";

const props = withDefaults(
  defineProps<{
    skills: Skill[];
    /** Skills found installed on disk that match no known Skill — rendered
     * as plain names with a "copy into catalog" action, deliberately not as
     * `SkillCard`s (see App.vue's `state.unrecognizedSkills`). */
    unrecognizedSkills?: UnrecognizedSkill[];
    /** The `path` of whichever unrecognized skill is currently being copied
     * (see App.vue's `copyingUnrecognizedPath`) — disables just that one
     * button so a slow copy can't be double-submitted. */
    copyingPath?: string | null;
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
    unrecognizedSkills: () => [],
    copyingPath: null,
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
  "copy-to-catalog": [skill: UnrecognizedSkill];
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
