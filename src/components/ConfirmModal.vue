<script setup lang="ts">
import { watch, nextTick, ref } from 'vue'
import { useEscapeKey } from '../composables/useEscapeKey'
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

useEscapeKey(() => props.isOpen, () => emit('cancel'))

watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    previousActiveElement.value = document.activeElement as HTMLElement
    nextTick(() => modalContentRef.value?.focus())
  } else {
    previousActiveElement.value?.focus()
  }
})
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[2000] flex items-center justify-center"
    @click.self="emit('cancel')"
  >
    <div
      ref="modalContentRef"
      class="w-[90%] max-w-[400px] rounded-md shadow-[0_10px_40px_rgba(0,0,0,0.5)] bg-card border border-divider flex flex-col"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="'modal-title-' + title"
      :aria-describedby="'modal-message-' + title"
      tabindex="-1"
    >
      <div class="px-6 pt-6 pb-2">
        <h3 class="text-xl font-semibold" :id="'modal-title-' + title">{{ title }}</h3>
      </div>
      <div class="px-6 pt-2 pb-6 text-muted" :id="'modal-message-' + title">
        <p>{{ message }}</p>
      </div>
      <div class="flex justify-end gap-4 py-4 px-6 bg-page border-t border-divider rounded-b-md">
        <BaseButton variant="secondary" @click="emit('cancel')">{{ cancelText || 'Cancel' }}</BaseButton>
        <BaseButton :variant="danger ? 'danger' : 'primary'" @click="emit('confirm')">{{ confirmText || 'Confirm' }}</BaseButton>
      </div>
    </div>
  </div>
</template>
