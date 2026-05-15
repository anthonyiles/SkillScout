<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useToast } from '../composables/useToast'

interface Skill {
  id: string
  name: string
  folder: string
  description: string | null
  file_path: string
  content: string
}

interface Project {
  id: number
  path: string
  agentIds: string[]
}

interface Agent {
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

const rules = ref<Skill[]>([])
const projects = ref<Project[]>([])
const availableAgents = ref<Agent[]>([])
const loading = ref(false)

const { error, success } = useToast()

import ContentModal from '../components/ContentModal.vue'
import TickBox from '../components/TickBox.vue'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import EmptyState from '../components/EmptyState.vue'

// Matrix: rule.id -> Set of project.id
const selectionMatrix = ref<Record<string, Set<number>>>({})

const activeRule = ref<Skill | null>(null)
const isModalOpen = ref(false)

let unlistenSync: () => void;

function openPreview(rule: Skill) {
  activeRule.value = rule
  isModalOpen.value = true
}


async function loadData() {
  try {
    const p = await invoke<Project[]>('get_projects')
    if (p) projects.value = p
  } catch (e) {
    console.error('Failed to load projects:', e)
  }

  try {
    const a = await invoke<Agent[]>('get_agents')
    if (a) availableAgents.value = a
  } catch (e) {
    console.error('Failed to load agents:', e)
  }

  try {
    const fetchedRules = await invoke<Skill[]>('get_repository_items', { folder: 'rules' })
    if (fetchedRules) {
      rules.value = fetchedRules
      await initializeMatrix()
    }
  } catch (e) {
    console.error('Failed to load rules:', e)
  }
}

onMounted(async () => {
  await loadData()
  
  unlistenSync = await listen('repo_synced', () => {
    loadData() // Refresh if background task syncs
  })
})

onUnmounted(() => {
  if (unlistenSync) unlistenSync()
})

async function initializeMatrix() {
  const nextMatrix: Record<string, Set<number>> = {}
  rules.value.forEach(rule => {
    nextMatrix[rule.id] = new Set()
  })

  try {
    const selections = await invoke<ItemSelection[]>('get_item_selections')
    if (selections) {
      for (const sel of selections) {
        if (nextMatrix[sel.item_id]) {
          nextMatrix[sel.item_id].add(sel.project_id)
        }
      }
    }
  } catch (e) {
    console.error('Failed to load selections:', e)
  }

  selectionMatrix.value = nextMatrix
}

async function syncRepo() {
  loading.value = true
  
  try {
    const repoUrl = await invoke<string | null>('get_setting', { key: 'repoUrl' })
    if (!repoUrl) {
      error('Please configure a repository URL in Settings first.')
      loading.value = false
      return
    }

    const count = await invoke<number>('sync_repo', { repoUrl })
    success(`Successfully synced repository! (${count} items processed)`)
    
    // Refresh local data
    await loadData()
  } catch (err: any) {
    console.error('Failed to sync repository:', err)
    error(typeof err === 'string' ? err : 'Failed to sync repository. Please try again.')
  } finally {
    loading.value = false
  }
}

async function toggleSelection(ruleId: string, projectId: number) {
  if (!selectionMatrix.value[ruleId]) {
    selectionMatrix.value[ruleId] = new Set()
  }
  
  if (selectionMatrix.value[ruleId].has(projectId)) {
    selectionMatrix.value[ruleId].delete(projectId)
  } else {
    selectionMatrix.value[ruleId].add(projectId)
  }
  
  try {
    await invoke('toggle_item_selection', { itemId: ruleId, projectId })
  } catch (e) {
    console.error('Failed to toggle selection in DB', e)
  }
}

// Selection persistence is handled per-toggle via invoke('toggle_item_selection')

function isSelected(ruleId: string, projectId: number) {
  return selectionMatrix.value[ruleId]?.has(projectId) || false
}

function getProjectName(path: string) {
  if (!path) return 'New Project'
  const parts = path.split(/[/\\]/).filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : 'New Project'
}

const applying = ref(false)

async function applyToProjects() {
  const tasks: any[] = []
  let missingConfigError = ''
  
  // Build tasks from matrix
  for (const rule of rules.value) {
    const selectedProjectIds = selectionMatrix.value[rule.id] || new Set()
    
    for (const project of projects.value) {
      if (!project.path) continue // Skip unconfigured
      
      const isSelected = selectedProjectIds.has(project.id)

      if (isSelected && (!project.agentIds || project.agentIds.length === 0)) {
        missingConfigError = `Project "${getProjectName(project.path)}" has no agents enabled in Settings.`
        break
      }

      if (!project.agentIds) continue

      for (const agentId of project.agentIds) {
        const agent = availableAgents.value.find(a => a.id === agentId)
        
        if (isSelected) {
          if (!agent) {
            missingConfigError = `Agent "${agentId}" not found for project "${getProjectName(project.path)}".`
            break
          }
          if (!agent.rulesPath) {
            missingConfigError = `Rules target path not configured for agent "${agent.name}" in project "${getProjectName(project.path)}".`
            break
          }
        }

        if (agent && agent.rulesPath) {
          if (isSelected) {
            tasks.push({
              source_file: rule.file_path,
              target_dir: `${project.path}/${agent.rulesPath}`,
              file_name: rule.name,
              remove: false
            })
          } else {
            tasks.push({
              source_file: null,
              target_dir: `${project.path}/${agent.rulesPath}`,
              file_name: rule.name,
              remove: true
            })
          }
        }
      }
      if (missingConfigError) break
    }
    if (missingConfigError) break
  }

  if (missingConfigError) {
    error(missingConfigError)
    return
  }

  if (tasks.length === 0) {
    error('No projects to apply rules to.')
    return
  }

  applying.value = true
  try {
    await invoke('apply_skills', { tasks })
    success(`Successfully updated rules across your projects!`)
  } catch (err: any) {
    console.error('Failed to apply rules:', err)
    error(typeof err === 'string' ? err : 'Failed to apply rules to projects.')
  } finally {
    applying.value = false
  }
}
</script>

<template>
  <PageLayout title="Rules">
    <template #actions>
      <BaseButton variant="secondary" @click="applyToProjects" :disabled="applying || loading">
        <svg v-if="!applying" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ applying ? 'Applying...' : 'Apply' }}
      </BaseButton>
      <BaseButton variant="primary" @click="syncRepo" :disabled="loading || applying">
        <svg v-if="!loading" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 21v-5h5"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ loading ? 'Syncing...' : 'Sync' }}
      </BaseButton>
    </template>

    <EmptyState
      v-if="rules.length === 0 && !loading"
      glass
      message="No rules loaded. Click 'Sync' to fetch rules from GitHub."
    />

    <div v-else-if="rules.length > 0" class="bg-card/70 backdrop-blur-md border border-white/10 rounded-md overflow-x-auto">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr>
            <th class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap w-[40%]">Rule</th>
            <th v-for="project in projects" :key="project.id" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              {{ getProjectName(project.path) }}
            </th>
            <th v-if="projects.length === 0" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              No projects configured
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="rule in rules" :key="rule.id" class="hover:bg-card-hover [&:last-child>td]:border-b-0">
            <td class="py-2 px-4 border-b border-divider">
              <div class="flex items-center gap-[0.4rem]">
                <span class="font-semibold text-base">{{ rule.name }}</span>
                <button
                  type="button"
                  class="bg-transparent border-0 p-0 cursor-pointer text-muted flex items-center transition-colors shrink-0 hover:text-accent"
                  @click="openPreview(rule)"
                  :aria-label="`Preview ${rule.name}`"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                </button>
              </div>
            </td>
            <td v-for="project in projects" :key="project.id" class="py-2 px-4 border-b border-divider text-center">
              <TickBox
                :checked="isSelected(rule.id, project.id)"
                @change="toggleSelection(rule.id, project.id)"
              />
            </td>
            <td v-if="projects.length === 0" class="py-2 px-4 border-b border-divider text-center text-muted text-sm">
              -
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <ContentModal
      :isOpen="isModalOpen"
      :title="activeRule?.name || ''"
      :content="activeRule?.content || ''"
      @close="isModalOpen = false"
    />
  </PageLayout>
</template>
