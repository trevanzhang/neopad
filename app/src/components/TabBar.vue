<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import type { AppMessages } from '../lib/i18n'
import type { NoteTab } from '../types/note'
import { isExternalTab, isNoteTab, isPromptTab } from '../lib/document-tab'

const props = defineProps<{
  tabs: NoteTab[]
  activeTabId: string
  exportingNote: boolean
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
  revealTab: [tabId: string]
  copyTabPath: [tabId: string]
  exportTab: [tabId: string, format: 'png' | 'pdf']
  updateTabColor: [tabId: string, color: string | null]
  newTab: []
  toggleLibrary: []
  previousTab: []
  nextTab: []
  reorderTabs: [orderedTabIds: string[]]
}>()

const contextMenu = ref<{ tabId: string; x: number; y: number } | null>(null)
const contextMenuElement = ref<HTMLElement | null>(null)
const colors = ['#F4B8B8', '#F6D58A', '#BFE3A4', '#A9DCEB', '#BFC7F3', '#DCB8ED']
const draggedTabId = ref<string | null>(null)
const dragTargetTabId = ref<string | null>(null)

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
    x: Math.max(4, Math.min(event.clientX, window.innerWidth - 210)),
    y: Math.max(4, Math.min(event.clientY, window.innerHeight - 305)),
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

function runContextAction(action: 'rename' | 'delete' | 'close' | 'archive' | 'unarchive' | 'reveal' | 'copy-path' | 'export-png' | 'export-pdf') {
  const tabId = contextMenu.value?.tabId
  if (!tabId) return
  contextMenu.value = null
  if (action === 'rename') emit('renameTab', tabId)
  else if (action === 'delete') emit('deleteTab', tabId)
  else if (action === 'close') emit('closeTab', tabId)
  else if (action === 'archive') emit('archiveTab', tabId)
  else if (action === 'unarchive') emit('unarchiveTab', tabId)
  else if (action === 'copy-path') emit('copyTabPath', tabId)
  else if (action === 'export-png') emit('exportTab', tabId, 'png')
  else if (action === 'export-pdf') emit('exportTab', tabId, 'pdf')
  else emit('revealTab', tabId)
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
  const tab = props.tabs.find((item) => item.id === tabId)
  if (isPromptTab(tab)) return
  if (tab?.archived) emit('unarchiveTab', tabId)
  else emit('archiveTab', tabId)
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

function contextTab() {
  return props.tabs.find((tab) => tab.id === contextMenu.value?.tabId)
}

function canRenameOrDelete(tab: NoteTab | undefined) {
  return Boolean(tab && tab.id !== 'inbox' && tab.id !== 'clipboard' && !isExternalTab(tab))
}

function tabTitle(tab: NoteTab) {
  if (isPromptTab(tab)) return `prompts/${tab.fileName}`
  if (tab.archived) return `${props.messages.archive}: ${tab.title}`
  return tab.externalPath ?? tab.title
}

function startTabDrag(event: DragEvent, tabId: string) {
  draggedTabId.value = tabId
  dragTargetTabId.value = tabId
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/neopad-tab', tabId)
  }
}

function dropTab(targetTabId: string) {
  const sourceId = draggedTabId.value
  if (!sourceId || sourceId === targetTabId) return endTabDrag()
  const ordered = props.tabs.map((tab) => tab.id)
  const sourceIndex = ordered.indexOf(sourceId)
  const targetIndex = ordered.indexOf(targetTabId)
  if (sourceIndex < 0 || targetIndex < 0) return endTabDrag()
  ordered.splice(sourceIndex, 1)
  ordered.splice(targetIndex, 0, sourceId)
  emit('reorderTabs', ordered)
  endTabDrag()
}

function endTabDrag() {
  draggedTabId.value = null
  dragTargetTabId.value = null
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
      :class="{
        active: tab.id === activeTabId,
        archived: tab.archived,
        dragging: tab.id === draggedTabId,
        'drag-target': tab.id === dragTargetTabId && tab.id !== draggedTabId,
      }"
      type="button"
      draggable="true"
      :aria-grabbed="tab.id === draggedTabId"
      :title="tabTitle(tab)"
      :style="tab.color ? { '--tab-color': tab.color } : undefined"
      @click="$emit('selectTab', tab.id)"
      @dblclick="$emit('titleDoubleClick', tab.id)"
      @contextmenu="openContextMenu($event, tab)"
      @dragstart="startTabDrag($event, tab.id)"
      @dragover.prevent="dragTargetTabId = tab.id"
      @drop.prevent="dropTab(tab.id)"
      @dragend="endTabDrag"
      @keydown.f8="renameTabWithShortcut($event, tab.id)"
      @keydown.delete="deleteTabWithShortcut($event, tab.id)"
      @keydown.f12="archiveTabWithShortcut($event, tab.id)"
      @keydown.ctrl.w="closeTabWithShortcut($event, tab.id)"
    >
      <span v-if="isExternalTab(tab)" class="tab-external-icon" aria-hidden="true" />
      <span v-else-if="isPromptTab(tab)" class="tab-prompt-icon" aria-hidden="true">P</span>
      <svg v-else-if="tab.archived" class="tab-archive-icon" aria-hidden="true" viewBox="0 0 16 16"><path d="M1.75 5.25h12.5v8.5H1.75zM1.25 2.5h13.5v2.75H1.25zM6.25 8.75h3.5" /></svg>
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
      @keydown.f8="renameTabWithShortcut($event, contextMenu.tabId)"
      @keydown.delete="deleteTabWithShortcut($event, contextMenu.tabId)"
      @keydown.f12="archiveTabWithShortcut($event, contextMenu.tabId)"
      @keydown.ctrl.w="closeTabWithShortcut($event, contextMenu.tabId)"
    >
      <button
        type="button"
        role="menuitem"
        :disabled="!canRenameOrDelete(contextTab())"
        @click="runContextAction('rename')"
      >
        <span>{{ messages.rename }}</span>
        <span class="tab-context-shortcut">{{ messages.f8 }}</span>
      </button>
      <button
        type="button"
        role="menuitem"
        :disabled="!canRenameOrDelete(contextTab())"
        @click="runContextAction('delete')"
      >
        <span>{{ messages.delete }}</span>
        <span class="tab-context-shortcut">{{ messages.altDel }}</span>
      </button>
      <button
        v-if="isNoteTab(contextTab()) && contextTab()?.archived"
        type="button"
        role="menuitem"
        @click="runContextAction('unarchive')"
      >
        <span>{{ messages.restore }}</span>
        <span class="tab-context-shortcut">{{ messages.f12 }}</span>
      </button>
      <button
        v-else
        type="button"
        role="menuitem"
        :disabled="contextMenu.tabId === 'inbox' || contextMenu.tabId === 'clipboard' || isPromptTab(contextTab())"
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
      <button type="button" role="menuitem" @click="runContextAction('reveal')">
        {{ messages.revealInFileManager }}
      </button>
      <button type="button" role="menuitem" @click="runContextAction('copy-path')">
        {{ messages.copyFilePath }}
      </button>
      <div class="menu-separator" role="separator" />
      <button type="button" role="menuitem" :disabled="exportingNote" @click="runContextAction('export-png')">
        {{ messages.exportAsPng }}
      </button>
      <button type="button" role="menuitem" :disabled="exportingNote" @click="runContextAction('export-pdf')">
        {{ messages.exportAsPdf }}
      </button>
      <template v-if="isNoteTab(contextTab())">
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
      </template>
    </div>
  </nav>
</template>
