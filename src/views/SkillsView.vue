<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useToast } from '../composables/useToast'
import { useProjects } from '../composables/useProjects'
import { useAgents } from '../composables/useAgents'
import { syncRepo, checkExisting, applySkills, getProjectFiles } from '../api'
import type { SyncTask } from '../api'
import type { Skill } from '../types'
import { getProjectName } from '../utils/formatters'
import ContentModal from '../components/ContentModal.vue'
import TickBox from '../components/TickBox.vue'
import BaseButton from '../components/BaseButton.vue'
import PageLayout from '../components/PageLayout.vue'
import EmptyState from '../components/EmptyState.vue'
import ConfirmModal from '../components/ConfirmModal.vue'

const { projects, load: loadProjects } = useProjects()
const { agents, load: loadAgents } = useAgents()
const { error, success } = useToast()

const skills = ref<Skill[]>([])
const loading = ref(false)
const scanning = ref(false)
const applying = ref(false)

// Matrix: skill.id -> Set of project.id
const selectionMatrix = ref<Record<string, Set<string>>>({})

const activeSkill = ref<Skill | null>(null)
const isModalOpen = ref(false)
const isConfirmOpen = ref(false)
const conflictingFiles = ref<string[]>([])
let pendingTasks: SyncTask[] = []

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
  loadProjects()
  loadAgents()

  const savedSkills = localStorage.getItem('skills')
  if (savedSkills) {
    try {
      skills.value = JSON.parse(savedSkills)
      initializeMatrix()
      scanLocal(true)
    } catch {
      skills.value = []
    }
  }
})

function initializeMatrix() {
  // Build fresh — prevents stale entries when skills are removed between syncs
  const fresh: Record<string, Set<string>> = {}
  skills.value.forEach(skill => { fresh[skill.id] = new Set() })

  const savedSelection = localStorage.getItem('skillsSelection')
  if (savedSelection) {
    try {
      const parsed = JSON.parse(savedSelection)
      for (const [key, arr] of Object.entries(parsed)) {
        if (fresh[key]) fresh[key] = new Set(arr as string[])
      }
    } catch {
      // leave fresh empty on parse failure
    }
  }

  selectionMatrix.value = fresh
}

async function doSync() {
  loading.value = true
  const repoUrl = localStorage.getItem('repoUrl')
  if (!repoUrl) {
    error('Please configure a repository URL in Settings first.')
    loading.value = false
    return
  }
  try {
    const fetched = await syncRepo(repoUrl)
    const filteredSkills = fetched.filter(s => s.folder === 'skills')
    if (filteredSkills.length === 0) {
      error('Repository synced, but no skills were found in the /skills folder.')
    } else {
      success(`Successfully synced ${filteredSkills.length} skills!`)
    }
    skills.value = filteredSkills
    localStorage.setItem('all_repo_data', JSON.stringify(fetched))
    localStorage.setItem('skills', JSON.stringify(filteredSkills))
    localStorage.setItem('rules', JSON.stringify(fetched.filter(s => s.folder === 'rules')))
    initializeMatrix()
  } catch (err: any) {
    error(typeof err === 'string' ? err : err.message ?? JSON.stringify(err))
  } finally {
    loading.value = false
  }
}

function toggleSelection(skillId: string, projectId: string) {
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
  const serialized: Record<string, string[]> = {}
  for (const [key, set] of Object.entries(selectionMatrix.value)) {
    serialized[key] = Array.from(set)
  }
  localStorage.setItem('skillsSelection', JSON.stringify(serialized))
}

