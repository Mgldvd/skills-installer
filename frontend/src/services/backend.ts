import { Channel } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import { call } from "./tauri/client";
import type {
  ApplicationConfig,
  DependencyStatus,
  InstallProgressEvent,
  InstallRequest,
  InstallResult,
  InstallScope,
  ParsedSkillSource,
  Project,
  Skill,
  SkillTag,
  UiPreferences,
  UninstallRequest,
  UninstallResult,
} from "../types";

export interface AddSkillArgs {
  url: string;
  displayName?: string | null;
  description?: string | null;
  groupId: string;
  tags?: string[];
  preselected?: boolean;
  enabled?: boolean;
}

export interface UpdateSkillArgs {
  skillId: string;
  url?: string | null;
  displayName?: string | null;
  description?: string | null;
  groupId?: string | null;
  tags?: string[] | null;
  preselected?: boolean | null;
  enabled?: boolean | null;
}

export async function getApplicationState(): Promise<ApplicationConfig> {
  return call("get_application_state");
}

export async function getSkills(): Promise<Skill[]> {
  return call("get_skills");
}

export async function getInstalledSkills(): Promise<Skill[]> {
  return call("get_installed_skills");
}

export async function previewSkillUrl(rawUrl: string): Promise<ParsedSkillSource> {
  return call("preview_skill_url", { rawUrl });
}

export interface PackSkillPreview {
  name: string;
  description: string;
}
export interface PackPreview {
  canonicalUrl: string;
  suggestedName: string;
  skills: PackSkillPreview[];
}
export interface ImportPackArgs {
  url: string;
  packName: string;
  color: string;
  groupId: string;
}
export interface PackImportResult {
  tag: SkillTag;
  added: Skill[];
  skipped: string[];
}

export async function previewPackUrl(rawUrl: string): Promise<PackPreview> {
  return call("preview_pack_url", { rawUrl });
}

export async function importPack(args: ImportPackArgs): Promise<PackImportResult> {
  return call("import_pack", { args });
}

export async function addSkill(args: AddSkillArgs): Promise<Skill> {
  return call("add_skill", { args });
}

export async function updateSkill(args: UpdateSkillArgs): Promise<Skill> {
  return call("update_skill", { args });
}

export async function deleteSkill(skillId: string): Promise<void> {
  return call("delete_skill", { skillId });
}

export async function createTag(name: string, color: string): Promise<SkillTag> {
  return call("create_tag", { args: { name, color } });
}
export async function updateTag(tagId: string, name: string, color: string): Promise<SkillTag> {
  return call("update_tag", { args: { tagId, name, color } });
}
export async function deleteTag(tagId: string): Promise<void> {
  return call("delete_tag", { tagId });
}
export async function reorderTags(tagIds: string[]): Promise<SkillTag[]> {
  return call("reorder_tags", { tagIds });
}
export async function assignTagToSkill(skillId: string, tagId: string): Promise<Skill> {
  return call("assign_tag_to_skill", { skillId, tagId });
}
export async function unassignTagFromSkill(skillId: string, tagId: string): Promise<Skill> {
  return call("unassign_tag_from_skill", { skillId, tagId });
}

export async function getPreferences(): Promise<UiPreferences> {
  return call("get_preferences");
}

export async function updatePreferences(preferences: UiPreferences): Promise<UiPreferences> {
  return call("update_preferences", { preferences });
}

export interface SaveProjectArgs {
  name: string;
  gitUrl?: string | null;
  skillNames: string[];
}

export async function getProjects(): Promise<Project[]> {
  return call("get_projects");
}
export async function saveProject(args: SaveProjectArgs): Promise<Project> {
  return call("save_project", { args });
}
export async function deleteProject(projectId: string): Promise<void> {
  return call("delete_project", { projectId });
}

export interface CliInstallResult {
  commandPath: string;
  executablePath: string;
  pathConfigured: boolean;
}

export async function installCliCommand(): Promise<CliInstallResult> {
  return call("install_cli_command");
}

export async function getDependencyStatus(): Promise<DependencyStatus> {
  return call("get_dependency_status");
}

export async function validateInstallation(request: InstallRequest): Promise<void> {
  return call("validate_installation", { request });
}

/**
 * Uses a `Channel` (scoped to this one invocation) rather than a global
 * `listen()` — progress from one install can never leak into an unrelated
 * listener.
 */
export async function installSkills(
  request: InstallRequest,
  onEvent: (event: InstallProgressEvent) => void,
): Promise<InstallResult> {
  const channel = new Channel<InstallProgressEvent>();
  channel.onmessage = onEvent;
  return call("install_skills", { request, onEvent: channel });
}

export async function cancelInstallation(): Promise<void> {
  return call("cancel_installation");
}

/** Uninstalls already-installed Skills from disk — distinct from
 * `deleteSkill`, which only removes a Skill from the catalog. */
export async function uninstallSkills(request: UninstallRequest): Promise<UninstallResult> {
  return call("uninstall_skills", { request });
}

export async function refresh(projectPath?: string, scope?: InstallScope): Promise<ApplicationConfig> {
  return call("refresh", { projectPath, scope });
}

/** Local-only, on-demand check — returns the ids of installed Local skills whose `.signature` no longer matches the Local Skill Source catalog. */
export async function checkLocalSkillUpdates(projectPath?: string): Promise<string[]> {
  return call("check_local_skill_updates", { projectPath });
}

export async function selectInstallationDirectory(defaultPath?: string): Promise<string | null> {
  const selected = await open({
    title: "Select installation folder",
    directory: true,
    multiple: false,
    defaultPath: defaultPath || undefined,
  });
  return typeof selected === "string" ? selected : null;
}
export async function selectLocalCatalogDirectory(defaultPath?: string): Promise<string | null> {
  const selected = await open({
    title: "Select local Skills catalog folder",
    directory: true,
    multiple: false,
    defaultPath: defaultPath || undefined,
  });
  return typeof selected === "string" ? selected : null;
}
export async function exportPortableConfiguration(): Promise<string> {
  return call("export_portable_configuration");
}
export async function importPortableConfiguration(content: string): Promise<void> {
  return call("import_portable_configuration", { content });
}
