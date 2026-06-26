<script setup lang="ts">
defineProps<{
  tabOrientation: 'horizontal' | 'vertical'
}>()
</script>

<template>
  <main class="app-shell" :class="`tab-${tabOrientation}`">
    <div class="window-chrome">
      <slot name="title" />
    </div>
    <slot name="tabs" />
    <slot />
    <slot name="status" />
  </main>
</template>
