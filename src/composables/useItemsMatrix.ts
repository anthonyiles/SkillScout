import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useToast } from './useToast'
import { formatError } from '../utils/formatError'

export interface RepositoryItem {
  id: string
  name: string
  folder: string
  description: string | null
  file_path: string
  content: string
}

export interface Project {
  id: number
  path: string
  agentIds: string[]
}

export interface Agent {
  id: string
  name: string
  skillsPath: string
  rulesPath: string
}

interface ItemSelection {
  item_id: string
  project_id: number
  applied_sha: string | null
}

export type ItemFolder = 'skills' | 'rules'

export function useItemsMatrix(folder: ItemFolder) {
  const items = ref<RepositoryItem[]>([])
  const projects = ref<Project[]>([])
  const agents = ref<Agent[]>([])
  const selectionMatrix = ref<Record<string, Set<number>>>({})
  const loading = ref(false)
  const applying = ref(false)
  const scanning = ref(false)

  const { error, success } = useToast()

  let unlistenSync: (() => void) | undefined

  function getProjectName(path: string): string {
    if (!path) return 'New Project'
    const parts = path.split(/[/\\]/).filter(Boolean)
    return parts.length > 0 ? parts[parts.length - 1] : 'New Project'
  }

  function isSelected(itemId: string, projectId: number): boolean {
    return selectionMatrix.value[itemId]?.has(projectId) || false
  }

  function agentPathForFolder(agent: Agent): string | undefined {
    return folder === 'skills' ? agent.skillsPath : agent.rulesPath
  }

  async function initializeMatrix() {
    const nextMatrix: Record<string, Set<number>> = {}
    items.value.forEach(item => { nextMatrix[item.id] = new Set() })

    try {
      const selections = await invoke<ItemSelection[]>('get_item_selections')
      if (selections) {
        for (const sel of selections) {
          if (nextMatrix[sel.item_id]) nextMatrix[sel.item_id].add(sel.project_id)
        }
      }
    } catch (err) {
      console.error('Failed to load selections:', err)
    }

    selectionMatrix.value = nextMatrix
  }

  async function loadData() {
    try {
      const fetchedProjects = await invoke<Project[]>('get_projects')
      if (fetchedProjects) projects.value = fetchedProjects
    } catch (err) {
      console.error('Failed to load projects:', err)
    }

    try {
      const fetchedAgents = await invoke<Agent[]>('get_agents')
      if (fetchedAgents) agents.value = fetchedAgents
    } catch (err) {
      console.error('Failed to load agents:', err)
    }

    try {
      const fetchedItems = await invoke<RepositoryItem[]>('get_repository_items', { folder })
      if (fetchedItems?.length) {
        items.value = fetchedItems
        await initializeMatrix()
        await scanLocal(true)
      }
    } catch (err) {
      console.error(`Failed to load ${folder}:`, err)
    }
  }

  async function syncRepo() {
    loading.value = true
    try {
      const repoUrl = await invoke<string | null>('get_setting', { key: 'repoUrl' })
      if (!repoUrl) {
        error('Please configure a repository URL in Settings first.')
        return
      }
      const count = await invoke<number>('sync_repo', { repoUrl })
      success(`Successfully synced repository! (${count} items processed)`)
      await loadData()
    } catch (err: unknown) {
      error(formatError(err, 'Failed to sync repository. Please try again.'))
    } finally {
      loading.value = false
    }
  }

  async function toggleSelection(itemId: string, projectId: number) {
    if (!selectionMatrix.value[itemId]) selectionMatrix.value[itemId] = new Set()
    if (selectionMatrix.value[itemId].has(projectId)) {
      selectionMatrix.value[itemId].delete(projectId)
    } else {
      selectionMatrix.value[itemId].add(projectId)
    }
    try {
      await invoke('toggle_item_selection', { itemId, projectId })
    } catch (err) {
      console.error('Failed to persist selection change', err)
    }
  }

  async function scanLocal(silent = false) {
    scanning.value = true
    let updated = false

    for (const project of projects.value) {
      if (!project.path || !project.agentIds?.length) continue

      const agentPaths = new Set<string>()
      for (const agentId of project.agentIds) {
        const agent = agents.value.find(agent => agent.id === agentId)
        const agentPath = agent && agentPathForFolder(agent)
        if (agentPath) agentPaths.add(agentPath)
      }

      if (agentPaths.size === 0) continue

      try {
        const foundFiles = await invoke<string[]>('get_project_files', {
          projectPath: project.path,
          subFolders: Array.from(agentPaths),
        })

        for (const item of items.value) {
          if (!selectionMatrix.value[item.id]) selectionMatrix.value[item.id] = new Set()
          const wasSelected = selectionMatrix.value[item.id].has(project.id)
          const isNowSelected = foundFiles.includes(item.name)

          if (isNowSelected !== wasSelected) {
            if (isNowSelected) {
              selectionMatrix.value[item.id].add(project.id)
            } else {
              selectionMatrix.value[item.id].delete(project.id)
            }
            await invoke('toggle_item_selection', { itemId: item.id, projectId: project.id })
            updated = true
          }
        }
      } catch (err: any) {
        console.error(`Failed to scan ${project.path}`, err)
      }
    }

    if (!silent) {
      if (updated) success('Matched tickboxes with local files!')
      else success('Tickboxes are already up to date.')
    }
    scanning.value = false
  }

  async function applyToProjects() {
    const tasks: any[] = []
    let missingConfigError = ''

    for (const item of items.value) {
      const selectedProjectIds = selectionMatrix.value[item.id] || new Set()

      for (const project of projects.value) {
        if (!project.path) continue

        const itemIsSelected = selectedProjectIds.has(project.id)

        if (itemIsSelected && !project.agentIds?.length) {
          missingConfigError = `Project "${getProjectName(project.path)}" has no agents enabled in Settings.`
          break
        }

        for (const agentId of project.agentIds ?? []) {
          const agent = agents.value.find(agent => agent.id === agentId)
          const agentPath = agent && agentPathForFolder(agent)

          if (itemIsSelected && !agent) {
            missingConfigError = `Agent "${agentId}" not found for project "${getProjectName(project.path)}".`
            break
          }
          if (itemIsSelected && !agentPath) {
            const label = folder === 'skills' ? 'Skills' : 'Rules'
            missingConfigError = `${label} path not configured for agent "${agent!.name}" in project "${getProjectName(project.path)}".`
            break
          }

          if (agentPath) {
            tasks.push({
              source_file: itemIsSelected ? item.file_path : null,
              target_dir: `${project.path}/${agentPath}`,
              file_name: item.name,
              remove: !itemIsSelected,
            })
          }
        }
        if (missingConfigError) break
      }
      if (missingConfigError) break
    }

    if (missingConfigError) { error(missingConfigError); return }
    if (tasks.length === 0) { error(`No projects to apply ${folder} to.`); return }

    applying.value = true
    try {
      await invoke('apply_skills', { tasks })
      success(`Successfully updated ${folder} across your projects!`)
    } catch (err: unknown) {
      error(formatError(err, `Failed to apply ${folder} to projects.`))
    } finally {
      applying.value = false
    }
  }

  onMounted(async () => {
    await loadData()
    unlistenSync = await listen('repo_synced', () => { loadData() })
  })

  onUnmounted(() => { unlistenSync?.() })

  return {
    items,
    projects,
    selectionMatrix,
    loading,
    applying,
    scanning,
    isSelected,
    getProjectName,
    syncRepo,
    toggleSelection,
    scanLocal,
    applyToProjects,
  }
}
