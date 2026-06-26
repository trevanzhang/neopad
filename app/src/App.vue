<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import AppShell from './components/AppShell.vue'
import EditorPane from './components/EditorPane.vue'
import MenuBar from './components/MenuBar.vue'
import type { PreviewMode } from './components/ModeSwitch.vue'
import PreviewPane from './components/PreviewPane.vue'
import SearchPanel from './components/SearchPanel.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import StatusBar from './components/StatusBar.vue'
import TabBar from './components/TabBar.vue'
import TitleBar from './components/TitleBar.vue'
import {
  createNote,
  getShortcutWarnings,
  getWorkspace,
  hideWindow,
  listNotes,
  readNote,
  saveClipboard,
  searchNotes,
  toggleAlwaysOnTop,
  writeNote,
} from './lib/invoke'
import { isTauriRuntime } from './lib/runtime'
import type { NoteTab, SearchResult } from './types/note'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const now = Date.now()
const tabs = ref<NoteTab[]>([
  {
    id: 'inbox',
    title: 'Inbox',
    fileName: 'inbox.md',
    createdAt: now,
    updatedAt: now,
    pinned: true,
    deleted: false,
  },
  {
    id: 'clipboard',
    title: 'Clipboard',
    fileName: 'clipboard.md',
    createdAt: now,
    updatedAt: now,
    pinned: true,
    deleted: false,
  },
])
const activeTabId = ref('inbox')
const content = ref('# Inbox\n\nStart typing...')
const saveState = ref<'Saved' | 'Saving' | 'Failed'>('Saved')
const isLoadingNote = ref(false)
const statusMessage = ref('Markdown')
const previewMode = ref<PreviewMode>('edit')
const searchOpen = ref(false)
const settingsOpen = ref(false)
const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const searching = ref(false)
const alwaysOnTop = ref(false)
const theme = ref<'system' | 'light' | 'dark'>('system')
const workspacePath = ref('~/.neopad')
const activeTab = computed(() => tabs.value.find((tab) => tab.id === activeTabId.value) ?? tabs.value[0])
let saveTimer: ReturnType<typeof window.setTimeout> | null = null
let searchTimer: ReturnType<typeof window.setTimeout> | null = null
let unlistenNotesChanged: UnlistenFn | null = null
let unlistenOpenSettings: UnlistenFn | null = null

onMounted(async () => {
  if (!isTauriRuntime()) {
    window.addEventListener('keydown', handleKeydown)
    return
  }

  await loadInitialNotes()
  await loadWorkspacePath()
  await loadShortcutWarnings()
  unlistenNotesChanged = await listen('neopad://notes-changed', async () => {
    await loadInitialNotes()
  })
  unlistenOpenSettings = await listen('neopad://open-settings', () => {
    openSettings()
  })
  window.addEventListener('keydown', handleKeydown)
  window.addEventListener('beforeunload', forceSaveOnExit)
  document.addEventListener('visibilitychange', forceSaveOnHide)
})

onBeforeUnmount(() => {
  clearSaveTimer()
  clearSearchTimer()
  void unlistenNotesChanged?.()
  void unlistenOpenSettings?.()
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('beforeunload', forceSaveOnExit)
  document.removeEventListener('visibilitychange', forceSaveOnHide)
})

watch(content, () => {
  if (isLoadingNote.value || !isTauriRuntime()) {
    return
  }

  scheduleSave()
})

watch(searchQuery, () => {
  if (!searchOpen.value || !isTauriRuntime()) {
    searchResults.value = []
    return
  }

  scheduleSearch()
})

async function selectTab(tabId: string) {
  if (tabId === activeTabId.value) {
    return
  }

  await forceSave()
  activeTabId.value = tabId
  await loadActiveNote()
}

