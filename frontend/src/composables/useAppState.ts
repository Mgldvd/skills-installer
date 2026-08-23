import { reactive } from "vue";

import type {
  DependencyStatus,
  InstallResult,
  OutputStream,
  Skill,
  SkillGroup,
  SkillTag,
  UiPreferences,
} from "../types";
import { defaultPreferences } from "../types";

export type PerSkillInstallStatus = "pending" | "installed" | "alreadyInstalled" | "failed" | "skipped";

export interface InstallOutputLine {
  skillId: string;
  line: string;
  stream: OutputStream | "command";
}

export interface InstallationState {
  isInstalling: boolean;
  currentSkillId: string | null;
  currentIndex: number;
  total: number;
  outputLines: InstallOutputLine[];
  perSkillStatus: Record<string, PerSkillInstallStatus>;
  displayNames: Record<string, string>;
  result: InstallResult | null;
  error: string | null;
}

interface AppState {
  skills: Skill[];
  groups: SkillGroup[];
  tags: SkillTag[];
  selectedSkillIds: Set<string>;
  /** Local-only: ids of installed Local skills whose catalog `.signature` has
   * changed since install, populated only by an explicit "Check for Updates" —
   * never computed on load/refresh (see `useSkills.checkForUpdates`). */
  skillsWithUpdates: Set<string>;
  searchQuery: string;
  skillBeingEditedId: string | null;
  groupBeingEditedId: string | null;
  isAddDialogOpen: boolean;
  preferences: UiPreferences;
  dependencyStatus: DependencyStatus | null;
  installation: InstallationState;
  sourcePath: string | null;
  isEmbeddedDefault: boolean;
  projectRoot: string;
  loaded: boolean;
}

function freshInstallationState(): InstallationState {
  return {
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
}

/**
 * A single module-level reactive store, not Pinia — the app's state shape
 * (requirement's explicit list: skills, groups, selectedSkillIds,
 * searchQuery, ...) is flat enough that plain
 * `reactive()` plus composables is simpler than adding a state-management
 * dependency. Revisit only if cross-component reactivity actually becomes
 * unwieldy.
 */
const state = reactive<AppState>({
  skills: [],
  groups: [],
  tags: [],
  selectedSkillIds: new Set<string>(),
  skillsWithUpdates: new Set<string>(),
  searchQuery: "",
  skillBeingEditedId: null,
  groupBeingEditedId: null,
  isAddDialogOpen: false,
  preferences: defaultPreferences(),
  dependencyStatus: null,
  installation: freshInstallationState(),
  sourcePath: null,
  isEmbeddedDefault: false,
  projectRoot: "",
  loaded: false,
});

export function useAppState() {
  return state;
}

export function resetInstallationState() {
  state.installation = freshInstallationState();
}
