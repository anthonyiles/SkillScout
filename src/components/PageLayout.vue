<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div class="p-6 w-full h-screen overflow-y-auto">
    <div
      class="mb-6"
      :class="{ 'flex justify-between items-center': $slots.actions }"
    >
      <div>
        <h1 class="text-[1.75rem] font-bold mb-3">{{ title }}</h1>
        <p v-if="description" class="text-muted mt-1">{{ description }}</p>
      </div>
      <div v-if="$slots.actions" class="flex gap-4 items-center">
        <slot name="actions"></slot>
      </div>
    </div>

    <slot></slot>
  </div>
</template>
