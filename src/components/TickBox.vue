<script setup lang="ts">
defineProps<{
  checked: boolean
  label?: string
}>()

const emit = defineEmits<{
  (e: 'update:checked', value: boolean): void
  (e: 'change', value: boolean): void
}>()

function onChange(event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:checked', target.checked)
  emit('change', target.checked)
}
</script>

<template>
  <label class="custom-checkbox-wrapper" :class="{ 'has-label': !!label }">
    <div class="custom-checkbox">
      <input 
        type="checkbox" 
        :checked="checked"
        @change="onChange"
      />
      <span class="checkmark"></span>
    </div>
    <span v-if="label" class="checkbox-label">{{ label }}</span>
  </label>
</template>

<style scoped>
.custom-checkbox-wrapper {
  display: inline-flex;
  align-items: center;
  cursor: pointer;
}

.custom-checkbox-wrapper.has-label {
  gap: 0.5rem;
}

.custom-checkbox {
  position: relative;
  display: inline-block;
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.custom-checkbox input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
  margin: 0;
  padding: 0;
}

.checkmark {
  position: absolute;
  top: 0;
  left: 0;
  height: 16px;
  width: 16px;
  background-color: var(--bg-base);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  transition: all var(--transition-fast);
  box-sizing: border-box;
}

.custom-checkbox-wrapper:hover .checkmark {
  border-color: var(--accent-primary);
}

.custom-checkbox input:checked ~ .checkmark {
  background-color: var(--accent-primary);
  border-color: var(--accent-primary);
}

.checkmark:after {
  content: "";
  position: absolute;
  display: none;
}

.custom-checkbox input:checked ~ .checkmark:after {
  display: block;
}

.custom-checkbox .checkmark:after {
  left: 4px;
  top: 0px;
  width: 4px;
  height: 9px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.checkbox-label {
  font-size: 0.85rem;
  color: var(--text-primary);
  user-select: none;
}
</style>
