import * as backend from '../services/backend'
import type { UiPreferences } from '../types'
import { useAppState } from './useAppState'

function applyFontScale(scale: number) {
  document.documentElement.style.setProperty('--font-scale', String(scale))
}
function applyAccent(accent: UiPreferences['accent']) { document.documentElement.dataset.accent = accent }

export function usePreferences() {
  const state = useAppState()

  async function load() {
    state.preferences = await backend.getPreferences()
    applyFontScale(state.preferences.fontScale)
    applyAccent(state.preferences.accent)
  }

  async function update(partial: Partial<UiPreferences>) {
    const previous = state.preferences
    state.preferences = { ...previous, ...partial }
    if (partial.fontScale !== undefined) applyFontScale(state.preferences.fontScale)
    if (partial.accent !== undefined) applyAccent(state.preferences.accent)
    try {
      state.preferences = await backend.updatePreferences(state.preferences)
    } catch (error) {
      state.preferences = previous
      applyFontScale(previous.fontScale)
      applyAccent(previous.accent)
      throw error
    }
  }

  return { load, update }
}
