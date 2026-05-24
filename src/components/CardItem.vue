<script setup lang="ts">
defineProps<{
  title?: string
  subtitle?: string
  glass?: boolean
}>()
</script>

<template>
  <div
    class="p-5 rounded-md border mb-4"
    :class="glass
      ? 'bg-card/70 backdrop-blur-md border-white/10'
      : 'bg-card-hover border-divider'"
  >
    <div v-if="title || $slots.title || $slots.actions" class="flex justify-between items-center mb-4 pb-3 border-b border-divider">
      <div class="flex items-center gap-3">
        <slot name="title">
          <h4 v-if="title" class="m-0 font-semibold text-accent">{{ title }}</h4>
        </slot>
        <span v-if="subtitle" class="bg-card-hover text-muted text-xs px-2 py-1 rounded-full">{{ subtitle }}</span>
      </div>
      <div v-if="$slots.actions" class="flex gap-2">
        <slot name="actions"></slot>
      </div>
    </div>

    <div class="w-full">
      <slot></slot>
    </div>
  </div>
</template>
