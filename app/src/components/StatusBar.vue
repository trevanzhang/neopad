<script setup lang="ts">
defineProps<{
  state: string
  characters: number
  mode: string
  editorMode: string
  charsLabel: string
}>()

defineEmits<{ cycleEditorMode: [] }>()
</script>

<template>
  <footer class="status-bar">
    <span>{{ state }}</span>
    <span>{{ characters }} {{ charsLabel }}</span>
    <span>{{ mode }}</span>
    <slot />
    <button type="button" class="status-mode" @click="$emit('cycleEditorMode')">{{ editorMode }}</button>
  </footer>
</template>
