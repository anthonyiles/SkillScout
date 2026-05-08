<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  variant: {
    type: String,
    default: 'default', // 'default', 'primary', 'secondary', 'danger', 'ghost'
  },
  icon: {
    type: Boolean,
    default: false
  },
  disabled: {
    type: Boolean,
    default: false
  },
  size: {
    type: String,
    default: 'md' // 'sm', 'md', 'lg'
  }
})

const buttonClasses = computed(() => {
  return [
    'base-button',
    `variant-${props.variant}`,
    `size-${props.size}`,
    { 'is-icon': props.icon }
  ]
})
</script>

<template>
  <button :class="buttonClasses" :disabled="disabled">
    <slot></slot>
  </button>
</template>

<style scoped>
.base-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.base-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Primary */
.variant-primary {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
}
.variant-primary:hover:not(:disabled) {
  background: var(--accent-primary-hover);
  border-color: var(--accent-primary-hover);
}

/* Secondary */
.variant-secondary {
  background: var(--bg-surface);
  border-color: var(--border-color);
  color: var(--text-primary);
}
.variant-secondary:hover:not(:disabled) {
  background: var(--bg-surface-hover);
  border-color: var(--text-secondary);
}

/* Danger */
.variant-danger {
  background: transparent;
  border-color: var(--accent-danger);
  color: #ffb3b3;
}
.variant-danger:hover:not(:disabled) {
  background: var(--accent-danger);
  color: white;
}

/* Ghost */
.variant-ghost {
  background: transparent;
  border-color: transparent;
  color: var(--text-secondary);
}
.variant-ghost:hover:not(:disabled) {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

/* Icon */
.is-icon {
  padding: 6px;
}

/* Sizes */
.size-sm {
  padding: 0.35rem 0.75rem;
  font-size: 0.8rem;
}

.size-md {
  /* Default padding and font size from global button */
}

.size-lg {
  padding: 0.75rem 1.25rem;
  font-size: 1rem;
}
</style>
