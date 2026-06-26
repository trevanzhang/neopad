<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import logoUrl from '../assets/neopad-logo-small.png'

const appWindow = getCurrentWindow()

function minimizeWindow() {
  void appWindow.minimize()
}

function toggleMaximizeWindow() {
  void appWindow.toggleMaximize()
}

function closeWindow() {
  void appWindow.close()
}
</script>

<template>
  <header class="title-bar" data-tauri-drag-region>
    <div class="brand-mark" data-tauri-drag-region>
      <img :src="logoUrl" alt="" aria-hidden="true" />
      <strong>NeoPad</strong>
    </div>
    <div class="window-glyphs" aria-label="Window controls">
      <button type="button" aria-label="Minimize" title="Minimize" @click="minimizeWindow">_</button>
      <button type="button" aria-label="Maximize" title="Maximize" @click="toggleMaximizeWindow">□</button>
      <button type="button" aria-label="Close" title="Close" @click="closeWindow">×</button>
    </div>
  </header>
</template>
