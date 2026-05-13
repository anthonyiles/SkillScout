<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useToast } from '../composables/useToast'
import { useAgents } from '../composables/useAgents'
import ConfirmModal from '../components/ConfirmModal.vue'
import BaseButton from '../components/BaseButton.vue'
import InputField from '../components/InputField.vue'
import PageLayout from '../components/PageLayout.vue'
import CardItem from '../components/CardItem.vue'

const { agents, load, save, reset } = useAgents()
const { success } = useToast()
const isConfirmOpen = ref(false)

onMounted(load)

function saveConfig() {
  save()
  success('Agent configurations saved successfully!')
}

function addAgent() {
  agents.value.push({
    id: `custom-${crypto.randomUUID()}`,
    name: 'New Agent',
    skillsPath: '',
    rulesPath: '',
  })
}

function removeAgent(id: string) {
  agents.value = agents.value.filter(a => a.id !== id)
}

function executeReset() {
  reset()
  success('Restored default agents.')
  isConfirmOpen.value = false
}
</script>

<template>
  <PageLayout title="Agents">
    <template #actions>
      <BaseButton variant="danger" @click="isConfirmOpen = true">Reset to Defaults</BaseButton>
      <BaseButton variant="primary" @click="saveConfig">Save</BaseButton>
    </template>

    <div class="settings-section glass">
      <p class="text-body mb-4">Define the relative folder paths where skills and rules should be copied for each AI agent.</p>

      <div class="agents-list">
        <CardItem v-for="agent in agents" :key="agent.id" :title="agent.name">
          <template #actions>
            <BaseButton variant="danger" icon @click="removeAgent(agent.id)" title="Remove Agent">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </BaseButton>
          </template>

          <div class="form-row">
            <InputField label="Agent Name" v-model="agent.name" placeholder="Agent Name" />
            <InputField label="Skills Target Path" v-model="agent.skillsPath" placeholder="e.g. .cursor/skills" />
            <InputField label="Rules Target Path" v-model="agent.rulesPath" placeholder="e.g. .cursor/rules" />
          </div>
        </CardItem>
      </div>

      <div class="actions">
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

<style scoped>
.settings-section {
  padding: 1.5rem;
  border-radius: var(--radius-md);
  margin-bottom: 1.5rem;
}

.mb-4 {
  margin-bottom: 1rem;
}

.form-row {
  display: flex;
  gap: 1rem;
}

.agents-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-bottom: 1rem;
}
</style>
