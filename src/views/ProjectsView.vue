<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast'
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

interface Project {
  id: number | null
  path: string
  agentIds: string[]
  _tempId?: string
}

const projects = ref<Project[]>([])
const availableAgents = ref<Agent[]>([])
const { success, error } = useToast()

async function loadData() {
  try {
    const fetchedProjects = await invoke<Project[]>('get_projects')
    if (fetchedProjects && fetchedProjects.length > 0) {
      projects.value = fetchedProjects.map(p => ({ ...p, _tempId: p.id?.toString() || crypto.randomUUID() }))
    } else {
      addProject()
    }
  } catch (e) {
    console.error('Failed to load projects:', e)
    addProject()
  }

  try {
    const fetchedAgents = await invoke<Agent[]>('get_agents')
    if (fetchedAgents) {
      availableAgents.value = fetchedAgents
    }
  } catch (e) {
    console.error('Failed to load agents:', e)
  }
}

onMounted(() => {
  loadData()
})

async function saveConfig() {
  try {
    const updatedProjects = []
    for (const project of projects.value) {
      if (!project.path) continue
      const saved = await invoke<Project>('save_project', {
        project: { id: project.id, path: project.path, agentIds: project.agentIds }
      })
      updatedProjects.push({ ...saved, _tempId: saved.id?.toString() || crypto.randomUUID() })
    }
    projects.value = updatedProjects
    if (projects.value.length === 0) addProject()
    success('Projects saved successfully!')
  } catch (e: any) {
    error(typeof e === 'string' ? e : 'Failed to save projects')
  }
}

function addProject() {
  projects.value.push({ id: null, path: '', agentIds: [], _tempId: crypto.randomUUID() })
}

async function removeProject(project: Project) {
  if (project.id) {
    try {
      await invoke('delete_project', { id: project.id })
    } catch (e) {
      error('Failed to delete project')
      return
    }
  }
  projects.value = projects.value.filter(p => p._tempId !== project._tempId)
  if (projects.value.length === 0) addProject()
}

function toggleAgent(project: Project, agentId: string) {
  if (project.agentIds.includes(agentId)) {
    project.agentIds = project.agentIds.filter(id => id !== agentId)
  } else {
    project.agentIds.push(agentId)
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
      <BaseButton @click="addProject">New</BaseButton>
      <BaseButton variant="primary" @click="saveConfig">Save</BaseButton>
    </template>

    <div class="glass p-6 rounded-md mb-6">
      <div class="flex flex-col gap-4">
        <CardItem
          v-for="project in projects"
          :key="project._tempId"
          :title="getProjectName(project.path)"
        >
          <template #actions>
            <BaseButton variant="danger" icon @click="removeProject(project)">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </BaseButton>
          </template>

          <div class="flex gap-4">
            <InputField label="Local Path" v-model="project.path" placeholder="/home/user/projects/app" class="[flex:2]" />
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
