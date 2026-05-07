<script setup lang="ts">
defineOptions({
  inheritAttrs: false
})

defineProps<{
  modelValue: string | number
  label?: string
  placeholder?: string
  type?: string
}>()

const emit = defineEmits(['update:modelValue'])
</script>

<template>
  <div class="form-group" :class="$attrs.class" :style="$attrs.style">
    <label v-if="label">{{ label }}</label>
    <input 
      :type="type || 'text'" 
      :value="modelValue" 
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
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
