<script setup lang="ts">
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import AppShell from './components/AppShell.vue'
import EditorPane from './components/EditorPane.vue'
import MenuBar from './components/MenuBar.vue'
import SearchPanel from './components/SearchPanel.vue'
import StatusBar from './components/StatusBar.vue'
import TabBar from './components/TabBar.vue'
import {
  createNote,
  completeStartup,
  exportAllNotesZip,
  getAppVersion,
  getShortcutWarnings,
  getWorkspace,
  hideWindow,
  listNotes,
  listRecentNotes,
  openTrash,
  openNote,
  openExternalMarkdown,
  quitApp,
  saveMarkdownFile,
  readExternalMarkdown,
  toggleMainWindowMaximize,
  toggleAlwaysOnTop,
  writeNote,
} from './lib/invoke'
import { messages, type AppLanguage } from './lib/i18n'
import { isTauriRuntime } from './lib/runtime'
import { useDocumentSession } from './composables/useDocumentSession'
import { useSearchState } from './composables/useSearchState'
import { useReminderState } from './composables/useReminderState'
import { useMcpService } from './composables/useMcpService'
import { useDialogs } from './composables/useDialogs'
import { useArchiveState } from './composables/useArchiveState'
import { usePreferenceState } from './composables/usePreferenceState'
import { useNativeSettings } from './composables/useNativeSettings'
import { useNoteLifecycle } from './composables/useNoteLifecycle'
import { editorBackgroundForTheme } from './lib/theme'
import { formatShortcutLabel, normalizeShortcutInput } from './lib/shortcut'
import {
  convertChinese,
  digestText,
  downloadText,
  renderInsertTemplate,
  safeFileName,
  titleFromFileName,
  toFullWidth,
  toHalfWidth,
} from './lib/document-utils'
import { simplifiedToTraditionalMap, traditionalToSimplifiedMap } from './lib/chinese-maps'
import { initialJsonSetting } from './lib/preferences'
import type { NoteTab, Reminder, SearchResult } from './types/note'
import {
  nextEditorMode,
  previewThemes,
  type EditorMode,
} from './types/editor'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { getCurrentWindow } from '@tauri-apps/api/window'

type HelpTopic = 'software' | 'markdown' | 'shortcuts' | 'expression' | 'about'

