<script setup lang="ts">
import { watch, onBeforeUnmount, ref, nextTick } from 'vue'
import { useFocusTrap } from '@vueuse/integrations/useFocusTrap'
import { useEscapeKey } from '../composables/useEscapeKey'
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

useEscapeKey(() => props.isOpen, close)

watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    await nextTick()
    activate()
  } else {
    deactivate()
  }
}, { immediate: true })

onBeforeUnmount(() => {
  deactivate()
})
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[1000] flex items-center justify-center"
    @click.self="close"
  >
    <div
      ref="modalRef"
      class="w-4/5 max-w-[800px] max-h-[80vh] flex flex-col rounded-md shadow-[0_10px_40px_rgba(0,0,0,0.5)] bg-card border border-divider"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="`modal-title-${$.uid}`"
    >
      <div class="flex justify-between items-center p-6 border-b border-divider">
        <h3 :id="`modal-title-${$.uid}`" class="text-xl font-semibold text-accent">{{ title }}</h3>
        <BaseButton variant="ghost" icon @click="close">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </BaseButton>
      </div>
      <div class="p-6 overflow-y-auto">
        <pre class="bg-page p-4 rounded-sm border border-divider text-white font-mono text-sm whitespace-pre-wrap break-words m-0">{{ content || 'No content found.' }}</pre>
      </div>
    </div>
  </div>
</template>
