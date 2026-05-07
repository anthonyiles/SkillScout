<script setup lang="ts">
defineProps<{
  message: string
  glass?: boolean
}>()
</script>

<template>
  <div class="empty-state" :class="{ 'glass': glass }">
    <div class="empty-icon">
      <slot name="icon">
        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="text-secondary"><circle cx="12" cy="12" r="10"/><line x1="8" y1="12" x2="16" y2="12"/></svg>
      </slot>
    </div>
    <p>{{ message }}</p>
    <div v-if="$slots.actions" class="empty-actions">
      <slot name="actions"></slot>
    </div>
  </div>
</template>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem 2rem;
  text-align: center;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.02);
  border: 1px dashed var(--border-color);
}

.empty-state.glass {
  background: rgba(30, 30, 30, 0.4);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.empty-icon {
  margin-bottom: 1.25rem;
  opacity: 0.5;
}

.empty-actions {
  margin-top: 1.5rem;
}

p {
  max-width: 400px;
  line-height: 1.5;
}
</style>
