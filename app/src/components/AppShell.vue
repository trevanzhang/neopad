<script setup lang="ts">
defineProps<{
  tabOrientation: 'horizontal' | 'vertical'
  windowOpacity: number
  theme: 'light' | 'dark'
}>()
</script>

<template>
  <main class="app-shell" :class="[`tab-${tabOrientation}`, `theme-${theme}`]" :style="{ opacity: windowOpacity }">
    <div class="window-chrome">
      <slot name="title" />
    </div>
    <slot name="tabs" />
    <slot />
    <slot name="status" />
  </main>
</template>
