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
  enabled: boolean;
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
