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
// The `globalPath` values below were originally transcribed from the
// `skills` CLI's own Supported Agents README table, like `projectPath`. That
// table turned out to be wrong about global scope — verified 2026-08-25 by
// actually running the real CLI (`npx skills add ... --global`) against a
// scratch `$HOME` for every agent id and inspecting what landed on disk (see
// Rust's `AGENT_GLOBAL_DIRS` for the discovery-side counterpart, kept in
// sync with this table by hand). `~/.agents/skills` turned out to be the
// CLI's shared global destination for nearly every agent, mirroring how
// `.agents/skills` already works at the project scope — including agents
// the README implied had a dedicated directory of their own. Only
// `claude-code`, `pi`, `openclaw`, `openhands`, `windsurf`, and
// `hermes-agent` get a genuinely separate dedicated copy in addition to
// being covered by the shared directory.
export const SUPPORTED_AGENTS: SupportedAgent[] = [
  { id: "universal", label: "Universal (.agents)", projectPath: ".agents/skills", globalPath: "~/.agents/skills" },
  { id: "claude-code", label: "Claude Code", projectPath: ".claude/skills", globalPath: "~/.claude/skills" },
  { id: "codex", label: "Codex", projectPath: ".agents/skills", globalPath: "~/.agents/skills" },
  { id: "gemini-cli", label: "Gemini CLI", projectPath: ".agents/skills", globalPath: "~/.agents/skills" },
  { id: "cursor", label: "Cursor", projectPath: ".agents/skills", globalPath: "~/.agents/skills" },
  { id: "windsurf", label: "Windsurf", projectPath: ".windsurf/skills", globalPath: "~/.codeium/windsurf/skills" },
  { id: "opencode", label: "OpenCode", projectPath: ".agents/skills", globalPath: "~/.agents/skills" },
  { id: "github-copilot", label: "GitHub Copilot", projectPath: ".agents/skills", globalPath: "~/.agents/skills" },
  // A separate row from `github-copilot` for how users actually think about
  // it, even though both install through the same underlying `skills` CLI
  // agent (see Rust's `domain::cli_agent_id`) — VS Code's Copilot Chat/agent
  // mode has no CLI id of its own. `.github/skills` is its own dedicated
  // *project*-scope directory per VS Code's docs, distinct from the generic
  // Copilot path — but at global scope it lands in the shared directory
  // like everything else.
  { id: "vscode", label: "VS Code", projectPath: ".github/skills", globalPath: "~/.agents/skills" },
  { id: "pi", label: "Pi", projectPath: ".pi/skills", globalPath: "~/.pi/agent/skills" },
  // Per the `skills` CLI's Supported Agents table: OpenClaw's project path
  // is a bare `skills/` at the project root, not `.agents/skills`.
  { id: "openclaw", label: "OpenClaw", projectPath: "skills", globalPath: "~/.openclaw/skills" },
  // Zed shares `.agents/skills` (project) / `~/.agents/skills` (global) with
  // Cline, Dexto, Kimi Code CLI, Loaf, and Warp per the `skills` CLI's table
  // — none of them have a dedicated directory of their own.
  { id: "zed", label: "Zed", projectPath: ".agents/skills", globalPath: "~/.agents/skills" },
  { id: "openhands", label: "OpenHands", projectPath: ".openhands/skills", globalPath: "~/.openhands/skills" },
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
