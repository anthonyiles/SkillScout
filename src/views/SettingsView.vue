<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useToast } from '../composables/useToast'
import { useUpdater } from '../composables/useUpdater'
import { getSetting, setSetting } from '../api'
import { formatError } from '../utils/formatError'
import BaseButton from '../components/BaseButton.vue'
import InputField from '../components/InputField.vue'
import PageLayout from '../components/PageLayout.vue'
import TickBox from '../components/TickBox.vue'

const repoUrl = ref('')
const saving = ref(false)
const { success, error } = useToast()
const { updateAvailable, checking, installing, installPercent, isBetaTester, checkForUpdate, installUpdate, setBetaTester } = useUpdater()

async function onBetaTesterChange(value: boolean) {
  try {
    await setBetaTester(value)
  } catch (e) {
    error(formatError(e, 'Failed to save beta tester preference'))
  }
}

const GITHUB_HTTPS_PATTERN = /^https:\/\/github\.com\/[\w.-]+\/[\w.-]+(\.git)?$/
const GITHUB_SSH_PATTERN = /^git@github\.com:[\w.-]+\/[\w.-]+(\.git)?$/

function isValidGitHubRepoUrl(url: string): boolean {
  const trimmed = url.trim()
  return GITHUB_HTTPS_PATTERN.test(trimmed) || GITHUB_SSH_PATTERN.test(trimmed)
}

onMounted(async () => {
  try {
    const savedRepo = await getSetting('repoUrl')
    if (savedRepo) repoUrl.value = savedRepo
  } catch (e) {
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
    await setSetting('repoUrl', repoUrl.value.trim())
    success('Configuration saved successfully!')
  } catch (err: unknown) {
    error(formatError(err, 'Failed to save configuration'))
  } finally {
    saving.value = false
  }
}

function installButtonLabel() {
  if (!installing.value) return 'Install & Relaunch'
  if (installPercent.value !== null) return `Downloading ${installPercent.value}%`
  return 'Downloading...'
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

    <div class="bg-card/70 backdrop-blur-md border border-white/10 p-6 rounded-md">
      <h2 class="text-xl font-semibold mb-2">App Updates</h2>
      <p class="text-sm text-muted mb-4">SkillScout checks for updates on launch.</p>

      <div class="mb-4">
        <TickBox
          :checked="isBetaTester"
          label="Receive beta updates"
          @change="onBetaTesterChange"
        />
        <p class="text-xs text-muted mt-1 ml-6">Beta releases may be less stable but include the latest features.</p>
      </div>


      <div v-if="updateAvailable" class="mb-4 p-4 bg-primary/10 border border-primary/30 rounded-md">
        <p class="text-sm font-medium mb-1">Version {{ updateAvailable.version }} is available</p>
        <p v-if="updateAvailable.notes" class="text-sm text-muted mb-3 whitespace-pre-line">{{ updateAvailable.notes }}</p>
        <BaseButton variant="primary" :loading="installing" :disabled="installing" @click="installUpdate">
          {{ installButtonLabel() }}
        </BaseButton>
      </div>

      <div v-else class="flex items-center gap-3">
        <BaseButton :loading="checking" :disabled="checking" @click="() => checkForUpdate()">
          {{ checking ? 'Checking...' : 'Check for Updates' }}
        </BaseButton>
        <span v-if="!checking" class="text-sm text-muted">SkillScout is up to date.</span>
      </div>
    </div>
  </PageLayout>
</template>
