<script setup lang="ts">
defineProps<{
  state: string
  characters: number
  mode: string
  editorMode: string
  charsLabel: string
  darkTheme: boolean
  themeLabel: string
  vimMode: string
}>()

defineEmits<{ cycleEditorMode: []; toggleTheme: [] }>()
</script>

<template>
  <footer class="status-bar">
    <span>{{ state }}</span>
    <span>{{ characters }} {{ charsLabel }}</span>
    <span>{{ mode }}</span>
    <span v-if="vimMode" class="status-vim-mode">{{ vimMode }}</span>
    <slot />
    <button type="button" class="status-mode" @click="$emit('cycleEditorMode')">{{ editorMode }}</button>
    <button type="button" class="status-theme" :title="themeLabel" :aria-label="themeLabel" @click="$emit('toggleTheme')">
      <svg v-if="darkTheme" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4" /><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" /></svg>
      <svg v-else viewBox="0 0 24 24" aria-hidden="true"><path d="M20.5 15.2A8.5 8.5 0 0 1 8.8 3.5 9 9 0 1 0 20.5 15.2Z" /></svg>
    </button>
  </footer>
</template>
