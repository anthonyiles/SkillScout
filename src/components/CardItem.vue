<script setup lang="ts">
defineProps<{
  title?: string
  subtitle?: string
  glass?: boolean
}>()
</script>

<template>
  <div class="card-item" :class="{ 'glass': glass }">
    <div v-if="title || $slots.title || $slots.actions" class="card-header">
      <div class="title-group">
        <slot name="title">
          <h4 v-if="title">{{ title }}</h4>
        </slot>
        <span v-if="subtitle" class="subtitle">{{ subtitle }}</span>
      </div>
      <div v-if="$slots.actions" class="card-actions">
        <slot name="actions"></slot>
      </div>
    </div>
    
    <div class="card-body">
      <slot></slot>
    </div>
  </div>
</template>

<style scoped>
.card-item {
  background: var(--bg-surface-hover);
  padding: 1.25rem;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  margin-bottom: 1rem;
}

.card-item.glass {
  background: rgba(30, 30, 30, 0.7);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid var(--border-color);
}

.title-group {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.card-header h4 {
  margin: 0;
  font-weight: 600;
  color: var(--accent-primary);
}

.subtitle {
  background: var(--bg-surface-hover);
  color: var(--text-secondary);
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
  border-radius: var(--radius-full);
}

.card-actions {
  display: flex;
  gap: 0.5rem;
}

.card-body {
  width: 100%;
}
</style>
