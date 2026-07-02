<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import AppShell from './components/AppShell.vue'
import ConfirmationDialog from './components/ConfirmationDialog.vue'
import EditorPane from './components/EditorPane.vue'
import InputDialog from './components/InputDialog.vue'
import MenuBar from './components/MenuBar.vue'
import PreviewPane from './components/PreviewPane.vue'
import SearchPanel from './components/SearchPanel.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import StatusBar from './components/StatusBar.vue'
import TabBar from './components/TabBar.vue'
import {
  createNote,
  deleteNote,
  getShortcutWarnings,
  getUiConfig,
  getWorkspace,
  hideWindow,
  listNotes,
  openTrash,
  quitApp,
  readNote,
  renameNote,
  saveClipboard,
  saveMarkdownFile,
  saveUiConfig,
  searchNotes,
  setNoteColor,
  setAutostart,
  setCloseToMinimize,
  setSnapToEdges,
  setWindowOpacity,
  toggleMainWindowMaximize,
  setTrayLanguage,
  toggleAlwaysOnTop,
  updateToggleShortcut,
  updateClipboardShortcut,
  writeNote,
} from './lib/invoke'
import type { AppTheme } from './lib/invoke'
import { messages, type AppLanguage } from './lib/i18n'
import { isTauriRuntime } from './lib/runtime'
import { AutosaveCoordinator } from './lib/autosave'
import { editorBackgroundForTheme } from './lib/theme'
import { formatShortcutLabel, normalizeShortcutInput, normalizeStoredShortcutKey } from './lib/shortcut'
import type { NoteTab, SearchResult } from './types/note'
import { isEditorMode, nextEditorMode, type EditorMode, type EditorModeShortcut } from './types/editor'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { getCurrentWindow } from '@tauri-apps/api/window'

type TabBarOrientation = 'horizontal' | 'vertical'
type HelpTopic = 'software' | 'shortcuts' | 'expression' | 'about'
type TitleDoubleClickAction = 'none' | 'delete' | 'rename'
type InputDialogState = { title: string; initialValue: string }
type ConfirmationDialogState = { title: string; message: string }

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
    systemTitle: false,
  },
  {
    id: 'clipboard',
    title: 'Clipboard',
    fileName: 'clipboard.md',
    createdAt: now,
    updatedAt: now,
    pinned: true,
    deleted: false,
    systemTitle: false,
  },
])
const activeTabId = ref('inbox')
const content = ref('# Inbox\n\nStart typing...')
const saveState = ref<'Saved' | 'Saving' | 'Failed'>('Saved')
const appReady = ref(false)
const isLoadingNote = ref(false)
const statusMessage = ref('Markdown')
const previewMode = ref<EditorMode>('edit')
const defaultEditorMode = ref<EditorMode>('edit')
const editorModeShortcut = ref<EditorModeShortcut>(initialEditorModeShortcut())
const searchOpen = ref(false)
const settingsOpen = ref(false)
const helpTopic = ref<HelpTopic | null>(null)
const inputDialog = ref<InputDialogState | null>(null)
let resolveInputDialog: ((value: string | null) => void) | null = null
const confirmationDialog = ref<ConfirmationDialogState | null>(null)
let resolveConfirmationDialog: ((confirmed: boolean) => void) | null = null
const immersiveMode = ref(false)
const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const searching = ref(false)
const alwaysOnTop = ref(false)
const theme = ref<AppTheme>(initialTheme())
const language = ref<AppLanguage>(initialLanguage())
const vimMode = ref(initialBooleanSetting('neopad.vimMode', false))
const vimUseCtrlShortcuts = ref(initialBooleanSetting('neopad.vimUseCtrlShortcuts', true))
const vimInsertExitKey = ref(initialStringSetting('neopad.vimInsertExitKey', 'jj'))
const activeVimMode = ref('')
const tabBarOrientation = ref<TabBarOrientation>(initialTabBarOrientation())
const wordWrap = ref(initialBooleanSetting('neopad.wordWrap', true))
const editorFontFamily = ref(initialStringSetting('neopad.editorFontFamily', '"Segoe UI", Arial, sans-serif'))
const editorBackgroundColor = ref(initialStringSetting('neopad.editorBackgroundColor', '#ffffff'))
const windowOpacity = ref(Number(initialStringSetting('neopad.windowOpacity', '1')))
const runAtStartup = ref(initialBooleanSetting('neopad.runAtStartup', false))
const closeToMinimize = ref(initialBooleanSetting('neopad.closeToMinimize', true))
const snapToEdges = ref(initialBooleanSetting('neopad.snapToEdges', false))
const transparencyEnabled = ref(initialBooleanSetting('neopad.transparencyEnabled', true))
const titleDoubleClickAction = ref<TitleDoubleClickAction>(initialTitleDoubleClickAction())
const shortcutBaseKey = ref(normalizeStoredShortcutKey(initialStringSetting('neopad.shortcutBaseKey', 'Z'), 'Z'))
const shortcutModifiers = ref<string[]>(initialJsonSetting('neopad.shortcutModifiers', ['Alt']))
const clipboardShortcutBaseKey = ref(normalizeStoredShortcutKey(initialStringSetting('neopad.clipboardShortcutBaseKey', 'V'), 'V'))
const clipboardShortcutModifiers = ref<string[]>(initialJsonSetting('neopad.clipboardShortcutModifiers', ['Ctrl', 'Shift']))
const insertSeparatorTemplate = ref(initialStringSetting('neopad.insertSeparatorTemplate', "crlf() + chars('-', 80) + crlf()"))
const insertDateTimeTemplate = ref(initialStringSetting('neopad.insertDateTimeTemplate', "date() + ' ' + time()"))
const legacyDateTimeSeparatorTemplate = "crlf() + chars('-', 29) + ' ' + date() + ' ' + time()"
const defaultDateTimeSeparatorTemplate = "crlf() + chars('-', 29) + ' ' + date() + ' ' + time() + ' ' + chars('-', 29) + crlf()"
const insertDateTimeSeparatorTemplate = ref(initialDateTimeSeparatorTemplate())
const customInsertTexts = ref<string[]>(initialJsonSetting('neopad.customInsertTexts', []))
const fileInput = ref<HTMLInputElement | null>(null)
const backgroundColorInput = ref<HTMLInputElement | null>(null)
const editorPane = ref<InstanceType<typeof EditorPane> | null>(null)
const workspacePath = ref('~/.neopad')
const activeTab = computed(() => tabs.value.find((tab) => tab.id === activeTabId.value) ?? tabs.value[0])
const t = computed(() => messages[language.value])
const displayTabs = computed(() => tabs.value.map((tab) => ({
  ...tab,
  title: tab.id === 'inbox'
      ? t.value.tabs.inbox
    : tab.id === 'clipboard'
      ? t.value.tabs.clipboard
      : tab.systemTitle && tab.title === 'Untitled'
        ? t.value.tabs.untitled
      : tab.title,
})))
const localizedSaveState = computed(() => {
  if (saveState.value === 'Saving') {
    return t.value.status.saving
  }
  if (saveState.value === 'Failed') {
    return t.value.status.failed
  }
  return t.value.status.saved
})
const helpContent = computed(() => getHelpContent(helpTopic.value, language.value))
const editorModeLabel = computed(() => {
  if (previewMode.value === 'preview') return t.value.status.previewMode
  if (previewMode.value === 'split') return t.value.status.hybridMode
  return t.value.status.editMode
})
const effectiveEditorBackground = computed(() => editorBackgroundForTheme(theme.value, editorBackgroundColor.value))
const themeToggleLabel = computed(() => theme.value === 'dark' ? t.value.status.switchToLight : t.value.status.switchToDark)
let searchTimer: number | null = null
let uiConfigTimer: number | null = null
let nativeSettingsTimer: number | null = null
let unlistenNewNoteRequested: UnlistenFn | null = null
let unlistenSaveClipboardRequested: UnlistenFn | null = null
let unlistenOpenSettings: UnlistenFn | null = null
let suppressedLoadedContent: string | null = null
let uiConfigLoaded = false
const autosave = new AutosaveCoordinator({
  delayMs: 500,
  save: ({ noteId, content: nextContent }: { noteId: string; content: string }) => writeNote(noteId, nextContent),
  onSaved: ({ noteId }, saved) => {
    const tab = tabs.value.find((item) => item.id === noteId)
    if (tab) tab.updatedAt = saved.updatedAt
  },
  onStateChange: (state) => {
    saveState.value = state
  },
})

