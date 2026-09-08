import * as backend from "../services/backend";
import type { InstallProgressEvent, InstallRequest } from "../types";
import { resetInstallationState, useAppState } from "./useAppState";

export function useInstallation() {
  const state = useAppState();

  function handleEvent(event: InstallProgressEvent, onSkillSuccess?: (skillId: string) => void) {
    switch (event.event) {
      case "start":
        state.installation.total = event.data.total;
        break;
      case "progress":
        state.installation.currentSkillId = event.data.skillId;
        state.installation.currentIndex = event.data.current;
        state.installation.perSkillStatus[event.data.skillId] = "pending";
        state.installation.displayNames[event.data.skillId] = event.data.displayName;
        break;
      case "command":
        state.installation.outputLines.push({
          skillId: event.data.skillId,
          line: `$ ${event.data.command}`,
          stream: "command",
        });
        break;
      case "output":
        state.installation.outputLines.push({
          skillId: event.data.skillId,
          line: event.data.line,
          stream: event.data.stream,
        });
        break;
      case "skill-success":
        state.installation.perSkillStatus[event.data.skillId] = "installed";
        state.installation.displayNames[event.data.skillId] = event.data.displayName;
        onSkillSuccess?.(event.data.skillId);
        break;
      case "skill-error":
        state.installation.perSkillStatus[event.data.skillId] = "failed";
        state.installation.displayNames[event.data.skillId] = event.data.displayName;
        state.installation.outputLines.push({
          skillId: event.data.skillId,
          line: `Error: ${event.data.message}`,
          stream: "stderr",
        });
        break;
      case "complete":
        state.installation.result = event.data.result;
        break;
    }
  }

  /** `onSkillSuccess` fires as each Skill in the batch finishes — the caller
   * uses it to move that Skill into the "Installed" section right away
   * instead of leaving the whole grid frozen until every Skill in the
   * selection is done (see App.vue's `runInstall`). */
  async function install(request: InstallRequest, onSkillSuccess?: (skillId: string) => void) {
    resetInstallationState();
    state.installation.isInstalling = true;
    state.installation.total = request.selection.skillIds.length;
    try {
      const result = await backend.installSkills(request, (event) => handleEvent(event, onSkillSuccess));
      state.installation.result = result;
      return result;
    } catch (error) {
      state.installation.error = error instanceof Error ? error.message : String(error);
      throw error;
    } finally {
      state.installation.isInstalling = false;
    }
  }

  async function cancel() {
    await backend.cancelInstallation();
  }

  async function checkDependencies() {
    state.dependencyStatus = await backend.getDependencyStatus();
    return state.dependencyStatus;
  }

  return { install, cancel, checkDependencies };
}
