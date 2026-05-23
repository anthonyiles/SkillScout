<script setup lang="ts">
import { ref, watch, onBeforeUnmount, nextTick } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { startGithubDeviceFlow, pollGithubToken } from '../api'
import { useFocusTrap } from '@vueuse/integrations/useFocusTrap'
import { useEscapeKey } from '../composables/useEscapeKey'
import BaseButton from './BaseButton.vue'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close', 'authenticated'])
const modalRef = ref<HTMLElement | null>(null)
const { activate, deactivate } = useFocusTrap(modalRef)

const isGenerating = ref(false)
const userCode = ref('')
const verificationUri = ref('')
const errorMsg = ref('')

let pollingIntervalId: ReturnType<typeof setTimeout> | null = null
const copySuccessMsg = ref('')

async function startFlow() {
  if (isGenerating.value) return
  isGenerating.value = true
  errorMsg.value = ''
  try {
    const res = await startGithubDeviceFlow()
    userCode.value = res.userCode
    verificationUri.value = res.verificationUri
    startPolling(res.deviceCode, res.interval)
  } catch (err: unknown) {
    errorMsg.value = err instanceof Error ? err.message : String(err)
  } finally {
    isGenerating.value = false
  }
}

function startPolling(deviceCode: string, intervalSeconds: number) {
  if (pollingIntervalId) clearTimeout(pollingIntervalId)
  let intervalMs = (intervalSeconds + 1) * 1000
  const poll = () => {
    pollingIntervalId = setTimeout(async () => {
      try {
        const res = await pollGithubToken(deviceCode)
        if (res.accessToken) {
          stopPolling(); emit('authenticated'); close()
        } else if (res.error) {
          if (res.error === 'authorization_pending') poll()
          else if (res.error === 'slow_down') { intervalMs += 5000; poll() }
          else if (res.error === 'expired_token') { errorMsg.value = 'The code expired. Please try again.'; stopPolling() }
          else { errorMsg.value = res.errorDescription || res.error; stopPolling() }
        }
      } catch (err: unknown) { errorMsg.value = err instanceof Error ? err.message : String(err); stopPolling() }
    }, intervalMs)
  }
  poll()
}

function stopPolling() {
  if (pollingIntervalId) { clearTimeout(pollingIntervalId); pollingIntervalId = null }
}

async function openGitHub() {
  if (verificationUri.value) {
    try {
      await openUrl(verificationUri.value)
    } catch { errorMsg.value = 'Failed to open browser.' }
  }
}

async function copyCode() {
  if (userCode.value) {
    try {
      await navigator.clipboard.writeText(userCode.value)
      copySuccessMsg.value = 'Code copied to clipboard!'
      setTimeout(() => copySuccessMsg.value = '', 3000)
    } catch { errorMsg.value = 'Failed to copy code to clipboard.' }
  }
}

async function copyUri() {
  if (verificationUri.value) {
    try {
      await navigator.clipboard.writeText(verificationUri.value)
      copySuccessMsg.value = 'URL copied to clipboard!'
      setTimeout(() => copySuccessMsg.value = '', 3000)
    } catch { errorMsg.value = 'Failed to copy URL to clipboard.' }
  }
}

function close() { stopPolling(); emit('close') }

useEscapeKey(() => props.isOpen, close)

watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    await nextTick(); activate(); startFlow()
  } else {
    deactivate(); stopPolling()
    userCode.value = ''; verificationUri.value = ''; errorMsg.value = ''; isGenerating.value = false
  }
}, { immediate: true })

onBeforeUnmount(() => { deactivate(); stopPolling() })
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[1000] flex items-center justify-center" @click.self="close">
    <div
      ref="modalRef"
      class="w-[90%] max-w-[450px] flex flex-col rounded-md shadow-[0_10px_40px_rgba(0,0,0,0.5)] bg-card border border-divider"
      role="dialog"
      aria-modal="true"
      aria-labelledby="github-modal-title"
    >
      <div class="flex justify-between items-center p-6 border-b border-divider">
        <h3 id="github-modal-title" class="text-xl font-semibold text-accent">Connect GitHub</h3>
        <BaseButton variant="ghost" icon @click="close" aria-label="Close">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </BaseButton>
      </div>

      <div class="py-8 px-6 flex flex-col items-center text-center">
        <!-- Generating -->
        <div v-if="isGenerating" class="flex flex-col items-center gap-4">
          <span class="inline-block w-[30px] h-[30px] rounded-full border-[3px] border-white/10 border-t-accent animate-spin"></span>
          <p class="text-muted">Generating authentication code...</p>
        </div>

        <!-- Error -->
        <div v-else-if="errorMsg" class="flex flex-col items-center gap-4">
          <p class="text-[#ff5555]">{{ errorMsg }}</p>
          <BaseButton @click="startFlow">Try Again</BaseButton>
        </div>

        <!-- Auth steps -->
        <div v-else-if="userCode" class="w-full flex flex-col items-center">
          <p class="text-muted mb-6 leading-relaxed">
            1. Copy the code below.<br>
            2. Click "Open GitHub" and paste the code to authorise this application.
          </p>

          <div class="flex items-center justify-center gap-4 bg-page py-4 px-6 rounded-sm border border-divider mb-8 w-full">
            <span class="font-mono text-[2rem] font-bold tracking-[4px] text-white">{{ userCode }}</span>
            <BaseButton variant="ghost" icon @click="copyCode" title="Copy to clipboard">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
            </BaseButton>
          </div>

          <div class="w-full mb-6 text-center">
            <p class="text-muted text-sm mb-2">Or manually go to:</p>
            <div class="flex items-center justify-center gap-2 bg-card-hover py-2 px-4 rounded-sm">
              <button type="button" @click="openGitHub" class="text-accent text-sm break-all bg-transparent border-0 p-0 cursor-pointer font-[inherit] hover:underline" title="Open GitHub authorization page" aria-label="Open GitHub authorization page">{{ verificationUri }}</button>
              <BaseButton variant="ghost" icon @click="copyUri" title="Copy URL" class="!p-1">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
              </BaseButton>
            </div>
          </div>

          <BaseButton class="w-full !py-4 mb-6 !text-[1.1rem]" @click="openGitHub">
            Open GitHub
          </BaseButton>

          <p v-if="copySuccessMsg" class="text-[#50fa7b] mb-4 text-sm">{{ copySuccessMsg }}</p>

          <p class="flex items-center justify-center gap-2 text-muted text-sm">
            <span class="inline-block w-4 h-4 rounded-full border-2 border-white/10 border-t-accent animate-spin"></span>
            Waiting for authorization...
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