async function createLocalTab() {
  if (isTauriRuntime()) {
    await forceSave()
    try {
      const note = await createNote()
      upsertTab({
        id: note.id,
        title: note.title,
        fileName: note.fileName,
        createdAt: note.updatedAt,
        updatedAt: note.updatedAt,
        pinned: false,
        deleted: false,
      })
      activeTabId.value = note.id
      setContentFromLoad(note.content)
      saveState.value = 'Saved'
      return
    } catch {
      saveState.value = 'Failed'
    }
  }

  const createdAt = Date.now()
  const index = tabs.value.length + 1
  const tab: NoteTab = {
    id: `draft-${createdAt}`,
    title: `Page ${index}`,
    fileName: `page-${createdAt}.md`,
    createdAt,
    updatedAt: createdAt,
    pinned: false,
    deleted: false,
  }

  tabs.value.push(tab)
  activeTabId.value = tab.id
  content.value = `# ${tab.title}\n\n`
}

async function saveCurrentClipboard() {
  if (!isTauriRuntime()) {
    statusMessage.value = 'Clipboard'
    return
  }

  try {
    await forceSave()
    const note = await saveClipboard()
    upsertTab({
      id: note.id,
      title: note.title,
      fileName: note.fileName,
      createdAt: note.updatedAt,
      updatedAt: note.updatedAt,
      pinned: true,
      deleted: false,
    })
    activeTabId.value = note.id
    setContentFromLoad(note.content)
    statusMessage.value = 'Clipboard saved'
  } catch {
    saveState.value = 'Failed'
  }
}

function showSearchPlaceholder() {
  searchOpen.value = true
  statusMessage.value = 'Search'
  if (searchQuery.value.trim()) {
    scheduleSearch()
  }
}

function showSettingsPlaceholder() {
  openSettings()
}

function closeSearch() {
  searchOpen.value = false
  searchResults.value = []
  clearSearchTimer()
}

async function selectSearchResult(result: SearchResult) {
  await forceSave()
  activeTabId.value = result.noteId
  await loadActiveNote()
  closeSearch()
}

function openSettings() {
  settingsOpen.value = true
  statusMessage.value = 'Settings'
}

function closeSettings() {
  settingsOpen.value = false
}

async function togglePin() {
  if (!isTauriRuntime()) {
    alwaysOnTop.value = !alwaysOnTop.value
    statusMessage.value = 'Always on top'
    return
  }

  try {
    const enabled = await toggleAlwaysOnTop()
    alwaysOnTop.value = enabled
    statusMessage.value = enabled ? 'Pinned' : 'Unpinned'
  } catch {
    saveState.value = 'Failed'
  }
}

async function copyMcpConfig(allowWrite: boolean) {
  const args = ['--workspace', workspacePath.value || '~/.neopad']
  if (allowWrite) {
    args.push('--allow-write')
  }

  const config = {
    mcpServers: {
      neopad: {
        command: 'neopad-mcp',
        args,
      },
    },
  }

  try {
    await navigator.clipboard.writeText(JSON.stringify(config, null, 2))
    statusMessage.value = allowWrite ? 'MCP write config copied' : 'MCP read-only config copied'
  } catch {
    saveState.value = 'Failed'
  }
}

async function loadInitialNotes() {
  try {
    const loadedTabs = await listNotes()
    if (loadedTabs.length > 0) {
      tabs.value = loadedTabs
      activeTabId.value = loadedTabs[0].id
    }
    await loadActiveNote()
  } catch {
    saveState.value = 'Failed'
  }
}

async function loadShortcutWarnings() {
  try {
    const warnings = await getShortcutWarnings()
    if (warnings.length > 0) {
      statusMessage.value = warnings[0]
    }
  } catch {
    saveState.value = 'Failed'
  }
}

async function loadWorkspacePath() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    const workspace = await getWorkspace()
    workspacePath.value = workspace.root
  } catch {
    saveState.value = 'Failed'
  }
}

async function loadActiveNote() {
  const tab = activeTab.value
  if (!tab) {
    return
  }

  isLoadingNote.value = true
  try {
    const note = await readNote(tab.id)
    setContentFromLoad(note.content)
    saveState.value = 'Saved'
  } catch {
    saveState.value = 'Failed'
  } finally {
    isLoadingNote.value = false
  }
}

