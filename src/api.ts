import { invoke } from '@tauri-apps/api/core'
import type { Skill, PrStatus } from './types'

export interface SyncTask {
  sourceFile: string | null
  targetDir: string
  fileName: string
  remove: boolean
}

export function syncRepo(repoUrl: string): Promise<Skill[]> {
  return invoke('sync_repo', { repoUrl })
}

export function checkExisting(tasks: SyncTask[]): Promise<string[]> {
  return invoke('check_existing', { tasks })
}

export function applySkills(tasks: SyncTask[]): Promise<number> {
  return invoke('apply_skills', { tasks })
}

export function getProjectFiles(projectPath: string, subFolders: string[]): Promise<string[]> {
  return invoke('get_project_files', { projectPath, subFolders })
}

export function startGithubDeviceFlow(): Promise<{ userCode: string; verificationUri: string; deviceCode: string; interval: number }> {
  return invoke('start_github_device_flow')
}

export function pollGithubToken(deviceCode: string): Promise<{ accessToken?: string; error?: string; errorDescription?: string }> {
  return invoke('poll_github_token', { deviceCode })
}

export function checkGithubAuth(): Promise<boolean> {
  return invoke('check_github_auth')
}

export function logoutGithub(): Promise<void> {
  return invoke('logout_github')
}

export function promoteItem(
  repoUrl: string,
  itemType: string,
  itemName: string,
  projectPath: string,
  subFolders: string[],
  updateMode?: boolean,
): Promise<{ url: string; branch: string }> {
  return invoke('promote_item', { repoUrl, itemType, itemName, projectPath, subFolders, updateMode })
}

export function checkPrStatus(prUrl: string): Promise<PrStatus> {
  return invoke('check_pr_status', { prUrl })
}
