import { computed } from "vue";

import type { Skill } from "../types";
import { useAppState } from "./useAppState";

function matchesSearch(skill: Skill, query: string): boolean {
  const haystacks = [skill.displayName, skill.skillName, skill.description, skill.repository, skill.skillsUrl];
  return haystacks.some((value) => value?.toLowerCase().includes(query));
}

/** Main-grid text search. Tags intentionally do not participate here. */
export function useFilters() {
  const state = useAppState();

  const filteredSkills = computed(() => {
    const query = state.searchQuery.trim().toLowerCase();
    if (!query) return state.skills;
    return state.skills.filter((skill) => matchesSearch(skill, query));
  });

  function setSearchQuery(query: string) {
    state.searchQuery = query;
  }

  return { filteredSkills, setSearchQuery };
}