onMounted(async () => {
  if (!isTauriRuntime()) {
    window.addEventListener('keydown', handleKeydown, { capture: true })
    appReady.value = true
    return
  }

  await resetWebviewZoom()
  await loadInitialNotes()
  await loadWorkspacePath()
  await loadShortcutWarnings()
  await loadNativeUiConfig()
  await Promise.allSettled([syncWindowOpacity(), syncToggleShortcut(), syncClipboardShortcut()])
  window.addEventListener('keydown', handleKeydown, { capture: true })
  window.addEventListener('beforeunload', forceSaveOnExit)
  document.addEventListener('visibilitychange', forceSaveOnHide)
  appReady.value = true
  nativeSettingsTimer = window.setTimeout(() => {
    nativeSettingsTimer = null
    void syncNativeSettings()
  }, 5000)
  void registerNativeEventListeners()
})

async function registerNativeEventListeners() {
  try {
    const [newNote, saveClipboardRequest, openSettingsRequest] = await Promise.all([
      listen('neopad://new-note-requested', () => {
        void createLocalTab()
      }),
      listen('neopad://save-clipboard-requested', () => {
        void saveCurrentClipboard()
      }),
      listen('neopad://open-settings', () => {
        openSettings()
      }),
    ])
    unlistenNewNoteRequested = newNote
    unlistenSaveClipboardRequested = saveClipboardRequest
    unlistenOpenSettings = openSettingsRequest
  } catch {
    saveState.value = 'Failed'
  }
}

async function resetWebviewZoom() {
  try {
    await getCurrentWebview().setZoom(1)
  } catch {
    saveState.value = 'Failed'
  }
}

onBeforeUnmount(() => {
  autosave.dispose()
  clearSearchTimer()
  if (uiConfigTimer) window.clearTimeout(uiConfigTimer)
  if (nativeSettingsTimer) window.clearTimeout(nativeSettingsTimer)
  void unlistenNewNoteRequested?.()
  void unlistenSaveClipboardRequested?.()
  void unlistenOpenSettings?.()
  window.removeEventListener('keydown', handleKeydown, { capture: true })
  window.removeEventListener('beforeunload', forceSaveOnExit)
  document.removeEventListener('visibilitychange', forceSaveOnHide)
})

watch(content, (nextContent) => {
  if (suppressedLoadedContent === nextContent) {
    suppressedLoadedContent = null
    return
  }
  if (isLoadingNote.value || !isTauriRuntime()) {
    return
  }

  const tab = activeTab.value
  if (tab) {
    autosave.markChanged({ noteId: tab.id, content: content.value })
  }
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
  if (isTauriRuntime()) {
    void syncTrayLanguage()
  }
  if (settingsOpen.value) {
    statusMessage.value = t.value.status.settings
  } else if (searchOpen.value) {
    statusMessage.value = t.value.status.search
  } else {
    statusMessage.value = t.value.status.markdown
  }
})

watch(tabBarOrientation, () => {
  window.localStorage.setItem('neopad.tabBarOrientation', tabBarOrientation.value)
})

watch(wordWrap, () => {
  window.localStorage.setItem('neopad.wordWrap', String(wordWrap.value))
})

watch(editorFontFamily, () => {
  window.localStorage.setItem('neopad.editorFontFamily', editorFontFamily.value)
})

watch(editorBackgroundColor, () => {
  window.localStorage.setItem('neopad.editorBackgroundColor', editorBackgroundColor.value)
})

watch(theme, () => {
  window.localStorage.setItem('neopad.theme', theme.value)
})

watch(windowOpacity, () => {
  window.localStorage.setItem('neopad.windowOpacity', String(windowOpacity.value))
  void syncWindowOpacity()
})

watch(runAtStartup, () => {
  window.localStorage.setItem('neopad.runAtStartup', String(runAtStartup.value))
  void syncAutostart()
})

watch(closeToMinimize, () => {
  window.localStorage.setItem('neopad.closeToMinimize', String(closeToMinimize.value))
  void syncCloseToMinimize()
})

watch(snapToEdges, () => {
  window.localStorage.setItem('neopad.snapToEdges', String(snapToEdges.value))
  void syncSnapToEdges()
})

watch(transparencyEnabled, () => {
  window.localStorage.setItem('neopad.transparencyEnabled', String(transparencyEnabled.value))
  void syncWindowOpacity()
})

watch(titleDoubleClickAction, () => {
  window.localStorage.setItem('neopad.titleDoubleClickAction', titleDoubleClickAction.value)
})

watch(shortcutBaseKey, () => {
  window.localStorage.setItem('neopad.shortcutBaseKey', shortcutBaseKey.value)
  void syncToggleShortcut()
})

watch(shortcutModifiers, () => {
  window.localStorage.setItem('neopad.shortcutModifiers', JSON.stringify(shortcutModifiers.value))
  void syncToggleShortcut()
}, { deep: true })

watch(clipboardShortcutBaseKey, () => {
  window.localStorage.setItem('neopad.clipboardShortcutBaseKey', clipboardShortcutBaseKey.value)
  void syncClipboardShortcut()
})

watch(clipboardShortcutModifiers, () => {
  window.localStorage.setItem('neopad.clipboardShortcutModifiers', JSON.stringify(clipboardShortcutModifiers.value))
  void syncClipboardShortcut()
}, { deep: true })

watch(insertSeparatorTemplate, () => {
  window.localStorage.setItem('neopad.insertSeparatorTemplate', insertSeparatorTemplate.value)
})

watch(insertDateTimeTemplate, () => {
  window.localStorage.setItem('neopad.insertDateTimeTemplate', insertDateTimeTemplate.value)
})

watch(insertDateTimeSeparatorTemplate, () => {
  window.localStorage.setItem('neopad.insertDateTimeSeparatorTemplate', insertDateTimeSeparatorTemplate.value)
})

watch(customInsertTexts, () => {
  window.localStorage.setItem('neopad.customInsertTexts', JSON.stringify(customInsertTexts.value))
}, { deep: true })

watch(editorModeShortcut, () => {
  window.localStorage.setItem('neopad.editorModeShortcut', editorModeShortcut.value)
})

watch(vimMode, () => {
  window.localStorage.setItem('neopad.vimMode', String(vimMode.value))
})

watch(vimUseCtrlShortcuts, () => {
  window.localStorage.setItem('neopad.vimUseCtrlShortcuts', String(vimUseCtrlShortcuts.value))
})

watch(vimInsertExitKey, () => {
  window.localStorage.setItem('neopad.vimInsertExitKey', vimInsertExitKey.value)
})

watch(
  [
    language,
    vimMode,
    vimUseCtrlShortcuts,
    vimInsertExitKey,
    tabBarOrientation,
    wordWrap,
    editorFontFamily,
    editorBackgroundColor,
    theme,
    windowOpacity,
    runAtStartup,
    closeToMinimize,
    snapToEdges,
    transparencyEnabled,
    titleDoubleClickAction,
    shortcutBaseKey,
    shortcutModifiers,
    clipboardShortcutBaseKey,
    clipboardShortcutModifiers,
    insertSeparatorTemplate,
    insertDateTimeTemplate,
    insertDateTimeSeparatorTemplate,
    customInsertTexts,
    defaultEditorMode,
    editorModeShortcut,
  ],
  () => {
    if (uiConfigLoaded && isTauriRuntime()) {
      void persistUiConfig()
    }
  },
  { deep: true },
)

function initialLanguage(): AppLanguage {
  if (typeof window === 'undefined') {
    return 'en'
  }

  return window.localStorage.getItem('neopad.language') === 'zh' ? 'zh' : 'en'
}

