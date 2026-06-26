<script setup lang="ts">
export type PreviewMode = 'edit' | 'preview' | 'split'

defineProps<{
  modelValue: PreviewMode
}>()

defineEmits<{
  'update:modelValue': [mode: PreviewMode]
}>()

const modes: PreviewMode[] = ['edit', 'preview', 'split']
</script>

<template>
  <div class="mode-switch" role="group" aria-label="Preview mode">
    <button
      v-for="mode in modes"
      :key="mode"
      type="button"
      :class="{ active: mode === modelValue }"
      @click="$emit('update:modelValue', mode)"
    >
      {{ mode }}
    </button>
  </div>
</template>
