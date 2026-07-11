<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import type { AppMessages } from '../lib/i18n'
import type { NoteTab } from '../types/note'

defineProps<{
  tabs: NoteTab[]
  activeTabId: string
  messages: AppMessages['tabs']
}>()

const emit = defineEmits<{
  selectTab: [tabId: string]
  titleDoubleClick: [tabId: string]
  renameTab: [tabId: string]
  deleteTab: [tabId: string]
  closeTab: [tabId: string]
  archiveTab: [tabId: string]
  unarchiveTab: [tabId: string]
  updateTabColor: [tabId: string, color: string | null]
  newTab: []
  toggleLibrary: []
  previousTab: []
  nextTab: []
}>()

const contextMenu = ref<{ tabId: string; x: number; y: number } | null>(null)
const contextMenuElement = ref<HTMLElement | null>(null)
const colors = ['#F4B8B8', '#F6D58A', '#BFE3A4', '#A9DCEB', '#BFC7F3', '#DCB8ED']

onMounted(() => {
  window.addEventListener('pointerdown', closeContextMenu)
  window.addEventListener('keydown', closeContextMenuOnEscape)
})
onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', closeContextMenu)
  window.removeEventListener('keydown', closeContextMenuOnEscape)
})

function openContextMenu(event: MouseEvent, tab: NoteTab) {
  event.preventDefault()
  emit('selectTab', tab.id)
  contextMenu.value = {
    tabId: tab.id,
    x: Math.min(event.clientX, window.innerWidth - 210),
    y: Math.min(event.clientY, window.innerHeight - 155),
  }
  void nextTick(() => contextMenuElement.value?.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus())
}

function closeContextMenu(event?: Event) {
  if (event && contextMenuElement.value?.contains(event.target as Node)) return
  contextMenu.value = null
}

function closeContextMenuOnEscape(event: KeyboardEvent) {
  if (event.key !== 'Escape' || !contextMenu.value) return
  event.preventDefault()
  event.stopImmediatePropagation()
  contextMenu.value = null
}

function runContextAction(action: 'rename' | 'delete' | 'close' | 'archive' | 'unarchive') {
  const tabId = contextMenu.value?.tabId
  if (!tabId) return
  contextMenu.value = null
  if (action === 'rename') emit('renameTab', tabId)
  else if (action === 'delete') emit('deleteTab', tabId)
  else if (action === 'close') emit('closeTab', tabId)
  else if (action === 'archive') emit('archiveTab', tabId)
  else emit('unarchiveTab', tabId)
}

function deleteTabWithShortcut(event: KeyboardEvent, tabId: string) {
  if (!event.altKey || event.ctrlKey || event.shiftKey || event.metaKey) return
  const target = event.target
  if (target instanceof HTMLElement && target.closest('.tab-context-menu') && !target.closest('[role="menuitem"]')) return
  event.preventDefault()
  event.stopPropagation()
  contextMenu.value = null
  emit('deleteTab', tabId)
}

function renameTabWithShortcut(event: KeyboardEvent, tabId: string) {
  if (event.ctrlKey || event.altKey || event.shiftKey || event.metaKey) return
  const target = event.target
  if (target instanceof HTMLElement && target.closest('.tab-context-menu') && !target.closest('[role="menuitem"]')) return
  event.preventDefault()
  event.stopPropagation()
  contextMenu.value = null
  emit('renameTab', tabId)
}

function archiveTabWithShortcut(event: KeyboardEvent, tabId: string) {
  if (event.ctrlKey || event.altKey || event.shiftKey || event.metaKey) return
  const target = event.target
  if (target instanceof HTMLElement && target.closest('.tab-context-menu') && !target.closest('[role="menuitem"]')) return
  event.preventDefault()
  event.stopPropagation()
  contextMenu.value = null
  emit('archiveTab', tabId)
}

function closeTabWithShortcut(event: KeyboardEvent, tabId: string) {
  if (!event.ctrlKey || event.altKey || event.shiftKey || event.metaKey) return
  const target = event.target
  if (target instanceof HTMLElement && target.closest('.tab-context-menu') && !target.closest('[role="menuitem"]')) return
  event.preventDefault()
  event.stopPropagation()
  contextMenu.value = null
  emit('closeTab', tabId)
}

