import type { Skill, SkillGroup } from "../src/types";

export function makeGroup(overrides: Partial<SkillGroup> = {}): SkillGroup {
  return {
    id: "testing",
    name: "Testing",
    color: "#6B82D9",
    order: 10,
    enabled: true,
    ...overrides,
  };
}

export function makeSkill(overrides: Partial<Skill> = {}): Skill {
  return {
    id: "triage",
    name: "Issue Triage",
    displayName: "Issue Triage",
    description: "Helps triage development issues.",
    source: { kind: "remote" },
    repository: "skills",
    repositoryUrl: "https://github.com/mattpocock/skills",
    skillName: "triage",
    skillsUrl: "https://www.skills.sh/mattpocock/skills/triage",
    groupId: "testing",
    tags: ["issues", "workflow"],
    preselected: false,
    local: false,
    installed: false,
    installedAgents: [],
    enabled: true,
    ...overrides,
  };
}
