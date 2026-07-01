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
  updateTabColor: [tabId: string, color: string | null]
  newTab: []
  toggleOrientation: []
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

function runContextAction(action: 'rename' | 'delete') {
  const tabId = contextMenu.value?.tabId
  if (!tabId) return
  contextMenu.value = null
  if (action === 'rename') emit('renameTab', tabId)
  else emit('deleteTab', tabId)
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
      title="Toggle tab bar display"
      aria-label="Toggle tab bar display"
      @click="$emit('toggleOrientation')"
    >
      ≡
    </button>
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="tab-item"
      :class="{ active: tab.id === activeTabId }"
      type="button"
      :style="tab.color ? { backgroundColor: tab.color } : undefined"
      @click="$emit('selectTab', tab.id)"
      @dblclick="$emit('titleDoubleClick', tab.id)"
      @contextmenu="openContextMenu($event, tab)"
    >
      {{ tab.title }}
    </button>
    <button class="tab-add" type="button" title="New note" aria-label="New note" @click="$emit('newTab')">
      +
    </button>
    <div class="tab-scroll-buttons" aria-hidden="true">
      <span>‹</span>
      <span>›</span>
    </div>
    <div
      v-if="contextMenu"
      ref="contextMenuElement"
      class="tab-context-menu"
      role="menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @contextmenu.prevent
    >
      <button
        type="button"
        role="menuitem"
        :disabled="contextMenu.tabId === 'inbox' || contextMenu.tabId === 'clipboard'"
        @click="runContextAction('rename')"
      >
        {{ messages.rename }}
      </button>
      <button
        type="button"
        role="menuitem"
        :disabled="contextMenu.tabId === 'inbox' || contextMenu.tabId === 'clipboard'"
        @click="runContextAction('delete')"
      >
        {{ messages.delete }}
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