function initialTheme(): AppTheme {
  if (typeof window === 'undefined') return 'light'
  const stored = window.localStorage.getItem('neopad.theme')
  if (stored === 'light' || stored === 'dark') return stored
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function initialTabBarOrientation(): TabBarOrientation {
  if (typeof window === 'undefined') {
    return 'horizontal'
  }

  return window.localStorage.getItem('neopad.tabBarOrientation') === 'vertical' ? 'vertical' : 'horizontal'
}

function initialBooleanSetting(key: string, fallback: boolean) {
  if (typeof window === 'undefined') {
    return fallback
  }

  const value = window.localStorage.getItem(key)
  return value === null ? fallback : value === 'true'
}

function initialStringSetting(key: string, fallback: string) {
  if (typeof window === 'undefined') {
    return fallback
  }

  return window.localStorage.getItem(key) || fallback
}

function initialJsonSetting<T>(key: string, fallback: T) {
  if (typeof window === 'undefined') {
    return fallback
  }

  try {
    const stored = window.localStorage.getItem(key)
    return stored ? JSON.parse(stored) as T : fallback
  } catch {
    return fallback
  }
}

function initialTitleDoubleClickAction(): TitleDoubleClickAction {
  const value = initialStringSetting('neopad.titleDoubleClickAction', 'rename')
  return value === 'none' || value === 'delete' || value === 'rename' ? value : 'rename'
}

function initialEditorModeShortcut(): EditorModeShortcut {
  const value = initialStringSetting('neopad.editorModeShortcut', 'F7')
  return value === 'Ctrl+Shift+M' || value === 'disabled' ? value : 'F7'
}

function initialDateTimeSeparatorTemplate() {
  const value = initialStringSetting('neopad.insertDateTimeSeparatorTemplate', defaultDateTimeSeparatorTemplate)
  return value === legacyDateTimeSeparatorTemplate ? defaultDateTimeSeparatorTemplate : value
}

async function selectTab(tabId: string) {
  if (tabId === activeTabId.value) {
    return
  }

  await forceSave()
  activeTabId.value = tabId
  await loadActiveNote()
}

function cycleTab(offset: -1 | 1) {
  const currentIndex = tabs.value.findIndex((tab) => tab.id === activeTabId.value)
  if (currentIndex < 0 || tabs.value.length < 2) return
  const nextIndex = (currentIndex + offset + tabs.value.length) % tabs.value.length
  const nextTab = tabs.value[nextIndex]
  if (nextTab) void selectTab(nextTab.id)
}

async function handleTabTitleDoubleClick(tabId: string) {
  if (titleDoubleClickAction.value === 'none') {
    return
  }

  const tab = tabs.value.find((item) => item.id === tabId)
  if (!tab) {
    return
  }

  if (titleDoubleClickAction.value === 'rename') {
    await renameTab(tab)
    return
  }

  if (titleDoubleClickAction.value === 'delete') {
    await deleteTab(tab)
  }
}

async function renameActivePage() {
  if (activeTab.value) await renameTab(activeTab.value)
}

async function deleteActivePage() {
  if (activeTab.value) await deleteTab(activeTab.value)
}

async function renamePageById(tabId: string) {
  const tab = tabs.value.find((item) => item.id === tabId)
  if (tab) await renameTab(tab)
}

async function deletePageById(tabId: string) {
  const tab = tabs.value.find((item) => item.id === tabId)
  if (tab) await deleteTab(tab)
}

async function updateTabColor(tabId: string, color: string | null) {
  const tab = tabs.value.find((item) => item.id === tabId)
  if (!tab) return
  if (isTauriRuntime()) {
    try {
      const updated = await setNoteColor(tabId, color)
      tab.color = updated.color
      tab.updatedAt = updated.updatedAt
    } catch {
      saveState.value = 'Failed'
    }
  } else {
    tab.color = color ?? undefined
  }
}

async function renameTab(tab: NoteTab) {
  if (tab.id === 'inbox' || tab.id === 'clipboard') return
  const nextTitle = (await requestInput(t.value.settings.renameTitle, tab.title))?.trim()
  if (!nextTitle) return

  if (isTauriRuntime()) {
    try {
      const renamed = await renameNote(tab.id, nextTitle)
      tab.title = renamed.title
      tab.updatedAt = renamed.updatedAt
    } catch {
      saveState.value = 'Failed'
    }
  } else {
    tab.title = nextTitle
    tab.updatedAt = Date.now()
  }
}

async function deleteTab(tab: NoteTab) {
  if (tab.id === 'inbox' || tab.id === 'clipboard') return
  const confirmed = await requestConfirmation(
    t.value.tabs.confirmDeleteTitle,
    t.value.tabs.confirmDeleteMessage.replace('{title}', tab.title),
  )
  if (!confirmed) return

  if (isTauriRuntime()) {
    try {
      await deleteNote(tab.id)
    } catch {
      saveState.value = 'Failed'
      return
    }
  }
  tabs.value = tabs.value.filter((item) => item.id !== tab.id)
  if (activeTabId.value === tab.id) {
    activeTabId.value = tabs.value[0]?.id ?? 'inbox'
    await loadActiveNote()
  }
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
        systemTitle: true,
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
    systemTitle: true,
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
      systemTitle: false,
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
        systemTitle: false,
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
  const fileName = `${safeFileName(title)}.md`

  try {
    if (isTauriRuntime()) {
      const saved = await saveMarkdownFile(fileName, content.value)
      if (!saved) return
    } else {
      downloadText(fileName, content.value)
    }
    statusMessage.value = t.value.status.savedAsFile
  } catch {
    saveState.value = 'Failed'
  }
}

async function exportAllNotes() {
  await forceSave()

  try {
    const sections: string[] = []
    for (const tab of tabs.value) {
      const noteContent = isTauriRuntime() ? (await readNote(tab.id)).content : tab.id === activeTabId.value ? content.value : ''
      sections.push(`## ${tab.title}\n\n<!-- file: ${tab.fileName} -->\n\n${noteContent.trimEnd()}\n`)
    }

    const exportContent = `# Exported from NeoPad\n\n${sections.join('\n---\n\n')}`
    if (isTauriRuntime()) {
      const saved = await saveMarkdownFile('neopad-export.md', exportContent)
      if (!saved) return
    } else {
      downloadText('neopad-export.md', exportContent)
    }
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
  editorPane.value?.openEditorFind()
}

function findNextMatch() {
  editorPane.value?.findNextMatch()
}

function openReplacePanel() {
  editorPane.value?.openEditorReplace()
}

function toggleTabBarOrientation() {
  tabBarOrientation.value = tabBarOrientation.value === 'horizontal' ? 'vertical' : 'horizontal'
}

async function promptEditorFont() {
  const nextFont = (await requestInput(t.value.menu.font, editorFontFamily.value))?.trim()
  if (!nextFont) {
    return
  }

  editorFontFamily.value = nextFont
  statusMessage.value = t.value.status.fontUpdated
}

function requestInput(title: string, initialValue: string) {
  resolveInputDialog?.(null)
  inputDialog.value = { title, initialValue }
  return new Promise<string | null>((resolve) => {
    resolveInputDialog = resolve
  })
}

function finishInputDialog(value: string | null) {
  const resolve = resolveInputDialog
  resolveInputDialog = null
  inputDialog.value = null
  resolve?.(value)
}

function requestConfirmation(title: string, message: string) {
  resolveConfirmationDialog?.(false)
  confirmationDialog.value = { title, message }
  return new Promise<boolean>((resolve) => {
    resolveConfirmationDialog = resolve
  })
}

function finishConfirmationDialog(confirmed: boolean) {
  const resolve = resolveConfirmationDialog
  resolveConfirmationDialog = null
  confirmationDialog.value = null
  resolve?.(confirmed)
}

async function editCustomInsertText(index: number) {
  const value = await requestInput(t.value.settings.custom, customInsertTexts.value[index] ?? '')
  if (value === null) return
  const next = [...customInsertTexts.value]
  next[index] = value
  customInsertTexts.value = next
}

function openBackgroundColorPicker() {
  backgroundColorInput.value?.click()
}

function updateEditorBackground(event: Event) {
  const input = event.target as HTMLInputElement
  editorBackgroundColor.value = input.value
  statusMessage.value = t.value.status.backgroundUpdated
}

function toggleWordWrap() {
  wordWrap.value = !wordWrap.value
  statusMessage.value = wordWrap.value ? t.value.status.wordWrapOn : t.value.status.wordWrapOff
}

function toggleTheme() {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
}

async function setImmersiveMode(enabled: boolean) {
  if (enabled) {
    closeSettings()
    closeSearch()
    closeHelp()
  }

  if (isTauriRuntime()) {
    await getCurrentWindow().setFullscreen(enabled)
  }
  immersiveMode.value = enabled
  if (enabled) {
    await nextTick()
    editorPane.value?.focusEditor()
  }
}

function toggleImmersiveMode() {
  void setImmersiveMode(!immersiveMode.value)
}

function insertSeparator() {
  insertEditorText(renderInsertTemplate(insertSeparatorTemplate.value))
}

function insertDateTime() {
  insertEditorText(renderInsertTemplate(insertDateTimeTemplate.value))
}

function insertDateTimeSeparator() {
  insertEditorText(renderInsertTemplate(insertDateTimeSeparatorTemplate.value))
}

function insertReminder() {
  insertEditorText(`- [ ] ${formatDateTime(new Date())} `)
}

function openInsertTextSettings() {
  openSettings()
}

function insertEditorText(text: string) {
  if (editorPane.value?.insertText(text)) {
    statusMessage.value = t.value.status.inserted
  }
}

function calculateCurrentLineExpression() {
  statusMessage.value = editorPane.value?.appendCurrentLineCalculation()
    ? t.value.status.expressionCalculated
    : t.value.status.expressionNotFound
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

function openHelpTopic(topic: HelpTopic) {
  helpTopic.value = topic
}

function closeHelp() {
  helpTopic.value = null
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

function openReminderList() {
  searchQuery.value = '- [ ]'
  showSearchPlaceholder()
}

async function processEditorText(action: string) {
  try {
    const processed = await editorPane.value?.transformText((text) => transformText(action, text))
    if (processed) {
      statusMessage.value = t.value.status.textProcessed
    }
  } catch {
    saveState.value = 'Failed'
  }
}

async function transformText(action: string, text: string) {
  switch (action) {
    case 'uppercase':
      return text.toUpperCase()
    case 'lowercase':
      return text.toLowerCase()
    case 'removeExtraSpaces':
      return text.replace(/[ \t]+/g, ' ')
    case 'trimLeadingSpaces':
      return text
        .split('\n')
        .map((line) => line.trim())
        .join('\n')
    case 'removeEmptyLines':
      return text
        .split('\n')
        .filter((line) => line.trim() !== '')
        .join('\n')
    case 'removeDuplicateEmptyLines':
      return text.replace(/(\n\s*){3,}/g, '\n\n')
    case 'sortLines':
      return text.split('\n').sort((a, b) => a.localeCompare(b)).join('\n')
    case 'uniqueLines':
      return Array.from(new Set(text.split('\n'))).join('\n')
    case 'toSimplified':
      return convertChinese(text, traditionalToSimplifiedMap)
    case 'toTraditional':
      return convertChinese(text, simplifiedToTraditionalMap)
    case 'toHalfWidth':
      return toHalfWidth(text)
    case 'toFullWidth':
      return toFullWidth(text)
    case 'addLineNumbers':
      return text
        .split('\n')
        .map((line, index) => `${index + 1}. ${line}`)
        .join('\n')
    case 'removeLineNumbers':
      return text.replace(/^\s*\d+[\).\u3001]\s*/gm, '')
    case 'urlEncode':
      return encodeURIComponent(text)
    case 'urlDecode':
      return decodeURIComponent(text)
    case 'base64Encode':
      return btoa(unescape(encodeURIComponent(text)))
    case 'base64Decode':
      return decodeURIComponent(escape(atob(text)))
    case 'md5Hash':
      return md5(text)
    case 'sha1Hash':
      return digestText('SHA-1', text)
    case 'sha256Hash':
      return digestText('SHA-256', text)
    default:
      return text
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

async function loadNativeUiConfig() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    const stored = await getUiConfig()
    if (!stored.initialized) {
      uiConfigLoaded = true
      persistUiConfig()
      return
    }
    const ui = stored.ui
    theme.value = stored.theme === 'dark'
      ? 'dark'
      : stored.theme === 'light'
        ? 'light'
        : window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    const storedMode = isEditorMode(stored.previewMode) ? stored.previewMode : 'edit'
    previewMode.value = storedMode
    defaultEditorMode.value = storedMode
    language.value = ui.language === 'zh' ? 'zh' : 'en'
    vimMode.value = ui.vimMode
    vimUseCtrlShortcuts.value = ui.vimUseCtrlShortcuts
    vimInsertExitKey.value = ui.vimInsertExitKey
    tabBarOrientation.value = ui.tabBarOrientation === 'vertical' ? 'vertical' : 'horizontal'
    wordWrap.value = ui.wordWrap
    editorFontFamily.value = ui.editorFontFamily
    editorBackgroundColor.value = ui.editorBackgroundColor
    windowOpacity.value = Math.min(1, Math.max(0.2, ui.windowOpacity))
    runAtStartup.value = ui.runAtStartup
    closeToMinimize.value = ui.closeToMinimize
    snapToEdges.value = ui.snapToEdges
    transparencyEnabled.value = ui.transparencyEnabled
    titleDoubleClickAction.value =
      ui.titleDoubleClickAction === 'none' || ui.titleDoubleClickAction === 'delete'
        ? ui.titleDoubleClickAction
        : 'rename'
    shortcutBaseKey.value = normalizeStoredShortcutKey(ui.shortcutBaseKey, 'Z')
    shortcutModifiers.value = ui.shortcutModifiers
    clipboardShortcutBaseKey.value = normalizeStoredShortcutKey(ui.clipboardShortcutBaseKey, 'V')
    clipboardShortcutModifiers.value = ui.clipboardShortcutModifiers
    insertSeparatorTemplate.value = ui.insertSeparatorTemplate
    insertDateTimeTemplate.value = ui.insertDateTimeTemplate
    insertDateTimeSeparatorTemplate.value = ui.insertDateTimeSeparatorTemplate === legacyDateTimeSeparatorTemplate
      ? defaultDateTimeSeparatorTemplate
      : ui.insertDateTimeSeparatorTemplate
    customInsertTexts.value = ui.customInsertTexts
    editorModeShortcut.value = ui.editorModeShortcut === 'Ctrl+Shift+M' || ui.editorModeShortcut === 'disabled'
      ? ui.editorModeShortcut
      : 'F7'
    uiConfigLoaded = true
  } catch {
    saveState.value = 'Failed'
  }
}

function persistUiConfig() {
  if (uiConfigTimer) {
    window.clearTimeout(uiConfigTimer)
  }
  uiConfigTimer = window.setTimeout(async () => {
    uiConfigTimer = null
    try {
      await saveUiConfig({
        language: language.value,
        vimMode: vimMode.value,
        vimUseCtrlShortcuts: vimUseCtrlShortcuts.value,
        vimInsertExitKey: vimInsertExitKey.value,
        tabBarOrientation: tabBarOrientation.value,
        wordWrap: wordWrap.value,
        editorFontFamily: editorFontFamily.value,
        editorBackgroundColor: editorBackgroundColor.value,
        windowOpacity: windowOpacity.value,
        runAtStartup: runAtStartup.value,
        closeToMinimize: closeToMinimize.value,
        snapToEdges: snapToEdges.value,
        transparencyEnabled: transparencyEnabled.value,
        titleDoubleClickAction: titleDoubleClickAction.value,
        shortcutBaseKey: shortcutBaseKey.value,
        shortcutModifiers: shortcutModifiers.value,
        clipboardShortcutBaseKey: clipboardShortcutBaseKey.value,
        clipboardShortcutModifiers: clipboardShortcutModifiers.value,
        insertSeparatorTemplate: insertSeparatorTemplate.value,
        insertDateTimeTemplate: insertDateTimeTemplate.value,
        insertDateTimeSeparatorTemplate: insertDateTimeSeparatorTemplate.value,
        customInsertTexts: customInsertTexts.value,
        editorModeShortcut: editorModeShortcut.value,
      }, defaultEditorMode.value, theme.value)
    } catch {
      saveState.value = 'Failed'
    }
  }, 150)
}

async function syncNativeSettings() {
  if (!isTauriRuntime()) {
    return
  }

  await Promise.allSettled([
    syncAutostart(),
    syncCloseToMinimize(),
    syncSnapToEdges(),
    syncWindowOpacity(),
    syncTrayLanguage(),
    syncToggleShortcut(),
    syncClipboardShortcut(),
  ])
}

async function syncTrayLanguage() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    await setTrayLanguage(language.value)
  } catch {
    saveState.value = 'Failed'
  }
}

async function syncAutostart() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    await setAutostart(runAtStartup.value)
  } catch {
    saveState.value = 'Failed'
  }
}

