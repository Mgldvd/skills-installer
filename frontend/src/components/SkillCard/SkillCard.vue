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
      'is-delete-mode': deleteMode,
      'is-delete-selected': deleteMode && deleteSelected,
    }"
  >
    <button
      v-if="deleteMode"
      type="button"
      class="skill-card__selection-surface"
      :aria-pressed="deleteSelected"
      :aria-label="deleteAriaLabel"
      :disabled="!skill.installed"
      @click="emit('toggle-delete', skill.id)"
    />
    <button
      v-else
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
      <div v-if="showUpdateButton" class="skill-card__meta">
        <button
          type="button"
          class="skill-card__update"
          :aria-label="`Update ${skill.displayName} to the local version`"
          title="Update to the local version"
          @click="emit('update', skill.id)"
        >
          Update
        </button>
      </div>
      <div class="skill-card__actions">
        <div
          v-if="collapsedAgentIcons.length || assignedTags.length"
          class="skill-card__agents"
          aria-label="Agent install status and assigned packs"
        >
          <span
            v-for="entry in collapsedAgentIcons"
            :key="entry.id"
            class="skill-card__agent-icon"
            :class="{ 'is-missing': entry.missing, 'is-partial': entry.partial }"
            :title="entry.title"
          >
            <AgentIcon :agent-id="entry.id" />
          </span>
          <PackBadge
            v-for="tag in assignedTags"
            :key="tag.id"
            :name="tag.name"
            :color="tag.color"
            :title="tag.name"
            icon-only
          />
        </div>
        <SourceIcon class="skill-card__source" :local="skill.local" />
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
      </div>
    </footer>
  </article>
</template>

<script setup lang="ts">
import { computed } from "vue";

import type { Skill, SkillTag } from "../../types";
import { agentLabel, isUniversalGroup } from "../../utils/agents";
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
    /** Bulk-uninstall mode (the footer's trash-can toggle) — while active,
     * the card's whole selection surface switches from "select to install"
     * to "select to uninstall": only installed Skills are selectable, and
     * the highlight is always red (`is-delete-selected`), never the
     * configurable accent color, so the two selection modes are never
     * visually ambiguous. */
    deleteMode?: boolean;
    deleteSelected?: boolean;
  }>(),
  {
    targetAgents: () => [],
    tags: () => [],
    compact: false,
    list: false,
    installing: false,
    hasUpdate: false,
    deleteMode: false,
    deleteSelected: false,
  },
);

const emit = defineEmits<{
  toggle: [skillId: string];
  edit: [skillId: string];
  update: [skillId: string];
  "toggle-delete": [skillId: string];
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
const deleteAriaLabel = computed(() => {
  if (!props.skill.installed) return `${props.skill.displayName}, not installed, nothing to uninstall`;
  return `${props.skill.displayName}, ${props.deleteSelected ? "selected for uninstall" : "not selected for uninstall"}`;
});
const assignedTags = computed(() => props.tags.filter((tag) => props.skill.tags.includes(tag.id)));
const missingAgentsTitle = computed(() => `Not yet installed for: ${missingAgents.value.map(agentLabel).join(", ")}`);
// Installed agents first, then — only on a card that's actually showing the
// "Missing N agents" badge — the still-missing target agents appended so the
// footer surfaces the whole gap. A skill with nothing installed yet has no
// gap to call out here; it keeps the original installed-only row. This is
// the full, uncollapsed breakdown — see EditSkillDialog for where it's
// shown in full; the card itself only shows `collapsedAgentIcons`.
const displayedAgents = computed(() => [
  ...props.skill.installedAgents,
  ...(isPartiallyInstalled.value ? missingAgents.value.filter((id) => !props.skill.installedAgents.includes(id)) : []),
]);
// `.agents/skills` (and similar shared conventions) mean installing once
// can mark half a dozen agent ids installed simultaneously — see
// discovery.rs's compat map — which is exactly what was cluttering the
// card with icons. Collapse every agent that shares Universal's own folder
// into one Universal icon; only agents with a genuinely distinct
// destination (Claude Code, Windsurf, Pi, ...) still get their own icon.
// The full per-agent truth stays available in `displayedAgents` for the
// Skill's detail view.
const collapsedAgentIcons = computed(() => {
  const universalMembers = displayedAgents.value.filter((id) => isUniversalGroup(id));
  const ownFolderAgents = displayedAgents.value.filter((id) => !isUniversalGroup(id));
  const entries: { id: string; missing: boolean; partial: boolean; title: string }[] = [];
  if (universalMembers.length) {
    const missingMembers = universalMembers.filter((id) => missingAgents.value.includes(id));
    const installedMembers = universalMembers.filter((id) => !missingAgents.value.includes(id));
    // Mixed state is possible (e.g. Cursor's own cross-compat reading of
    // `.claude/skills` covers it while Gemini CLI, which doesn't read that
    // directory, stays missing) — any real coverage reads as "installed", so
    // the icon keeps its normal look rather than the full `is-missing`
    // treatment, but `is-partial` still adds a small dot: real coverage
    // doesn't mean *every* target agent in this group actually has it.
    const isMissing = installedMembers.length === 0;
    const isPartial = !isMissing && missingMembers.length > 0;
    const title = isMissing
      ? `Not yet installed for: ${missingMembers.map(agentLabel).join(", ")}`
      : `Installed for: ${installedMembers.map(agentLabel).join(", ")}` +
        (missingMembers.length ? ` (not yet for: ${missingMembers.map(agentLabel).join(", ")})` : "");
    entries.push({ id: "universal", missing: isMissing, partial: isPartial, title });
  }
  for (const id of ownFolderAgents) {
    const missing = missingAgents.value.includes(id);
    entries.push({
      id,
      missing,
      partial: false,
      title: missing ? `Not yet installed for ${agentLabel(id)}` : `Installed for ${agentLabel(id)}`,
    });
  }
  return entries;
});
// Defensive against a stale `skillsWithUpdates` (e.g. after switching the
// selected project folder changes what's installed): only ever show the
// Update button for a skill that's still both local and installed.
const showUpdateButton = computed(() => props.hasUpdate && props.skill.local && props.skill.installed);
</script>

<style scoped lang="scss" src="./SkillCard.scss"></style>
