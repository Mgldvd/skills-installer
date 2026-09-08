import * as backend from "../services/backend";
import { useAppState } from "./useAppState";

export function usePresets() {
  const state = useAppState();

  async function loadAll() {
    state.presets = await backend.getPresets();
  }

  async function save(name: string, gitUrl: string | null, skillNames: string[]) {
    const preset = await backend.savePreset({ name, gitUrl, skillNames });
    state.presets = [...state.presets, preset].sort((a, b) => a.name.localeCompare(b.name));
    return preset;
  }

  async function update(presetId: string, skillNames: string[]) {
    const preset = await backend.updatePreset(presetId, skillNames);
    state.presets = state.presets.map((p) => (p.id === presetId ? preset : p));
    return preset;
  }

  async function remove(presetId: string) {
    await backend.deletePreset(presetId);
    state.presets = state.presets.filter((preset) => preset.id !== presetId);
  }

  return { loadAll, save, update, remove };
}
