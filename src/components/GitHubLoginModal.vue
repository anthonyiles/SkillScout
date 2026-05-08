<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useFocusTrap } from '@vueuse/integrations/useFocusTrap'
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

let pollingIntervalId: ReturnType<typeof setInterval> | null = null

async function startFlow() {
  if (isGenerating.value) return
  isGenerating.value = true
  errorMsg.value = ''
  
  try {
    const res: any = await invoke('start_github_device_flow')
    userCode.value = res.user_code
    verificationUri.value = res.verification_uri
    
    // Start polling for token
    startPolling(res.device_code, res.interval)
  } catch (e: any) {
    errorMsg.value = e.toString()
  } finally {
    isGenerating.value = false
  }
}

function startPolling(deviceCode: string, intervalSeconds: number) {
  if (pollingIntervalId) clearInterval(pollingIntervalId)
  
  // Convert interval from seconds to milliseconds, adding a slight buffer
  const intervalMs = (intervalSeconds + 1) * 1000
  
  pollingIntervalId = setInterval(async () => {
    try {
      const res: any = await invoke('poll_github_token', { 
        deviceCode 
      })
      
      if (res.access_token) {
        // Success!
        stopPolling()
        emit('authenticated')
        close()
      } else if (res.error) {
        if (res.error === 'authorization_pending') {
          // Keep waiting
        } else if (res.error === 'slow_down') {
          // In a real app we'd increase the interval dynamically, 
          // but for simplicity we'll just keep polling with the current one 
          // (or you could implement dynamic interval adjustments here)
        } else if (res.error === 'expired_token') {
          errorMsg.value = 'The code expired. Please try again.'
          stopPolling()
        } else {
          errorMsg.value = res.error_description || res.error
          stopPolling()
        }
      }
    } catch (e: any) {
      errorMsg.value = e.toString()
      stopPolling()
    }
  }, intervalMs)
}

function stopPolling() {
  if (pollingIntervalId) {
    clearInterval(pollingIntervalId)
    pollingIntervalId = null
  }
}

function openGitHub() {
  if (verificationUri.value) {
    openUrl(verificationUri.value)
  }
}

async function copyCode() {
  if (userCode.value) {
    await navigator.clipboard.writeText(userCode.value)
  }
}

async function copyUri() {
  if (verificationUri.value) {
    await navigator.clipboard.writeText(verificationUri.value)
  }
}

function close() {
  stopPolling()
  emit('close')
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.isOpen) {
    close()
  }
}

watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    window.addEventListener('keydown', handleKeydown)
    activate()
    startFlow()
  } else {
    window.removeEventListener('keydown', handleKeydown)
    deactivate()
    stopPolling()
    userCode.value = ''
    verificationUri.value = ''
    errorMsg.value = ''
  }
}, { immediate: true })

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  deactivate()
  stopPolling()
})
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div 
      ref="modalRef"
      class="modal-content glass"
      role="dialog"
      aria-modal="true"
      aria-labelledby="github-modal-title"
    >
      <div class="modal-header">
        <h3 id="github-modal-title" class="text-h2">Connect GitHub</h3>
        <BaseButton variant="ghost" icon @click="close" aria-label="Close">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </BaseButton>
      </div>
      
      <div class="modal-body text-center">
        <div v-if="isGenerating" class="loading-state">
          <div class="spinner"></div>
          <p>Generating authentication code...</p>
        </div>
        
        <div v-else-if="errorMsg" class="error-state">
          <p class="error-text">{{ errorMsg }}</p>
          <BaseButton @click="startFlow">Try Again</BaseButton>
        </div>
        
        <div v-else-if="userCode" class="auth-steps">
          <p class="instructions">
            1. Copy the code below.<br>
            2. Click "Open GitHub" and paste the code to authorize this application.
          </p>
          
          <div class="code-box">
            <span class="user-code">{{ userCode }}</span>
            <BaseButton variant="ghost" icon @click="copyCode" title="Copy to clipboard">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
            </BaseButton>
          </div>
          
          <div class="manual-link-container">
            <p class="manual-text">Or manually go to:</p>
            <div class="manual-link-box">
              <a :href="verificationUri" target="_blank" rel="noopener noreferrer" class="manual-link">{{ verificationUri }}</a>
              <BaseButton variant="ghost" icon @click="copyUri" title="Copy URL" class="copy-small">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
              </BaseButton>
            </div>
          </div>
          
          <BaseButton class="open-github-btn" @click="openGitHub">
            Open GitHub
          </BaseButton>
          
          <p class="waiting-text">
            <span class="spinner small"></span> Waiting for authorization...
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-content {
  width: 90%;
  max-width: 450px;
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-md);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  color: var(--accent-primary);
}

.modal-body {
  padding: 2rem 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.text-center {
  text-align: center;
}

.instructions {
  color: var(--text-secondary);
  margin-bottom: 1.5rem;
  line-height: 1.5;
}

.code-box {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  background: var(--bg-base);
  padding: 1rem 1.5rem;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
  margin-bottom: 2rem;
}

.user-code {
  font-family: monospace;
  font-size: 2rem;
  font-weight: bold;
  letter-spacing: 4px;
  color: var(--text-primary);
}

.open-github-btn {
  width: 100%;
  margin-bottom: 1.5rem;
  padding: 1rem;
  font-size: 1.1rem;
}

.manual-link-container {
  width: 100%;
  margin-bottom: 1.5rem;
  text-align: center;
}

.manual-text {
  color: var(--text-secondary);
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
}

.manual-link-box {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  background: var(--bg-surface-hover);
  padding: 0.5rem 1rem;
  border-radius: var(--radius-sm);
}

.manual-link {
  color: var(--accent-primary);
  font-size: 0.85rem;
  text-decoration: none;
  word-break: break-all;
}

.manual-link:hover {
  text-decoration: underline;
}

.copy-small {
  padding: 0.25rem;
  height: auto;
  min-height: 0;
  color: var(--text-secondary);
}

.waiting-text {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.error-text {
  color: #ff5555;
  margin-bottom: 1rem;
}

/* Simple Spinner */
.spinner {
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top: 3px solid var(--accent-primary);
  border-radius: 50%;
  width: 30px;
  height: 30px;
  animation: spin 1s linear infinite;
  margin: 0 auto 1rem;
}

.spinner.small {
  width: 16px;
  height: 16px;
  border-width: 2px;
  margin: 0;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>
