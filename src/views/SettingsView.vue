<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast'
import { formatError } from '../utils/formatError'
import BaseButton from '../components/BaseButton.vue'
import InputField from '../components/InputField.vue'
import PageLayout from '../components/PageLayout.vue'

const repoUrl = ref('')
const saving = ref(false)
const { success, error } = useToast()

const GITHUB_HTTPS_PATTERN = /^https:\/\/github\.com\/[\w.-]+\/[\w.-]+(\.git)?$/
const GITHUB_SSH_PATTERN = /^git@github\.com:[\w.-]+\/[\w.-]+(\.git)?$/

function isValidGitHubRepoUrl(url: string): boolean {
  const trimmed = url.trim()
  return GITHUB_HTTPS_PATTERN.test(trimmed) || GITHUB_SSH_PATTERN.test(trimmed)
}

onMounted(async () => {
  try {
    const savedRepo = await invoke<string | null>('get_setting', { key: 'repoUrl' })
    if (savedRepo) repoUrl.value = savedRepo
  } catch (e: any) {
    console.error('Failed to load settings:', e)
  }
})

async function saveConfig() {
  if (repoUrl.value && !isValidGitHubRepoUrl(repoUrl.value)) {
    error('Repository URL must be a GitHub HTTPS or SSH URL (e.g. git@github.com:org/repo.git)')
    return
  }
  saving.value = true
  try {
    await invoke('set_setting', { key: 'repoUrl', value: repoUrl.value.trim() })
    success('Configuration saved successfully!')
  } catch (err: unknown) {
    error(formatError(err, 'Failed to save configuration'))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <PageLayout title="Settings">
    <template #actions>
      <BaseButton variant="primary" @click="saveConfig" :disabled="saving">{{ saving ? 'Saving...' : 'Save' }}</BaseButton>
    </template>

    <div class="bg-card/70 backdrop-blur-md border border-white/10 p-6 rounded-md mb-6">
      <h2 class="text-xl font-semibold mb-2">Repository Settings</h2>
      <p class="text-sm text-muted mb-4">Configure the private GitHub repository where skills are stored. We use your local SSH agent for authentication.</p>
      <InputField label="Clone URL (HTTPS or SSH)" v-model="repoUrl" placeholder="git@github.com:org/repo.git" />
    </div>
  </PageLayout>
</template>
