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

const selectionMatrix = ref<Record<string, Set<number>>>({})
const activeSkill = ref<Skill | null>(null)
const isModalOpen = ref(false)
let unlistenSync: () => void;

function openPreview(skill: Skill) {
  activeSkill.value = skill
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
    const fetchedSkills = await invoke<Skill[]>('get_repository_items', { folder: 'skills' })
    if (fetchedSkills) {
      skills.value = fetchedSkills
      await initializeMatrix()
      await scanLocal(true)
    }
  } catch (e) {
    console.error('Failed to load skills:', e)
  }
}

onMounted(async () => {
  await loadData()
  unlistenSync = await listen('repo_synced', () => { loadData() })
})

onUnmounted(() => { if (unlistenSync) unlistenSync() })

async function initializeMatrix() {
  const nextMatrix: Record<string, Set<number>> = {}
  skills.value.forEach(skill => { nextMatrix[skill.id] = new Set() })
  try {
    const selections = await invoke<ItemSelection[]>('get_item_selections')
    if (selections) {
      for (const sel of selections) {
        if (nextMatrix[sel.item_id]) nextMatrix[sel.item_id].add(sel.project_id)
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
    if (!repoUrl) { error('Please configure a repository URL in Settings first.'); loading.value = false; return }
    const count = await invoke<number>('sync_repo', { repoUrl })
    success(`Successfully synced repository! (${count} items processed)`)
    await loadData()
  } catch (err: any) {
    error(typeof err === 'string' ? err : 'Failed to sync repository. Please try again.')
  } finally {
    loading.value = false
  }
}

async function toggleSelection(skillId: string, projectId: number) {
  if (!selectionMatrix.value[skillId]) selectionMatrix.value[skillId] = new Set()
  if (selectionMatrix.value[skillId].has(projectId)) {
    selectionMatrix.value[skillId].delete(projectId)
  } else {
    selectionMatrix.value[skillId].add(projectId)
  }
  try {
    await invoke('toggle_item_selection', { itemId: skillId, projectId })
  } catch (e) {
    console.error('Failed to toggle selection in DB', e)
  }
}

async function scanLocal(silent = false) {
  scanning.value = true
  let updated = false
  for (const project of projects.value) {
    if (!project.path || !project.agentIds) continue
    const skillPathsToScan = new Set<string>()
    for (const agentId of project.agentIds) {
      const agent = availableAgents.value.find(a => a.id === agentId)
      if (agent && agent.skillsPath) skillPathsToScan.add(agent.skillsPath)
    }
    if (skillPathsToScan.size === 0) continue
    try {
      const foundFiles: string[] = await invoke('get_project_files', { projectPath: project.path, subFolders: Array.from(skillPathsToScan) })
      for (const skill of skills.value) {
        if (!selectionMatrix.value[skill.id]) selectionMatrix.value[skill.id] = new Set()
        const wasSelected = selectionMatrix.value[skill.id].has(project.id)
        const isNowSelected = foundFiles.includes(skill.name)
        if (isNowSelected && !wasSelected) {
          selectionMatrix.value[skill.id].add(project.id)
          await invoke('toggle_item_selection', { itemId: skill.id, projectId: project.id })
          updated = true
        } else if (!isNowSelected && wasSelected) {
          selectionMatrix.value[skill.id].delete(project.id)
          await invoke('toggle_item_selection', { itemId: skill.id, projectId: project.id })
          updated = true
        }
      }
    } catch (e: any) {
      console.error(`Failed to scan ${project.path}`, e)
    }
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
  for (const skill of skills.value) {
    const selectedProjectIds = selectionMatrix.value[skill.id] || new Set()
    for (const project of projects.value) {
      if (!project.path) continue
      const isSelected = selectedProjectIds.has(project.id)
      if (isSelected && (!project.agentIds || project.agentIds.length === 0)) {
        missingConfigError = `Project "${getProjectName(project.path)}" has no agents enabled in Settings.`
        break
      }
      if (!project.agentIds) continue
      for (const agentId of project.agentIds) {
        const agent = availableAgents.value.find(a => a.id === agentId)
        if (agent && agent.skillsPath) {
          tasks.push({ source_file: isSelected ? skill.file_path : null, target_dir: `${project.path}/${agent.skillsPath}`, file_name: skill.name, remove: !isSelected })
        }
      }
    }
    if (missingConfigError) break
  }
  if (missingConfigError) { error(missingConfigError); return }
  if (tasks.length === 0) { error('No projects to apply skills to.'); return }
  applying.value = true
  try {
    await invoke('apply_skills', { tasks })
    success(`Successfully updated skills across your projects!`)
  } catch (err: any) {
    error(typeof err === 'string' ? err : 'Failed to apply skills to projects.')
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
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ applying ? 'Applying...' : 'Apply' }}
      </BaseButton>
      <BaseButton variant="secondary" @click="scanLocal(false)" :disabled="loading || applying || scanning">
        <svg v-if="!scanning" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12h3"/><path d="M19 12h3"/><path d="M12 2v3"/><path d="M12 19v3"/><path d="m4.93 4.93 2.12 2.12"/><path d="m16.95 16.95 2.12 2.12"/><path d="m4.93 19.07 2.12-2.12"/><path d="m16.95 7.05 2.12-2.12"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ scanning ? 'Scanning...' : 'Scan files' }}
      </BaseButton>
      <BaseButton variant="primary" @click="syncRepo" :disabled="loading || applying || scanning">
        <svg v-if="!loading" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 21v-5h5"/></svg>
        <span v-else class="inline-block w-[14px] h-[14px] rounded-full border-2 border-white/30 border-t-white animate-spin"></span>
        {{ loading ? 'Syncing...' : 'Sync' }}
      </BaseButton>
    </template>

    <EmptyState
      v-if="skills.length === 0 && !loading"
      glass
      message="No skills loaded. Click 'Sync' to fetch skills from GitHub."
    />

    <div v-else-if="skills.length > 0" class="glass rounded-md overflow-x-auto">
      <table class="w-full border-collapse text-left">
        <thead>
          <tr>
            <th class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap w-[40%]">Skill</th>
            <th v-for="project in projects" :key="project.id" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              {{ getProjectName(project.path) }}
            </th>
            <th v-if="projects.length === 0" class="py-2 px-4 border-b border-divider font-semibold text-muted bg-black/20 whitespace-nowrap text-center w-[150px]">
              No projects configured
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="skill in skills" :key="skill.id" class="hover:bg-card-hover [&:last-child>td]:border-b-0">
            <td class="py-2 px-4 border-b border-divider">
              <div class="flex flex-col gap-1">
                <div class="flex items-center gap-[0.4rem]">
                  <span class="font-semibold text-base">{{ skill.name }}</span>
                  <button
                    type="button"
                    class="bg-transparent border-0 p-0 cursor-pointer text-muted flex items-center transition-colors shrink-0 hover:text-accent"
                    @click="openPreview(skill)"
                    :aria-label="`Preview ${skill.name}`"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                  </button>
                </div>
              </div>
            </td>
            <td v-for="project in projects" :key="project.id" class="py-2 px-4 border-b border-divider text-center">
              <TickBox
                :checked="isSelected(skill.id, project.id)"
                @change="toggleSelection(skill.id, project.id)"
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
      :title="activeSkill?.name || ''"
      :content="activeSkill?.content || ''"
      @close="isModalOpen = false"
    />
  </PageLayout>
</template>
