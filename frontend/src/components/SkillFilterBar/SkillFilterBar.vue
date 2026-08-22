<template>
  <section class="skill-filter-bar" aria-label="Skill sort and filters">
    <div class="skill-filter-bar__group skill-filter-bar__group--sort">
      <span class="skill-filter-bar__label">Sort</span>
      <label class="skill-filter-bar__sort">
        <span class="sr-only">Sort by</span>
        <select
          :value="sortBy"
          aria-label="Sort Skills"
          @change="emit('update:sortBy', ($event.target as HTMLSelectElement).value as SortMode)">
          <option value="name">Name</option>
          <option value="pack">Pack</option>
          <option value="local">Local</option>
          <option value="remote">Remote</option>
        </select>
      </label>
      <div class="skill-filter-bar__view" role="group" aria-label="Skill view">
        <button
          type="button"
          :aria-pressed="view === 'grid'"
          title="Grid view"
          aria-label="Grid view"
          @click="emit('update:view', 'grid')">
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <rect x="1.5" y="1.5" width="5.5" height="5.5" rx="1" />
            <rect x="9" y="1.5" width="5.5" height="5.5" rx="1" />
            <rect x="1.5" y="9" width="5.5" height="5.5" rx="1" />
            <rect x="9" y="9" width="5.5" height="5.5" rx="1" />
          </svg>
        </button>
        <button
          type="button"
          :aria-pressed="view === 'list'"
          title="List view"
          aria-label="List view"
          @click="emit('update:view', 'list')">
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <rect x="1.5" y="2.25" width="13" height="2.5" rx="1" />
            <rect x="1.5" y="6.75" width="13" height="2.5" rx="1" />
            <rect x="1.5" y="11.25" width="13" height="2.5" rx="1" />
          </svg>
        </button>
      </div>
    </div>

    <div class="skill-filter-bar__group skill-filter-bar__group--filter">
      <span class="skill-filter-bar__label">Filter</span>
      <label class="skill-filter-bar__search">
        <span class="sr-only">Filter Skills</span>
        <svg viewBox="0 0 20 20" aria-hidden="true">
          <circle cx="8.5" cy="8.5" r="5.5" /><path d="m13 13 4 4" />
        </svg>
        <input
          type="search"
          :value="query"
          placeholder="Filter skills..."
          @input="emit('update:query', ($event.target as HTMLInputElement).value)" />
      </label>
      <div class="skill-filter-bar__source" role="group" aria-label="Filter by source">
        <button
          type="button"
          :aria-pressed="sourceFilter === 'local'"
          @click="emit('update:sourceFilter', sourceFilter === 'local' ? 'all' : 'local')">
          Local
        </button>
        <button
          type="button"
          :aria-pressed="sourceFilter === 'remote'"
          @click="emit('update:sourceFilter', sourceFilter === 'remote' ? 'all' : 'remote')">
          Remote
        </button>
      </div>
      <label class="skill-filter-bar__pack">
        <span class="sr-only">Filter by Pack</span>
        <select
          :value="packFilter ?? ''"
          aria-label="Filter by Pack"
          @change="emit('update:packFilter', ($event.target as HTMLSelectElement).value || null)">
          <option value="">All Packs</option>
          <option v-for="tag in tags" :key="tag.id" :value="tag.id">{{ tag.name }}</option>
        </select>
      </label>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { SkillTag } from "../../types";

type SortMode = "name" | "pack" | "local" | "remote";
type ViewMode = "grid" | "list";
type SourceFilter = "all" | "local" | "remote";

withDefaults(
  defineProps<{
    query?: string;
    sortBy?: SortMode;
    view?: ViewMode;
    sourceFilter?: SourceFilter;
    packFilter?: string | null;
    tags?: SkillTag[];
  }>(),
  {
    query: "",
    sortBy: "name",
    view: "grid",
    sourceFilter: "all",
    packFilter: null,
    tags: () => [],
  },
);

const emit = defineEmits<{
  "update:query": [value: string];
  "update:sortBy": [value: SortMode];
  "update:view": [value: ViewMode];
  "update:sourceFilter": [value: SourceFilter];
  "update:packFilter": [value: string | null];
}>();
</script>

<style scoped lang="scss" src="./SkillFilterBar.scss"></style>
