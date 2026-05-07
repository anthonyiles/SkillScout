<script setup lang="ts">
import { ref, onMounted } from 'vue'
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
  id: number
  path: string
  agentIds: string[]
}

const projects = ref<Project[]>([])
const availableAgents = ref<Agent[]>([])
const { success } = useToast()

onMounted(() => {
  const savedProjects = localStorage.getItem('projects')
  if (savedProjects) {
    // Map legacy projects if any
    const parsed = JSON.parse(savedProjects)
    projects.value = parsed.map((p: any) => ({
      ...p,
      agentIds: p.agentIds || []
    }))
  } else {
    addProject()
  }
  
  const savedAgents = localStorage.getItem('agents')
  if (savedAgents) {
    availableAgents.value = JSON.parse(savedAgents)
  }
})

function saveConfig() {
  localStorage.setItem('projects', JSON.stringify(projects.value))
  success('Projects saved successfully!')
}

function addProject() {
  projects.value.push({ id: Date.now(), path: '', agentIds: [] })
}

function removeProject(id: number) {
  projects.value = projects.value.filter(p => p.id !== id)
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
  <PageLayout title="Projects" style="padding-bottom: 5rem">
    <template #actions>
      <BaseButton @click="addProject">New</BaseButton>
      <BaseButton variant="primary" @click="saveConfig">Save</BaseButton>
    </template>

    <div class="settings-section glass">
      <div class="projects-list">
        <CardItem 
          v-for="project in projects" 
          :key="project.id" 
          :title="getProjectName(project.path)"
        >
          <template #actions>
            <BaseButton variant="danger" icon @click="removeProject(project.id)">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </BaseButton>
          </template>
          
          <div class="form-row">
            <InputField label="Local Path" v-model="project.path" placeholder="/home/user/projects/app" style="flex: 2" />
          </div>
          
          <div class="checkbox-group mt-3">
            <label>Supported Agents</label>
            <div v-if="availableAgents.length === 0" class="text-secondary text-sm">
              No agents configured. Please add them in the Agents tab.
            </div>
            <div class="agent-checkboxes">
              <TickBox 
                v-for="agent in availableAgents" 
                :key="agent.id" 
                class="agent-cb"
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

<style scoped>


.settings-section {
  padding: 1.5rem;
  border-radius: var(--radius-md);
  margin-bottom: 1.5rem;
}

.mt-3 {
  margin-top: 0.75rem;
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  margin-bottom: 0.75rem;
}

.checkbox-group label {
  font-size: 0.85rem;
  color: var(--text-secondary);
  font-weight: 500;
}

.form-row {
  display: flex;
  gap: 1rem;
}

.agent-checkboxes {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

:deep(.agent-cb) {
  background: var(--bg-base);
  padding: 0.4rem 0.75rem;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
  transition: all var(--transition-fast);
}

:deep(.agent-cb:hover) {
  border-color: var(--accent-primary);
}

.projects-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}







.text-sm { font-size: 0.85rem; }
.text-secondary { color: var(--text-secondary); }
</style>
