// Sample data for the style guide only — shaped exactly like the real
// backend responses so every component below renders with the same props
// it receives in the real app, just without a Tauri backend behind it.
import type { InstallationState } from "../composables/useAppState";
import {
  defaultInstallOptions,
  defaultPreferences,
  type InstallOptions,
  type Preset,
  type Skill,
  type SkillGroup,
  type SkillTag,
  type UiPreferences,
} from "../types";

export const targetAgents = ["universal", "claude-code", "codex"];

export const mockGroups: SkillGroup[] = [
  { id: "frontend", name: "Frontend", color: "#3B82F6", order: 0, enabled: true },
  { id: "backend", name: "Backend", color: "#22C55E", order: 1, enabled: true },
  { id: "docs", name: "Documentation", color: "#F59E0B", order: 2, enabled: true },
  { id: "other", name: "Other", color: "#A855F7", order: 3, enabled: true },
];

export const mockTags: SkillTag[] = [
  { id: "tag-review", name: "Code Review", color: "#F43F75", order: 0, enabled: true },
  { id: "tag-testing", name: "Testing", color: "#14B8A6", order: 1, enabled: true },
  { id: "tag-writing", name: "Writing", color: "#A855F7", order: 2, enabled: true },
  { id: "tag-archived", name: "Archived", color: "#84CC16", order: 3, enabled: false },
];

export const mockSkills: Skill[] = [
  {
    id: "skill-code-review",
    name: "code-review",
    displayName: "Code Review",
    description: "Reviews a diff for correctness bugs and simplification opportunities before it ships.",
    source: { kind: "remote" },
    repository: "acme/skills",
    repositoryUrl: "https://github.com/acme/skills",
    skillName: "code-review",
    skillsUrl: "https://skills.sh/acme/skills/code-review",
    groupId: "frontend",
    tags: ["tag-review"],
    preselected: true,
    local: false,
    installed: true,
    installedAgents: ["universal", "claude-code", "codex"],
    enabled: true,
  },
  {
    id: "skill-tdd",
    name: "test-driven-development",
    displayName: "Test-Driven Development",
    description: "Drives feature work through a red-green-refactor loop with integration tests first.",
    source: { kind: "remote" },
    repository: "acme/skills",
    repositoryUrl: "https://github.com/acme/skills",
    skillName: "test-driven-development",
    skillsUrl: "https://skills.sh/acme/skills/test-driven-development",
    groupId: "backend",
    tags: ["tag-testing"],
    preselected: false,
    local: false,
    installed: true,
    installedAgents: ["universal"],
    enabled: true,
  },
  {
    id: "skill-style-guide",
    name: "style-guide",
    displayName: "Local Style Guide",
    description: "Enforces this repository's formatting and naming conventions on generated code.",
    source: { kind: "local", path: "~/.control/skill/style-guide" },
    repository: "",
    repositoryUrl: "",
    skillName: "style-guide",
    skillsUrl: "",
    groupId: "frontend",
    tags: ["tag-review"],
    preselected: false,
    local: true,
    installed: true,
    installedAgents: ["universal", "claude-code", "codex"],
    enabled: true,
  },
  {
    id: "skill-readme",
    name: "readme-instructions",
    displayName: "README Instructions",
    description: "",
    source: { kind: "remote" },
    repository: "acme/skills",
    repositoryUrl: "https://github.com/acme/skills",
    skillName: "readme-instructions",
    skillsUrl: "https://skills.sh/acme/skills/readme-instructions",
    groupId: "docs",
    tags: ["tag-writing"],
    preselected: false,
    local: false,
    installed: false,
    installedAgents: [],
    enabled: true,
  },
  {
    id: "skill-domain-modeling",
    name: "domain-modeling",
    displayName: "Domain Modeling",
    description: "Builds and sharpens a project's domain model and its CONTEXT.md.",
    source: { kind: "remote" },
    repository: "acme/skills",
    repositoryUrl: "https://github.com/acme/skills",
    skillName: "domain-modeling",
    skillsUrl: "https://skills.sh/acme/skills/domain-modeling",
    groupId: "backend",
    tags: [],
    preselected: true,
    local: false,
    installed: false,
    installedAgents: [],
    enabled: true,
  },
  {
    id: "skill-security-review",
    name: "security-review",
    displayName: "Security Review",
    description: "Completes a security review of the pending changes on the current branch.",
    source: { kind: "remote" },
    repository: "acme/skills",
    repositoryUrl: "https://github.com/acme/skills",
    skillName: "security-review",
    skillsUrl: "https://skills.sh/acme/skills/security-review",
    groupId: "backend",
    tags: ["tag-review", "tag-testing"],
    preselected: false,
    local: false,
    installed: true,
    installedAgents: ["claude-code"],
    enabled: true,
  },
  {
    id: "skill-legacy-agent",
    name: "legacy-agent-notes",
    displayName: "Legacy Agent Notes",
    description: "Disabled while its instructions are being rewritten for the new agent roster.",
    source: { kind: "remote" },
    repository: "acme/skills",
    repositoryUrl: "https://github.com/acme/skills",
    skillName: "legacy-agent-notes",
    skillsUrl: "https://skills.sh/acme/skills/legacy-agent-notes",
    groupId: "other",
    tags: ["tag-archived"],
    preselected: false,
    local: false,
    installed: false,
    installedAgents: [],
    enabled: false,
  },
];

