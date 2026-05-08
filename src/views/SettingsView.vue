<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useToast } from '../composables/useToast'
import BaseButton from '../components/BaseButton.vue'
import InputField from '../components/InputField.vue'
import PageLayout from '../components/PageLayout.vue'

const repoUrl = ref('')
const { success } = useToast()

onMounted(() => {
  const savedRepo = localStorage.getItem('repoUrl')
  if (savedRepo) repoUrl.value = savedRepo
})

function saveConfig() {
  localStorage.setItem('repoUrl', repoUrl.value)
  success('Configuration saved successfully!')
}
</script>

<template>
  <PageLayout title="Settings">
    <template #actions>
      <BaseButton variant="primary" @click="saveConfig">Save</BaseButton>
    </template>

    <div class="settings-section glass">
      <h2 class="text-h2">Repository Settings</h2>
      <p class="text-body mb-4">Configure the private GitHub repository where skills are stored. We use your local SSH agent for authentication.</p>
      
      <InputField label="SSH Clone URL" v-model="repoUrl" placeholder="git@github.com:org/repo.git" />
    </div>
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



.text-sm { font-size: 0.85rem; }
.text-secondary { color: var(--text-secondary); }
</style>
