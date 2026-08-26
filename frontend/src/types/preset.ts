// Named `Preset`, not `Project`: `InstallScope` already uses "Project" for
// the active install destination/scope selected in the header — see
// REFACTOR_PROJECT_GLOBAL_SCOPE.md for why this type was renamed.
export interface Preset {
  id: string;
  name: string;
  gitUrl: string | null;
  skillNames: string[];
}