async function scanLocal(silent = false) {
  scanning.value = true
  let updated = false
  for (const project of projects.value) {
    if (!project.path || !project.agentIds) continue
    const skillPaths = new Set<string>()
    for (const agentId of project.agentIds) {
      const agent = agents.value.find(a => a.id === agentId)
      if (agent?.skillsPath) skillPaths.add(agent.skillsPath)
    }
    if (skillPaths.size === 0) continue
    try {
      const foundFiles = await getProjectFiles(project.path, Array.from(skillPaths))
      for (const skill of skills.value) {
        if (!selectionMatrix.value[skill.id]) selectionMatrix.value[skill.id] = new Set()
        const wasSelected = selectionMatrix.value[skill.id].has(project.id)
        const isNowSelected = foundFiles.includes(skill.name)
        if (isNowSelected && !wasSelected) { selectionMatrix.value[skill.id].add(project.id); updated = true }
        else if (!isNowSelected && wasSelected) { selectionMatrix.value[skill.id].delete(project.id); updated = true }
      }
    } catch (e: any) {
      console.error(`Failed to scan ${project.path}`, e)
    }
  }
  if (updated) saveSelection()
  if (!silent) success(updated ? 'Matched tickboxes with local files!' : 'Tickboxes are already up to date.')
  scanning.value = false
}

function isSelected(skillId: string, projectId: string) {
  return selectionMatrix.value[skillId]?.has(projectId) ?? false
}

function buildTasks(): { tasks: SyncTask[]; error: string } {
  const tasks: SyncTask[] = []
  for (const skill of skills.value) {
    const selectedIds = selectionMatrix.value[skill.id] ?? new Set()
    for (const project of projects.value) {
      if (!project.path) continue
      const selected = selectedIds.has(project.id)
      if (selected && (!project.agentIds || project.agentIds.length === 0)) {
        return { tasks: [], error: `Project "${getProjectName(project.path)}" has no agents enabled in Settings.` }
      }
      for (const agentId of project.agentIds ?? []) {
        const agent = agents.value.find(a => a.id === agentId)
        if (agent?.skillsPath) {
          tasks.push({
            sourceFile: selected ? skill.filePath : null,
            targetDir: `${project.path}/${agent.skillsPath}`,
            fileName: skill.name,
            remove: !selected,
          })
        }
      }
    }
  }
  return { tasks, error: '' }
}

async function applyToProjects() {
  const { tasks, error: buildError } = buildTasks()
  if (buildError) { error(buildError); return }
  if (tasks.length === 0) { error('No projects to apply skills to.'); return }

  // Check for conflicts before applying
  const addTasks = tasks.filter(t => !t.remove)
  if (addTasks.length > 0) {
    try {
      const conflicts = await checkExisting(addTasks)
      if (conflicts.length > 0) {
        conflictingFiles.value = conflicts
        pendingTasks = tasks
        isConfirmOpen.value = true
        return
      }
    } catch {
      // If check fails, proceed — apply will handle errors
    }
  }

  await runApply(tasks)
}

async function runApply(tasks: SyncTask[]) {
  applying.value = true
  try {
    const count = await applySkills(tasks)
    success(`Successfully updated ${count} file(s) across your projects!`)
  } catch (err: any) {
    error(typeof err === 'string' ? err : err.message ?? JSON.stringify(err))
  } finally {
    applying.value = false
  }
}

async function confirmOverwrite() {
  isConfirmOpen.value = false
  await runApply(pendingTasks)
  pendingTasks = []
}

function cancelOverwrite() {
  isConfirmOpen.value = false
  pendingTasks = []
  conflictingFiles.value = []
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
      <BaseButton variant="primary" @click="doSync" :disabled="loading || applying || scanning">
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
            <th v-if="projects.length === 0" class="project-col text-secondary">No projects configured</th>
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
            <td v-if="projects.length === 0" class="checkbox-cell text-secondary text-sm">-</td>
          </tr>
        </tbody>
      </table>
    </div>

    <ContentModal
      :isOpen="isModalOpen"
      :title="activeSkill?.name ?? ''"
      :content="activeSkill?.content ?? ''"
      @close="isModalOpen = false"
    />

    <ConfirmModal
      :isOpen="isConfirmOpen"
      title="Overwrite existing files?"
      :message="`The following files already exist and will be overwritten:\n\n${conflictingFiles.join('\n')}`"
      confirmText="Overwrite"
      :danger="true"
      @confirm="confirmOverwrite"
      @cancel="cancelOverwrite"
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

.skill-col { width: 40%; }
.project-col { text-align: center; width: 150px; }
.checkbox-cell { text-align: center; }

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

.skill-preview-line:hover { color: var(--accent-primary); }
.text-secondary { color: var(--text-secondary); }
.text-sm { font-size: 0.875rem; }
</style>
