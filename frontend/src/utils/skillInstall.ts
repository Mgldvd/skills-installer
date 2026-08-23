import type { Skill } from "../types";

/** Which of `targetAgents` this skill isn't installed for yet. Installing it
 * again for the same target agent list only ever closes this gap — the
 * Skills CLI is idempotent per agent — so this is also what decides whether
 * an already-`installed` skill can still be selected. */
export function agentsNeedingInstall(skill: Skill, targetAgents: string[]): string[] {
  return targetAgents.filter((id) => !skill.installedAgents.includes(id));
}
