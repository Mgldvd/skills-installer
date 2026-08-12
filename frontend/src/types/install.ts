import type { SkillSelection } from './skill'

export type InstallScope = 'project' | 'global'

export interface InstallOptions {
  agents: string[]
  projectPath: string | null
  copy: boolean
  scope: InstallScope
  dryRun: boolean
  confirm: boolean
  continueOnError: boolean
}

export interface InstallRequest {
  selection: SkillSelection
  options: InstallOptions
}

export type SkillInstallStatus = 'installed' | 'alreadyInstalled' | 'failed' | 'skipped'

export interface SkillInstallOutcome {
  skillId: string
  displayName: string
  status: SkillInstallStatus
  message: string | null
  commandPreview: string
}

export interface InstallResult {
  requested: number
  installed: number
  alreadyInstalled: number
  failed: number
  cancelled: boolean
  perSkill: SkillInstallOutcome[]
}

export type OutputStream = 'stdout' | 'stderr'

export type InstallProgressEvent =
  | { event: 'start'; data: { total: number } }
  | { event: 'progress'; data: { current: number; total: number; skillId: string; displayName: string } }
  | { event: 'output'; data: { skillId: string; line: string; stream: OutputStream } }
  | { event: 'skill-success'; data: { skillId: string; displayName: string } }
  | { event: 'skill-error'; data: { skillId: string; displayName: string; message: string } }
  | { event: 'complete'; data: { result: InstallResult } }

export function defaultInstallOptions(): InstallOptions {
  return {
    agents: ['universal'],
    projectPath: null,
    copy: true,
    scope: 'project',
    dryRun: false,
    confirm: true,
    continueOnError: true,
  }
}
