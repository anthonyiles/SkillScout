import { invoke } from '@tauri-apps/api/core'
import type { Agent, PrStatus } from './types'

// Shared response types matching Rust model serialization

export interface Project {
  id: number
  path: string
  agentIds: string[]
}

export interface RepositoryItem {
  id: string
  name: string
  folder: string
  description: string | null
  file_path: string
  content: string
  sha: string
  last_synced: string | null
}

export interface PromotedItem {
  id?: number
  name: string
  path: string
  itemType: string
  repository_item_id?: string | null
  url?: string | null
  branch: string
  subFolder?: string | null
}

export interface FileHash {
  name: string
  sha: string
  folder: string
  content: string
}

export interface ItemSelection {
  item_id: string
  project_id: number
  applied_sha: string | null
}

export interface SyncTask {
  source_file: string | null
  target_dir: string
  file_name: string
  remove: boolean
}

// Auth

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

// Settings

export function getSetting(key: string): Promise<string | null> {
  return invoke('get_setting', { key })
}

export function setSetting(key: string, value: string): Promise<void> {
  return invoke('set_setting', { key, value })
}

// Projects & Agents

export function getProjects(): Promise<Project[]> {
  return invoke('get_projects')
}

export function saveProject(project: { id: number | null; path: string; agentIds: string[] }): Promise<Project> {
  return invoke('save_project', { project })
}

export function deleteProject(id: number): Promise<void> {
  return invoke('delete_project', { id })
}

export function getAgents(): Promise<Agent[]> {
  return invoke('get_agents')
}

// Repository items

export function getRepositoryItems(folder?: string): Promise<RepositoryItem[]> {
  return invoke('get_repository_items', { folder })
}

export function syncRepo(repoUrl: string): Promise<number> {
  return invoke('sync_repo', { repoUrl })
}

// Selections

export function getItemSelections(): Promise<ItemSelection[]> {
  return invoke('get_item_selections')
}

export function toggleItemSelection(itemId: string, projectId: number): Promise<void> {
  return invoke('toggle_item_selection', { itemId, projectId })
}

// File operations

export function checkExisting(tasks: SyncTask[]): Promise<string[]> {
  return invoke('check_existing', { tasks })
}

export function applySkills(tasks: SyncTask[]): Promise<number> {
  return invoke('apply_skills', { tasks })
}

export function getProjectFiles(projectPath: string, subFolders: string[]): Promise<string[]> {
  return invoke('get_project_files', { projectPath, subFolders })
}

export function getProjectFileHashes(projectPath: string, subFolders: string[]): Promise<FileHash[]> {
  return invoke('get_project_file_hashes', { projectPath, subFolders })
}

// Promoted items

export function getPromotedItems(): Promise<PromotedItem[]> {
  return invoke('get_promoted_items')
}

export function addPromotedItem(item: Omit<PromotedItem, 'id'>): Promise<PromotedItem> {
  return invoke('add_promoted_item', { item })
}

export function removePromotedItem(id: number): Promise<void> {
  return invoke('remove_promoted_item', { id })
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
