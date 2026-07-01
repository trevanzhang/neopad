<script setup lang="ts">
defineProps<{
  tabOrientation: 'horizontal' | 'vertical'
  theme: 'light' | 'dark'
  immersive: boolean
}>()
</script>

<template>
  <main class="app-shell" :class="[`tab-${tabOrientation}`, `theme-${theme}`, { immersive }]">
    <div class="window-chrome">
      <slot name="title" />
    </div>
    <slot name="tabs" />
    <slot />
    <slot name="status" />
  </main>
</template>
