<script setup lang="ts">
import type { PreviewMode } from './ModeSwitch.vue'

defineProps<{
  previewMode: PreviewMode
}>()

defineEmits<{
  newNote: []
  saveClipboard: []
  search: []
  settings: []
  togglePin: []
  updatePreviewMode: [mode: PreviewMode]
}>()
</script>

<template>
  <nav class="menu-bar" aria-label="Application menu">
    <div class="menu-root">
      <button type="button" class="menu-title">File(F)</button>
      <div class="menu-popover">
        <button type="button" @click="$emit('newNote')">New Page</button>
        <button type="button" @click="$emit('saveClipboard')">Save Clipboard</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">Edit(E)</button>
      <div class="menu-popover">
        <button type="button" disabled>Undo</button>
        <button type="button" disabled>Redo</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">View(V)</button>
      <div class="menu-popover">
        <button type="button" :class="{ checked: previewMode === 'edit' }" @click="$emit('updatePreviewMode', 'edit')">
          Edit
        </button>
        <button
          type="button"
          :class="{ checked: previewMode === 'split' }"
          @click="$emit('updatePreviewMode', 'split')"
        >
          Split
        </button>
        <button
          type="button"
          :class="{ checked: previewMode === 'preview' }"
          @click="$emit('updatePreviewMode', 'preview')"
        >
          Preview
        </button>
        <button type="button" @click="$emit('togglePin')">Always on Top</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">Page(P)</button>
      <div class="menu-popover">
        <button type="button" @click="$emit('newNote')">New Page</button>
        <button type="button" disabled>Rename Page</button>
        <button type="button" disabled>Delete Page</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">Format(O)</button>
      <div class="menu-popover">
        <button type="button" disabled>Plain Text</button>
        <button type="button" disabled>Markdown</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">Insert(I)</button>
      <div class="menu-popover">
        <button type="button" disabled>Date</button>
        <button type="button" disabled>Separator</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">Tools(T)</button>
      <div class="menu-popover">
        <button type="button" @click="$emit('search')">Search</button>
        <button type="button" @click="$emit('settings')">Settings</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">Help(H)</button>
      <div class="menu-popover">
        <button type="button" disabled>About NeoPad</button>
      </div>
    </div>
  </nav>
</template>
