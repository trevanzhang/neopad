<script setup lang="ts">
import type { EditorMode } from '../types/editor'

defineProps<{
  modelValue: EditorMode
}>()

defineEmits<{
  'update:modelValue': [mode: EditorMode]
}>()

const modes: EditorMode[] = ['edit', 'preview', 'split']
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
