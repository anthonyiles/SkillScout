<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div class="view-container">
    <div class="header" :class="{ 'has-actions': $slots.actions }">
      <div class="title-section">
        <h1 class="text-h1">{{ title }}</h1>
        <p v-if="description" class="text-secondary mt-1">{{ description }}</p>
      </div>
      <div v-if="$slots.actions" class="actions">
        <slot name="actions"></slot>
      </div>
    </div>
    
    <div class="content">
      <slot></slot>
    </div>
  </div>
</template>

<style scoped>
.view-container {
  padding: 1.5rem;
  width: 100%;
  height: 100vh;
  overflow-y: auto;
}

.header {
  margin-bottom: 1.5rem;
}

.header.has-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.actions {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.mt-1 {
  margin-top: 0.25rem;
}
</style>
