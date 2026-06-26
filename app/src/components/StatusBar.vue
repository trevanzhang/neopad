<script setup lang="ts">
defineProps<{
  state: string
  characters: number
  mode: string
}>()
</script>

<template>
  <footer class="status-bar">
    <span>{{ state }}</span>
    <span>{{ characters }} chars</span>
    <span>{{ mode }}</span>
    <slot />
  </footer>
</template>
