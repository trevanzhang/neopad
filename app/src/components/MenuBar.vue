<script setup lang="ts">
import type { AppMessages } from '../lib/i18n'
import type { PreviewMode } from './ModeSwitch.vue'

defineProps<{
  previewMode: PreviewMode
  messages: AppMessages['menu']
}>()

defineEmits<{
  newNote: []
  saveClipboard: []
  loadFile: []
  saveAsFile: []
  exportAll: []
  openTrash: []
  hideWindow: []
  exitApp: []
  search: []
  settings: []
  togglePin: []
  updatePreviewMode: [mode: PreviewMode]
}>()
</script>

<template>
  <nav class="menu-bar" aria-label="Application menu">
    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.file }}</button>
      <div class="menu-popover">
        <button type="button" @click="$emit('loadFile')">{{ messages.loadFromFile }}</button>
        <button type="button" @click="$emit('saveAsFile')">{{ messages.saveAsFile }}</button>
        <div class="menu-separator" role="separator" />
        <button type="button" @click="$emit('exportAll')">{{ messages.exportAll }}</button>
        <div class="menu-separator" role="separator" />
        <button type="button" @click="$emit('openTrash')">{{ messages.trash }}</button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" @click="$emit('hideWindow')">
          <span>{{ messages.hide }}</span>
          <span class="menu-shortcut">{{ messages.esc }}</span>
        </button>
        <button type="button" @click="$emit('exitApp')">{{ messages.exit }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.edit }}</button>
      <div class="menu-popover">
        <button type="button" disabled>{{ messages.undo }}</button>
        <button type="button" disabled>{{ messages.redo }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.view }}</button>
      <div class="menu-popover">
        <button type="button" :class="{ checked: previewMode === 'edit' }" @click="$emit('updatePreviewMode', 'edit')">
          {{ messages.editMode }}
        </button>
        <button
          type="button"
          :class="{ checked: previewMode === 'split' }"
          @click="$emit('updatePreviewMode', 'split')"
        >
          {{ messages.splitMode }}
        </button>
        <button
          type="button"
          :class="{ checked: previewMode === 'preview' }"
          @click="$emit('updatePreviewMode', 'preview')"
        >
          {{ messages.previewMode }}
        </button>
        <button type="button" @click="$emit('togglePin')">{{ messages.alwaysOnTop }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.page }}</button>
      <div class="menu-popover">
        <button type="button" @click="$emit('newNote')">{{ messages.newPage }}</button>
        <button type="button" disabled>{{ messages.renamePage }}</button>
        <button type="button" disabled>{{ messages.deletePage }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.format }}</button>
      <div class="menu-popover">
        <button type="button" disabled>{{ messages.plainText }}</button>
        <button type="button" disabled>{{ messages.markdown }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.insert }}</button>
      <div class="menu-popover">
        <button type="button" disabled>{{ messages.date }}</button>
        <button type="button" disabled>{{ messages.separator }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.tools }}</button>
      <div class="menu-popover">
        <button type="button" @click="$emit('search')">{{ messages.search }}</button>
        <button type="button" @click="$emit('settings')">{{ messages.settings }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.help }}</button>
      <div class="menu-popover">
        <button type="button" disabled>{{ messages.about }}</button>
      </div>
    </div>
  </nav>
</template>