async function syncCloseToMinimize() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    await setCloseToMinimize(closeToMinimize.value)
  } catch {
    saveState.value = 'Failed'
  }
}

async function syncSnapToEdges() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    await setSnapToEdges(snapToEdges.value)
  } catch {
    saveState.value = 'Failed'
  }
}

async function syncWindowOpacity() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    await setWindowOpacity(transparencyEnabled.value ? windowOpacity.value : 1)
    statusMessage.value = t.value.status.opacityUpdated
  } catch {
    saveState.value = 'Failed'
  }
}

async function syncToggleShortcut() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    await updateToggleShortcut(shortcutBaseKey.value, shortcutModifiers.value)
  } catch {
    saveState.value = 'Failed'
  }
}

async function syncClipboardShortcut() {
  if (!isTauriRuntime()) {
    return
  }

  try {
    await updateClipboardShortcut(clipboardShortcutBaseKey.value, clipboardShortcutModifiers.value)
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
  if (content.value !== nextContent) {
    suppressedLoadedContent = nextContent
  }
  content.value = nextContent
  autosave.markLoaded()
}

async function forceSave() {
  if (!isTauriRuntime()) {
    saveState.value = 'Saved'
    return
  }
  await autosave.flush()
}

function forceSaveOnExit() {
  void forceSave()
}

function forceSaveOnHide() {
  if (document.visibilityState === 'hidden') {
    void forceSave()
  }
}

function setEditorMode(mode: EditorMode) {
  previewMode.value = mode
  if (mode !== 'preview') {
    void nextTick(() => editorPane.value?.focusEditor())
  }
}

function setDefaultEditorMode(mode: EditorMode) {
  defaultEditorMode.value = mode
  setEditorMode(mode)
}

function cycleEditorMode() {
  setEditorMode(nextEditorMode(previewMode.value))
}

function matchesEditorModeShortcut(event: KeyboardEvent) {
  if (editorModeShortcut.value === 'disabled') return false
  if (editorModeShortcut.value === 'F7') {
    return event.key === 'F7' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey
  }
  return event.key.toLowerCase() === 'm' && event.ctrlKey && event.shiftKey && !event.altKey && !event.metaKey
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Tab' && event.ctrlKey && !event.altKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    cycleTab(event.shiftKey ? -1 : 1)
    return
  }

  if (event.key === 'F9' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    toggleTheme()
    return
  }

  if (event.key === 'F11' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    toggleImmersiveMode()
    return
  }

  if (event.key === 'Escape') {
    if (confirmationDialog.value) {
      event.preventDefault()
      event.stopPropagation()
      finishConfirmationDialog(false)
      return
    }

    if (inputDialog.value) {
      event.preventDefault()
      event.stopPropagation()
      finishInputDialog(null)
      return
    }

    if (immersiveMode.value) {
      event.preventDefault()
      event.stopPropagation()
      void setImmersiveMode(false)
      return
    }

    if (settingsOpen.value) {
      event.preventDefault()
      event.stopPropagation()
      closeSettings()
      return
    }

    if (helpTopic.value) {
      event.preventDefault()
      event.stopPropagation()
      closeHelp()
      return
    }

    if (searchOpen.value) {
      event.preventDefault()
      event.stopPropagation()
      closeSearch()
      return
    }

    // Menus and tab context menus own Escape while they are open. Their
    // component listeners close the surface later in the same event dispatch.
    if (document.querySelector('.menu-root:focus-within, .tab-context-menu')) return
  }

  if (matchesEditorModeShortcut(event)) {
    event.preventDefault()
    event.stopPropagation()
    cycleEditorMode()
    return
  }

  if (event.key === 'Enter' && event.altKey && !event.ctrlKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    if (isTauriRuntime()) {
      void toggleMainWindowMaximize()
    }
    return
  }

  if (event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    const key = event.key.toLowerCase()
    if (key === 'n') {
      event.preventDefault()
      event.stopPropagation()
      void createLocalTab()
      return
    }
    if (key === 'w') {
      event.preventDefault()
      event.stopPropagation()
      void deleteActivePage()
      return
    }
    if (key === 'o') {
      event.preventDefault()
      event.stopPropagation()
      triggerLoadFile()
      return
    }
  }

  if (vimMode.value && editorPane.value?.isEditorFocused()) {
    const isApplicationFunctionKey = event.key === 'F3' || event.key === 'F6' || event.key === 'F8' || event.key === 'F10'
    const shouldHideFromNormalMode = event.key === 'Escape' && activeVimMode.value === 'normal'
    const isPreservedCtrlShortcut = vimUseCtrlShortcuts.value && (event.ctrlKey || event.metaKey)
    if (!isApplicationFunctionKey && !shouldHideFromNormalMode && !isPreservedCtrlShortcut) {
      return
    }
  }

  if (vimMode.value && vimUseCtrlShortcuts.value && editorPane.value?.isEditorFocused() && event.ctrlKey && !event.altKey) {
    const key = event.key.toLowerCase()
    if (key === 'f') {
      event.preventDefault()
      event.stopPropagation()
      if (event.shiftKey) showSearchPlaceholder()
      else openFindPanel()
      return
    }
    if (key === 'r' && !event.shiftKey) {
      event.preventDefault()
      event.stopPropagation()
      openReplacePanel()
      return
    }
    if (key === 'c') { event.preventDefault(); void copyEditorSelection(); return }
    if (key === 'x') { event.preventDefault(); void cutEditorSelection(); return }
    if (key === 'v' && !event.shiftKey) { event.preventDefault(); void pasteIntoEditor(); return }
    if (key === 'a') { event.preventDefault(); selectAllEditorText(); return }
  }

  if (!vimMode.value && event.key.toLowerCase() === 'f' && event.ctrlKey && !event.altKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    if (event.shiftKey) showSearchPlaceholder()
    else openFindPanel()
    return
  }

  if (!vimMode.value && event.key.toLowerCase() === 'r' && event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    openReplacePanel()
    return
  }

  if (event.key === 'F3' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    findNextMatch()
    return
  }

  if (event.key === 'Enter' && event.ctrlKey) {
    event.preventDefault()
    event.stopPropagation()
    calculateCurrentLineExpression()
    return
  }

  if (event.key === 'Escape' && isTauriRuntime()) {
    event.preventDefault()
    void hideMainWindow()
  }

  if (event.key === 'F10') {
    event.preventDefault()
    toggleTabBarOrientation()
  }

  if (event.key === 'F6') {
    event.preventDefault()
    void togglePin()
  }

  if (event.key === 'F8') {
    event.preventDefault()
    openSettings()
  }

  if (event.code === 'Minus' && event.ctrlKey && event.shiftKey) {
    event.preventDefault()
    insertDateTimeSeparator()
  } else if (event.code === 'Minus' && event.ctrlKey) {
    event.preventDefault()
    insertSeparator()
  }

  if (event.key.toLowerCase() === 'd' && event.ctrlKey) {
    event.preventDefault()
    insertDateTime()
  }

  if (event.key.toLowerCase() === 'e' && event.ctrlKey) {
    event.preventDefault()
    insertReminder()
  }

  if (event.key.toLowerCase() === 'v' && event.ctrlKey && event.shiftKey) {
    event.preventDefault()
    void saveCurrentClipboard()
  }
}

