<script setup lang="ts">
import { watch, onBeforeUnmount, ref } from 'vue'
import { useFocusTrap } from '@vueuse/integrations/useFocusTrap'
import BaseButton from './BaseButton.vue'

const props = defineProps<{
  isOpen: boolean
  title: string
  content: string
}>()

const emit = defineEmits(['close'])
const modalRef = ref<HTMLElement | null>(null)
const { activate, deactivate } = useFocusTrap(modalRef)

function close() {
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
  } else {
    window.removeEventListener('keydown', handleKeydown)
    deactivate()
  }
}, { immediate: true })

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  deactivate()
})
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div 
      ref="modalRef"
      class="modal-content glass"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="`modal-title-${$.uid}`"
    >
      <div class="modal-header">
        <h3 :id="`modal-title-${$.uid}`" class="text-h2">{{ title }}</h3>
        <BaseButton variant="ghost" icon @click="close">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </BaseButton>
      </div>
      <div class="modal-body">
        <pre class="content-preview">{{ content || 'No content found.' }}</pre>
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
  width: 80%;
  max-width: 800px;
  max-height: 80vh;
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
  padding: 1.5rem;
  overflow-y: auto;
}

.content-preview {
  background: var(--bg-base);
  padding: 1rem;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  font-family: monospace;
  font-size: 0.85rem;
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
}
</style>
