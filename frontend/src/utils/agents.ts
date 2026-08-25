import { SUPPORTED_AGENTS, type AgentScopeSelection } from "../types";

export function agentLabel(id: string): string {
  return SUPPORTED_AGENTS.find((agent) => agent.id === id)?.label ?? id;
}

export function formatAgents(ids: string[]): string {
  return ids.map(agentLabel).join(", ");
}

/** The saved custom order (if any), with unknown ids dropped and any
 * `SUPPORTED_AGENTS` id missing from it (a newly added agent, or simply an
 * empty/never-customized preference) appended in the built-in order. Always
 * returns every supported agent id exactly once. */
export function resolveAgentOrder(order: string[]): string[] {
  const knownIds = new Set(SUPPORTED_AGENTS.map((agent) => agent.id));
  const seen = new Set<string>();
  const resolved: string[] = [];
  for (const id of order) {
    if (knownIds.has(id) && !seen.has(id)) {
      resolved.push(id);
      seen.add(id);
    }
  }
  for (const agent of SUPPORTED_AGENTS) {
    if (!seen.has(agent.id)) resolved.push(agent.id);
  }
  return resolved;
}

/** Sorts agent ids by their position in `order` (see `resolveAgentOrder`) —
 * ids not present in `order` sort after every id that is. */
export function sortByAgentOrder(ids: string[], order: string[]): string[] {
  const rank = new Map(order.map((id, index) => [id, index]));
  return [...ids].sort((a, b) => (rank.get(a) ?? order.length) - (rank.get(b) ?? order.length));
}

/** An agent id missing from `scopes` (a newly added agent, or simply never
 * customized) falls back to Project-only — mirrors the identical rule on
 * the Rust side (`InstallOptions::scopes_for`). Project and Global aren't
 * exclusive: both flags can be true at once. */
export function resolveAgentScopes(agentId: string, scopes: Record<string, AgentScopeSelection>): AgentScopeSelection {
  return scopes[agentId] ?? { project: true, global: false };
}

/** Whether `agentId` installs into the *exact* directory Universal itself
 * reads from, for either scope — a static fact about `SUPPORTED_AGENTS`,
 * independent of any user's live scope selection. `universal` itself always
 * counts. Used to collapse redundant agent icons on a Skill card down to
 * one "Universal" icon, and to build the Agents dialog's summary of which
 * agents that covers — see `SkillCard.vue` and `AgentsDialog.vue`. */
export function isUniversalGroup(agentId: string): boolean {
  if (agentId === "universal") return true;
  const universal = SUPPORTED_AGENTS.find((candidate) => candidate.id === "universal");
  const agent = SUPPORTED_AGENTS.find((candidate) => candidate.id === agentId);
  if (!universal || !agent) return false;
  return agent.projectPath === universal.projectPath || agent.globalPath === universal.globalPath;
}
