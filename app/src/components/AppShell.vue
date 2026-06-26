<script setup lang="ts">
defineProps<{
  tabOrientation: 'horizontal' | 'vertical'
  windowOpacity: number
}>()
</script>

<template>
  <main class="app-shell" :class="`tab-${tabOrientation}`" :style="{ opacity: windowOpacity }">
    <div class="window-chrome">
      <slot name="title" />
    </div>
    <slot name="tabs" />
    <slot />
    <slot name="status" />
  </main>
</template>