function setContentFromLoad(nextContent: string) {
  clearSaveTimer()
  content.value = nextContent
}

function scheduleSave() {
  clearSaveTimer()
  saveState.value = 'Saving'
  saveTimer = window.setTimeout(() => {
    void forceSave()
  }, 500)
}

async function forceSave() {
  clearSaveTimer()
  if (!isTauriRuntime()) {
    saveState.value = 'Saved'
    return
  }

  const tab = activeTab.value
  if (!tab) {
    return
  }

  saveState.value = 'Saving'
  try {
    const saved = await writeNote(tab.id, content.value)
    tab.updatedAt = saved.updatedAt
    saveState.value = 'Saved'
  } catch {
    saveState.value = 'Failed'
  }
}

function forceSaveOnExit() {
  void forceSave()
}

function forceSaveOnHide() {
  if (document.visibilityState === 'hidden') {
    void forceSave()
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && isTauriRuntime()) {
    event.preventDefault()
    void hideWindow()
  }

  if (event.key.toLowerCase() === 'v' && event.ctrlKey && event.shiftKey) {
    event.preventDefault()
    void saveCurrentClipboard()
  }
}

function clearSaveTimer() {
  if (saveTimer) {
    window.clearTimeout(saveTimer)
    saveTimer = null
  }
}

function scheduleSearch() {
  clearSearchTimer()
  searching.value = true
  searchTimer = window.setTimeout(() => {
    void runSearch()
  }, 200)
}

async function runSearch() {
  clearSearchTimer()
  const query = searchQuery.value.trim()
  if (!query) {
    searchResults.value = []
    searching.value = false
    return
  }

  try {
    searchResults.value = await searchNotes(query, 100)
  } catch {
    saveState.value = 'Failed'
  } finally {
    searching.value = false
  }
}

function clearSearchTimer() {
  if (searchTimer) {
    window.clearTimeout(searchTimer)
    searchTimer = null
  }
}

function upsertTab(tab: NoteTab) {
  const index = tabs.value.findIndex((existing) => existing.id === tab.id)
  if (index === -1) {
    tabs.value.push(tab)
  } else {
    tabs.value[index] = tab
  }
}
</script>

<template>
  <AppShell>
    <template #title>
      <TitleBar />
      <MenuBar
        :preview-mode="previewMode"
        @new-note="createLocalTab"
        @save-clipboard="saveCurrentClipboard"
        @search="showSearchPlaceholder"
        @settings="showSettingsPlaceholder"
        @toggle-pin="togglePin"
        @update-preview-mode="previewMode = $event"
      />
    </template>

    <template #tabs>
      <TabBar
        :tabs="tabs"
        :active-tab-id="activeTabId"
        @select-tab="selectTab"
        @new-tab="createLocalTab"
      />
    </template>

    <div class="workspace-pane" :class="`mode-${previewMode}`">
      <EditorPane
        v-if="previewMode !== 'preview'"
        v-model="content"
        :title="activeTab?.title ?? 'Untitled'"
      />
      <PreviewPane v-if="previewMode !== 'edit'" :content="content" />
    </div>

    <SearchPanel
      v-if="searchOpen"
      v-model:query="searchQuery"
      :results="searchResults"
      :searching="searching"
      @close="closeSearch"
      @select="selectSearchResult"
    />

    <SettingsPanel
      v-if="settingsOpen"
      :always-on-top="alwaysOnTop"
      :theme="theme"
      :preview-mode="previewMode"
      :workspace-path="workspacePath"
      @close="closeSettings"
      @toggle-always-on-top="togglePin"
      @update:theme="theme = $event"
      @update:preview-mode="previewMode = $event"
      @copy-mcp-config="copyMcpConfig"
    />

    <template #status>
      <StatusBar :state="saveState" :characters="content.length" :mode="statusMessage" />
    </template>
  </AppShell>
</template>
