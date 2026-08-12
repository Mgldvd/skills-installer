export type DependencySource = 'installedExecutable' | 'npx' | 'unavailable'

export interface DependencyStatus {
  available: boolean
  source: DependencySource
  executablePath: string | null
  version: string | null
  detail: string | null
}
