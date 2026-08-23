import type { useAppState } from "../src/composables/useAppState";
import { defaultPreferences } from "../src/types";

type AppState = ReturnType<typeof useAppState>;

/**
 * `useAppState` is a module-level singleton (by design — see its own
 * comment), so tests that run in the same worker must reset it between
 * cases instead of relying on a fresh module instance. Kept in the test
 * tree rather than exported from production code.
 */
export function resetState(state: AppState) {
  state.skills = [];
  state.groups = [];
  state.tags = [];
  state.selectedSkillIds = new Set();
  state.skillsWithUpdates = new Set();
  state.searchQuery = "";
  state.skillBeingEditedId = null;
  state.groupBeingEditedId = null;
  state.isAddDialogOpen = false;
  state.preferences = defaultPreferences();
  state.dependencyStatus = null;
  state.installation = {
    isInstalling: false,
    currentSkillId: null,
    currentIndex: 0,
    total: 0,
    outputLines: [],
    perSkillStatus: {},
    displayNames: {},
    result: null,
    error: null,
  };
  state.sourcePath = null;
  state.isEmbeddedDefault = false;
  state.projectRoot = "";
  state.loaded = false;
}
