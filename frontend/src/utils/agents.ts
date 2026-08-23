import { SUPPORTED_AGENTS } from "../types";

export function agentLabel(id: string): string {
  return SUPPORTED_AGENTS.find((agent) => agent.id === id)?.label ?? id;
}

export function formatAgents(ids: string[]): string {
  return ids.map(agentLabel).join(", ");
}
