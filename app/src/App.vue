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
  openTrash,
  quitApp,
  readNote,
  saveClipboard,
  searchNotes,
  toggleAlwaysOnTop,
  writeNote,
} from './lib/invoke'
import { messages, type AppLanguage } from './lib/i18n'
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
const language = ref<AppLanguage>(initialLanguage())
const fileInput = ref<HTMLInputElement | null>(null)
const editorPane = ref<InstanceType<typeof EditorPane> | null>(null)
const workspacePath = ref('~/.neopad')
const activeTab = computed(() => tabs.value.find((tab) => tab.id === activeTabId.value) ?? tabs.value[0])
const t = computed(() => messages[language.value])
const localizedSaveState = computed(() => {
  if (saveState.value === 'Saving') {
    return t.value.status.saving
  }
  if (saveState.value === 'Failed') {
    return t.value.status.failed
  }
  return t.value.status.saved
})
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

watch(language, () => {
  window.localStorage.setItem('neopad.language', language.value)
  if (settingsOpen.value) {
    statusMessage.value = t.value.status.settings
  } else if (searchOpen.value) {
    statusMessage.value = t.value.status.search
  } else {
    statusMessage.value = t.value.status.markdown
  }
})

function initialLanguage(): AppLanguage {
  if (typeof window === 'undefined') {
    return 'en'
  }

  return window.localStorage.getItem('neopad.language') === 'zh' ? 'zh' : 'en'
}

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
    statusMessage.value = t.value.status.clipboard
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
    statusMessage.value = t.value.status.clipboardSaved
  } catch {
    saveState.value = 'Failed'
  }
}

function triggerLoadFile() {
  fileInput.value?.click()
}

async function loadFileFromInput(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''

  if (!file) {
    return
  }

  try {
    await forceSave()
    const fileContent = await file.text()
    const title = titleFromFileName(file.name)

    if (isTauriRuntime()) {
      const created = await createNote(title)
      const saved = await writeNote(created.id, fileContent)
      upsertTab({
        id: saved.id,
        title: saved.title,
        fileName: saved.fileName,
        createdAt: saved.updatedAt,
        updatedAt: saved.updatedAt,
        pinned: false,
        deleted: false,
      })
      activeTabId.value = saved.id
      setContentFromLoad(saved.content)
    } else {
      createLocalTabFromContent(title, fileContent)
    }

    saveState.value = 'Saved'
    statusMessage.value = t.value.status.loadedFromFile
  } catch {
    saveState.value = 'Failed'
  }
}

async function saveAsFile() {
  await forceSave()
  const title = activeTab.value?.title || 'Untitled'
  downloadText(`${safeFileName(title)}.md`, content.value)
  statusMessage.value = t.value.status.savedAsFile
}

async function exportAllNotes() {
  await forceSave()

  try {
    const sections: string[] = []
    for (const tab of tabs.value) {
      const noteContent = isTauriRuntime() ? (await readNote(tab.id)).content : tab.id === activeTabId.value ? content.value : ''
      sections.push(`## ${tab.title}\n\n<!-- file: ${tab.fileName} -->\n\n${noteContent.trimEnd()}\n`)
    }

    downloadText('neopad-export.md', `# Exported from NeoPad\n\n${sections.join('\n---\n\n')}`)
    statusMessage.value = t.value.status.exported
  } catch {
    saveState.value = 'Failed'
  }
}

async function openTrashFolder() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    await openTrash()
    statusMessage.value = t.value.status.trashOpened
  } catch {
    saveState.value = 'Failed'
  }
}

async function hideMainWindow() {
  if (!isTauriRuntime()) {
    return
  }

  await forceSave()
  await hideWindow()
}

async function exitApp() {
  await forceSave()
  if (isTauriRuntime()) {
    await quitApp()
  }
}

function undoEditor() {
  editorPane.value?.undoEdit()
}

function cutEditorSelection() {
  void editorPane.value?.cutSelection()
}

function copyEditorSelection() {
  void editorPane.value?.copySelection()
}

function pasteIntoEditor() {
  void editorPane.value?.pasteClipboard()
}

