<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useToast } from '../composables/useToast'
import { formatError } from '../utils/formatError'
import {
  getProjects,
  getAgents,
  saveProject,
  deleteProject,
  getRepositoryItems,
  getItemSelections,
  applySkills,
  pickProjectFolder,
} from '../api'
import type { RepositoryItem, ItemSelection, SyncTask } from '../api'
import TickBox from '../components/TickBox.vue'
import BaseButton from '../components/BaseButton.vue'
import InputField from '../components/InputField.vue'
import PageLayout from '../components/PageLayout.vue'
import CardItem from '../components/CardItem.vue'
import EmptyState from '../components/EmptyState.vue'

interface Agent {
  id: string
  name: string
  skillsPath: string
  rulesPath: string
}

interface LocalProject {
  id: number | null
  path: string
  agentIds: string[]
  _tempId: string
}

const projects = ref<LocalProject[]>([])
const availableAgents = ref<Agent[]>([])
const saving = ref(false)
const lastSavedAgentIds = ref<Map<string, string[]>>(new Map())
const { success, error } = useToast()

async function loadData() {
  try {
    const fetchedProjects = await getProjects()
    if (fetchedProjects && fetchedProjects.length > 0) {
      const withTempIds = fetchedProjects.map(project => ({
        ...project,
        _tempId: project.id?.toString() || crypto.randomUUID(),
      }))
      projects.value = withTempIds

      const snapshot = new Map<string, string[]>()
      for (const p of withTempIds) {
        snapshot.set(p._tempId, [...p.agentIds])
      }
      lastSavedAgentIds.value = snapshot
    } else {
      addProject()
    }
  } catch (err) {
    console.error('Failed to load projects:', err)
    addProject()
  }

  try {
    const fetchedAgents = await getAgents()
    if (fetchedAgents) {
      availableAgents.value = fetchedAgents
    }
  } catch (err) {
    console.error('Failed to load agents:', err)
  }
}

onMounted(() => {
  loadData()
})

async function applyAgentSync(
  project: LocalProject,
  addedAgentIds: string[],
  removedAgentIds: string[],
  allItems: RepositoryItem[],
  allSelections: ItemSelection[],
) {
  const selectedItemIds = new Set(
    allSelections.filter(s => s.project_id === project.id).map(s => s.item_id)
  )

  const tasks: SyncTask[] = []

  for (const agentId of removedAgentIds) {
    const agent = availableAgents.value.find(a => a.id === agentId)
    if (!agent) continue
    for (const item of allItems) {
      if (!selectedItemIds.has(item.id)) continue
      const agentPath = item.folder === 'skills' ? agent.skillsPath : agent.rulesPath
      if (!agentPath) continue
      tasks.push({ source_file: null, target_dir: `${project.path}/${agentPath}`, file_name: item.name, remove: true })
    }
  }

  for (const agentId of addedAgentIds) {
    const agent = availableAgents.value.find(a => a.id === agentId)
    if (!agent) continue
    for (const item of allItems) {
      if (!selectedItemIds.has(item.id)) continue
      const agentPath = item.folder === 'skills' ? agent.skillsPath : agent.rulesPath
      if (!agentPath) continue
      tasks.push({ source_file: item.file_path, target_dir: `${project.path}/${agentPath}`, file_name: item.name, remove: false })
    }
  }

  if (tasks.length === 0) return
  await applySkills(tasks)
}

