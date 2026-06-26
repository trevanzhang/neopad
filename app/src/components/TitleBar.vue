<script setup lang="ts">
import logoUrl from '../assets/neopad-logo-small.png'
</script>

<template>
  <header class="title-bar">
    <div class="brand-mark">
      <img :src="logoUrl" alt="" aria-hidden="true" />
      <strong>NeoPad</strong>
    </div>
    <div class="window-glyphs" aria-hidden="true">
      <span>_</span>
      <span>□</span>
      <span>×</span>
    </div>
  </header>
</template>
