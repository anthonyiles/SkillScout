<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
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

const skills = ref<Skill[]>([])
const projects = ref<Project[]>([])
const availableAgents = ref<Agent[]>([])
const loading = ref(false)
const scanning = ref(false)

const { error, success } = useToast()

import ContentModal from '../components/ContentModal.vue'
import TickBox from '../components/TickBox.vue'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import EmptyState from '../components/EmptyState.vue'

// Matrix: skill.id -> Set of project.id
const selectionMatrix = ref<Record<string, Set<number>>>({})

const activeSkill = ref<Skill | null>(null)
const isModalOpen = ref(false)

function openPreview(skill: Skill) {
  activeSkill.value = skill
  isModalOpen.value = true
}

function truncate(str: string, length: number) {
  if (!str) return 'No preview available'
  const clean = str.replace(/\n/g, ' ').trim()
  return clean.length > length ? clean.substring(0, length) + '...' : clean
}

onMounted(() => {
  // Load projects from local storage
  const savedProjects = localStorage.getItem('projects')
  if (savedProjects) {
    try {
      projects.value = JSON.parse(savedProjects)
    } catch (e) {
      console.error('Failed to parse projects from localStorage:', e)
      projects.value = []
    }
  }

  const savedAgents = localStorage.getItem('agents')
  if (savedAgents) {
    try {
      availableAgents.value = JSON.parse(savedAgents)
    } catch (e) {
      console.error('Failed to parse agents from localStorage:', e)
      availableAgents.value = []
    }
  }

  // Load cached skills from local storage
  const savedSkills = localStorage.getItem('skills')
  if (savedSkills) {
    try {
      skills.value = JSON.parse(savedSkills)
      initializeMatrix()
      scanLocal(true)
    } catch (e) {
      console.error('Failed to parse skills from localStorage:', e)
      skills.value = []
    }
  }
})

function initializeMatrix() {
  skills.value.forEach(skill => {
    if (!selectionMatrix.value[skill.id]) {
      selectionMatrix.value[skill.id] = new Set()
    }
  })
  
  const savedSelection = localStorage.getItem('skillsSelection')
  if (savedSelection) {
    try {
      const parsed = JSON.parse(savedSelection)
      for (const [key, arr] of Object.entries(parsed)) {
        selectionMatrix.value[key] = new Set(arr as number[])
      }
    } catch(e) {
      console.error('Failed to parse skillsSelection from localStorage:', e, savedSelection)
      selectionMatrix.value = {}
    }
  }
}

async function syncRepo() {
  loading.value = true
  
  const repoUrl = localStorage.getItem('repoUrl')
  if (!repoUrl) {
    error('Please configure a repository URL in Settings first.')
    loading.value = false
    return
  }

  try {
    const fetchedSkills: Skill[] = await invoke('sync_repo', { repoUrl })
    const filteredSkills = fetchedSkills.filter(s => s.folder === 'skills')
    
    if (filteredSkills.length === 0) {
      error('Repository synced, but no skills were found in the /skills folder.')
    } else {
      success(`Successfully synced ${filteredSkills.length} skills!`)
    }
    skills.value = filteredSkills
    
    // We can store all fetched to let RulesView access them without re-syncing
    localStorage.setItem('all_repo_data', JSON.stringify(fetchedSkills))
    localStorage.setItem('skills', JSON.stringify(filteredSkills))
    localStorage.setItem('rules', JSON.stringify(fetchedSkills.filter(s => s.folder === 'rules')))
    initializeMatrix()
  } catch (err: any) {
    error(typeof err === 'string' ? err : err.message || JSON.stringify(err))
  } finally {
    loading.value = false
  }
}

function toggleSelection(skillId: string, projectId: number) {
  if (!selectionMatrix.value[skillId]) {
    selectionMatrix.value[skillId] = new Set()
  }
  
  if (selectionMatrix.value[skillId].has(projectId)) {
    selectionMatrix.value[skillId].delete(projectId)
  } else {
    selectionMatrix.value[skillId].add(projectId)
  }
  
  saveSelection()
}

function saveSelection() {
  const serialized: Record<string, number[]> = {}
  for (const [key, set] of Object.entries(selectionMatrix.value)) {
    serialized[key] = Array.from(set)
  }
  localStorage.setItem('skillsSelection', JSON.stringify(serialized))
}

