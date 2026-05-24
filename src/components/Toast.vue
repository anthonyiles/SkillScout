<script setup lang="ts">
import { useToast } from '../composables/useToast'
import BaseButton from './BaseButton.vue'

const { toasts, removeToast } = useToast()

const borderClass: Record<string, string> = {
  error:   'border-l-danger',
  success: 'border-l-success',
  info:    'border-l-accent',
}

const iconClass: Record<string, string> = {
  error:   'text-danger',
  success: 'text-success',
  info:    'text-accent',
}
</script>

<template>
  <div class="fixed bottom-4 right-4 z-[9999] flex flex-col gap-3 w-[350px] max-w-[calc(100vw-2rem)] pointer-events-none">
    <TransitionGroup
      enter-active-class="transition-all duration-300 ease-in-out"
      leave-active-class="transition-all duration-300 ease-in-out"
      enter-from-class="opacity-0 translate-x-full"
      leave-to-class="opacity-0 -translate-y-5"
    >
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="pointer-events-auto flex items-start gap-3 p-4 rounded-md shadow-[0_4px_12px_rgba(0,0,0,0.3)] bg-card/70 backdrop-blur-md border border-white/10 border-l-4"
        :class="borderClass[toast.type] ?? 'border-l-accent'"
      >
        <div :class="iconClass[toast.type] ?? 'text-accent'">
          <svg v-if="toast.type === 'error'" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
          <svg v-if="toast.type === 'success'" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          <svg v-if="toast.type === 'info'" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
        </div>
        <div class="flex-1 text-sm leading-[1.4] break-words">{{ toast.message }}</div>
        <BaseButton variant="ghost" icon @click="removeToast(toast.id)">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </BaseButton>
      </div>
    </TransitionGroup>
  </div>
</template>
