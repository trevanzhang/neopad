<script setup lang="ts">
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import AppShell from './components/AppShell.vue'
import EditorPane from './components/EditorPane.vue'
import MenuBar from './components/MenuBar.vue'
import NoteLibrary from './components/NoteLibrary.vue'
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
  listLibraryNotes,
  listNotes,
  listRecentNotes,
  openTrash,
  openNote,
  openExternalMarkdown,
  saveMarkdownFile,
  readExternalMarkdown,
  toggleMainWindowMaximize,
  toggleAlwaysOnTop,
  writeNote,
} from './lib/invoke'
import { messages } from './lib/i18n'
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
import { useWindowLifecycle } from './composables/useWindowLifecycle'
import { editorBackgroundForTheme } from './lib/theme'
import { normalizeShortcutInput } from './lib/shortcut'
import { downloadText, renderInsertTemplate, safeFileName, titleFromFileName } from './lib/document-utils'
import { transformText } from './lib/text-transform'
import { getHelpContent, type HelpTopic } from './lib/help-content'
import { createKeyboardHandler, isEditableElement } from './lib/keyboard-shortcuts'
import { initialJsonSetting } from './lib/preferences'
import type { NoteTab, Reminder, SearchResult } from './types/note'
import {
  nextEditorMode,
  previewThemes,
  type EditorMode,
} from './types/editor'
import { getCurrentWindow } from '@tauri-apps/api/window'


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
  vimUseCtrlShortcuts, vimInsertExitKey, wordWrap, editorFontFamily,
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
const noteLibraryOpen = ref(false)
const libraryNotes = ref<NoteTab[]>([])
const libraryLoading = ref(false)
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
  refreshRecentNotes, refreshArchivedNotes, refreshLibrary: refreshNoteLibrary, upsertTab,
})
const {
  registerNativeEventListeners,
  resetWebviewZoom,
  hideMainWindow,
  exitApp,
  disposeWindowLifecycle,
} = useWindowLifecycle({
  closeToMinimize,
  createLocalTab,
  saveCurrentClipboard,
  openSettings,
  saveBeforeWindowAction,
  onError: () => { saveState.value = 'Failed' },
})
const handleKeydown = createKeyboardHandler({
  state: {
    modalOpen: () => Boolean(reminderDialogOpen.value || confirmationDialog.value || inputDialog.value || fontDialogOpen.value),
    reminderDialogOpen: () => reminderDialogOpen.value,
    reminderListOpen: () => reminderListOpen.value,
    archiveListOpen: () => archiveListOpen.value,
    confirmationOpen: () => Boolean(confirmationDialog.value),
    inputOpen: () => Boolean(inputDialog.value),
    fontDialogOpen: () => fontDialogOpen.value,
    immersiveMode: () => immersiveMode.value,
    settingsOpen: () => settingsOpen.value,
    helpOpen: () => Boolean(helpTopic.value),
    searchOpen: () => searchOpen.value,
    vimMode: () => vimMode.value,
    vimUseCtrlShortcuts: () => vimUseCtrlShortcuts.value,
    vimNormalMode: () => activeVimMode.value === 'normal',
    editorFocused: () => Boolean(editorPane.value?.isEditorFocused()),
    editableTarget: isEditableElement,
    menuOrContextOpen: () => Boolean(document.querySelector('.menu-root:focus-within, .tab-context-menu')),
    tabContextOpen: () => Boolean(document.querySelector('.tab-context-menu')),
    findPanelOpen: () => Boolean(document.querySelector('.np-find-panel')),
    nativeRuntime: isTauriRuntime,
  },
  actions: {
    openShortcutHelp: () => openHelpTopic('shortcuts'),
    cycleTab,
    toggleTheme,
    togglePreviewTheme,
    openReminderList,
    closeReminderList,
    toggleImmersiveMode,
    archiveActivePage,
    deleteActivePage,
    closeReminderDialog,
    closeArchiveList,
    cancelConfirmation: () => finishConfirmationDialog(false),
    cancelInput: () => finishInputDialog(null),
    closeFontDialog,
    exitImmersiveMode: () => setImmersiveMode(false),
    closeSettings,
    closeHelp,
    closeSearch,
    closeEditorFind: () => editorPane.value?.closeEditorFind(),
    cycleEditorMode,
    renameActivePage,
    toggleMainWindowMaximize,
    createLocalTab,
    closeActivePage,
    triggerLoadFile,
    showSearch: showSearchPlaceholder,
    openFind: openFindPanel,
    openReplace: openReplacePanel,
    copy: copyEditorSelection,
    cut: cutEditorSelection,
    paste: pasteIntoEditor,
    selectAll: selectAllEditorText,
    findNext: findNextMatch,
    calculateExpression: calculateCurrentLineExpression,
    hideMainWindow,
    toggleNoteLibrary,
    togglePin,
    openSettings,
    insertDateTimeSeparator,
    insertSeparator,
    insertDateTime,
    insertReminder,
    saveCurrentClipboard,
  },
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
const helpContent = computed(() => getHelpContent(helpTopic.value, language.value, {
  appVersion: appVersion.value,
  shortcutBaseKey: shortcutBaseKey.value,
  shortcutModifiers: shortcutModifiers.value,
  clipboardShortcutBaseKey: clipboardShortcutBaseKey.value,
  clipboardShortcutModifiers: clipboardShortcutModifiers.value,
}))
const editorModeLabel = computed(() => {
  if (previewMode.value === 'preview') return t.value.status.previewMode
  if (previewMode.value === 'split') return t.value.status.hybridMode
  return t.value.status.editMode
})
const effectiveEditorBackground = computed(() => editorBackgroundForTheme(theme.value, editorBackgroundColor.value))
const themeToggleLabel = computed(() => theme.value === 'dark' ? t.value.status.switchToLight : t.value.status.switchToDark)

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

function toggleNoteLibrary() {
  noteLibraryOpen.value = !noteLibraryOpen.value
  if (noteLibraryOpen.value) void refreshNoteLibrary()
}

async function refreshNoteLibrary() {
  libraryLoading.value = true
  try {
    if (isTauriRuntime()) {
      const [notes] = await Promise.all([listLibraryNotes(), refreshArchivedNotes()])
      libraryNotes.value = notes
    } else {
      libraryNotes.value = tabs.value.filter((tab) => !tab.archived && !tab.deleted && !tab.external)
      archivedNotes.value = tabs.value.filter((tab) => tab.archived && !tab.deleted && !tab.external)
    }
  } catch {
    saveState.value = 'Failed'
  } finally {
    libraryLoading.value = false
  }
}

async function selectLibraryNote(noteId: string) {
  if (noteId === activeTabId.value) {
    focusEditorAfterPageAction()
    return
  }
  const generation = nextNoteLoadGeneration()
  if (!(await forceSave()) || !isCurrentNoteLoad(generation)) return
  try {
    const opened = isTauriRuntime() ? await openNote(noteId) : libraryNotes.value.find((tab) => tab.id === noteId)
    if (!opened || !isCurrentNoteLoad(generation)) return
    upsertTab(opened)
    activeTabId.value = opened.id
    await loadActiveNote(generation)
    await refreshRecentNotes()
    focusEditorAfterPageAction()
  } catch {
    saveState.value = 'Failed'
  }
}

async function restoreLibraryNote(tab: NoteTab) {
  await unarchiveTab(tab)
  await refreshNoteLibrary()
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
    const processed = await editorPane.value?.transformText((text) => transformText(action, text, t.value.status.unsupportedHash))
    if (processed) {
      statusMessage.value = t.value.status.textProcessed
    }
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

</script>

<template>
  <AppShell
    :library-open="noteLibraryOpen"
    :data-ready="appReady ? 'true' : 'false'"
    :theme="theme"
    :immersive="immersiveMode"
  >
    <template #title>
      <MenuBar
        :preview-mode="previewMode"
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
        @toggle-note-library="toggleNoteLibrary"
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
        @toggle-library="toggleNoteLibrary"
        @previous-tab="cycleTab(-1)"
        @next-tab="cycleTab(1)"
      />
    </template>

    <template #library>
      <NoteLibrary
        v-if="noteLibraryOpen"
        :notes="libraryNotes"
        :archived-notes="archivedNotes"
        :active-note-id="activeTabId"
        :loading="libraryLoading"
        :messages="t.library"
        @select="selectLibraryNote"
        @restore="restoreLibraryNote"
        @new-note="createLocalTab"
        @refresh="refreshNoteLibrary"
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

    <div v-if="helpTopic" class="input-dialog-backdrop" role="presentation" @mousedown.self="closeHelp()">
      <section class="input-dialog help-panel" role="dialog" aria-modal="true" :aria-label="helpContent.title">
        <header class="input-dialog-header">
          <strong>{{ helpContent.title }}</strong>
          <button type="button" :aria-label="t.settings.close" :title="t.settings.close" @click="closeHelp()"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg></button>
        </header>
        <div class="input-dialog-body help-body">
          <p v-for="line in helpContent.lines" :key="line">{{ line }}</p>
        </div>
      </section>
    </div>

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

