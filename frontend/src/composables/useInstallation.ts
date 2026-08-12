import * as backend from '../services/backend'
import type { InstallProgressEvent, InstallRequest } from '../types'
import { resetInstallationState, useAppState } from './useAppState'

export function useInstallation() {
  const state = useAppState()

  function handleEvent(event: InstallProgressEvent) {
    switch (event.event) {
      case 'start':
        state.installation.total = event.data.total
        break
      case 'progress':
        state.installation.currentSkillId = event.data.skillId
        state.installation.currentIndex = event.data.current
        state.installation.perSkillStatus[event.data.skillId] = 'pending'
        state.installation.displayNames[event.data.skillId] = event.data.displayName
        break
      case 'output':
        state.installation.outputLines.push({
          skillId: event.data.skillId,
          line: event.data.line,
          stream: event.data.stream,
        })
        break
      case 'skill-success':
        state.installation.perSkillStatus[event.data.skillId] = 'installed'
        state.installation.displayNames[event.data.skillId] = event.data.displayName
        break
      case 'skill-error':
        state.installation.perSkillStatus[event.data.skillId] = 'failed'
        state.installation.displayNames[event.data.skillId] = event.data.displayName
        break
      case 'complete':
        state.installation.result = event.data.result
        break
    }
  }

  async function install(request: InstallRequest) {
    resetInstallationState()
    state.installation.isInstalling = true
    state.installation.total = request.selection.skillIds.length
    try {
      const result = await backend.installSkills(request, handleEvent)
      state.installation.result = result
      return result
    } catch (error) {
      state.installation.error = error instanceof Error ? error.message : String(error)
      throw error
    } finally {
      state.installation.isInstalling = false
    }
  }

  async function cancel() {
    await backend.cancelInstallation()
  }

  async function checkDependencies() {
    state.dependencyStatus = await backend.getDependencyStatus()
    return state.dependencyStatus
  }

  return { install, cancel, checkDependencies }
}
