<template>
  <article
    class="skill-card"
    :class="{
      'is-selected': selected,
      'is-disabled': !skill.enabled,
      'is-compact': compact,
      'is-list': list,
      'is-installed': skill.installed,
      'is-partially-installed': isPartiallyInstalled,
      'is-installing': installing,
    }"
  >
    <button
      type="button"
      class="skill-card__selection-surface"
      :aria-pressed="selected"
      :aria-label="ariaLabel"
      :disabled="!skill.enabled || isFullyInstalled"
      @click="emit('toggle', skill.id)"
    />

    <span class="skill-card__selection-ribbon" aria-hidden="true" />

    <header class="skill-card__top">
      <h3 class="skill-card__title">{{ skill.displayName }}</h3>
      <!-- A fully-installed, non-partial card stays quiet here on purpose:
           the green ribbon/border already say "done" — a stale `installing`
           prop lingering a beat after completion shouldn't flash a spinner
           back on top of it. -->
      <span v-if="installing && !isFullyInstalled" class="skill-card__installing">
        <span class="skill-card__spinner" aria-hidden="true" />
        Installing…
      </span>
      <span v-else-if="isPartiallyInstalled" class="skill-card__partial" :title="missingAgentsTitle">
        Missing {{ missingAgents.length }} agent{{ missingAgents.length === 1 ? "" : "s" }}
      </span>
    </header>

    <div v-if="!compact && !list" class="skill-card__description">
      <span class="skill-card__description-text">{{ skill.description || "No description provided." }}</span>
    </div>

    <footer class="skill-card__footer">
      <div class="skill-card__meta">
        <SourceIcon class="skill-card__source" :local="skill.local" />
        <button
          v-if="showUpdateButton"
          type="button"
          class="skill-card__update"
          :aria-label="`Update ${skill.displayName} to the local version`"
          title="Update to the local version"
          @click="emit('update', skill.id)"
        >
          Update
        </button>
        <div v-if="assignedTags.length" class="skill-card__packs" aria-label="Assigned packs">
          <PackBadge v-for="tag in assignedTags" :key="tag.id" :name="tag.name" :color="tag.color" compact />
        </div>
      </div>
      <div v-if="skill.installedAgents.length" class="skill-card__agents" :title="installedAgentsTitle" aria-label="Installed for">
        <span v-for="id in skill.installedAgents" :key="id" class="skill-card__agent-icon">
          <AgentIcon :agent-id="id" />
        </span>
      </div>
      <button
        type="button"
        class="skill-card__edit"
        :aria-label="`View and edit ${skill.displayName}`"
        title="View and edit Skill"
        @click="emit('edit', skill.id)"
      >
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
import { agentLabel } from "../../utils/agents";
import { agentsNeedingInstall } from "../../utils/skillInstall";
import AgentIcon from "../AgentIcon/AgentIcon.vue";
import PackBadge from "../PackBadge/PackBadge.vue";
import SourceIcon from "../SourceIcon/SourceIcon.vue";

const props = withDefaults(
  defineProps<{
    skill: Skill;
    selected: boolean;
    targetAgents?: string[];
    tags?: SkillTag[];
    compact?: boolean;
    list?: boolean;
    installing?: boolean;
    hasUpdate?: boolean;
  }>(),
  { targetAgents: () => [], tags: () => [], compact: false, list: false, installing: false, hasUpdate: false },
);

const emit = defineEmits<{
  toggle: [skillId: string];
  edit: [skillId: string];
  update: [skillId: string];
}>();

// Installed for *some* but not *every* currently targeted agent — still
// selectable, so re-installing can close the gap (see `agentsNeedingInstall`).
const missingAgents = computed(() => agentsNeedingInstall(props.skill, props.targetAgents));
const isPartiallyInstalled = computed(() => props.skill.installed && missingAgents.value.length > 0);
const isFullyInstalled = computed(() => props.skill.installed && !isPartiallyInstalled.value);

const ariaLabel = computed(() => {
  if (isFullyInstalled.value) return `${props.skill.displayName}, already installed`;
  if (props.installing) return `${props.skill.displayName}, installing`;
  if (isPartiallyInstalled.value) {
    return `${props.skill.displayName}, installed for ${props.skill.installedAgents
      .map(agentLabel)
      .join(", ")}, not yet installed for ${missingAgents.value.map(agentLabel).join(", ")}`;
  }
  return `${props.skill.displayName}${props.selected ? ", selected" : ", not selected"}`;
});
const assignedTags = computed(() => props.tags.filter((tag) => props.skill.tags.includes(tag.id)));
const installedAgentsTitle = computed(() => `Installed for: ${props.skill.installedAgents.map(agentLabel).join(", ")}`);
const missingAgentsTitle = computed(() => `Not yet installed for: ${missingAgents.value.map(agentLabel).join(", ")}`);
// Defensive against a stale `skillsWithUpdates` (e.g. after switching the
// selected project folder changes what's installed): only ever show the
// Update button for a skill that's still both local and installed.
const showUpdateButton = computed(() => props.hasUpdate && props.skill.local && props.skill.installed);
</script>

<style scoped lang="scss" src="./SkillCard.scss"></style>
