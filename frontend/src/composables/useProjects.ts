import * as backend from "../services/backend";
import { useAppState } from "./useAppState";

export function useProjects() {
  const state = useAppState();

  async function loadAll() {
    state.projects = await backend.getProjects();
  }

  async function save(name: string, gitUrl: string | null, skillNames: string[]) {
    const project = await backend.saveProject({ name, gitUrl, skillNames });
    state.projects = [...state.projects, project].sort((a, b) => a.name.localeCompare(b.name));
    return project;
  }

  async function remove(projectId: string) {
    await backend.deleteProject(projectId);
    state.projects = state.projects.filter((project) => project.id !== projectId);
  }

  return { loadAll, save, remove };
}
