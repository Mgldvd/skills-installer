import type { InstallScope } from "./install";
import type { SkillGroup } from "./group";
import type { Skill, UnrecognizedSkill } from "./skill";
import type { SkillTag } from "./tag";

export interface ConfigDefaults {
  agent: string | null;
  copy: boolean;
  scope: InstallScope;
}

export interface ApplicationConfig {
  version: number;
  defaults: ConfigDefaults;
  groups: SkillGroup[];
  tags: SkillTag[];
  skills: Skill[];
  unrecognizedSkills: UnrecognizedSkill[];
  localSkillTags?: Record<string, string[]>;
  sourcePath: string | null;
  isEmbeddedDefault: boolean;
  projectRoot: string;
}
