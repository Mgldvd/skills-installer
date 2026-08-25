import type { SkillSelection } from "./skill";

export type InstallScope = "project" | "global";

/** An agent's scope isn't exclusive — it can install to the project *and*
 * the user's global directory from one action, so this is two independent
 * flags rather than a single `InstallScope`. Mirrors Rust's
 * `AgentScopeSelection`. */
export interface AgentScopeSelection {
  project: boolean;
  global: boolean;
}

export interface InstallOptions {
  agents: string[];
  /** Per-agent scope override — see `UiPreferences.agentScopes` and
   * `resolveAgentScopes`. An agent id missing here falls back to
   * `{ project: true, global: false }`. */
  agentScopes: Record<string, AgentScopeSelection>;
  projectPath: string | null;
  copy: boolean;
  dryRun: boolean;
  confirm: boolean;
  continueOnError: boolean;
}

export interface InstallRequest {
  selection: SkillSelection;
  options: InstallOptions;
}

export type SkillInstallStatus = "installed" | "alreadyInstalled" | "failed" | "skipped";

export interface SkillInstallOutcome {
  skillId: string;
  displayName: string;
  status: SkillInstallStatus;
  message: string | null;
  commandPreview: string;
}

export interface InstallResult {
  requested: number;
  installed: number;
  alreadyInstalled: number;
  failed: number;
  cancelled: boolean;
  perSkill: SkillInstallOutcome[];
}

export type OutputStream = "stdout" | "stderr";

export type InstallProgressEvent =
  | { event: "start"; data: { total: number } }
  | { event: "progress"; data: { current: number; total: number; skillId: string; displayName: string } }
  | { event: "command"; data: { skillId: string; command: string } }
  | { event: "output"; data: { skillId: string; line: string; stream: OutputStream } }
  | { event: "skill-success"; data: { skillId: string; displayName: string } }
  | { event: "skill-error"; data: { skillId: string; displayName: string; message: string } }
  | { event: "complete"; data: { result: InstallResult } };

// Uninstalls already-installed Skills from disk (the real `skills remove`)
// — distinct from the catalog-only "Delete Skill" action, which just edits
// the local skills.yaml and never touches installed files.
export interface UninstallRequest {
  selection: SkillSelection;
  projectPath: string | null;
}

export interface UninstallResult {
  requested: number;
  removed: number;
  failed: number;
  message: string | null;
}

export function defaultInstallOptions(): InstallOptions {
  return {
    agents: ["universal"],
    agentScopes: {},
    projectPath: null,
    copy: true,
    dryRun: false,
    confirm: true,
    continueOnError: true,
  };
}
