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
  <label class="inline-flex items-center cursor-pointer group" :class="{ 'gap-2': !!label }">
    <div class="relative w-4 h-4 flex-shrink-0">
      <input
        type="checkbox"
        class="peer absolute opacity-0 w-0 h-0 m-0 p-0"
        :checked="checked"
        @change="onChange"
      />
      <span class="
        absolute inset-0 w-4 h-4 bg-page border border-divider rounded-[4px] transition-all box-border
        group-hover:border-accent
        peer-checked:bg-accent peer-checked:border-accent
        after:content-[''] after:absolute after:hidden
        peer-checked:after:block
        after:left-1 after:top-0 after:w-1 after:h-[9px]
        after:border-solid after:border-white after:[border-width:0_2px_2px_0] after:rotate-45
      "></span>
    </div>
    <span v-if="label" class="text-sm select-none">{{ label }}</span>
  </label>
</template>