function selectAllEditorText() {
  editorPane.value?.selectAllText()
}

function openFindPanel() {
  showSearchPlaceholder()
}

function findNextMatch() {
  showSearchPlaceholder()
}

function openReplacePanel() {
  showSearchPlaceholder()
}

function showSearchPlaceholder() {
  searchOpen.value = true
  statusMessage.value = t.value.status.search
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
  statusMessage.value = t.value.status.settings
}

function closeSettings() {
  settingsOpen.value = false
}

async function togglePin() {
  if (!isTauriRuntime()) {
    alwaysOnTop.value = !alwaysOnTop.value
    statusMessage.value = t.value.status.alwaysOnTop
    return
  }

  try {
    const enabled = await toggleAlwaysOnTop()
    alwaysOnTop.value = enabled
    statusMessage.value = enabled ? t.value.status.pinned : t.value.status.unpinned
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
    statusMessage.value = allowWrite ? t.value.status.mcpWriteCopied : t.value.status.mcpReadOnlyCopied
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
    void hideMainWindow()
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

function createLocalTabFromContent(title: string, nextContent: string) {
  const createdAt = Date.now()
  const tab: NoteTab = {
    id: `draft-${createdAt}`,
    title,
    fileName: `${safeFileName(title)}.md`,
    createdAt,
    updatedAt: createdAt,
    pinned: false,
    deleted: false,
  }

  tabs.value.push(tab)
  activeTabId.value = tab.id
  content.value = nextContent
}

function titleFromFileName(fileName: string) {
  const withoutExtension = fileName.replace(/\.[^/.]+$/, '')
  return withoutExtension.trim() || 'Untitled'
}

function safeFileName(title: string) {
  return title.trim().replace(/[<>:"/\\|?*\u0000-\u001f]/g, '-').replace(/\s+/g, ' ') || 'Untitled'
}

function downloadText(fileName: string, text: string) {
  const url = URL.createObjectURL(new Blob([text], { type: 'text/markdown;charset=utf-8' }))
  const link = document.createElement('a')
  link.href = url
  link.download = fileName
  link.click()
  URL.revokeObjectURL(url)
}
</script>

<template>
  <AppShell>
    <template #title>
      <TitleBar />
      <MenuBar
        :preview-mode="previewMode"
        :messages="t.menu"
        @new-note="createLocalTab"
        @save-clipboard="saveCurrentClipboard"
        @load-file="triggerLoadFile"
        @save-as-file="saveAsFile"
        @export-all="exportAllNotes"
        @open-trash="openTrashFolder"
        @hide-window="hideMainWindow"
        @exit-app="exitApp"
        @undo="undoEditor"
        @cut="cutEditorSelection"
        @copy="copyEditorSelection"
        @paste="pasteIntoEditor"
        @find="openFindPanel"
        @find-next="findNextMatch"
        @replace="openReplacePanel"
        @global-search="showSearchPlaceholder"
        @select-all="selectAllEditorText"
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
      <input
        ref="fileInput"
        class="file-loader"
        type="file"
        accept=".md,.markdown,.txt,text/markdown,text/plain"
        @change="loadFileFromInput"
      />
      <EditorPane
        v-if="previewMode !== 'preview'"
        ref="editorPane"
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
      :messages="t.search"
      @close="closeSearch"
      @select="selectSearchResult"
    />

    <SettingsPanel
      v-if="settingsOpen"
      :always-on-top="alwaysOnTop"
      :theme="theme"
      :preview-mode="previewMode"
      :language="language"
      :workspace-path="workspacePath"
      :messages="t.settings"
      :menu-messages="t.menu"
      @close="closeSettings"
      @toggle-always-on-top="togglePin"
      @update:theme="theme = $event"
      @update:preview-mode="previewMode = $event"
      @update:language="language = $event"
      @copy-mcp-config="copyMcpConfig"
    />

    <template #status>
      <StatusBar
        :state="localizedSaveState"
        :characters="content.length"
        :mode="statusMessage"
        :chars-label="t.status.chars"
      />
    </template>
  </AppShell>
</template>
