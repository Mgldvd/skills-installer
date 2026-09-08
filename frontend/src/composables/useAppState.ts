import { reactive } from "vue";

import type {
  DependencyStatus,
  InstallResult,
  OutputStream,
  Preset,
  Skill,
  SkillGroup,
  SkillTag,
  UiPreferences,
  UnrecognizedSkill,
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
  /** Display names of skills found installed on disk (for the active
   * project/scope) that match no known Skill — installed by some other
   * means and not tracked by this app's catalog. Purely informational; see
   * `useSkills.loadAll`/`refresh`. */
  unrecognizedSkills: UnrecognizedSkill[];
  groups: SkillGroup[];
  tags: SkillTag[];
  selectedSkillIds: Set<string>;
  /** Local-only: ids of installed Local skills whose catalog `.signature` has
   * changed since install — populated by `useSkills.checkForUpdates`, which
   * runs both from the "Check for Updates" button and once automatically
   * after the initial load (see `App.vue`). */
  skillsWithUpdates: Set<string>;
  /** Whether the Local Skill Source catalog is a git repo, from the same
   * check — `null` until that check has run at least once. Drives the
   * "not versioned" / "you have uncommitted changes" nudges. */
  localCatalogVersioned: boolean | null;
  localCatalogDirtySkillNames: string[];
  searchQuery: string;
  skillBeingEditedId: string | null;
  groupBeingEditedId: string | null;
  isAddDialogOpen: boolean;
  preferences: UiPreferences;
  /** Saved skill preselections, independent of any one project folder —
   * see `usePresets`. */
  presets: Preset[];
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
  unrecognizedSkills: [],
  groups: [],
  tags: [],
  selectedSkillIds: new Set<string>(),
  skillsWithUpdates: new Set<string>(),
  localCatalogVersioned: null,
  localCatalogDirtySkillNames: [],
  searchQuery: "",
  skillBeingEditedId: null,
  groupBeingEditedId: null,
  isAddDialogOpen: false,
  preferences: defaultPreferences(),
  presets: [],
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
