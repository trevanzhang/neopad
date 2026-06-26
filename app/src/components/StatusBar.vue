<script setup lang="ts">
defineProps<{
  state: string
  characters: number
  mode: string
  charsLabel: string
}>()
</script>

<template>
  <footer class="status-bar">
    <span>{{ state }}</span>
    <span>{{ characters }} {{ charsLabel }}</span>
    <span>{{ mode }}</span>
    <slot />
  </footer>
</template>