export const mockSkillsWithUpdates = new Set<string>(["skill-style-guide"]);

export const mockPresets: Preset[] = [
  {
    id: "preset-marketing",
    name: "Marketing Site",
    gitUrl: "https://github.com/acme/marketing-site",
    skillNames: ["code-review", "readme-instructions"],
  },
  {
    id: "preset-internal-tools",
    name: "Internal Tools",
    gitUrl: "https://gitlab.com/acme/internal-tools",
    skillNames: ["test-driven-development", "security-review", "domain-modeling"],
  },
  {
    id: "preset-sandbox",
    name: "Sandbox",
    gitUrl: null,
    skillNames: [],
  },
];

export const mockPreferences: UiPreferences = {
  ...defaultPreferences(),
  defaultAgents: targetAgents,
};

export const mockInstallOptions: InstallOptions = {
  ...defaultInstallOptions(),
  agents: targetAgents,
};

export const mockInstallationInstalling: InstallationState = {
  isInstalling: true,
  currentSkillId: "skill-tdd",
  currentIndex: 2,
  total: 4,
  outputLines: [
    { skillId: "skill-code-review", line: "npx @skills-sh/cli add code-review --agent universal", stream: "command" },
    { skillId: "skill-code-review", line: "Fetching acme/skills/code-review...", stream: "stdout" },
    { skillId: "skill-code-review", line: "Installed to .agents/skills/code-review", stream: "stdout" },
    { skillId: "skill-tdd", line: "npx @skills-sh/cli add test-driven-development --agent universal", stream: "command" },
    { skillId: "skill-tdd", line: "Fetching acme/skills/test-driven-development...", stream: "stdout" },
  ],
  perSkillStatus: {
    "skill-code-review": "installed",
    "skill-tdd": "pending",
    "skill-domain-modeling": "pending",
    "skill-security-review": "pending",
  },
  displayNames: {
    "skill-code-review": "Code Review",
    "skill-tdd": "Test-Driven Development",
    "skill-domain-modeling": "Domain Modeling",
    "skill-security-review": "Security Review",
  },
  result: null,
  error: null,
};

export const mockInstallationSuccess: InstallationState = {
  isInstalling: false,
  currentSkillId: null,
  currentIndex: 4,
  total: 4,
  outputLines: mockInstallationInstalling.outputLines,
  perSkillStatus: {
    "skill-code-review": "installed",
    "skill-tdd": "installed",
    "skill-domain-modeling": "alreadyInstalled",
    "skill-security-review": "installed",
  },
  displayNames: mockInstallationInstalling.displayNames,
  result: {
    requested: 4,
    installed: 3,
    alreadyInstalled: 1,
    failed: 0,
    cancelled: false,
    perSkill: [
      {
        skillId: "skill-code-review",
        displayName: "Code Review",
        status: "installed",
        message: null,
        commandPreview: "npx @skills-sh/cli add code-review",
      },
    ],
  },
  error: null,
};

export const mockInstallationError: InstallationState = {
  isInstalling: false,
  currentSkillId: null,
  currentIndex: 1,
  total: 2,
  outputLines: [
    { skillId: "skill-legacy-agent", line: "npx @skills-sh/cli add legacy-agent-notes --agent universal", stream: "command" },
    { skillId: "skill-legacy-agent", line: "Error: permission denied writing to .agents/skills", stream: "stderr" },
  ],
  perSkillStatus: {
    "skill-legacy-agent": "failed",
  },
  displayNames: {
    "skill-legacy-agent": "Legacy Agent Notes",
  },
  result: null,
  error: "Skills CLI exited with code 1: permission denied writing to .agents/skills",
};
