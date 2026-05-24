<script setup lang="ts">
defineProps<{
  message: string
  glass?: boolean
}>()
</script>

<template>
  <div
    class="flex flex-col items-center justify-center py-12 px-8 text-center text-muted rounded-md"
    :class="glass
      ? 'bg-card/40 backdrop-blur border border-white/10'
      : 'bg-white/[0.02] border border-dashed border-divider'"
  >
    <div class="mb-5 opacity-50">
      <slot name="icon">
        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false"><circle cx="12" cy="12" r="10"/><line x1="8" y1="12" x2="16" y2="12"/></svg>
      </slot>
    </div>
    <p class="max-w-[400px] leading-normal">{{ message }}</p>
    <div v-if="$slots.actions" class="mt-6">
      <slot name="actions"></slot>
    </div>
  </div>
</template>