const ArchiveList = defineAsyncComponent(() => import('./components/ArchiveList.vue'))
const ConfirmationDialog = defineAsyncComponent(() => import('./components/ConfirmationDialog.vue'))
const FontDialog = defineAsyncComponent(() => import('./components/FontDialog.vue'))
const InputDialog = defineAsyncComponent(() => import('./components/InputDialog.vue'))
const PreviewPane = defineAsyncComponent(() => import('./components/PreviewPane.vue'))
const ReminderDialog = defineAsyncComponent(() => import('./components/ReminderDialog.vue'))
const ReminderList = defineAsyncComponent(() => import('./components/ReminderList.vue'))
const SettingsPanel = defineAsyncComponent(() => import('./components/SettingsPanel.vue'))

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
    archived: false,
    open: true,
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
    archived: false,
    open: true,
    systemTitle: false,
  },
])
const activeTabId = ref('inbox')
const recentNotes = ref<NoteTab[]>([])
const externalRecentNotes = ref<NoteTab[]>(initialJsonSetting<NoteTab[]>('neopad.externalRecentNotes', []))
const appReady = ref(false)
const statusMessage = ref('Markdown')
const settingsOpen = ref(false)
const helpTopic = ref<HelpTopic | null>(null)
const appVersion = ref('')
const fontDialogOpen = ref(false)
const {
  inputDialog,
  confirmationDialog,
  requestInput,
  finishInputDialog,
  requestConfirmation,
  finishConfirmationDialog,
} = useDialogs()
const immersiveMode = ref(false)
const alwaysOnTop = ref(false)
const activeVimMode = ref('')
const preferenceState = usePreferenceState({
  onLanguageChanged: () => {
    if (isTauriRuntime()) void syncTrayLanguage()
    statusMessage.value = settingsOpen.value
      ? t.value.status.settings
      : searchOpen.value ? t.value.status.search : t.value.status.markdown
  },
  onWindowOpacityChanged: () => void syncWindowOpacity(),
  onAutostartChanged: () => void syncAutostart(),
  onStartHiddenChanged: () => void syncStartHidden(),
  onCloseToMinimizeChanged: () => void syncCloseToMinimize(),
  onSnapToEdgesChanged: () => void syncSnapToEdges(),
  onToggleShortcutChanged: () => void syncToggleShortcut(),
  onClipboardShortcutChanged: () => void syncClipboardShortcut(),
  onPersistRequested: () => {
    if (uiConfigLoaded.value && isTauriRuntime()) void persistUiConfig()
  },
})
const {
  previewMode, defaultEditorMode, editorModeShortcut, theme, language, vimMode,
  vimUseCtrlShortcuts, vimInsertExitKey, tabBarOrientation, wordWrap, editorFontFamily,
  editorFontSize, editorBackgroundColor, previewTheme, previewFontFamily, previewFontSize,
  previewLineHeight, previewContentWidth, windowOpacity, runAtStartup, startHidden,
  closeToMinimize, snapToEdges, transparencyEnabled, titleDoubleClickAction,
  shortcutBaseKey, shortcutModifiers, clipboardShortcutBaseKey, clipboardShortcutModifiers,
  insertSeparatorTemplate, insertDateTimeTemplate, insertDateTimeSeparatorTemplate,
  customInsertTexts,
} = preferenceState
const fileInput = ref<HTMLInputElement | null>(null)
const backgroundColorInput = ref<HTMLInputElement | null>(null)
const editorPane = ref<InstanceType<typeof EditorPane> | null>(null)
const searchPanel = ref<InstanceType<typeof SearchPanel> | null>(null)
const workspacePath = ref('~/.neopad')
const activeTab = computed(() => tabs.value.find((tab) => tab.id === activeTabId.value) ?? tabs.value[0])
const t = computed(() => messages[language.value])
const {
  content,
  saveState,
  nextNoteLoadGeneration,
  isCurrentNoteLoad,
  loadActiveNote,
  setContentFromLoad,
  forceSave,
  disposeDocumentSession,
} = useDocumentSession({
  tabs,
  activeTabId,
  activeTab,
  statusMessage,
  failedMessage: () => t.value.status.failed,
  rememberExternalDocument,
})
const {
  searchOpen,
  searchQuery,
  searchResults,
  searching,
  searchHasMore,
  scheduleSearch,
  loadMoreSearchResults,
  clearSearch,
  disposeSearchState,
} = useSearchState(() => {
  saveState.value = 'Failed'
})
const {
  reminderDialogOpen,
  reminderListOpen,
  reminders,
  remindersLoading,
  refreshReminders,
  completeReminderItem,
  reopenReminderItem,
  completeAllDueReminders,
  startReminderPolling,
  disposeReminderState,
} = useReminderState({
  activeTabId,
  forceSave,
  loadActiveNote,
  notificationTitle: () => t.value.reminders.notificationTitle,
  onError: () => {
    saveState.value = 'Failed'
  },
})
const {
  mcpStatus,
  mcpUiError,
  loadMcpStatus,
  updateMcpEnabled,
  refreshMcpToken,
  copyMcpConfig,
} = useMcpService({
  stoppedLabel: () => t.value.settings.stopped,
  onUpdated: () => {
    statusMessage.value = t.value.status.mcpUpdated
  },
  onCopied: () => {
    statusMessage.value = t.value.status.mcpConfigCopied
  },
  onError: () => {
    saveState.value = 'Failed'
  },
})
const {
  archiveListOpen,
  archivedNotes,
  archiveLoading,
  refreshArchivedNotes,
} = useArchiveState(forceSave, () => {
  saveState.value = 'Failed'
})
const {
  uiConfigLoaded,
  loadNativeUiConfig,
  persistUiConfig,
  syncTrayLanguage,
  syncAutostart,
  syncStartHidden,
  syncCloseToMinimize,
  syncSnapToEdges,
  syncWindowOpacity,
  syncToggleShortcut,
  syncClipboardShortcut,
  scheduleNativeSettingsSync,
  disposeNativeSettings,
} = useNativeSettings({
  preferences: preferenceState,
  onError: () => { saveState.value = 'Failed' },
  onOpacityUpdated: () => { statusMessage.value = t.value.status.opacityUpdated },
})
const {
  selectTab,
  cycleTab,
  handleTabTitleDoubleClick,
  renameActivePage,
  deleteActivePage,
  closeActivePage,
  archiveActivePage,
  unarchiveActivePage,
  renamePageById,
  deletePageById,
  closePageById,
  archivePageById,
  updateTabColor,
  unarchiveTab,
  createLocalTab,
  saveCurrentClipboard,
} = useNoteLifecycle({
  tabs, activeTabId, activeTab, content, saveState, statusMessage, language,
  titleDoubleClickAction, archiveListOpen, text: () => t.value, forceSave,
  nextNoteLoadGeneration, isCurrentNoteLoad, loadActiveNote, setContentFromLoad,
  requestInput, requestConfirmation, focusEditor: focusEditorAfterPageAction,
  refreshRecentNotes, refreshArchivedNotes, upsertTab,
})
const displayTabs = computed(() => tabs.value.map((tab) => ({
  ...tab,
  title: tab.id === 'inbox'
      ? t.value.tabs.inbox
    : tab.id === 'clipboard'
      ? t.value.tabs.clipboard
      : tab.systemTitle && /^Untitled(?: (\d+))?$/.test(tab.title)
        ? `${t.value.tabs.untitled}${tab.title.slice('Untitled'.length)}`
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
let unlistenNewNoteRequested: UnlistenFn | null = null
let unlistenSaveClipboardRequested: UnlistenFn | null = null
let unlistenOpenSettings: UnlistenFn | null = null
let unlistenCloseRequested: UnlistenFn | null = null
let unlistenHideRequested: UnlistenFn | null = null
let unlistenQuitRequested: UnlistenFn | null = null

onMounted(async () => {
  if (!isTauriRuntime()) {
    window.addEventListener('keydown', handleKeydown, { capture: true })
    appReady.value = true
    return
  }

  await resetWebviewZoom()
  await loadInitialNotes()
  await loadAppVersion()
  await loadWorkspacePath()
  await loadMcpStatus()
  await loadShortcutWarnings()
  await loadNativeUiConfig()
  await Promise.allSettled([syncWindowOpacity(), syncToggleShortcut(), syncClipboardShortcut()])
  window.addEventListener('keydown', handleKeydown, { capture: true })
  window.addEventListener('beforeunload', forceSaveOnExit)
  document.addEventListener('visibilitychange', forceSaveOnHide)
  await registerNativeEventListeners()
  appReady.value = true
  await nextTick()
  await completeStartup().catch(() => {
    saveState.value = 'Failed'
  })
  scheduleNativeSettingsSync()
  await startReminderPolling()
})

async function registerNativeEventListeners() {
  try {
    const [
      newNote,
      saveClipboardRequest,
      openSettingsRequest,
      closeRequest,
      hideRequest,
      quitRequest,
    ] = await Promise.all([
      listen('neopad://new-note-requested', () => {
        void createLocalTab()
      }),
      listen('neopad://save-clipboard-requested', () => {
        void saveCurrentClipboard()
      }),
      listen('neopad://open-settings', () => {
        openSettings()
      }),
      listen('neopad://close-requested', () => {
        void handleCloseRequested()
      }),
      listen('neopad://hide-requested', () => {
        void handleHideRequested()
      }),
      listen('neopad://quit-requested', () => {
        void handleQuitRequested()
      }),
    ])
    unlistenNewNoteRequested = newNote
    unlistenSaveClipboardRequested = saveClipboardRequest
    unlistenOpenSettings = openSettingsRequest
    unlistenCloseRequested = closeRequest
    unlistenHideRequested = hideRequest
    unlistenQuitRequested = quitRequest
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
  disposeDocumentSession()
  disposeSearchState()
  disposeNativeSettings()
  disposeReminderState()
  void unlistenNewNoteRequested?.()
  void unlistenSaveClipboardRequested?.()
  void unlistenOpenSettings?.()
  void unlistenCloseRequested?.()
  void unlistenHideRequested?.()
  void unlistenQuitRequested?.()
  window.removeEventListener('keydown', handleKeydown, { capture: true })
  window.removeEventListener('beforeunload', forceSaveOnExit)
  document.removeEventListener('visibilitychange', forceSaveOnHide)
})

function focusEditorAfterPageAction() {
  if (previewMode.value === 'preview') return
  void nextTick(() => editorPane.value?.focusEditor())
}

function triggerLoadFile() {
  if (isTauriRuntime()) {
    void openExternalDocument()
    return
  }
  fileInput.value?.click()
}

async function openExternalDocument() {
  try {
    const document = await openExternalMarkdown()
    if (!document) return
    await openExternalDocumentPath(document.path, document)
  } catch {
    saveState.value = 'Failed'
  }
}

async function openExternalDocumentPath(path: string, loaded?: import('./types/note').ExternalDocument) {
  const generation = nextNoteLoadGeneration()
  if (!(await forceSave()) || !isCurrentNoteLoad(generation)) return
  try {
    const document = loaded ?? await readExternalMarkdown(path)
    if (!isCurrentNoteLoad(generation)) return
    const tab: NoteTab = {
      id: `external:${document.path}`,
      title: document.title,
      fileName: document.fileName,
      createdAt: document.updatedAt,
      updatedAt: document.updatedAt,
      pinned: false,
      deleted: false,
      archived: false,
      open: true,
      systemTitle: false,
      external: true,
      externalPath: document.path,
      externalRevision: document.revision,
    }
    upsertTab(tab)
    activeTabId.value = tab.id
    setContentFromLoad(document.content)
    rememberExternalDocument(tab)
    saveState.value = 'Saved'
    statusMessage.value = t.value.status.loadedFromFile
    focusEditorAfterPageAction()
  } catch {
    saveState.value = 'Failed'
  }
}

async function loadFileFromInput(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''

  if (!file) {
    return
  }

  const generation = nextNoteLoadGeneration()
  try {
    if (!(await forceSave())) return
    if (!isCurrentNoteLoad(generation)) return
    const fileContent = await file.text()
    if (!isCurrentNoteLoad(generation)) return
    const title = titleFromFileName(file.name)

    if (isTauriRuntime()) {
      const created = await createNote(title)
      if (!isCurrentNoteLoad(generation)) return
      const saved = await writeNote(created.id, fileContent, created.updatedAt)
      if (!isCurrentNoteLoad(generation)) return
      upsertTab({
        id: saved.id,
        title: saved.title,
        fileName: saved.fileName,
        createdAt: saved.updatedAt,
        updatedAt: saved.updatedAt,
        pinned: false,
        deleted: false,
        archived: false,
        open: true,
        systemTitle: false,
      })
      activeTabId.value = saved.id
      setContentFromLoad(saved.content)
    } else {
      createLocalTabFromContent(title, fileContent)
    }

    saveState.value = 'Saved'
    statusMessage.value = t.value.status.loadedFromFile
    focusEditorAfterPageAction()
  } catch {
    saveState.value = 'Failed'
  }
}

async function saveAsFile() {
  if (!(await forceSave())) return
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
  if (!(await forceSave())) return

  try {
    if (isTauriRuntime()) {
      const saved = await exportAllNotesZip()
      if (!saved) return
    } else {
      const sections: string[] = []
      for (const tab of tabs.value) {
        const noteContent = tab.id === activeTabId.value ? content.value : ''
        sections.push(`## ${tab.title}\n\n<!-- file: ${tab.fileName} -->\n\n${noteContent.trimEnd()}\n`)
      }
      const exportContent = `# Exported from NeoPad\n\n${sections.join('\n---\n\n')}`
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

  if (!(await saveBeforeWindowAction())) return
  await hideWindow()
}

async function exitApp() {
  if (!(await saveBeforeWindowAction())) return
  if (isTauriRuntime()) {
    await quitApp()
  }
}

async function handleCloseRequested() {
  if (closeToMinimize.value) {
    await handleHideRequested()
  } else {
    await handleQuitRequested()
  }
}

async function handleHideRequested() {
  if (!isTauriRuntime()) return
  if (!(await saveBeforeWindowAction())) return
  await hideWindow()
}

async function handleQuitRequested() {
  if (!(await saveBeforeWindowAction())) return
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

function promptEditorFont() {
  fontDialogOpen.value = true
}

function confirmEditorFont(nextFont: { fontFamily: string; fontSize: number }) {
  fontDialogOpen.value = false
  editorFontFamily.value = nextFont.fontFamily
  editorFontSize.value = Math.min(22, Math.max(12, Number(nextFont.fontSize) || 14))
  statusMessage.value = t.value.status.fontUpdated
  focusEditorAfterPageAction()
}

function closeFontDialog() {
  fontDialogOpen.value = false
  focusEditorAfterPageAction()
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

function togglePreviewTheme() {
  const currentIndex = previewThemes.indexOf(previewTheme.value)
  previewTheme.value = previewThemes[(currentIndex + 1) % previewThemes.length]
}

async function setImmersiveMode(enabled: boolean) {
  if (enabled) {
    closeSettings(false)
    closeSearch(false)
    closeHelp(false)
  }

  if (isTauriRuntime()) {
    await getCurrentWindow().setFullscreen(enabled)
  }
  immersiveMode.value = enabled
  if (enabled && previewMode.value !== 'preview') {
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
  reminderDialogOpen.value = true
}

function closeReminderDialog() {
  reminderDialogOpen.value = false
  focusEditorAfterPageAction()
}

function confirmReminder(value: { content: string; dueText: string }) {
  reminderDialogOpen.value = false
  if (editorPane.value?.insertLine(`- [ ] @remind ${value.dueText} ${value.content}`)) {
    statusMessage.value = t.value.status.inserted
  }
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
  void nextTick(() => searchPanel.value?.focusSearchInput())
}

function showSettingsPlaceholder() {
  openSettings()
}

function closeSearch(returnFocusToEditor = true) {
  searchOpen.value = false
  clearSearch()
  if (returnFocusToEditor) focusEditorAfterPageAction()
}

async function selectSearchResult(result: SearchResult) {
  const generation = nextNoteLoadGeneration()
  if (!(await forceSave())) return
  if (!isCurrentNoteLoad(generation)) return
  try {
    const opened = isTauriRuntime() ? await openNote(result.noteId) : tabs.value.find((tab) => tab.id === result.noteId)
    if (!opened) return
    upsertTab(opened)
    activeTabId.value = result.noteId
    await refreshRecentNotes()
  } catch {
    saveState.value = 'Failed'
    return
  }
  await loadActiveNote(generation)
  closeSearch(false)
  focusEditorAfterPageAction()
}

function openSettings() {
  settingsOpen.value = true
  statusMessage.value = t.value.status.settings
  void loadMcpStatus()
}

function closeSettings(returnFocusToEditor = true) {
  settingsOpen.value = false
  if (returnFocusToEditor) focusEditorAfterPageAction()
}

function openHelpTopic(topic: HelpTopic) {
  helpTopic.value = topic
}

function closeHelp(returnFocusToEditor = true) {
  helpTopic.value = null
  if (returnFocusToEditor) focusEditorAfterPageAction()
}

async function loadAppVersion() {
  try {
    appVersion.value = await getAppVersion()
  } catch {
    appVersion.value = ''
  }
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

async function openReminderList() {
  closeSearch(false)
  closeSettings(false)
  closeHelp(false)
  reminderListOpen.value = true
  await refreshReminders()
}

async function openArchiveList() {
  closeSearch(false)
  closeSettings(false)
  closeHelp(false)
  closeReminderList(false)
  archiveListOpen.value = true
  await refreshArchivedNotes()
}

function closeArchiveList(returnFocusToEditor = true) {
  archiveListOpen.value = false
  if (returnFocusToEditor) focusEditorAfterPageAction()
}

async function restoreArchivedNote(tab: NoteTab) {
  await unarchiveTab(tab)
}

function closeReminderList(returnFocusToEditor = true) {
  reminderListOpen.value = false
  if (returnFocusToEditor) focusEditorAfterPageAction()
}

async function selectReminder(reminder: Reminder) {
  const generation = nextNoteLoadGeneration()
  if (!(await forceSave())) return
  if (!isCurrentNoteLoad(generation)) return
  activeTabId.value = reminder.noteId
  await loadActiveNote(generation)
  closeReminderList(false)
  if (previewMode.value === 'preview') previewMode.value = 'edit'
  await nextTick()
  editorPane.value?.goToLine(reminder.lineNumber)
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
      return digestText('SHA-1', text, t.value.status.unsupportedHash)
    case 'sha256Hash':
      return digestText('SHA-256', text, t.value.status.unsupportedHash)
    default:
      return text
  }
}

async function loadInitialNotes() {
  try {
    const loadedTabs = await listNotes()
    if (loadedTabs.length > 0) {
      tabs.value = loadedTabs
      activeTabId.value = loadedTabs[0].id
    }
    await refreshRecentNotes()
    await loadActiveNote()
  } catch {
    saveState.value = 'Failed'
  }
}

async function refreshRecentNotes() {
  if (!isTauriRuntime()) return
  const internal = await listRecentNotes()
  recentNotes.value = [...externalRecentNotes.value, ...internal]
    .sort((left, right) => (right.lastOpenedAt ?? right.updatedAt) - (left.lastOpenedAt ?? left.updatedAt))
    .slice(0, 20)
}

async function openRecentNote(noteId: string) {
  const external = externalRecentNotes.value.find((note) => note.id === noteId)
  if (external?.externalPath) {
    await openExternalDocumentPath(external.externalPath)
    return
  }
  const generation = nextNoteLoadGeneration()
  if (!(await forceSave()) || !isCurrentNoteLoad(generation)) return
  try {
    const opened = isTauriRuntime() ? await openNote(noteId) : null
    if (!opened || !isCurrentNoteLoad(generation)) return
    upsertTab(opened)
    activeTabId.value = opened.id
    await loadActiveNote(generation)
    await refreshRecentNotes()
    focusEditorAfterPageAction()
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    if (message.includes('note file does not exist')) {
      statusMessage.value = t.value.status.noteFileMissing
      await refreshRecentNotes()
    }
    saveState.value = 'Failed'
  }
}

function rememberExternalDocument(tab: NoteTab) {
  const recent = { ...tab, lastOpenedAt: Date.now() }
  externalRecentNotes.value = [recent, ...externalRecentNotes.value.filter((item) => item.id !== recent.id)].slice(0, 20)
  window.localStorage.setItem('neopad.externalRecentNotes', JSON.stringify(externalRecentNotes.value))
  void refreshRecentNotes()
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

async function saveBeforeWindowAction() {
  const saved = await forceSave()
  if (!saved) {
    saveState.value = 'Failed'
  }
  return saved
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
  return event.key === 'F4' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey
}

function matchesDeletePageShortcut(event: KeyboardEvent) {
  return event.key === 'Delete' && event.altKey && !event.ctrlKey && !event.shiftKey && !event.metaKey
}

function isEditableElement(element: EventTarget | null) {
  if (!(element instanceof HTMLElement)) return false
  return (
    element instanceof HTMLInputElement ||
    element instanceof HTMLTextAreaElement ||
    element instanceof HTMLSelectElement ||
    element.isContentEditable
  )
}

function handleKeydown(event: KeyboardEvent) {
  const modalOpen = Boolean(reminderDialogOpen.value || confirmationDialog.value || inputDialog.value || fontDialogOpen.value)
  if (modalOpen && event.key !== 'Escape') {
    const key = event.key.toLowerCase()
    const isCtrlShortcut =
      event.ctrlKey && !event.altKey && !event.metaKey && (key === 'tab' || key === 'n' || key === 'w' || key === 'o')
    const isFunctionShortcut =
      !event.ctrlKey &&
      !event.altKey &&
      !event.shiftKey &&
      !event.metaKey &&
      (event.key === 'F4' || event.key === 'F5' || event.key === 'F7' || event.key === 'F9' || event.key === 'F11' || event.key === 'F12')
    const isDeletePageShortcut = matchesDeletePageShortcut(event)
    const isEditorModeShortcut = matchesEditorModeShortcut(event)
    if (isCtrlShortcut || isFunctionShortcut || isDeletePageShortcut || isEditorModeShortcut) {
      event.preventDefault()
      event.stopPropagation()
    }
    return
  }

  if (
    event.key === 'F1' &&
    !event.ctrlKey &&
    !event.altKey &&
    !event.shiftKey &&
    !event.metaKey &&
    (!isEditableElement(event.target) || Boolean(editorPane.value?.isEditorFocused())) &&
    !document.querySelector('.menu-root:focus-within, .tab-context-menu')
  ) {
    event.preventDefault()
    event.stopPropagation()
    openHelpTopic('shortcuts')
    return
  }

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

  if (event.key === 'F7' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    togglePreviewTheme()
    return
  }

  if (event.key === 'F5' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    if (reminderListOpen.value) closeReminderList()
    else void openReminderList()
    return
  }

  if (event.key === 'F11' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault()
    event.stopPropagation()
    toggleImmersiveMode()
    return
  }

  if (
    event.key === 'F12' &&
    !event.ctrlKey &&
    !event.altKey &&
    !event.shiftKey &&
    !event.metaKey &&
    !reminderListOpen.value &&
    !archiveListOpen.value &&
    !settingsOpen.value &&
    !helpTopic.value &&
    !searchOpen.value &&
    !document.querySelector('.menu-root:focus-within, .tab-context-menu')
  ) {
    event.preventDefault()
    event.stopPropagation()
    void archiveActivePage()
    return
  }

  if (
    matchesDeletePageShortcut(event) &&
    !reminderListOpen.value &&
    !archiveListOpen.value &&
    !settingsOpen.value &&
    !helpTopic.value &&
    !searchOpen.value &&
    !document.querySelector('.tab-context-menu')
  ) {
    event.preventDefault()
    event.stopPropagation()
    void deleteActivePage()
    return
  }

  if (event.key === 'Escape') {
    if (reminderDialogOpen.value) {
      event.preventDefault()
      event.stopPropagation()
      closeReminderDialog()
      return
    }

    if (reminderListOpen.value) {
      event.preventDefault()
      event.stopPropagation()
      closeReminderList()
      return
    }

    if (archiveListOpen.value) {
      event.preventDefault()
      event.stopPropagation()
      closeArchiveList()
      return
    }

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

    if (fontDialogOpen.value) {
      event.preventDefault()
      event.stopPropagation()
      closeFontDialog()
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

    if (document.querySelector('.np-find-panel')) {
      event.preventDefault()
      event.stopPropagation()
      editorPane.value?.closeEditorFind()
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

  if (
    event.key === 'F2' &&
    !event.ctrlKey &&
    !event.altKey &&
    !event.shiftKey &&
    !event.metaKey &&
    !reminderListOpen.value &&
    !archiveListOpen.value &&
    !settingsOpen.value &&
    !helpTopic.value &&
    !searchOpen.value &&
    !document.querySelector('.menu-root:focus-within, .tab-context-menu')
  ) {
    event.preventDefault()
    event.stopPropagation()
    void renameActivePage()
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
      if (document.querySelector('.tab-context-menu')) return
      event.preventDefault()
      event.stopPropagation()
      void closeActivePage()
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

function upsertTab(tab: NoteTab) {
  const index = tabs.value.findIndex((existing) => existing.id === tab.id)
  if (index === -1) {
    tabs.value.push(tab)
  } else {
    tabs.value[index] = tab
  }
}

function createLocalTabFromContent(title: string, nextContent: string) {
  nextNoteLoadGeneration()
  const createdAt = Date.now()
  const tab: NoteTab = {
    id: `draft-${createdAt}`,
    title,
    fileName: `${safeFileName(title)}.md`,
    createdAt,
    updatedAt: createdAt,
    pinned: false,
    deleted: false,
    archived: false,
    open: true,
    systemTitle: false,
  }

  tabs.value.push(tab)
  activeTabId.value = tab.id
  content.value = nextContent
  focusEditorAfterPageAction()
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

function getHelpContent(topic: HelpTopic | null, currentLanguage: AppLanguage) {
  const zh = currentLanguage === 'zh'

  if (topic === 'shortcuts') {
    return {
      title: zh ? '\u5feb\u6377\u952e\u5217\u8868' : 'Shortcut List',
      lines: [
        `${formatShortcutLabel(shortcutBaseKey.value, shortcutModifiers.value)} - ` + (zh ? '\u663e\u793a/\u9690\u85cf\u7a97\u53e3' : 'Show/hide window'),
        `${formatShortcutLabel(clipboardShortcutBaseKey.value, clipboardShortcutModifiers.value)} - ` + (zh ? '\u4fdd\u5b58\u526a\u8d34\u677f' : 'Save clipboard'),
        'F1 - ' + (zh ? '\u6253\u5f00\u5feb\u6377\u952e\u5e2e\u52a9' : 'Open shortcut help'),
        'Alt+Enter - ' + (zh ? '\u6700\u5927\u5316/\u8fd8\u539f\u7a97\u53e3' : 'Maximize/restore window'),
        'Ctrl+N - ' + (zh ? '\u65b0\u5efa\u6807\u7b7e\u9875' : 'New tab'),
        'F2 - ' + (zh ? '\u91cd\u547d\u540d\u6807\u7b7e\u9875' : 'Rename tab'),
        'Alt+Del - ' + (zh ? '\u5c06\u5f53\u524d\u6807\u7b7e\u9875\u79fb\u81f3\u56de\u6536\u7ad9' : 'Move current tab to Trash'),
        'Ctrl+W - ' + (zh ? '\u5173\u95ed\u6807\u7b7e\u9875' : 'Close tab'),
        'Ctrl+O - ' + (zh ? '\u4ece\u6587\u4ef6\u8f7d\u5165' : 'Load from file'),
        'Ctrl+Tab / Ctrl+Shift+Tab - ' + (zh ? '\u5207\u6362\u4e0b\u4e00\u4e2a/\u4e0a\u4e00\u4e2a\u6807\u7b7e\u9875' : 'Switch next/previous tab'),
        'Ctrl+F - ' + (zh ? '\u67e5\u627e' : 'Find'),
        'Ctrl+Shift+F - ' + (zh ? '\u5168\u5c40\u641c\u7d22' : 'Global search'),
        'Ctrl+D - ' + (zh ? '\u63d2\u5165\u65e5\u671f\u65f6\u95f4' : 'Insert date time'),
        'Ctrl+- - ' + (zh ? '\u63d2\u5165\u5206\u9694\u884c' : 'Insert separator'),
        'Ctrl+Shift+- - ' + (zh ? '\u63d2\u5165\u65e5\u671f\u65f6\u95f4\u5206\u9694\u884c' : 'Insert date time separator'),
        'Ctrl+E - ' + (zh ? '\u63d2\u5165\u63d0\u9192' : 'Insert reminder'),
        'F4 - ' + (zh ? '\u5faa\u73af\u5207\u6362\u7f16\u8f91\u5668\u6a21\u5f0f' : 'Cycle editor mode'),
        'F5 - ' + (zh ? '\u6253\u5f00/\u5173\u95ed\u63d0\u9192\u5217\u8868' : 'Open/close reminder list'),
        'F6 - ' + (zh ? '\u5207\u6362\u7a97\u53e3\u7f6e\u9876' : 'Toggle window on top'),
        'F7 - ' + (zh ? '\u5207\u6362\u9884\u89c8\u4e3b\u9898' : 'Toggle preview theme'),
        'F8 - ' + (zh ? '\u6253\u5f00\u8bbe\u7f6e' : 'Open settings'),
        'F9 - ' + (zh ? '\u5207\u6362\u65e5\u95f4/\u591c\u95f4\u6a21\u5f0f' : 'Toggle day/night mode'),
        'F11 - ' + (zh ? '\u5207\u6362\u6c89\u6d78\u5f0f\u5168\u5c4f' : 'Toggle immersive fullscreen'),
        'F12 - ' + (zh ? '\u5f52\u6863\u5f53\u524d\u6807\u7b7e\u9875' : 'Archive current tab'),
        'F10 - ' + (zh ? '\u5207\u6362\u6807\u7b7e\u680f\u65b9\u5411' : 'Toggle tab bar orientation'),
        'Esc - ' + (zh ? '\u9690\u85cf\u7a97\u53e3' : 'Hide window'),
      ],
    }
  }

  if (topic === 'markdown') {
    return {
      title: zh ? 'Markdown 简明指南' : 'Markdown Quick Guide',
      lines: zh
        ? [
            '# 一级标题；## 二级标题；### 三级标题',
            '**粗体**；*斜体*；~~删除线~~',
            '- 无序列表；1. 有序列表；- [ ] 待办；- [x] 已完成',
            '[链接文字](https://example.com)；![图片说明](图片地址)',
            '> 引用文字；`行内代码`；三个反引号包裹代码块',
            '--- 单独一行可插入分隔线。段落之间空一行。',
          ]
        : [
            '# Heading 1; ## Heading 2; ### Heading 3',
            '**bold**; *italic*; ~~strikethrough~~',
            '- Bulleted list; 1. numbered list; - [ ] task; - [x] done',
            '[link text](https://example.com); ![image description](image-url)',
            '> Quote; `inline code`; wrap code blocks in three backticks',
            'Use --- on its own line for a divider. Leave a blank line between paragraphs.',
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
            ...(appVersion.value ? [`\u7248\u672c\uff1a${appVersion.value}`] : []),
            '\u4f5c\u8005\uff1aTrevanZhang',
            '\u5f00\u6e90\u9879\u76ee\uff1ahttps://github.com/trevanzhang/neopad',
            '\u5f00\u6e90\u534f\u8bae\uff1aMIT License',
            '\u6280\u672f\u6808\uff1aTauri 2, Vue 3, TypeScript, Rust',
          ]
        : [
            'NeoPad - a lightweight, local-first Markdown desktop note pad.',
            ...(appVersion.value ? [`Version: ${appVersion.value}`] : []),
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
        :active-tab-archived="Boolean(activeTab?.archived)"
        :recent-notes="recentNotes"
        :messages="t.menu"
        @new-note="createLocalTab"
        @rename-page="renameActivePage"
        @delete-page="deleteActivePage"
        @close-page="closeActivePage"
        @archive-page="archiveActivePage"
        @unarchive-page="unarchiveActivePage"
        @open-recent="openRecentNote"
        @save-clipboard="saveCurrentClipboard"
        @load-file="triggerLoadFile"
        @save-as-file="saveAsFile"
        @export-all="exportAllNotes"
        @view-archive="openArchiveList"
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
        @toggle-preview-theme="togglePreviewTheme"
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
        @close-tab="closePageById"
        @archive-tab="archivePageById"
        @unarchive-tab="unarchiveTab(tabs.find((tab) => tab.id === $event)!)"
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
        v-show="previewMode !== 'preview'"
        ref="editorPane"
        v-model="content"
        :title="activeTab?.title ?? 'Untitled'"
        :word-wrap="wordWrap"
        :font-family="editorFontFamily"
        :font-size="editorFontSize"
        :background-color="effectiveEditorBackground"
        :vim-mode="vimMode"
        :vim-insert-exit-key="vimInsertExitKey"
        @vim-mode-change="activeVimMode = $event"
        @vim-tab-switch="cycleTab"
      />
      <PreviewPane
        v-if="previewMode !== 'edit'"
        :content="content"
        :editor-font-family="editorFontFamily"
        :preview-theme="previewTheme"
        :preview-font-family="previewFontFamily"
        :preview-font-size="previewFontSize"
        :preview-line-height="previewLineHeight"
        :preview-content-width="previewContentWidth"
      />
    </div>

    <SearchPanel
      v-if="searchOpen"
      ref="searchPanel"
      v-model:query="searchQuery"
      :results="searchResults"
      :searching="searching"
      :has-more="searchHasMore"
      :messages="t.search"
      @close="closeSearch"
      @select="selectSearchResult"
      @load-more="loadMoreSearchResults"
    />

    <ReminderList
      v-if="reminderListOpen"
      :reminders="reminders"
      :loading="remindersLoading"
      :messages="t.reminders"
      @close="closeReminderList"
      @refresh="refreshReminders"
      @select="selectReminder"
      @complete="completeReminderItem"
      @reopen="reopenReminderItem"
      @complete-due="completeAllDueReminders"
    />

    <ArchiveList
      v-if="archiveListOpen"
      :notes="archivedNotes"
      :loading="archiveLoading"
      :messages="t.archive"
      @close="closeArchiveList"
      @refresh="refreshArchivedNotes"
      @restore="restoreArchivedNote"
    />

    <SettingsPanel
      v-if="settingsOpen"
      :always-on-top="alwaysOnTop"
      :vim-mode="vimMode"
      :vim-use-ctrl-shortcuts="vimUseCtrlShortcuts"
      :vim-insert-exit-key="vimInsertExitKey"
      :preview-mode="defaultEditorMode"
      :language="language"
      :workspace-path="workspacePath"
      :run-at-startup="runAtStartup"
      :start-hidden="startHidden"
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
      :preview-theme="previewTheme"
      :preview-font-family="previewFontFamily"
      :preview-font-size="previewFontSize"
      :preview-line-height="previewLineHeight"
      :preview-content-width="previewContentWidth"
      :mcp-status="mcpStatus"
      :mcp-error="mcpUiError"
      :messages="t.settings"
      :menu-messages="t.menu"
      @close="closeSettings"
      @toggle-always-on-top="togglePin"
      @update:vim-mode="vimMode = $event"
      @update:vim-use-ctrl-shortcuts="vimUseCtrlShortcuts = $event"
      @update:vim-insert-exit-key="setVimInsertExitKey"
      @update:preview-mode="setDefaultEditorMode"
      @update:language="language = $event"
      @update:run-at-startup="runAtStartup = $event"
      @update:start-hidden="startHidden = $event"
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
      @update:preview-theme="previewTheme = $event"
      @update:preview-font-family="previewFontFamily = $event"
      @update:preview-font-size="previewFontSize = $event"
      @update:preview-line-height="previewLineHeight = $event"
      @update:preview-content-width="previewContentWidth = $event"
      @edit-custom-text="editCustomInsertText"
      @update-mcp-enabled="updateMcpEnabled"
      @copy-mcp-config="copyMcpConfig"
      @regenerate-mcp-token="refreshMcpToken"
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

    <FontDialog
      v-if="fontDialogOpen"
      :title="t.menu.font"
      :field-label="t.settings.editorFont"
      :size-label="t.settings.editorFontSize"
      :sample-text="t.settings.fontSample"
      :font-family="editorFontFamily"
      :font-size="editorFontSize"
      :confirm-label="t.settings.ok"
      :cancel-label="t.settings.cancel"
      @confirm="confirmEditorFont"
      @cancel="closeFontDialog"
    />

    <ReminderDialog
      v-if="reminderDialogOpen"
      :title="t.reminders.createTitle"
      :content-label="t.reminders.contentLabel"
      :date-label="t.reminders.dateLabel"
      :time-label="t.reminders.timeLabel"
      :confirm-label="t.reminders.insert"
      :cancel-label="t.reminders.cancel"
      @confirm="confirmReminder"
      @cancel="closeReminderDialog"
    />

    <ConfirmationDialog
      v-if="confirmationDialog"
      :title="confirmationDialog.title"
      :message="confirmationDialog.message"
      :confirm-label="confirmationDialog.confirmLabel"
      :cancel-label="t.settings.cancel"
      :danger="confirmationDialog.danger"
      @confirm="finishConfirmationDialog(true)"
      @cancel="finishConfirmationDialog(false)"
    />

    <section v-if="helpTopic" class="help-panel" role="dialog" aria-modal="true" :aria-label="helpContent.title">
      <header class="help-header">
        <strong>{{ helpContent.title }}</strong>
        <button type="button" @click="closeHelp()">{{ t.settings.close }}</button>
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