async function saveConfig() {
  saving.value = true
  try {
    const [allItems, allSelections] = await Promise.all([getRepositoryItems(), getItemSelections()])

    const updatedProjects: LocalProject[] = []
    for (const project of projects.value) {
      if (!project.path) continue

      const oldIds = lastSavedAgentIds.value.get(project._tempId) ?? []
      const newIds = project.agentIds
      const addedAgentIds = newIds.filter(id => !oldIds.includes(id))
      const removedAgentIds = oldIds.filter(id => !newIds.includes(id))

      const saved = await saveProject({ id: project.id, path: project.path, agentIds: project.agentIds })
      const savedWithTemp: LocalProject = { ...saved, id: saved.id ?? null, _tempId: saved.id?.toString() || crypto.randomUUID() }
      updatedProjects.push(savedWithTemp)

      if (savedWithTemp.id !== null && (addedAgentIds.length > 0 || removedAgentIds.length > 0)) {
        try {
          await applyAgentSync(savedWithTemp, addedAgentIds, removedAgentIds, allItems, allSelections)
        } catch (err: unknown) {
          error(formatError(err, 'Projects saved, but failed to sync some skill/rule files.'))
        }
      }
    }

    const newSnapshot = new Map<string, string[]>()
    for (const p of updatedProjects) {
      newSnapshot.set(p._tempId, [...p.agentIds])
    }
    lastSavedAgentIds.value = newSnapshot

    projects.value = updatedProjects
    if (projects.value.length === 0) addProject()
    success('Projects saved successfully!')
  } catch (err: unknown) {
    error(formatError(err, 'Failed to save projects'))
  } finally {
    saving.value = false
  }
}

function addProject() {
  projects.value.push({ id: null, path: '', agentIds: [], _tempId: crypto.randomUUID() })
}

async function removeProject(project: LocalProject) {
  if (project.id) {
    try {
      await deleteProject(project.id)
    } catch (err) {
      error('Failed to delete project')
      return
    }
  }
  lastSavedAgentIds.value.delete(project._tempId)
  projects.value = projects.value.filter(existing => existing._tempId !== project._tempId)
  if (projects.value.length === 0) addProject()
}

function toggleAgent(project: LocalProject, agentId: string) {
  if (project.agentIds.includes(agentId)) {
    project.agentIds = project.agentIds.filter(id => id !== agentId)
  } else {
    project.agentIds.push(agentId)
  }
}

async function browseForProject(project: LocalProject) {
  try {
    const selected = await pickProjectFolder()
    if (selected) project.path = selected
  } catch (err: unknown) {
    error(formatError(err, 'Failed to open folder picker'))
  }
}

function getProjectName(path: string) {
  if (!path) return 'New Project'
  const parts = path.split(/[/\\]/).filter(Boolean)
  return parts.length > 0 ? parts[parts.length - 1] : 'New Project'
}
</script>

<template>
  <PageLayout title="Projects" class="pb-20">
    <template #actions>
      <BaseButton @click="addProject" :disabled="saving">New</BaseButton>
      <BaseButton variant="primary" @click="saveConfig" :disabled="saving">{{ saving ? 'Saving...' : 'Save' }}</BaseButton>
    </template>

    <div class="bg-card/70 backdrop-blur-md border border-white/10 p-6 rounded-md mb-6">
      <div class="flex flex-col gap-4">
        <CardItem
          v-for="project in projects"
          :key="project.id?.toString() ?? project._tempId"
          :title="getProjectName(project.path)"
        >
          <template #actions>
            <BaseButton variant="danger" icon @click="removeProject(project)">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </BaseButton>
          </template>

          <div class="flex gap-2 items-end">
            <InputField label="Local Path" v-model="project.path" placeholder="/home/user/projects/app" class="flex-1" />
            <BaseButton variant="secondary" @click="browseForProject(project)" title="Browse for folder">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
              Browse
            </BaseButton>
          </div>

          <div class="flex flex-col gap-1 mb-3 mt-3">
            <label class="text-sm text-muted font-medium">Supported Agents</label>
            <div v-if="availableAgents.length === 0" class="text-sm text-muted">
              No agents configured. Please add them in the Agents tab.
            </div>
            <div class="flex flex-wrap gap-2">
              <TickBox
                v-for="agent in availableAgents"
                :key="agent.id"
                class="bg-page py-[0.4rem] px-3 rounded-sm border border-divider transition-all hover:border-accent"
                :label="agent.name"
                :checked="project.agentIds.includes(agent.id)"
                @change="toggleAgent(project, agent.id)"
              />
            </div>
          </div>
        </CardItem>

        <EmptyState
          v-if="projects.length === 0"
          message="No projects added yet. Click 'New' to get started."
        />
      </div>
    </div>
  </PageLayout>
</template>
