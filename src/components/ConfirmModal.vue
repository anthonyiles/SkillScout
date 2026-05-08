<script setup lang="ts">
import { watch, nextTick, ref, onBeforeUnmount } from 'vue'
import BaseButton from './BaseButton.vue'

const props = defineProps<{
  isOpen: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  danger?: boolean
}>()

const emit = defineEmits(['confirm', 'cancel'])
const modalContentRef = ref<HTMLElement | null>(null)
const previousActiveElement = ref<HTMLElement | null>(null)

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.isOpen) {
    emit('cancel')
  }
}

watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    previousActiveElement.value = document.activeElement as HTMLElement
    window.addEventListener('keydown', handleKeydown)
    nextTick(() => {
      modalContentRef.value?.focus()
    })
  } else {
    window.removeEventListener('keydown', handleKeydown)
    previousActiveElement.value?.focus()
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="emit('cancel')">
    <div class="modal-content glass" role="dialog" aria-modal="true" :aria-labelledby="'modal-title-' + title" :aria-describedby="'modal-message-' + title" tabindex="-1" ref="modalContentRef">
      <div class="modal-header">
        <h3 class="text-h2" :id="'modal-title-' + title">{{ title }}</h3>
      </div>
      <div class="modal-body">
        <p class="text-body" :id="'modal-message-' + title">{{ message }}</p>
      </div>
      <div class="modal-actions">
        <BaseButton variant="secondary" @click="emit('cancel')">{{ cancelText || 'Cancel' }}</BaseButton>
        <BaseButton :variant="danger ? 'danger' : 'primary'" @click="emit('confirm')">{{ confirmText || 'Confirm' }}</BaseButton>
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
  z-index: 2000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-content {
  width: 90%;
  max-width: 400px;
  border-radius: var(--radius-md);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: 1.5rem 1.5rem 0.5rem;
}

.modal-header h3 {
  margin: 0;
  color: var(--text-primary);
}

.modal-body {
  padding: 0.5rem 1.5rem 1.5rem;
  color: var(--text-secondary);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  padding: 1rem 1.5rem;
  background: var(--bg-base);
  border-top: 1px solid var(--border-color);
  border-bottom-left-radius: var(--radius-md);
  border-bottom-right-radius: var(--radius-md);
}


</style>
