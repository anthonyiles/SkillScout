<script setup lang="ts">
defineOptions({
  inheritAttrs: false
})

import { computed } from 'vue'

const props = defineProps<{
  modelValue: string | number
  label?: string
  placeholder?: string
  type?: string
  id?: string
}>()

const fallbackInputId = `input-${Math.random().toString(36).substring(2, 9)}`
const inputId = computed(() => props.id || fallbackInputId)

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

function handleInput(event: Event) {
  const target = event.target as HTMLInputElement
  const value = target.value

  if (props.type === 'number') {
    // Preserve intermediate states like "-", ".", or "-."
    if (/^-?$|^-?\d*\.?\d*$/.test(value) && value !== '') {
      emit('update:modelValue', value)
    } else {
      const numValue = value === '' ? '' : Number(value)
      emit('update:modelValue', numValue === '' || isNaN(numValue as number) ? '' : numValue)
    }
  } else {
    emit('update:modelValue', value)
  }
}
</script>

<template>
  <div class="form-group" :class="$attrs.class" :style="$attrs.style">
    <label v-if="label" :for="inputId">{{ label }}</label>
    <input
      :id="inputId"
      :type="type || 'text'"
      :value="modelValue"
      @input="handleInput"
      :placeholder="placeholder"
      v-bind="{ ...$attrs, class: undefined, style: undefined }"
    />
  </div>
</template>

<style scoped>
.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  margin-bottom: 0.75rem;
  flex: 1;
}

.form-group label {
  font-size: 0.85rem;
  color: var(--text-secondary);
  font-weight: 500;
}
</style>
