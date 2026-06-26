<script setup lang="ts">
import type { AppMessages } from '../lib/i18n'
import type { PreviewMode } from './ModeSwitch.vue'

defineProps<{
  previewMode: PreviewMode
  tabBarOrientation: 'horizontal' | 'vertical'
  wordWrap: boolean
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
  undo: []
  cut: []
  copy: []
  paste: []
  find: []
  findNext: []
  replace: []
  globalSearch: []
  selectAll: []
  search: []
  settings: []
  togglePin: []
  toggleTabBarOrientation: []
  updateTabBarOrientation: [orientation: 'horizontal' | 'vertical']
  formatFont: []
  formatBackground: []
  toggleWordWrap: []
  insertSeparator: []
  insertDateTime: []
  insertDateTimeSeparator: []
  insertReminder: []
  insertTextSettings: []
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
        <button type="button" class="menu-command menu-muted" @click="$emit('undo')">
          <span>{{ messages.undo }}</span>
          <span class="menu-shortcut">{{ messages.ctrlZ }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" @click="$emit('cut')">
          <span>{{ messages.cut }}</span>
          <span class="menu-shortcut">{{ messages.ctrlX }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('copy')">
          <span>{{ messages.copy }}</span>
          <span class="menu-shortcut">{{ messages.ctrlC }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('paste')">
          <span>{{ messages.paste }}</span>
          <span class="menu-shortcut">{{ messages.ctrlV }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" @click="$emit('find')">
          <span>{{ messages.find }}</span>
          <span class="menu-shortcut">{{ messages.ctrlF }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('findNext')">
          <span>{{ messages.findNext }}</span>
          <span class="menu-shortcut">{{ messages.f3 }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('replace')">
          <span>{{ messages.replace }}</span>
          <span class="menu-shortcut">{{ messages.ctrlR }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('globalSearch')">
          <span>{{ messages.globalSearch }}</span>
          <span class="menu-shortcut">{{ messages.ctrlShiftF }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command menu-muted" @click="$emit('selectAll')">
          <span>{{ messages.selectAll }}</span>
          <span class="menu-shortcut">{{ messages.ctrlA }}</span>
        </button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.view }}</button>
      <div class="menu-popover">
        <button type="button" class="menu-command" @click="$emit('toggleTabBarOrientation')">
          <span>{{ messages.toggleTabBarDisplay }}</span>
          <span class="menu-shortcut">{{ messages.f10 }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <div class="menu-subroot">
          <button type="button" class="menu-command">
            <span>{{ messages.tabBarDisplay }}</span>
            <span class="menu-arrow">&rsaquo;</span>
          </button>
          <div class="menu-popover menu-subpopover">
            <button
              type="button"
              :class="{ checked: tabBarOrientation === 'horizontal' }"
              @click="$emit('updateTabBarOrientation', 'horizontal')"
            >
              {{ messages.horizontal }}
            </button>
            <button
              type="button"
              :class="{ checked: tabBarOrientation === 'vertical' }"
              @click="$emit('updateTabBarOrientation', 'vertical')"
            >
              {{ messages.vertical }}
            </button>
          </div>
        </div>
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
        <button type="button" @click="$emit('formatFont')">{{ messages.font }}</button>
        <button type="button" @click="$emit('formatBackground')">{{ messages.backgroundColor }}</button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" :class="{ checked: wordWrap }" @click="$emit('toggleWordWrap')">
          <span>{{ messages.wordWrap }}</span>
          <span class="menu-shortcut">{{ messages.ctrlW }}</span>
        </button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.insert }}</button>
      <div class="menu-popover">
        <button type="button" class="menu-command" @click="$emit('insertSeparator')">
          <span>{{ messages.insertSeparator }}</span>
          <span class="menu-shortcut">{{ messages.ctrlDash }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('insertDateTime')">
          <span>{{ messages.dateTime }}</span>
          <span class="menu-shortcut">{{ messages.ctrlD }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('insertDateTimeSeparator')">
          <span>{{ messages.dateTimeSeparator }}</span>
          <span class="menu-shortcut">{{ messages.ctrlShiftDash }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('insertReminder')">
          <span>{{ messages.reminder }}</span>
          <span class="menu-shortcut">{{ messages.ctrlE }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" @click="$emit('insertTextSettings')">{{ messages.insertTextSettings }}</button>
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
