<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  variant: {
    type: String,
    default: 'default',
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
    default: 'md'
  }
})

const variantClasses: Record<string, string> = {
  default:   'bg-card-hover border-divider enabled:hover:bg-accent enabled:hover:border-accent',
  primary:   'bg-accent border-accent enabled:hover:bg-accent-active enabled:hover:border-accent-active',
  secondary: 'bg-card border-divider enabled:hover:bg-card-hover enabled:hover:border-muted',
  danger:    'bg-transparent border-danger text-[#ffb3b3] enabled:hover:bg-danger enabled:hover:text-white',
  ghost:     'bg-transparent border-transparent text-muted enabled:hover:bg-card-hover enabled:hover:text-white',
}

const sizeClasses: Record<string, string> = {
  sm: 'py-[0.35rem] px-3 text-[0.8rem]',
  md: '',
  lg: 'py-3 px-5 text-base',
}

const buttonClasses = computed(() => [
  'inline-flex items-center justify-center gap-2 disabled:opacity-60 disabled:cursor-not-allowed',
  variantClasses[props.variant] ?? variantClasses.default,
  sizeClasses[props.size] ?? '',
  props.icon ? 'p-[6px]' : '',
])
</script>

<template>
  <button :class="buttonClasses" :disabled="disabled">
    <slot></slot>
  </button>
</template>