function updateColor(color: string | null) {
  const tabId = contextMenu.value?.tabId
  if (!tabId) return
  emit('updateTabColor', tabId, color)
  contextMenu.value = null
}
</script>

<template>
  <nav class="tab-bar" aria-label="Pages">
    <button
      class="tab-menu"
      type="button"
      :title="messages.library"
      :aria-label="messages.library"
      @click="$emit('toggleLibrary')"
    >
      ≡
    </button>
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="tab-item"
      :class="{ active: tab.id === activeTabId }"
      type="button"
      :title="tab.externalPath ?? tab.title"
      :style="tab.color ? { '--tab-color': tab.color } : undefined"
      @click="$emit('selectTab', tab.id)"
      @dblclick="$emit('titleDoubleClick', tab.id)"
      @contextmenu="openContextMenu($event, tab)"
      @keydown.f2="renameTabWithShortcut($event, tab.id)"
      @keydown.delete="deleteTabWithShortcut($event, tab.id)"
      @keydown.f12="archiveTabWithShortcut($event, tab.id)"
      @keydown.ctrl.w="closeTabWithShortcut($event, tab.id)"
    >
      <span v-if="tab.external" class="tab-external-icon" aria-hidden="true" />
      <span class="tab-label">{{ tab.title }}</span>
    </button>
    <button class="tab-add" type="button" title="New note" aria-label="New note" @click="$emit('newTab')">
      +
    </button>
    <div class="tab-scroll-buttons">
      <button type="button" :title="messages.previous" :aria-label="messages.previous" @click="$emit('previousTab')">
        &lsaquo;
      </button>
      <button type="button" :title="messages.next" :aria-label="messages.next" @click="$emit('nextTab')">
        &rsaquo;
      </button>
    </div>
    <div
      v-if="contextMenu"
      ref="contextMenuElement"
      class="tab-context-menu"
      role="menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @contextmenu.prevent
      @keydown.f2="renameTabWithShortcut($event, contextMenu.tabId)"
      @keydown.delete="deleteTabWithShortcut($event, contextMenu.tabId)"
      @keydown.f12="archiveTabWithShortcut($event, contextMenu.tabId)"
      @keydown.ctrl.w="closeTabWithShortcut($event, contextMenu.tabId)"
    >
      <button
        type="button"
        role="menuitem"
      :disabled="contextMenu.tabId === 'inbox' || contextMenu.tabId === 'clipboard'"
      @click="runContextAction('rename')"
    >
        <span>{{ messages.rename }}</span>
        <span class="tab-context-shortcut">{{ messages.f2 }}</span>
      </button>
      <button
        type="button"
        role="menuitem"
        :disabled="contextMenu.tabId === 'inbox' || contextMenu.tabId === 'clipboard'"
        @click="runContextAction('delete')"
      >
        <span>{{ messages.delete }}</span>
        <span class="tab-context-shortcut">{{ messages.altDel }}</span>
      </button>
      <button
        type="button"
        role="menuitem"
        :disabled="contextMenu.tabId === 'inbox' || contextMenu.tabId === 'clipboard'"
        @click="runContextAction('archive')"
      >
        <span>{{ messages.archive }}</span>
        <span class="tab-context-shortcut">{{ messages.f12 }}</span>
      </button>
      <button
        type="button"
        role="menuitem"
        :disabled="contextMenu.tabId === 'inbox' || contextMenu.tabId === 'clipboard'"
        @click="runContextAction('close')"
      >
        <span>{{ messages.close }}</span>
        <span class="tab-context-shortcut">{{ messages.ctrlW }}</span>
      </button>
      <div class="menu-separator" role="separator" />
      <span class="tab-context-label">{{ messages.color }}</span>
      <div class="tab-color-palette">
        <button
          type="button"
          class="tab-color-reset"
          :title="messages.defaultColor"
          :aria-label="messages.defaultColor"
          @click="updateColor(null)"
        >
          &times;
        </button>
        <button
          v-for="color in colors"
          :key="color"
          type="button"
          class="tab-color-swatch"
          :style="{ backgroundColor: color }"
          :aria-label="color"
          @click="updateColor(color)"
        />
      </div>
    </div>
  </nav>
</template>
