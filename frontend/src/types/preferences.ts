import type { InstallScope } from './install'

export interface UiPreferences {
  fontScale: number
  defaultAgents: string[]
  copyByDefault: boolean
  defaultScope: InstallScope
  confirmBeforeInstall: boolean
  continueAfterFailure: boolean
  accent: AccentColor
  localSourcePath: string | null
}

export type AccentColor = 'pink' | 'coral' | 'blue' | 'teal' | 'violet' | 'green'
export interface SupportedAgent { id:string; label:string; projectPath:string; globalPath:string }
export const SUPPORTED_AGENTS: SupportedAgent[] = [
  { id:'universal', label:'Universal (.agents)', projectPath:'.agents/skills', globalPath:'~/.config/agents/skills' },
  { id:'claude-code', label:'Claude Code', projectPath:'.claude/skills', globalPath:'~/.claude/skills' },
  { id:'codex', label:'Codex', projectPath:'.agents/skills', globalPath:'~/.codex/skills' },
  { id:'gemini-cli', label:'Gemini CLI', projectPath:'.agents/skills', globalPath:'~/.gemini/skills' },
  { id:'cursor', label:'Cursor', projectPath:'.agents/skills', globalPath:'~/.cursor/skills' },
  { id:'windsurf', label:'Windsurf', projectPath:'.windsurf/skills', globalPath:'~/.codeium/windsurf/skills' },
  { id:'opencode', label:'OpenCode', projectPath:'.agents/skills', globalPath:'~/.config/opencode/skills' },
  { id:'github-copilot', label:'GitHub Copilot', projectPath:'.agents/skills', globalPath:'~/.copilot/skills' },
]

export interface FontScalePreset {
  label: string
  value: number
}

export const FONT_SCALE_PRESETS: FontScalePreset[] = [
  { label: 'Small', value: 0.9 },
  { label: 'Default', value: 1.0 },
  { label: 'Large', value: 1.1 },
  { label: 'Larger', value: 1.25 },
  { label: 'Extra', value: 1.4 },
]

export function defaultPreferences(): UiPreferences {
  return {
    fontScale: 1.0,
    defaultAgents: ['universal'],
    copyByDefault: true,
    defaultScope: 'project',
    confirmBeforeInstall: true,
    continueAfterFailure: true,
    accent: 'pink',
    localSourcePath: null,
  }
}