async function scanLocal(silent = false) {
  scanning.value = true
  let updated = false
  for (const project of projects.value) {
    if (!project.path || !project.agentIds) continue;
    
    const skillPathsToScan = new Set<string>();
    for (const agentId of project.agentIds) {
      const agent = availableAgents.value.find(a => a.id === agentId);
      if (agent && agent.skillsPath) {
        skillPathsToScan.add(agent.skillsPath);
      }
    }
    
    if (skillPathsToScan.size === 0) continue;
    
    try {
      const foundFiles: string[] = await invoke('get_project_files', { 
        projectPath: project.path, 
        subFolders: Array.from(skillPathsToScan) 
      });
      
      for (const skill of skills.value) {
        if (!selectionMatrix.value[skill.id]) {
          selectionMatrix.value[skill.id] = new Set();
        }
        
        const wasSelected = selectionMatrix.value[skill.id].has(project.id);
        const isNowSelected = foundFiles.includes(skill.name);
        
        if (isNowSelected && !wasSelected) {
          selectionMatrix.value[skill.id].add(project.id);
          updated = true;
        } else if (!isNowSelected && wasSelected) {
          selectionMatrix.value[skill.id].delete(project.id);
          updated = true;
        }
      }
    } catch (e: any) {
      console.error(`Failed to scan ${project.path}`, e);
    }
  }
  
  if (updated) {
    saveSelection();
  }
  if (!silent) {
    if (updated) success('Matched tickboxes with local files!')
    else success('Tickboxes are already up to date.')
  }
  scanning.value = false
}

function isSelected(skillId: string, projectId: number) {
  return selectionMatrix.value[skillId]?.has(projectId) || false
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
  for (const skill of skills.value) {
    const selectedProjectIds = selectionMatrix.value[skill.id] || new Set()
    
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
        if (agent && agent.skillsPath) {
          if (isSelected) {
            tasks.push({
              source_file: skill.file_path,
              target_dir: `${project.path}/${agent.skillsPath}`,
              file_name: skill.name,
              remove: false
            })
          } else {
            tasks.push({
              source_file: null,
              target_dir: `${project.path}/${agent.skillsPath}`,
              file_name: skill.name,
              remove: true
            })
          }
        }
      }
    }
    if (missingConfigError) break
  }

  if (missingConfigError) {
    error(missingConfigError)
    return
  }

  if (tasks.length === 0) {
    error('No projects to apply skills to.')
    return
  }

  applying.value = true
  try {
    await invoke('apply_skills', { tasks })
    success(`Successfully updated skills across your projects!`)
  } catch (err: any) {
    error(typeof err === 'string' ? err : err.message || JSON.stringify(err))
  } finally {
    applying.value = false
  }
}
</script>

<template>
  <PageLayout title="Skills">
    <template #actions>
      <BaseButton variant="secondary" @click="applyToProjects" :disabled="applying || loading || scanning">
        <svg v-if="!applying" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>
        <span v-else class="loader"></span>
        {{ applying ? 'Applying...' : 'Apply' }}
      </BaseButton>
      <BaseButton variant="secondary" @click="scanLocal(false)" :disabled="loading || applying || scanning">
        <svg v-if="!scanning" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12h3"/><path d="M19 12h3"/><path d="M12 2v3"/><path d="M12 19v3"/><path d="m4.93 4.93 2.12 2.12"/><path d="m16.95 16.95 2.12 2.12"/><path d="m4.93 19.07 2.12-2.12"/><path d="m16.95 7.05 2.12-2.12"/></svg>
        <span v-else class="loader"></span>
        {{ scanning ? 'Scanning...' : 'Scan files' }}
      </BaseButton>
      <BaseButton variant="primary" @click="syncRepo" :disabled="loading || applying || scanning">
        <svg v-if="!loading" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 21v-5h5"/></svg>
        <span v-else class="loader"></span>
        {{ loading ? 'Syncing...' : 'Sync' }}
      </BaseButton>
    </template>

    <EmptyState 
      v-if="skills.length === 0 && !loading" 
      glass
      message="No skills loaded. Click 'Sync' to fetch skills from GitHub." 
    />

    <div v-else-if="skills.length > 0" class="matrix-container glass">
      <table class="matrix-table">
        <thead>
          <tr>
            <th class="skill-col">Skill</th>
            <th v-for="project in projects" :key="project.id" class="project-col">
              {{ getProjectName(project.path) }}
            </th>
            <th v-if="projects.length === 0" class="project-col text-secondary">
              No projects configured
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="skill in skills" :key="skill.id">
            <td class="skill-info">
              <div class="skill-name">{{ skill.name }}</div>
              <div class="skill-preview-line" @click="openPreview(skill)" title="Click to view full content">
                {{ truncate(skill.content, 35) }}
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/><line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/></svg>
              </div>
            </td>
            <td v-for="project in projects" :key="project.id" class="checkbox-cell">
              <TickBox 
                :checked="isSelected(skill.id, project.id)"
                @change="toggleSelection(skill.id, project.id)"
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
      :title="activeSkill?.name || ''" 
      :content="activeSkill?.content || ''" 
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

.text-secondary {
  color: var(--text-secondary);
}
.text-sm {
  font-size: 0.875rem;
}
</style>
