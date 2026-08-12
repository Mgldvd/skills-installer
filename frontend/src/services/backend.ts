import { Channel } from '@tauri-apps/api/core'

import { call } from './tauri/client'
import type {
  ApplicationConfig,
  DependencyStatus,
  InstallProgressEvent,
  InstallRequest,
  InstallResult,
  ParsedSkillSource,
  Skill,
  SkillTag,
  UiPreferences,
} from '../types'

export interface AddSkillArgs {
  url: string
  displayName?: string | null
  description?: string | null
  groupId: string
  tags?: string[]
  preselected?: boolean
  enabled?: boolean
}

export interface UpdateSkillArgs {
  skillId: string
  url?: string | null
  displayName?: string | null
  description?: string | null
  groupId?: string | null
  tags?: string[] | null
  preselected?: boolean | null
  enabled?: boolean | null
}

export async function getApplicationState(): Promise<ApplicationConfig> {
  return call('get_application_state')
}

export async function getSkills(): Promise<Skill[]> {
  return call('get_skills')
}

export async function getInstalledSkills(): Promise<Skill[]> {
  return call('get_installed_skills')
}

export async function previewSkillUrl(rawUrl: string): Promise<ParsedSkillSource> {
  return call('preview_skill_url', { rawUrl })
}

export async function addSkill(args: AddSkillArgs): Promise<Skill> {
  return call('add_skill', { args })
}

export async function updateSkill(args: UpdateSkillArgs): Promise<Skill> {
  return call('update_skill', { args })
}

export async function deleteSkill(skillId: string): Promise<void> {
  return call('delete_skill', { skillId })
}

export async function createTag(name: string, color: string): Promise<SkillTag> {
  return call('create_tag', { args: { name, color } })
}
export async function updateTag(tagId: string, name: string, color: string): Promise<SkillTag> {
  return call('update_tag', { args: { tagId, name, color } })
}
export async function deleteTag(tagId: string): Promise<void> { return call('delete_tag', { tagId }) }
export async function assignTagToSkill(skillId: string, tagId: string): Promise<Skill> {
  return call('assign_tag_to_skill', { skillId, tagId })
}
export async function unassignTagFromSkill(skillId: string, tagId: string): Promise<Skill> {
  return call('unassign_tag_from_skill', { skillId, tagId })
}

export async function getPreferences(): Promise<UiPreferences> {
  return call('get_preferences')
}

export async function updatePreferences(preferences: UiPreferences): Promise<UiPreferences> {
  return call('update_preferences', { preferences })
}

export async function getDependencyStatus(): Promise<DependencyStatus> {
  return call('get_dependency_status')
}

export async function validateInstallation(request: InstallRequest): Promise<void> {
  return call('validate_installation', { request })
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
  const channel = new Channel<InstallProgressEvent>()
  channel.onmessage = onEvent
  return call('install_skills', { request, onEvent: channel })
}

export async function cancelInstallation(): Promise<void> {
  return call('cancel_installation')
}

export async function refresh(): Promise<ApplicationConfig> {
  return call('refresh')
}
export async function exportPortableConfiguration():Promise<string>{return call('export_portable_configuration')}
export async function importPortableConfiguration(content:string):Promise<void>{return call('import_portable_configuration',{content})}
