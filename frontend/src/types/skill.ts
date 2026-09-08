export type SkillSource = { kind: "remote" } | { kind: "local"; path: string };

export interface Skill {
  id: string;
  name: string;
  displayName: string;
  description: string;
  source: SkillSource;
  repository: string;
  repositoryUrl: string;
  skillName: string;
  skillsUrl: string;
  groupId: string;
  tags: string[];
  preselected: boolean;
  local: boolean;
  installed: boolean;
  installedAgents: string[];
  enabled: boolean;
}

/** A skill found installed on disk that matches no known catalog skill —
 * see `ApplicationConfig.unrecognizedSkills`. `path` is what "copy into
 * catalog" sends back to the `copyUnrecognizedSkill` backend call. */
export interface UnrecognizedSkill {
  skillName: string;
  displayName: string;
  path: string;
}

export interface InstalledSkill {
  name: string;
  displayName: string;
  description: string;
  path: string;
  agent: string | null;
}

export interface SkillSelection {
  skillIds: string[];
}

/** Result of `checkLocalSkillUpdates` — see `useSkills.checkForUpdates`.
 * `catalogVersioned` is `null` when there's no catalog directory to check
 * at all (nothing to suggest `git init`-ing). */
export interface LocalUpdatesReport {
  outdatedSkillIds: string[];
  catalogVersioned: boolean | null;
  catalogDirtySkillNames: string[];
}
