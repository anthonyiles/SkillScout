<script setup lang="ts">
import { onMounted } from 'vue'
import { useToast } from '../composables/useToast'
import { useProjects } from '../composables/useProjects'
import { useAgents } from '../composables/useAgents'
import type { Project } from '../types'
import { getProjectName } from '../utils/formatters'
import TickBox from '../components/TickBox.vue'
import BaseButton from '../components/BaseButton.vue'
import InputField from '../components/InputField.vue'
import PageLayout from '../components/PageLayout.vue'
import CardItem from '../components/CardItem.vue'
import EmptyState from '../components/EmptyState.vue'

const { projects, load: loadProjects, save: saveProjects, add: addProject, remove: removeProject } = useProjects()
const { agents, load: loadAgents } = useAgents()
const { success } = useToast()

onMounted(() => {
  loadProjects()
  loadAgents()
})

function saveConfig() {
  saveProjects()
  success('Projects saved successfully!')
}

function toggleAgent(project: Project, agentId: string) {
  if (project.agentIds.includes(agentId)) {
    project.agentIds = project.agentIds.filter(id => id !== agentId)
  } else {
    project.agentIds.push(agentId)
  }
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
            <div v-if="agents.length === 0" class="text-secondary text-sm">
              No agents configured. Please add them in the Agents tab.
            </div>
            <div class="agent-checkboxes">
              <TickBox
                v-for="agent in agents"
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
