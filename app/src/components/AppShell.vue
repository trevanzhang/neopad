<script setup lang="ts">
defineProps<{
  libraryOpen: boolean
  theme: 'light' | 'dark'
  immersive: boolean
}>()
</script>

<template>
  <main class="app-shell" :class="[`theme-${theme}`, { immersive, 'note-library-open': libraryOpen }]">
    <div class="window-chrome">
      <slot name="title" />
    </div>
    <slot name="tabs" />
    <slot name="library" />
    <slot />
    <slot name="status" />
  </main>
</template>
