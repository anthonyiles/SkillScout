<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast'
import { formatError } from '../utils/formatError'
import ConfirmModal from '../components/ConfirmModal.vue'
import BaseButton from '../components/BaseButton.vue'
import InputField from '../components/InputField.vue'
import PageLayout from '../components/PageLayout.vue'
import CardItem from '../components/CardItem.vue'

interface Agent {
  id: string
  name: string
  skillsPath: string
  rulesPath: string
}

const agents = ref<Agent[]>([])
const { success, error } = useToast()

async function loadAgents() {
  try {
    const fetched = await invoke<Agent[]>('get_agents')
    if (fetched) {
      agents.value = fetched
    }
  } catch (err) {
    console.error('Failed to load agents:', err)
  }
}

onMounted(() => {
  loadAgents()
})

async function saveConfig() {
  try {
    for (const agent of agents.value) {
      await invoke('save_agent', { agent })
    }
    success('Agent configurations saved successfully!')
  } catch (err: unknown) {
    error(formatError(err, 'Failed to save agents'))
  }
}

function addAgent() {
  const uniqueId = `custom-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  agents.value.push({
    id: uniqueId,
    name: 'New Agent',
    skillsPath: '',
    rulesPath: ''
  })
}

async function removeAgent(id: string) {
  try {
    await invoke('delete_agent', { id })
    agents.value = agents.value.filter(agent => agent.id !== id)
    success('Agent removed.')
  } catch {
    error('Failed to remove agent')
  }
}

const isConfirmOpen = ref(false)

function resetToDefaults() {
  isConfirmOpen.value = true
}

async function executeReset() {
  try {
    await invoke('reset_agents_to_defaults')
    await loadAgents()
    success('Restored default agents.')
  } catch (err) {
    error('Failed to reset to defaults')
  } finally {
    isConfirmOpen.value = false
  }
}
</script>

<template>
  <PageLayout title="Agents">
    <template #actions>
      <BaseButton variant="danger" @click="resetToDefaults">Reset to Defaults</BaseButton>
      <BaseButton variant="primary" @click="saveConfig">Save</BaseButton>
    </template>

    <div class="bg-card/70 backdrop-blur-md border border-white/10 p-6 rounded-md mb-6">
      <p class="text-sm text-muted mb-4">Define the relative folder paths where skills and rules should be copied for each AI agent.</p>

      <div class="flex flex-col gap-4 mb-4">
        <CardItem v-for="agent in agents" :key="agent.id">
          <template #title>
            <input
              v-model="agent.name"
              placeholder="Agent Name"
              aria-label="Agent name"
              class="text-[1.1rem] font-semibold text-accent bg-transparent border border-transparent px-2 py-1 rounded-sm outline-none transition-colors focus:bg-page focus:border-accent"
            />
          </template>
          <template #actions>
            <BaseButton variant="danger" icon @click="removeAgent(agent.id)" title="Remove Agent">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </BaseButton>
          </template>

          <div class="flex gap-4">
            <InputField label="Skills Target Path" v-model="agent.skillsPath" placeholder="e.g. .cursor/skills" />
            <InputField label="Rules Target Path" v-model="agent.rulesPath" placeholder="e.g. .cursor/rules" />
          </div>
        </CardItem>
      </div>

      <div>
        <BaseButton @click="addAgent">Add Custom Agent</BaseButton>
      </div>
    </div>

    <ConfirmModal
      :isOpen="isConfirmOpen"
      title="Reset to Defaults?"
      message="This will erase all custom agents and restore the defaults. Are you sure you want to proceed?"
      confirmText="Reset Agents"
      :danger="true"
      @confirm="executeReset"
      @cancel="isConfirmOpen = false"
    />
  </PageLayout>
</template>
