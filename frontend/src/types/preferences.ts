import type { InstallScope } from "./install";

export interface UiPreferences {
  fontScale: number;
  defaultAgents: string[];
  copyByDefault: boolean;
  confirmBeforeInstall: boolean;
  continueAfterFailure: boolean;
  accent: string;
  localSourcePath: string | null;
  compactCards: boolean;
  /** User-customized agent display order (Agents dialog + Skill card icons).
   * Empty means "no customization yet" — see `resolveAgentOrder`. */
  agentOrder: string[];
  /** The app's single active scope, remembered across launches — replaced
   * the old per-agent `agentScopes` map (see
   * REFACTOR_PROJECT_GLOBAL_SCOPE.md). */
  lastScope: InstallScope;
  /** The last project folder selected in the header, remembered across
   * launches. */
  lastProjectPath: string | null;
}

export interface SupportedAgent {
  id: string;
  label: string;
  projectPath: string;
  globalPath: string;
}
export const SUPPORTED_AGENTS: SupportedAgent[] = [
  {
    id: "universal",
    label: "Universal (.agents)",
    projectPath: ".agents/skills",
    globalPath: "~/.config/agents/skills",
  },
  { id: "claude-code", label: "Claude Code", projectPath: ".claude/skills", globalPath: "~/.claude/skills" },
  // Per the `skills` CLI's own Supported Agents table (vercel-labs/skills
  // README) — the authoritative source for every path below, since that CLI
  // is what actually performs the install: Codex's project path is the
  // shared `.agents/skills`, but its *global* path is its own dedicated
  // `~/.codex/skills`, not shared with Universal.
  { id: "codex", label: "Codex", projectPath: ".agents/skills", globalPath: "~/.codex/skills" },
  { id: "gemini-cli", label: "Gemini CLI", projectPath: ".agents/skills", globalPath: "~/.gemini/skills" },
  { id: "cursor", label: "Cursor", projectPath: ".agents/skills", globalPath: "~/.cursor/skills" },
  { id: "windsurf", label: "Windsurf", projectPath: ".windsurf/skills", globalPath: "~/.codeium/windsurf/skills" },
  { id: "opencode", label: "OpenCode", projectPath: ".agents/skills", globalPath: "~/.config/opencode/skills" },
  { id: "github-copilot", label: "GitHub Copilot", projectPath: ".agents/skills", globalPath: "~/.copilot/skills" },
  // A separate row from `github-copilot` for how users actually think about
  // it, even though both install through the same underlying `skills` CLI
  // agent (see Rust's `domain::cli_agent_id`) — VS Code's Copilot Chat/agent
  // mode has no CLI id of its own. `.github/skills` is its own dedicated
  // directory per VS Code's docs, distinct from the generic Copilot path.
  { id: "vscode", label: "VS Code", projectPath: ".github/skills", globalPath: "~/.copilot/skills" },
  { id: "pi", label: "Pi", projectPath: ".pi/skills", globalPath: "~/.pi/agent/skills" },
  // Per the `skills` CLI's Supported Agents table: OpenClaw's project path
  // is a bare `skills/` at the project root, not `.agents/skills`.
  { id: "openclaw", label: "OpenClaw", projectPath: "skills", globalPath: "~/.openclaw/skills" },
  // Zed shares `.agents/skills` (project) / `~/.agents/skills` (global) with
  // Cline, Dexto, Kimi Code CLI, Loaf, and Warp per the `skills` CLI's table
  // — none of them have a dedicated directory of their own.
  { id: "zed", label: "Zed", projectPath: ".agents/skills", globalPath: "~/.agents/skills" },
  // Per the `skills` CLI's table, OpenHands has its own dedicated directory
  // at both scopes — not the shared `.agents/skills` convention.
  { id: "openhands", label: "OpenHands", projectPath: ".openhands/skills", globalPath: "~/.openhands/skills" },
  // Per the `skills` CLI's table, Hermes Agent has its own dedicated
  // directory at both scopes — not the shared `.agents/skills` convention.
  { id: "hermes-agent", label: "Hermes Agent", projectPath: ".hermes/skills", globalPath: "~/.hermes/skills" },
];

export interface FontScalePreset {
  label: string;
  value: number;
}

export const FONT_SCALE_PRESETS: FontScalePreset[] = [
  { label: "Small", value: 0.9 },
  { label: "Default", value: 1.0 },
  { label: "Large", value: 1.1 },
  { label: "Larger", value: 1.25 },
  { label: "Extra", value: 1.4 },
];

export function defaultPreferences(): UiPreferences {
  return {
    fontScale: 1.0,
    defaultAgents: ["universal"],
    copyByDefault: true,
    confirmBeforeInstall: true,
    continueAfterFailure: true,
    accent: "#F43F75",
    localSourcePath: null,
    compactCards: false,
    agentOrder: [],
    lastScope: "project",
    lastProjectPath: null,
  };
}
