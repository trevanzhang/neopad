<script setup lang="ts">
import logoUrl from '../assets/neopad-logo-small.png'

defineEmits<{
  pin: []
  search: []
  settings: []
}>()
</script>

<template>
  <header class="title-bar">
    <div class="brand-mark">
      <img :src="logoUrl" alt="" aria-hidden="true" />
      <strong>NeoPad</strong>
    </div>
    <div class="title-actions">
      <button type="button" title="Always on top" aria-label="Always on top" @click="$emit('pin')">
        Pin
      </button>
      <button type="button" title="Search" aria-label="Search" @click="$emit('search')">Search</button>
      <button type="button" title="Settings" aria-label="Settings" @click="$emit('settings')">
        Settings
      </button>
    </div>
  </header>
</template>
