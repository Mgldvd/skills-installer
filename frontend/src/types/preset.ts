// Named `Preset`, not `Project`: `InstallScope` already uses "Project" for
// the active install destination/scope selected in the header, so this type
// was renamed to avoid the clash.
export interface Preset {
  id: string;
  name: string;
  gitUrl: string | null;
  skillNames: string[];
}