function setVimInsertExitKey(key: string) {
  vimInsertExitKey.value = Array.from(key)
    .filter((character) => character.length === 1 && !/\s/.test(character))
    .slice(0, 8)
    .join('')
}

function setShortcutBaseKey(value: string) {
  shortcutBaseKey.value = normalizeShortcutInput(value)
}

function setClipboardShortcutBaseKey(value: string) {
  clipboardShortcutBaseKey.value = normalizeShortcutInput(value)
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
    systemTitle: false,
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

function formatDateTime(date: Date) {
  const pad = (value: number) => String(value).padStart(2, '0')
  const year = date.getFullYear()
  const month = pad(date.getMonth() + 1)
  const day = pad(date.getDate())
  const hours = pad(date.getHours())
  const minutes = pad(date.getMinutes())
  return `${year}-${month}-${day} ${hours}:${minutes}`
}

function renderInsertTemplate(template: string) {
  const now = new Date()
  const parts = template.split(/\s*\+\s*/g)
  return parts.map((part) => renderInsertTemplatePart(part.trim(), now)).join('')
}

function renderInsertTemplatePart(part: string, date: Date) {
  if (part === 'crlf()') {
    return '\n'
  }
  if (part === 'date()') {
    return formatDate(date)
  }
  if (part === 'time()') {
    return formatTime(date)
  }

  const charsMatch = part.match(/^chars\(['"](.+)['"],\s*(\d+)\)$/)
  if (charsMatch) {
    return charsMatch[1].repeat(Number(charsMatch[2]))
  }

  const quotedMatch = part.match(/^['"](.*)['"]$/)
  if (quotedMatch) {
    return quotedMatch[1]
  }

  return part
}

function formatDate(date: Date) {
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
}

function formatTime(date: Date) {
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`
}

function toHalfWidth(text: string) {
  return text.replace(/[\uff01-\uff5e]/g, (char) => String.fromCharCode(char.charCodeAt(0) - 0xfee0)).replace(/\u3000/g, ' ')
}

function toFullWidth(text: string) {
  return text.replace(/[!-~]/g, (char) => String.fromCharCode(char.charCodeAt(0) + 0xfee0)).replace(/ /g, '\u3000')
}

function convertChinese(text: string, map: Record<string, string>) {
  return text.replace(/./g, (char) => map[char] ?? char)
}

async function digestText(algorithm: AlgorithmIdentifier, text: string) {
  if (!crypto.subtle) {
    throw new Error(t.value.status.unsupportedHash)
  }

  const hash = await crypto.subtle.digest(algorithm, new TextEncoder().encode(text))
  return Array.from(new Uint8Array(hash))
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')
}

function md5(text: string) {
  const rotateLeft = (value: number, shift: number) => (value << shift) | (value >>> (32 - shift))
  const add = (left: number, right: number) => (left + right) & 0xffffffff
  const cmn = (q: number, a: number, b: number, x: number, s: number, t: number) => add(rotateLeft(add(add(a, q), add(x, t)), s), b)
  const ff = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) => cmn((b & c) | (~b & d), a, b, x, s, t)
  const gg = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) => cmn((b & d) | (c & ~d), a, b, x, s, t)
  const hh = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) => cmn(b ^ c ^ d, a, b, x, s, t)
  const ii = (a: number, b: number, c: number, d: number, x: number, s: number, t: number) => cmn(c ^ (b | ~d), a, b, x, s, t)
  const words = md5Words(text)
  let a = 1732584193
  let b = -271733879
  let c = -1732584194
  let d = 271733878

  for (let i = 0; i < words.length; i += 16) {
    const aa = a
    const bb = b
    const cc = c
    const dd = d
    a = ff(a, b, c, d, words[i], 7, -680876936)
    d = ff(d, a, b, c, words[i + 1], 12, -389564586)
    c = ff(c, d, a, b, words[i + 2], 17, 606105819)
    b = ff(b, c, d, a, words[i + 3], 22, -1044525330)
    a = ff(a, b, c, d, words[i + 4], 7, -176418897)
    d = ff(d, a, b, c, words[i + 5], 12, 1200080426)
    c = ff(c, d, a, b, words[i + 6], 17, -1473231341)
    b = ff(b, c, d, a, words[i + 7], 22, -45705983)
    a = ff(a, b, c, d, words[i + 8], 7, 1770035416)
    d = ff(d, a, b, c, words[i + 9], 12, -1958414417)
    c = ff(c, d, a, b, words[i + 10], 17, -42063)
    b = ff(b, c, d, a, words[i + 11], 22, -1990404162)
    a = ff(a, b, c, d, words[i + 12], 7, 1804603682)
    d = ff(d, a, b, c, words[i + 13], 12, -40341101)
    c = ff(c, d, a, b, words[i + 14], 17, -1502002290)
    b = ff(b, c, d, a, words[i + 15], 22, 1236535329)
    a = gg(a, b, c, d, words[i + 1], 5, -165796510)
    d = gg(d, a, b, c, words[i + 6], 9, -1069501632)
    c = gg(c, d, a, b, words[i + 11], 14, 643717713)
    b = gg(b, c, d, a, words[i], 20, -373897302)
    a = gg(a, b, c, d, words[i + 5], 5, -701558691)
    d = gg(d, a, b, c, words[i + 10], 9, 38016083)
    c = gg(c, d, a, b, words[i + 15], 14, -660478335)
    b = gg(b, c, d, a, words[i + 4], 20, -405537848)
    a = gg(a, b, c, d, words[i + 9], 5, 568446438)
    d = gg(d, a, b, c, words[i + 14], 9, -1019803690)
    c = gg(c, d, a, b, words[i + 3], 14, -187363961)
    b = gg(b, c, d, a, words[i + 8], 20, 1163531501)
    a = gg(a, b, c, d, words[i + 13], 5, -1444681467)
    d = gg(d, a, b, c, words[i + 2], 9, -51403784)
    c = gg(c, d, a, b, words[i + 7], 14, 1735328473)
    b = gg(b, c, d, a, words[i + 12], 20, -1926607734)
    a = hh(a, b, c, d, words[i + 5], 4, -378558)
    d = hh(d, a, b, c, words[i + 8], 11, -2022574463)
    c = hh(c, d, a, b, words[i + 11], 16, 1839030562)
    b = hh(b, c, d, a, words[i + 14], 23, -35309556)
    a = hh(a, b, c, d, words[i + 1], 4, -1530992060)
    d = hh(d, a, b, c, words[i + 4], 11, 1272893353)
    c = hh(c, d, a, b, words[i + 7], 16, -155497632)
    b = hh(b, c, d, a, words[i + 10], 23, -1094730640)
    a = hh(a, b, c, d, words[i + 13], 4, 681279174)
    d = hh(d, a, b, c, words[i], 11, -358537222)
    c = hh(c, d, a, b, words[i + 3], 16, -722521979)
    b = hh(b, c, d, a, words[i + 6], 23, 76029189)
    a = hh(a, b, c, d, words[i + 9], 4, -640364487)
    d = hh(d, a, b, c, words[i + 12], 11, -421815835)
    c = hh(c, d, a, b, words[i + 15], 16, 530742520)
    b = hh(b, c, d, a, words[i + 2], 23, -995338651)
    a = ii(a, b, c, d, words[i], 6, -198630844)
    d = ii(d, a, b, c, words[i + 7], 10, 1126891415)
    c = ii(c, d, a, b, words[i + 14], 15, -1416354905)
    b = ii(b, c, d, a, words[i + 5], 21, -57434055)
    a = ii(a, b, c, d, words[i + 12], 6, 1700485571)
    d = ii(d, a, b, c, words[i + 3], 10, -1894986606)
    c = ii(c, d, a, b, words[i + 10], 15, -1051523)
    b = ii(b, c, d, a, words[i + 1], 21, -2054922799)
    a = ii(a, b, c, d, words[i + 8], 6, 1873313359)
    d = ii(d, a, b, c, words[i + 15], 10, -30611744)
    c = ii(c, d, a, b, words[i + 6], 15, -1560198380)
    b = ii(b, c, d, a, words[i + 13], 21, 1309151649)
    a = ii(a, b, c, d, words[i + 4], 6, -145523070)
    d = ii(d, a, b, c, words[i + 11], 10, -1120210379)
    c = ii(c, d, a, b, words[i + 2], 15, 718787259)
    b = ii(b, c, d, a, words[i + 9], 21, -343485551)
    a = add(a, aa)
    b = add(b, bb)
    c = add(c, cc)
    d = add(d, dd)
  }

  return [a, b, c, d].map((value) => md5Hex(value)).join('')
}

function md5Words(text: string) {
  const bytes = Array.from(new TextEncoder().encode(text))
  const words: number[] = []
  bytes.forEach((byte, index) => {
    words[index >> 2] = (words[index >> 2] || 0) | (byte << ((index % 4) * 8))
  })
  words[bytes.length >> 2] = (words[bytes.length >> 2] || 0) | (0x80 << ((bytes.length % 4) * 8))
  words[(((bytes.length + 8) >> 6) + 1) * 16 - 2] = bytes.length * 8
  return words
}

function md5Hex(value: number) {
  let output = ''
  for (let i = 0; i < 4; i += 1) {
    output += ((value >> (i * 8)) & 0xff).toString(16).padStart(2, '0')
  }
  return output
}

const simplifiedToTraditionalMap: Record<string, string> = {
  '\u4e07': '\u842c',
  '\u4e0e': '\u8207',
  '\u4e13': '\u5c08',
  '\u4e1a': '\u696d',
  '\u4e1c': '\u6771',
  '\u4e24': '\u5169',
  '\u4e25': '\u56b4',
  '\u4e2a': '\u500b',
  '\u4e3a': '\u70ba',
  '\u4e49': '\u7fa9',
  '\u4e50': '\u6a02',
  '\u4e60': '\u7fd2',
  '\u4e66': '\u66f8',
  '\u4e70': '\u8cb7',
  '\u4e89': '\u722d',
  '\u4e8e': '\u65bc',
  '\u4e91': '\u96f2',
  '\u4ea7': '\u7522',
  '\u4eb2': '\u89aa',
  '\u4ebf': '\u5104',
  '\u4ec5': '\u50c5',
  '\u4ece': '\u5f9e',
  '\u4ed3': '\u5009',
  '\u4eea': '\u5100',
  '\u4eec': '\u5011',
  '\u4ef7': '\u50f9',
  '\u4f17': '\u773e',
  '\u4f18': '\u512a',
  '\u4f1a': '\u6703',
  '\u4f20': '\u50b3',
  '\u4f24': '\u50b7',
  '\u4f53': '\u9ad4',
  '\u513f': '\u5152',
  '\u515a': '\u9ee8',
  '\u5170': '\u862d',
  '\u5173': '\u95dc',
  '\u5174': '\u8208',
  '\u5199': '\u5beb',
  '\u519b': '\u8ecd',
  '\u519c': '\u8fb2',
  '\u51b2': '\u885d',
  '\u51b3': '\u6c7a',
  '\u51c6': '\u6e96',
  '\u51e0': '\u5e7e',
  '\u5219': '\u5247',
  '\u521a': '\u525b',
  '\u521b': '\u5275',
  '\u5220': '\u522a',
  '\u522b': '\u5225',
  '\u5267': '\u5287',
  '\u529e': '\u8fa6',
  '\u52a1': '\u52d9',
  '\u52a8': '\u52d5',
  '\u533a': '\u5340',
  '\u533b': '\u91ab',
  '\u534e': '\u83ef',
  '\u5355': '\u55ae',
  '\u5356': '\u8ce3',
  '\u536b': '\u885b',
  '\u53d1': '\u767c',
  '\u53d8': '\u8b8a',
  '\u53f7': '\u865f',
  '\u540e': '\u5f8c',
  '\u542c': '\u807d',
  '\u542f': '\u555f',
  '\u5458': '\u54e1',
  '\u56fd': '\u570b',
  '\u56fe': '\u5716',
  '\u5706': '\u5713',
  '\u575a': '\u5805',
  '\u575b': '\u58c7',
  '\u5757': '\u584a',
  '\u58f0': '\u8072',
  '\u5907': '\u5099',
  '\u590d': '\u5fa9',
  '\u5934': '\u982d',
  '\u593a': '\u596a',
  '\u594b': '\u596e',
  '\u5956': '\u734e',
  '\u5987': '\u5a66',
  '\u5988': '\u5abd',
  '\u5a31': '\u5a1b',
  '\u5b59': '\u5b6b',
  '\u5b66': '\u5b78',
  '\u5b81': '\u5be7',
  '\u5b9d': '\u5bf6',
  '\u5b9e': '\u5be6',
  '\u5bf9': '\u5c0d',
  '\u5bfc': '\u5c0e',
  '\u5c14': '\u723e',
  '\u5c3d': '\u76e1',
  '\u5c42': '\u5c64',
  '\u5c5e': '\u5c6c',
  '\u5c81': '\u6b72',
  '\u5c9b': '\u5cf6',
  '\u5e01': '\u5e63',
  '\u5e08': '\u5e2b',
  '\u5e26': '\u5e36',
  '\u5e2e': '\u5e6b',
  '\u5e7f': '\u5ee3',
  '\u5e86': '\u6176',
  '\u5e93': '\u5eab',
  '\u5e94': '\u61c9',
  '\u5f00': '\u958b',
  '\u5f20': '\u5f35',
  '\u5f52': '\u6b78',
  '\u5f53': '\u7576',
  '\u5f55': '\u9304',
  '\u5fc6': '\u61b6',
  '\u6001': '\u614b',
  '\u603b': '\u7e3d',
  '\u604b': '\u6200'
}

const traditionalToSimplifiedMap = Object.fromEntries(Object.entries(simplifiedToTraditionalMap).map(([key, value]) => [value, key]))

function getHelpContent(topic: HelpTopic | null, currentLanguage: AppLanguage) {
  const zh = currentLanguage === 'zh'

  if (topic === 'shortcuts') {
    return {
      title: zh ? '\u5feb\u6377\u952e\u5217\u8868' : 'Shortcut List',
      lines: [
        `${formatShortcutLabel(shortcutBaseKey.value, shortcutModifiers.value)} - ` + (zh ? '\u663e\u793a/\u9690\u85cf\u7a97\u53e3' : 'Show/hide window'),
        `${formatShortcutLabel(clipboardShortcutBaseKey.value, clipboardShortcutModifiers.value)} - ` + (zh ? '\u4fdd\u5b58\u526a\u8d34\u677f' : 'Save clipboard'),
        'Alt+Enter - ' + (zh ? '\u6700\u5927\u5316/\u8fd8\u539f\u7a97\u53e3' : 'Maximize/restore window'),
        'Ctrl+N - ' + (zh ? '\u65b0\u5efa\u6807\u7b7e\u9875' : 'New tab'),
        'Ctrl+W - ' + (zh ? '\u5173\u95ed\u6807\u7b7e\u9875' : 'Close tab'),
        'Ctrl+O - ' + (zh ? '\u4ece\u6587\u4ef6\u8f7d\u5165' : 'Load from file'),
        'Ctrl+Tab / Ctrl+Shift+Tab - ' + (zh ? '\u5207\u6362\u4e0b\u4e00\u4e2a/\u4e0a\u4e00\u4e2a\u6807\u7b7e\u9875' : 'Switch next/previous tab'),
        'Ctrl+F - ' + (zh ? '\u67e5\u627e' : 'Find'),
        'Ctrl+Shift+F - ' + (zh ? '\u5168\u5c40\u641c\u7d22' : 'Global search'),
        'Ctrl+D - ' + (zh ? '\u63d2\u5165\u65e5\u671f\u65f6\u95f4' : 'Insert date time'),
        'Ctrl+- - ' + (zh ? '\u63d2\u5165\u5206\u9694\u884c' : 'Insert separator'),
        'Ctrl+Shift+- - ' + (zh ? '\u63d2\u5165\u65e5\u671f\u65f6\u95f4\u5206\u9694\u884c' : 'Insert date time separator'),
        'Ctrl+E - ' + (zh ? '\u63d2\u5165\u63d0\u9192' : 'Insert reminder'),
        'F6 - ' + (zh ? '\u5207\u6362\u7a97\u53e3\u7f6e\u9876' : 'Toggle window on top'),
        (editorModeShortcut.value === 'disabled' ? (zh ? '\u672a\u7ed1\u5b9a' : 'Unbound') : editorModeShortcut.value) +
          ' - ' + (zh ? '\u5faa\u73af\u5207\u6362\u7f16\u8f91\u5668\u6a21\u5f0f' : 'Cycle editor mode'),
        'F8 - ' + (zh ? '\u6253\u5f00\u8bbe\u7f6e' : 'Open settings'),
        'F9 - ' + (zh ? '\u5207\u6362\u65e5\u95f4/\u591c\u95f4\u6a21\u5f0f' : 'Toggle light/dark theme'),
        'F11 - ' + (zh ? '\u5207\u6362\u6c89\u6d78\u5f0f\u5168\u5c4f' : 'Toggle immersive fullscreen'),
        'F10 - ' + (zh ? '\u5207\u6362\u6807\u7b7e\u680f\u65b9\u5411' : 'Toggle tab bar orientation'),
        'Esc - ' + (zh ? '\u9690\u85cf\u7a97\u53e3' : 'Hide window'),
      ],
    }
  }

  if (topic === 'expression') {
    return {
      title: zh ? '\u8868\u8fbe\u5f0f\u8ba1\u7b97\u6307\u5357' : 'Expression Guide',
      lines: zh
        ? [
            '\u5728\u7f16\u8f91\u6a21\u5f0f\u4e0b\uff0c\u8f93\u5165\u4e00\u884c\u6570\u5b66\u8868\u8fbe\u5f0f\u540e\u6309 Ctrl+Enter\uff0cNeoPad \u4f1a\u5728\u884c\u5c3e\u8ffd\u52a0\u8ba1\u7b97\u7ed3\u679c\u3002',
            '\u652f\u6301 +, -, *, /, %, ^ \u548c\u62ec\u53f7\uff0c\u4e5f\u652f\u6301 \u00d7 \u548c \u00f7 \u7b26\u53f7\u3002',
            '\u793a\u4f8b\uff1a899*565-451 \u6309 Ctrl+Enter \u540e\u53d8\u4e3a 899*565-451 = 507484\u3002',
            '\u5982\u679c\u884c\u5185\u5305\u542b\u975e\u8868\u8fbe\u5f0f\u6587\u5b57\uff0c\u4f1a\u5c3d\u91cf\u8ba1\u7b97\u53ef\u8bc6\u522b\u7684\u524d\u7f00\u90e8\u5206\u3002',
          ]
        : [
            'In edit mode, type a math expression on one line and press Ctrl+Enter. NeoPad appends the result to that line.',
            'Supported operators: +, -, *, /, %, ^, parentheses, ×, and ÷.',
            'Example: 899*565-451 becomes 899*565-451 = 507484.',
            'If the line contains non-expression text, NeoPad tries to calculate the recognizable expression prefix.',
          ],
    }
  }

  if (topic === 'about') {
    return {
      title: zh ? '\u5173\u4e8e NeoPad' : 'About NeoPad',
      lines: zh
        ? [
            'NeoPad - \u8f7b\u91cf\u3001\u672c\u5730\u4f18\u5148\u7684 Markdown \u684c\u9762\u4fbf\u7b7e\u3002',
            '\u4f5c\u8005\uff1aTrevanZhang',
            '\u5f00\u6e90\u9879\u76ee\uff1ahttps://github.com/trevanzhang/neopad',
            '\u5f00\u6e90\u534f\u8bae\uff1aMIT License',
            '\u6280\u672f\u6808\uff1aTauri 2, Vue 3, TypeScript, Rust',
          ]
        : [
            'NeoPad - a lightweight, local-first Markdown desktop note pad.',
            'Author: TrevanZhang',
            'Open source: https://github.com/trevanzhang/neopad',
            'License: MIT License',
            'Built with Tauri 2, Vue 3, TypeScript, and Rust.',
          ],
    }
  }

  return {
    title: zh ? '\u8f6f\u4ef6\u8bf4\u660e' : 'Software Help',
    lines: zh
      ? [
          'NeoPad \u662f\u4e00\u6b3e\u8f7b\u91cf\u7684\u672c\u5730\u4f18\u5148\u684c\u9762\u4fbf\u7b7e\uff0c\u4e13\u6ce8\u4e8e\u5feb\u901f\u8bb0\u5f55\u548c\u67e5\u627e\u3002',
          '\u7b14\u8bb0\u4ee5 Markdown \u6587\u4ef6\u81ea\u52a8\u4fdd\u5b58\u5728\u672c\u5730\uff0c\u65e0\u9700\u8d26\u53f7\uff0c\u4e0d\u4f9d\u8d56\u4e91\u670d\u52a1\u3002',
          '\u652f\u6301\u591a\u6807\u7b7e\u9875\u3001\u5168\u6587\u641c\u7d22\u3001\u526a\u8d34\u677f\u91c7\u96c6\u3001Markdown \u9884\u89c8\u3001Vim \u952e\u4f4d\u548c\u884c\u5185\u8ba1\u7b97\u3002',
          '\u72ec\u7acb MCP \u670d\u52a1\u5668\u53ef\u4f9b\u672c\u5730 AI \u5de5\u5177\u8bbf\u95ee\u540c\u4e00\u7b14\u8bb0\u5de5\u4f5c\u533a\uff0c\u9ed8\u8ba4\u53ea\u8bfb\u3002',
        ]
      : [
          'NeoPad is a lightweight, local-first desktop note pad focused on fast capture and retrieval.',
          'Notes are autosaved locally as Markdown files. No account or cloud service is required.',
          'It supports tabs, full-text search, clipboard capture, Markdown preview, Vim keys, and inline calculations.',
          'A standalone, read-only-by-default MCP server lets local AI tools access the same note workspace.',
        ],
  }
}

</script>

<template>
  <AppShell
    :tab-orientation="tabBarOrientation"
    :data-ready="appReady ? 'true' : 'false'"
    :theme="theme"
    :immersive="immersiveMode"
  >
    <template #title>
      <MenuBar
        :preview-mode="previewMode"
        :tab-bar-orientation="tabBarOrientation"
        :word-wrap="wordWrap"
        :always-on-top="alwaysOnTop"
        :page-actions-enabled="Boolean(activeTab && activeTab.id !== 'inbox' && activeTab.id !== 'clipboard')"
        :messages="t.menu"
        @new-note="createLocalTab"
        @rename-page="renameActivePage"
        @delete-page="deleteActivePage"
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
        @update-tab-bar-orientation="tabBarOrientation = $event"
        @format-font="promptEditorFont"
        @format-background="openBackgroundColorPicker"
        @toggle-word-wrap="toggleWordWrap"
        @toggle-theme="toggleTheme"
        @insert-separator="insertSeparator"
        @insert-date-time="insertDateTime"
        @insert-date-time-separator="insertDateTimeSeparator"
        @insert-reminder="insertReminder"
        @insert-text-settings="openInsertTextSettings"
        @window-opacity="openSettings"
        @reminder-list="openReminderList"
        @process-text="processEditorText"
        @help-topic="openHelpTopic"
        @update-preview-mode="setEditorMode"
      />
    </template>

    <template #tabs>
      <TabBar
        :tabs="displayTabs"
        :active-tab-id="activeTabId"
        :messages="t.tabs"
        @select-tab="selectTab"
        @title-double-click="handleTabTitleDoubleClick"
        @rename-tab="renamePageById"
        @delete-tab="deletePageById"
        @update-tab-color="updateTabColor"
        @new-tab="createLocalTab"
        @toggle-orientation="toggleTabBarOrientation"
        @previous-tab="cycleTab(-1)"
        @next-tab="cycleTab(1)"
      />
    </template>

    <div class="workspace-pane" :class="[`mode-${previewMode}`, { immersive: immersiveMode }]">
      <input
        ref="fileInput"
        class="file-loader"
        type="file"
        accept=".md,.markdown,.txt,text/markdown,text/plain"
        @change="loadFileFromInput"
      />
      <input
        ref="backgroundColorInput"
        class="file-loader"
        type="color"
        :value="editorBackgroundColor"
        @input="updateEditorBackground"
      />
      <EditorPane
        v-show="immersiveMode || previewMode !== 'preview'"
        ref="editorPane"
        v-model="content"
        :title="activeTab?.title ?? 'Untitled'"
        :word-wrap="wordWrap"
        :font-family="editorFontFamily"
        :background-color="effectiveEditorBackground"
        :vim-mode="vimMode"
        :vim-insert-exit-key="vimInsertExitKey"
        @vim-mode-change="activeVimMode = $event"
      />
      <PreviewPane v-if="!immersiveMode && previewMode !== 'edit'" :content="content" />
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
      :vim-mode="vimMode"
      :vim-use-ctrl-shortcuts="vimUseCtrlShortcuts"
      :vim-insert-exit-key="vimInsertExitKey"
      :preview-mode="defaultEditorMode"
      :editor-mode-shortcut="editorModeShortcut"
      :language="language"
      :workspace-path="workspacePath"
      :run-at-startup="runAtStartup"
      :close-to-minimize="closeToMinimize"
      :snap-to-edges="snapToEdges"
      :transparency-enabled="transparencyEnabled"
      :window-opacity-percent="Math.round(windowOpacity * 100)"
      :title-double-click-action="titleDoubleClickAction"
      :shortcut-base-key="shortcutBaseKey"
      :shortcut-modifiers="shortcutModifiers"
      :clipboard-shortcut-base-key="clipboardShortcutBaseKey"
      :clipboard-shortcut-modifiers="clipboardShortcutModifiers"
      :insert-separator-template="insertSeparatorTemplate"
      :insert-date-time-template="insertDateTimeTemplate"
      :insert-date-time-separator-template="insertDateTimeSeparatorTemplate"
      :custom-insert-texts="customInsertTexts"
      :messages="t.settings"
      :menu-messages="t.menu"
      @close="closeSettings"
      @toggle-always-on-top="togglePin"
      @update:vim-mode="vimMode = $event"
      @update:vim-use-ctrl-shortcuts="vimUseCtrlShortcuts = $event"
      @update:vim-insert-exit-key="setVimInsertExitKey"
      @update:preview-mode="setDefaultEditorMode"
      @update:editor-mode-shortcut="editorModeShortcut = $event"
      @update:language="language = $event"
      @update:run-at-startup="runAtStartup = $event"
      @update:close-to-minimize="closeToMinimize = $event"
      @update:snap-to-edges="snapToEdges = $event"
      @update:transparency-enabled="transparencyEnabled = $event"
      @update:window-opacity-percent="windowOpacity = $event / 100"
      @update:title-double-click-action="titleDoubleClickAction = $event"
      @update:shortcut-base-key="setShortcutBaseKey"
      @update:shortcut-modifiers="shortcutModifiers = $event"
      @update:clipboard-shortcut-base-key="setClipboardShortcutBaseKey"
      @update:clipboard-shortcut-modifiers="clipboardShortcutModifiers = $event"
      @update:insert-separator-template="insertSeparatorTemplate = $event"
      @update:insert-date-time-template="insertDateTimeTemplate = $event"
      @update:insert-date-time-separator-template="insertDateTimeSeparatorTemplate = $event"
      @update:custom-insert-texts="customInsertTexts = $event"
      @edit-custom-text="editCustomInsertText"
      @copy-mcp-config="copyMcpConfig"
    />

    <InputDialog
      v-if="inputDialog"
      :title="inputDialog.title"
      :initial-value="inputDialog.initialValue"
      :confirm-label="t.settings.ok"
      :cancel-label="t.settings.cancel"
      @confirm="finishInputDialog"
      @cancel="finishInputDialog(null)"
    />

    <ConfirmationDialog
      v-if="confirmationDialog"
      :title="confirmationDialog.title"
      :message="confirmationDialog.message"
      :confirm-label="t.tabs.delete"
      :cancel-label="t.settings.cancel"
      @confirm="finishConfirmationDialog(true)"
      @cancel="finishConfirmationDialog(false)"
    />

    <section v-if="helpTopic" class="help-panel" role="dialog" aria-modal="true" :aria-label="helpContent.title">
      <header class="help-header">
        <strong>{{ helpContent.title }}</strong>
        <button type="button" @click="closeHelp">{{ t.settings.close }}</button>
      </header>
      <div class="help-body">
        <p v-for="line in helpContent.lines" :key="line">{{ line }}</p>
      </div>
    </section>

    <template #status>
      <StatusBar
        :state="localizedSaveState"
        :characters="content.length"
        :mode="statusMessage"
        :editor-mode="editorModeLabel"
        :chars-label="t.status.chars"
        :dark-theme="theme === 'dark'"
        :theme-label="themeToggleLabel"
        :vim-mode="activeVimMode ? activeVimMode.toUpperCase() : ''"
        @cycle-editor-mode="cycleEditorMode"
        @toggle-theme="toggleTheme"
      />
    </template>
  </AppShell>
</template>

