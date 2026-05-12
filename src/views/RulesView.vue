<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useToast } from '../composables/useToast'

interface Skill {
  id: string
  name: string
  folder: string
  description: string
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

function truncate(str: string, length: number) {
  if (!str) return 'No preview available'
  const clean = str.replace(/\n/g, ' ').trim()
  return clean.length > length ? clean.substring(0, length) + '...' : clean
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
  rules.value.forEach(rule => {
    if (!selectionMatrix.value[rule.id]) {
      selectionMatrix.value[rule.id] = new Set()
    }
  })
  
  try {
    const selections = await invoke<ItemSelection[]>('get_item_selections')
    if (selections) {
      for (const sel of selections) {
        if (!selectionMatrix.value[sel.item_id]) {
          selectionMatrix.value[sel.item_id] = new Set()
        }
        selectionMatrix.value[sel.item_id].add(sel.project_id)
      }
    }
  } catch (e) {
    console.error('Failed to load selections:', e)
  }
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
    error(typeof err === 'string' ? err : err.message || JSON.stringify(err))
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

async function saveSelection() {
  // Now handled dynamically on toggle
}

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
    error(typeof err === 'string' ? err : err.message || JSON.stringify(err))
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
        <span v-else class="loader"></span>
        {{ applying ? 'Applying...' : 'Apply' }}
      </BaseButton>
      <BaseButton variant="primary" @click="syncRepo" :disabled="loading || applying">
        <svg v-if="!loading" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 21v-5h5"/></svg>
        <span v-else class="loader"></span>
        {{ loading ? 'Syncing...' : 'Sync' }}
      </BaseButton>
    </template>

    <EmptyState 
      v-if="rules.length === 0 && !loading" 
      glass
      message="No rules loaded. Click 'Sync' to fetch rules from GitHub." 
    />

    <div v-else-if="rules.length > 0" class="matrix-container glass">
      <table class="matrix-table">
        <thead>
          <tr>
            <th class="skill-col">Rule</th>
            <th v-for="project in projects" :key="project.id" class="project-col">
              {{ getProjectName(project.path) }}
            </th>
            <th v-if="projects.length === 0" class="project-col text-secondary">
              No projects configured
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="rule in rules" :key="rule.id">
            <td class="skill-info">
              <div class="skill-name">{{ rule.name }}</div>
              <button
                type="button"
                class="preview-button skill-preview-line"
                @click="openPreview(rule)"
                title="Click to view full content"
                :aria-label="`Preview rule: ${rule.name}`"
              >
                {{ truncate(rule.content, 35) }}
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/><line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/></svg>
              </button>
            </td>
            <td v-for="project in projects" :key="project.id" class="checkbox-cell">
              <TickBox 
                :checked="isSelected(rule.id, project.id)"
                @change="toggleSelection(rule.id, project.id)"
              />
            </td>
            <td v-if="projects.length === 0" class="checkbox-cell text-secondary text-sm">
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

<style scoped>




.loader {
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top: 2px solid white;
  border-radius: 50%;
  width: 14px;
  height: 14px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}



.matrix-container {
  border-radius: var(--radius-md);
  overflow-x: auto;
}

.matrix-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
}

.matrix-table th, .matrix-table td {
  padding: 0.5rem 1rem;
  border-bottom: 1px solid var(--border-color);
}

.matrix-table th {
  font-weight: 600;
  color: var(--text-secondary);
  background: rgba(0, 0, 0, 0.2);
  white-space: nowrap;
}

.matrix-table tr:last-child td {
  border-bottom: none;
}

.matrix-table tbody tr:hover {
  background: var(--bg-surface-hover);
}

.skill-col {
  width: 40%;
}

.project-col {
  text-align: center;
  width: 150px;
}

.checkbox-cell {
  text-align: center;
}

.skill-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.skill-name {
  font-weight: 600;
  font-size: 1rem;
}

.skill-preview-line {
  font-size: 0.8rem;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 0.25rem;
  transition: color var(--transition-fast);
}

.skill-preview-line:hover {
  color: var(--accent-primary);
}

.preview-button {
  width: 100%;
  padding: 0;
  border: 0;
  background: transparent;
  text-align: left;
  font-family: inherit;
}

.text-secondary {
  color: var(--text-secondary);
}
.text-sm {
  font-size: 0.875rem;
}
</style>
