<script setup lang="ts">
import type { AppLanguage, AppMessages } from '../lib/i18n'
import type { PreviewMode } from './ModeSwitch.vue'

defineProps<{
  alwaysOnTop: boolean
  theme: 'system' | 'light' | 'dark'
  previewMode: PreviewMode
  language: AppLanguage
  workspacePath: string
  messages: AppMessages['settings']
  menuMessages: AppMessages['menu']
}>()

defineEmits<{
  close: []
  toggleAlwaysOnTop: []
  'update:theme': [theme: 'system' | 'light' | 'dark']
  'update:previewMode': [mode: PreviewMode]
  'update:language': [language: AppLanguage]
  copyMcpConfig: [allowWrite: boolean]
}>()
</script>

<template>
  <aside class="settings-panel" :aria-label="messages.title">
    <header class="settings-header">
      <strong>{{ messages.title }}</strong>
      <button type="button" :aria-label="messages.close" :title="messages.close" @click="$emit('close')">
        {{ messages.close }}
      </button>
    </header>

    <section class="settings-section">
      <h2>{{ messages.general }}</h2>
      <label class="settings-row">
        <span>{{ messages.language }}</span>
        <select
          :value="language"
          @change="$emit('update:language', ($event.target as HTMLSelectElement).value as AppLanguage)"
        >
          <option value="en">{{ messages.english }}</option>
          <option value="zh">{{ messages.chinese }}</option>
        </select>
      </label>
      <label class="settings-row">
        <span>{{ messages.alwaysOnTop }}</span>
        <input :checked="alwaysOnTop" type="checkbox" @change="$emit('toggleAlwaysOnTop')" />
      </label>
      <label class="settings-row">
        <span>{{ messages.theme }}</span>
        <select
          :value="theme"
          @change="$emit('update:theme', ($event.target as HTMLSelectElement).value as 'system' | 'light' | 'dark')"
        >
          <option value="system">{{ messages.system }}</option>
          <option value="light">{{ messages.light }}</option>
          <option value="dark">{{ messages.dark }}</option>
        </select>
      </label>
    </section>

    <section class="settings-section">
      <h2>{{ messages.shortcuts }}</h2>
      <div class="settings-row">
        <span>{{ messages.toggleWindow }}</span>
        <kbd>Alt+Z</kbd>
      </div>
      <div class="settings-row">
        <span>{{ messages.saveClipboard }}</span>
        <kbd>Ctrl+Shift+V</kbd>
      </div>
      <div class="settings-row">
        <span>{{ messages.hideWindow }}</span>
        <kbd>Esc</kbd>
      </div>
    </section>

    <section class="settings-section">
      <h2>{{ messages.editor }}</h2>
      <label class="settings-row">
        <span>{{ messages.previewMode }}</span>
        <select
          :value="previewMode"
          @change="$emit('update:previewMode', ($event.target as HTMLSelectElement).value as PreviewMode)"
        >
          <option value="edit">{{ menuMessages.editMode }}</option>
          <option value="preview">{{ menuMessages.previewMode }}</option>
          <option value="split">{{ menuMessages.splitMode }}</option>
        </select>
      </label>
    </section>

    <section class="settings-section">
      <h2>{{ messages.mcp }}</h2>
      <div class="settings-row">
        <span>{{ messages.workspace }}</span>
        <code>{{ workspacePath || '~/.neopad' }}</code>
      </div>
      <div class="settings-actions">
        <button type="button" @click="$emit('copyMcpConfig', false)">{{ messages.copyReadOnlyConfig }}</button>
        <button type="button" @click="$emit('copyMcpConfig', true)">{{ messages.copyWriteConfig }}</button>
      </div>
    </section>
  </aside>
</template>
