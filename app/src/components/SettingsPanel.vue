<script setup lang="ts">
import type { PreviewMode } from './ModeSwitch.vue'

defineProps<{
  alwaysOnTop: boolean
  theme: 'system' | 'light' | 'dark'
  previewMode: PreviewMode
  workspacePath: string
}>()

defineEmits<{
  close: []
  toggleAlwaysOnTop: []
  'update:theme': [theme: 'system' | 'light' | 'dark']
  'update:previewMode': [mode: PreviewMode]
  copyMcpConfig: [allowWrite: boolean]
}>()
</script>

<template>
  <aside class="settings-panel" aria-label="Settings">
    <header class="settings-header">
      <strong>Settings</strong>
      <button type="button" aria-label="Close settings" title="Close" @click="$emit('close')">Close</button>
    </header>

    <section class="settings-section">
      <h2>General</h2>
      <label class="settings-row">
        <span>Always on top</span>
        <input :checked="alwaysOnTop" type="checkbox" @change="$emit('toggleAlwaysOnTop')" />
      </label>
      <label class="settings-row">
        <span>Theme</span>
        <select
          :value="theme"
          @change="$emit('update:theme', ($event.target as HTMLSelectElement).value as 'system' | 'light' | 'dark')"
        >
          <option value="system">System</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
      </label>
    </section>

    <section class="settings-section">
      <h2>Shortcuts</h2>
      <div class="settings-row">
        <span>Toggle window</span>
        <kbd>Alt+Z</kbd>
      </div>
      <div class="settings-row">
        <span>Save clipboard</span>
        <kbd>Ctrl+Shift+V</kbd>
      </div>
      <div class="settings-row">
        <span>Hide window</span>
        <kbd>Esc</kbd>
      </div>
    </section>

    <section class="settings-section">
      <h2>Editor</h2>
      <label class="settings-row">
        <span>Preview mode</span>
        <select
          :value="previewMode"
          @change="$emit('update:previewMode', ($event.target as HTMLSelectElement).value as PreviewMode)"
        >
          <option value="edit">Edit</option>
          <option value="preview">Preview</option>
          <option value="split">Split</option>
        </select>
      </label>
    </section>

    <section class="settings-section">
      <h2>MCP</h2>
      <div class="settings-row">
        <span>Workspace</span>
        <code>{{ workspacePath || '~/.neopad' }}</code>
      </div>
      <div class="settings-actions">
        <button type="button" @click="$emit('copyMcpConfig', false)">Copy read-only config</button>
        <button type="button" @click="$emit('copyMcpConfig', true)">Copy write config</button>
      </div>
    </section>
  </aside>
</template>
