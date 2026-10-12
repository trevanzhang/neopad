<script setup lang="ts">
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import AppShell from './components/AppShell.vue'
import EditorPane from './components/EditorPane.vue'
import MenuBar from './components/MenuBar.vue'
import NoteLibrary from './components/NoteLibrary.vue'
import SearchPanel from './components/SearchPanel.vue'
import StatusBar from './components/StatusBar.vue'
import TabBar from './components/TabBar.vue'
import {
  listProjectDirectories, createProjectDirectory, moveProjectDirectory, renameProjectDirectory, deleteProjectDirectory,
  archiveProjectDirectory, moveNoteToProject, moveNoteToCapture, createProjectNote,
  createArchiveDirectory,
  deleteArchiveDirectory,
  createNote,
  clearTrash,
  completeStartup,
  copyExternalMarkdownPathToClipboard,
  copyNotePathToClipboard,
  exportAllNotesZip,
  getAppVersion,
  getShortcutWarnings,
  getWorkspace,
  listLibraryNotes,
  listArchiveDirectories,
  listTrashedNotes,
  listNotes,
  listRecentNotes,
  listRecoverableNoteWrites,
  openWorkspaceInFileManager,
  openTrash,
  openNote,
  openExternalMarkdown,
  openExternalMarkdownPaths,
  saveMarkdownFile,
  readExternalMarkdown,
  restoreRecoverableNoteWrite,
  revealExternalMarkdownInFileManager,
  revealNoteInFileManager,
  takePendingExternalMarkdownPaths,
  toggleMainWindowMaximize,
  toggleAlwaysOnTop,
  restoreNoteFromTrash,
  renameNoteWithHeading,
  moveArchiveDirectory,
  moveArchivedNote,
  renameArchiveDirectory,
  archiveNoteToDirectory,
  reorderOpenNotes,
  writeNote,
} from './lib/invoke'
import { messages } from './lib/i18n'
import { isTauriRuntime } from './lib/runtime'
import { useDocumentSession } from './composables/useDocumentSession'
import { useSearchState } from './composables/useSearchState'
import { useReminderState } from './composables/useReminderState'
import { useAgentTools } from './composables/useAgentTools'
import { useFinancialTools } from './composables/useFinancialTools'
import { financeText } from './lib/finance-ui'
import type { FinancialInsertTarget } from './types/finance'
import { useAiAssistant } from './composables/useAiAssistant'
import { useDialogs } from './composables/useDialogs'
import { useArchiveState } from './composables/useArchiveState'
import { usePreferenceState } from './composables/usePreferenceState'
import { useNativeSettings } from './composables/useNativeSettings'
import { useNoteLifecycle } from './composables/useNoteLifecycle'
import { useNoteExport } from './composables/useNoteExport'
import { useWindowLifecycle } from './composables/useWindowLifecycle'
import { editorBackgroundForTheme } from './lib/theme'
import { normalizeShortcutInput } from './lib/shortcut'
import { downloadText, renderInsertTemplate, safeFileName, titleFromFileName } from './lib/document-utils'
import { transformText } from './lib/text-transform'
import { getHelpContent, getReferenceHelp, getShortcutHelpGroups, type HelpTopic } from './lib/help-content'
import { createKeyboardHandler, isEditableElement } from './lib/keyboard-shortcuts'
import { createAiInlinePlan, type AiInlinePlan } from './lib/ai-inline-command'
import { normalizeAiNoteTitle } from './lib/ai-title'
import { isCurrentAiRequest } from './lib/ai-request-state'
import { initialJsonSetting } from './lib/preferences'
import { isExternalTab, isNoteTab } from './lib/document-tab'
import type { NoteTab, Reminder, SearchResult } from './types/note'
import type {
  AiChatState,
  AiContextKind,
  AiContextScope,
  AiConversationMessage,
  AiInlineCommandName,
  AiPanelSession,
} from './types/ai'
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
const FinancialToolsPanel = defineAsyncComponent(() => import('./components/FinancialToolsPanel.vue'))
const SettingsPanel = defineAsyncComponent(() => import('./components/SettingsPanel.vue'))
const AiAssistantPanel = defineAsyncComponent(() => import('./components/AiAssistantPanel.vue'))

interface AiInlineSession {
  requestId: number
  noteId: string
  command: AiInlineCommandName
  plan: AiInlinePlan | null
}

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
const settingsInitialTab = ref<'general' | 'ai'>('general')
const aiPanelSession = ref<AiPanelSession | null>(null)
const aiChatSessions = ref<Record<string, AiChatState>>({})
const aiInlineSession = ref<AiInlineSession | null>(null)
const aiRenamingTabId = ref<string | null>(null)
let aiInlineRequestId = 0
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
const financialTools = useFinancialTools()
const { category: financeCategory, draft: financeDraft, result: financeResult, error: financeError, busy: financeBusy, changed: financeChanged } = financialTools
const financialToolsOpen = ref(false)
const financialInsertTarget = ref<FinancialInsertTarget | null>(null)
const financialNotice = ref('')
const searchPanel = ref<InstanceType<typeof SearchPanel> | null>(null)
const workspacePath = ref('~/.neopad')
const noteLibraryOpen = ref(false)
const libraryNotes = ref<NoteTab[]>([])
const trashedNotes = ref<NoteTab[]>([])
const projectNotes = ref<NoteTab[]>([])
const projectDirectories = ref<string[]>([])
const projectMenuLoading = ref(false)
const projectMenuFailed = ref(false)
const archiveDirectories = ref<string[]>([])
const libraryLoading = ref(false)
const activeTab = computed(() => tabs.value.find((tab) => tab.id === activeTabId.value) ?? tabs.value[0])
const t = computed(() => messages[language.value])
const {
  content,
  saveState,
  isLoadingNote,
  nextNoteLoadGeneration,
  isCurrentNoteLoad,
  loadActiveNote,
  setContentFromLoad,
  forceSave,
  disposeDocumentSession,
  refreshActiveNote,
} = useDocumentSession({
  tabs,
  activeTabId,
  activeTab,
  statusMessage,
  failedMessage: () => t.value.status.failed,
  rememberExternalDocument,
})
const { exportingNote, exportNote } = useNoteExport({
  tabs,
  activeTabId,
  content,
  isLoadingNote,
  statusMessage,
  previewTheme,
  editorFontFamily,
  previewFontFamily,
  previewFontSize,
  previewLineHeight,
  forceSave,
  text: () => t.value.status,
  safeFileName,
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
  agentToolsStatus, agentToolsError, agentToolsBusy,
  loadAgentToolsStatus, repairAgentCli, copyAgentCliPath, openAgentGuide,
} = useAgentTools(() => { statusMessage.value = t.value.status.agentCliPathCopied }, openAgentGuideDocument)
const {
  aiConfig,
  aiError,
  aiTesting,
  aiTestSucceeded,
  loadAiConfig,
  updateAiConfig,
  storeApiKey,
  removeApiKey,
  checkConnection,
  requestAiText,
  disposeAiAssistant,
} = useAiAssistant()
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
  renameTab,
  archiveTab,
  deleteTab,
  createLocalTab,
  saveCurrentClipboard,
  importCaptureFromClipboard,
} = useNoteLifecycle({
  tabs, activeTabId, activeTab, content, saveState, statusMessage, language,
  titleDoubleClickAction, archiveListOpen, text: () => t.value, forceSave,
  nextNoteLoadGeneration, isCurrentNoteLoad, loadActiveNote, setContentFromLoad,
  requestInput, focusEditor: focusEditorAfterPageAction,
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
  importCaptureFromClipboard,
  openExternalDocuments: openExternalDocumentPaths,
  openPendingExternalDocuments,
  openSettings,
  saveBeforeWindowAction,
  onError: () => { saveState.value = 'Failed' },
})
const handleKeydown = createKeyboardHandler({
  state: {
    financialToolsOpen: () => financialToolsOpen.value,
    modalOpen: () => Boolean(reminderDialogOpen.value || confirmationDialog.value || inputDialog.value || fontDialogOpen.value || aiPanelSession.value || aiInlineSession.value || helpTopic.value),
    reminderDialogOpen: () => reminderDialogOpen.value,
    reminderListOpen: () => reminderListOpen.value,
    archiveListOpen: () => archiveListOpen.value,
    confirmationOpen: () => Boolean(confirmationDialog.value),
    inputOpen: () => Boolean(inputDialog.value),
    fontDialogOpen: () => fontDialogOpen.value,
    aiPanelOpen: () => Boolean(aiPanelSession.value),
    aiInlineOpen: () => Boolean(aiInlineSession.value),
    immersiveMode: () => immersiveMode.value,
    settingsOpen: () => settingsOpen.value,
    helpOpen: () => Boolean(helpTopic.value),
    searchOpen: () => searchOpen.value,
    vimMode: () => vimMode.value,
    vimUseCtrlShortcuts: () => vimUseCtrlShortcuts.value,
    vimNormalMode: () => activeVimMode.value === 'normal',
    editorFocused: () => Boolean(editorPane.value?.isEditorFocused()),
    editableTarget: isEditableElement,
    menuOrContextOpen: () => Boolean(document.querySelector('.menu-root:focus-within, .tab-context-menu, .editor-context-menu')),
    tabContextOpen: () => Boolean(document.querySelector('.tab-context-menu')),
    findPanelOpen: () => Boolean(document.querySelector('.np-find-panel')),
    nativeRuntime: isTauriRuntime,
  },
  actions: {
    openFinancialTools,
    closeFinancialTools,
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
    closeAiPanel,
    cancelAiInlineCommand,
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
    undo: undoEditor,
    redo: redoEditor,
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
    openAiPanel,
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
const shortcutHelpGroups = computed(() => getShortcutHelpGroups(language.value, {
  appVersion: appVersion.value,
  shortcutBaseKey: shortcutBaseKey.value,
  shortcutModifiers: shortcutModifiers.value,
  clipboardShortcutBaseKey: clipboardShortcutBaseKey.value,
  clipboardShortcutModifiers: clipboardShortcutModifiers.value,
}))
const referenceHelp = computed(() => helpTopic.value === 'markdown' || helpTopic.value === 'vim' || helpTopic.value === 'expression'
  ? getReferenceHelp(helpTopic.value, language.value)
  : null)
const editorModeLabel = computed(() => {
  if (previewMode.value === 'preview') return t.value.status.previewMode
  if (previewMode.value === 'split') return t.value.status.hybridMode
  return t.value.status.editMode
})
const effectiveEditorBackground = computed(() => editorBackgroundForTheme(theme.value, editorBackgroundColor.value))
const themeToggleLabel = computed(() => theme.value === 'dark' ? t.value.status.switchToLight : t.value.status.switchToDark)
const aiReady = computed(() => aiConfig.value.enabled && Boolean(aiConfig.value.baseUrl.trim()) && Boolean(aiConfig.value.model.trim()))
const aiSlashLabels = computed(() => ({
  polish: t.value.ai.polishCommand,
  summarize: t.value.ai.summarizeCommand,
  translate: t.value.ai.translateCommand,
  continue: t.value.ai.continueCommand,
}))
const aiInlinePrompts = computed<Record<AiInlineCommandName, string>>(() => ({
  polish: t.value.ai.polishPrompt,
  summarize: t.value.ai.summarizePrompt,
  translate: t.value.ai.translatePrompt,
  continue: t.value.ai.continuePrompt,
}))

watch(activeTabId, (next, previous) => {
  if (next !== previous && aiInlineSession.value) cancelAiInlineCommand(false)
  if (next !== previous) financialInsertTarget.value = null
})

watch([financeCategory, financeDraft], () => { financialNotice.value = '' }, { deep: true })

let workspaceRefreshTimer: ReturnType<typeof setInterval> | null = null
let workspaceRefreshing = false

async function refreshWorkspaceNotes() {
  if (workspaceRefreshing || isLoadingNote.value || saveState.value !== 'Saved' || aiInlineSession.value) return
  workspaceRefreshing = true
  try {
    await refreshActiveNote()
    if (isLoadingNote.value || saveState.value !== 'Saved') return
    const id = activeTabId.value
    const previousContent = content.value
    const previousOrder = tabs.value.map((tab) => tab.id).join('\n')
    const loaded = await listNotes()
    if (activeTabId.value !== id || isLoadingNote.value || saveState.value !== 'Saved'
      || content.value !== previousContent || tabs.value.map((tab) => tab.id).join('\n') !== previousOrder) return
    const active = tabs.value.find((tab) => tab.id === id)
    // Keep the revision belonging to the displayed content until the next read.
    const notes = loaded.map((tab) => tab.id === id && active ? { ...tab, updatedAt: active.updatedAt } : tab)
    const incoming = new Map(notes.map((tab) => [tab.id, tab]))
    // Preserve external tabs and the user's current tab order.
    const current = tabs.value.flatMap((tab) => {
      if (!isNoteTab(tab)) return [tab]
      const updated = incoming.get(tab.id)
      return updated ? [updated] : []
    })
    const existing = new Set(current.map((tab) => tab.id))
    tabs.value = [...current, ...notes.filter((tab) => !existing.has(tab.id))]
    if (!tabs.value.some((tab) => tab.id === id) && tabs.value.length) {
      activeTabId.value = tabs.value[0].id
      await loadActiveNote()
    }
    if (noteLibraryOpen.value) await refreshNoteLibrary()
  } catch { /* Retry on the next refresh without discarding editor content. */ }
  finally { workspaceRefreshing = false }
}

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
  await loadAgentToolsStatus()
  await loadAiConfig()
  await loadShortcutWarnings()
  await loadNativeUiConfig()
  await Promise.allSettled([syncWindowOpacity(), syncToggleShortcut(), syncClipboardShortcut()])
  window.addEventListener('keydown', handleKeydown, { capture: true })
  window.addEventListener('beforeunload', forceSaveOnExit)
  document.addEventListener('visibilitychange', forceSaveOnHide)
  await registerNativeEventListeners()
  await openPendingExternalDocuments()
  appReady.value = true
  await nextTick()
  await completeStartup().catch(() => {
    saveState.value = 'Failed'
  })
  await recoverPendingNoteWrites()
  scheduleNativeSettingsSync()
  await startReminderPolling()
  workspaceRefreshTimer = setInterval(() => { void refreshWorkspaceNotes() }, 2000)
})

onBeforeUnmount(() => {
  if (workspaceRefreshTimer) clearInterval(workspaceRefreshTimer)

  window.removeEventListener('keydown', handleKeydown, true)
  window.removeEventListener('beforeunload', forceSaveOnExit)
  document.removeEventListener('visibilitychange', forceSaveOnHide)
  disposeReminderState()
  disposeNativeSettings()
  disposeSearchState()
  disposeDocumentSession()
  disposeWindowLifecycle()
  disposeAiAssistant()
})

async function recoverPendingNoteWrites() {
  try {
    const recoveries = await listRecoverableNoteWrites()
    for (const recovery of recoveries) {
      const message = t.value.recovery.message.replace('{fileName}', recovery.targetFileName)
      const confirmed = await requestConfirmation(
        t.value.recovery.title,
        message,
        t.value.recovery.restore,
      )
      if (!confirmed) break
      await restoreRecoverableNoteWrite(recovery.recoveryFileName)
      await loadInitialNotes()
      statusMessage.value = t.value.recovery.restored
    }
  } catch {
    saveState.value = 'Failed'
  }
}

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

async function openAgentGuideDocument(path: string) {
  const [document] = await openExternalMarkdownPaths([path])
  if (!document) throw new Error('NeoPad Agent guide could not be opened')
  closeSettings(false)
  await openExternalDocumentPath(document.path, document)
}

async function openPendingExternalDocuments() {
  try {
    const paths = await takePendingExternalMarkdownPaths()
    if (paths.length > 0) await openExternalDocumentPaths(paths)
  } catch {
    saveState.value = 'Failed'
  }
}

async function openExternalDocumentPaths(paths: string[]) {
  if (paths.length === 0) return
  try {
    const documents = await openExternalMarkdownPaths(paths)
    for (const document of documents) {
      await openExternalDocumentPath(document.path, document)
    }
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
      kind: 'external',
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

async function openWorkspaceFolder() {
  if (!isTauriRuntime()) return
  try {
    await openWorkspaceInFileManager()
    statusMessage.value = t.value.status.workspaceOpened
  } catch {
    saveState.value = 'Failed'
  }
}

async function revealNoteInExplorer(noteId: string) {
  if (!isTauriRuntime() || !(await forceSave())) return
  try {
    const tab = tabs.value.find((item) => item.id === noteId)
    if (tab?.externalPath) await revealExternalMarkdownInFileManager(tab.externalPath)
    else await revealNoteInFileManager(noteId)
  } catch (error) {
    statusMessage.value = displayAiRequestError(error)
  }
}

async function copyNoteFilePath(noteId: string) {
  const tab = tabs.value.find((item) => item.id === noteId)
  if (!tab) return

  try {
    if (isTauriRuntime()) {
      if (tab.externalPath) await copyExternalMarkdownPathToClipboard(tab.externalPath)
      else await copyNotePathToClipboard(noteId)
    } else {
      await navigator.clipboard.writeText(tab.externalPath ?? (tab.projectRelativePath ? `projects/${tab.projectRelativePath}` : tab.fileName))
    }
    statusMessage.value = t.value.status.notePathCopied
  } catch {
    saveState.value = 'Failed'
  }
}

function undoEditor() {
  editorPane.value?.undoEdit()
}

function redoEditor() {
  editorPane.value?.redoEdit()
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
    const [notes, trashed, archiveFolders, projectFolders] = isTauriRuntime()
      ? await Promise.all([listLibraryNotes(), listTrashedNotes(), listArchiveDirectories(), listProjectDirectories()])
      : [tabs.value.filter(tab => isNoteTab(tab) && !tab.archived && !tab.deleted), tabs.value.filter(tab => tab.deleted), archiveDirectories.value, projectDirectories.value]
    libraryNotes.value = notes.filter(note => !note.projectRelativePath)
    projectNotes.value = notes.filter(note => !!note.projectRelativePath)
    trashedNotes.value = trashed
    archiveDirectories.value = archiveFolders
    projectDirectories.value = projectFolders
    await refreshArchivedNotes()
  } catch (error) { saveState.value = 'Failed'; statusMessage.value = displayAiRequestError(error) }
  finally { libraryLoading.value = false }
}
async function selectLibraryNote(noteId: string): Promise<boolean> {
  if (noteId === activeTabId.value) {
    focusEditorAfterPageAction()
    return true
  }
  const generation = nextNoteLoadGeneration()
  if (!(await forceSave()) || !isCurrentNoteLoad(generation)) return false
  try {
    const opened = isTauriRuntime()
      ? await openNote(noteId)
      : [...libraryNotes.value, ...projectNotes.value, ...archivedNotes.value].find((tab) => tab.id === noteId)
    if (!opened || !isCurrentNoteLoad(generation)) return false
    upsertTab(opened)
    activeTabId.value = opened.id
    if (!(await loadActiveNote(generation))) return false
    await refreshRecentNotes()
    focusEditorAfterPageAction()
    return true
  } catch {
    saveState.value = 'Failed'
    return false
  }
}



async function createLibraryDirectory(kind: 'archive' | 'project', parent = '') {
  const name = (await requestInput(t.value.library.newFolder, ''))?.trim()
  if (!name) return
  const relativePath = parent ? `${parent}/${name}` : name
  try {
    if (kind === 'archive') await createArchiveDirectory(relativePath)
    else await createProjectDirectory(relativePath)
    await refreshNoteLibrary()
  } catch (error) { statusMessage.value = displayAiRequestError(error) }
}
function directoryName(path: string) {
  return path.slice(path.lastIndexOf('/') + 1)
}

async function moveLibraryDirectory(kind: 'archive' | 'project', path: string, targetParent: string) {
  if (!(await forceSave())) return
  try {
    if (kind === 'archive') await moveArchiveDirectory(path, targetParent)
    else await moveProjectDirectory(path, targetParent)
    await refreshWorkspaceNotes()
    await refreshNoteLibrary()
  } catch (error) { statusMessage.value = displayAiRequestError(error) }
}
async function renameLibraryDirectory(kind: 'archive' | 'project', path: string) {
  const name = (await requestInput(t.value.library.renameFolderTitle, directoryName(path)))?.trim()
  if (!name || !(await forceSave())) return
  try {
    if (kind === 'archive') await renameArchiveDirectory(path, name)
    else await renameProjectDirectory(path, name)
    await refreshWorkspaceNotes()
    await refreshNoteLibrary()
  } catch (error) { statusMessage.value = displayAiRequestError(error) }
}
async function deleteLibraryDirectory(kind: 'archive' | 'project', path: string) {
  if (!(await requestConfirmation(t.value.library.deleteFolderTitle,
    t.value.library.deleteFolderMessage.replace('{name}', directoryName(path)), t.value.library.deleteFolderConfirm, true))
    || !(await forceSave())) return
  try {
    if (kind === 'archive') await deleteArchiveDirectory(path)
    else await deleteProjectDirectory(path)
    await refreshWorkspaceNotes()
    await refreshNoteLibrary()
  } catch (error) { statusMessage.value = displayAiRequestError(error); await refreshNoteLibrary() }
}
async function archiveLibraryNotesTo(notes: NoteTab[], directory: string) {
  try {
    if (notes.some((note) => tabs.value.some((tab) => tab.id === note.id)) && !(await forceSave())) return
    for (const note of notes) {
      const updated = note.archived
        ? await moveArchivedNote(note.id, directory)
        : await archiveNoteToDirectory(note.id, directory)
      const openIndex = tabs.value.findIndex((tab) => tab.id === note.id)
      if (openIndex >= 0) tabs.value[openIndex] = updated
      if (openIndex >= 0) {
        tabs.value = tabs.value.filter((tab) => tab.id !== note.id)
        if (note.id === activeTabId.value) activeTabId.value = tabs.value[0]?.id ?? 'inbox'
      }
    }
    await loadActiveNote()
    await refreshNoteLibrary()
  } catch (error) {
    saveState.value = 'Failed'
    statusMessage.value = displayAiRequestError(error)
    await loadActiveNote()
    await refreshNoteLibrary().catch(() => undefined)
  }
}

async function moveLibraryNotesToProject(notes: NoteTab[], directory: string) {
  if (!directory || !(await forceSave())) return
  try {
    for (const note of notes) {
      const moved = await moveNoteToProject(note.id, directory)
      const index = tabs.value.findIndex(tab => tab.id === note.id)
      if (index >= 0) tabs.value[index] = moved
    }
    await refreshWorkspaceNotes()
    await refreshNoteLibrary()
  } catch (error) { statusMessage.value = displayAiRequestError(error); await refreshNoteLibrary() }
}
async function refreshProjectMenu() {
  if (projectMenuLoading.value) return
  projectMenuLoading.value = true
  projectMenuFailed.value = false
  try {
    if (isTauriRuntime()) projectDirectories.value = await listProjectDirectories()
  } catch (error) {
    projectMenuFailed.value = true
    statusMessage.value = displayAiRequestError(error)
  } finally { projectMenuLoading.value = false }
}
async function moveTabToProject(tabId: string, directory: string) {
  const tab = tabs.value.find(tab => tab.id === tabId)
  if (!tab || !isNoteTab(tab) || tab.pinned || tab.deleted) return
  await moveLibraryNotesToProject([tab], directory)
  if (activeTabId.value === tabId) focusEditorAfterPageAction()
}
async function moveLibraryNotesToCapture(notes: NoteTab[]) {
  if (!(await forceSave())) return
  try {
    for (const note of notes) await moveNoteToCapture(note.id)
    await refreshWorkspaceNotes()
    await refreshNoteLibrary()
  } catch (error) { statusMessage.value = displayAiRequestError(error) }
}
async function archiveLibraryProject(path: string) {
  if (!(await forceSave())) return
  try {
    await archiveProjectDirectory(path)
    await refreshWorkspaceNotes()
    await refreshNoteLibrary()
  } catch (error) { statusMessage.value = displayAiRequestError(error) }
}
async function createLibraryProjectNote(directory = '') {
  if (!directory) directory = (await requestInput(t.value.library.projects, ''))?.trim() ?? ''
  if (!directory) return
  const title = (await requestInput(t.value.library.newProjectTitle, ''))?.trim()
  if (!title || !(await forceSave())) return
  try {
    const created = await createProjectNote(directory, title)
    await refreshNoteLibrary()
    await selectLibraryNote(created.id)
  } catch (error) { statusMessage.value = displayAiRequestError(error) }
}
async function reorderTabs(orderedIds: string[]) {
  const byId = new Map(tabs.value.map((tab) => [tab.id, tab]))
  const ordered = orderedIds.map((id) => byId.get(id)).filter((tab): tab is NoteTab => Boolean(tab))
  if (ordered.length !== tabs.value.length) return
  tabs.value = ordered
  if (!isTauriRuntime()) return
  try {
    await reorderOpenNotes(ordered.filter(isNoteTab).map((tab) => tab.id))
  } catch {
    saveState.value = 'Failed'
  }
}






async function restoreLibraryNote(notes: NoteTab[]) {
  for (const tab of notes) await unarchiveTab(tab)
  await refreshNoteLibrary()
}

async function restoreTrashedLibraryNotes(notes: NoteTab[]) {
  try {
    for (const tab of notes) {
      const restored = isTauriRuntime() ? await restoreNoteFromTrash(tab.id) : { ...tab, deleted: false, open: true }
      if (!restored.archived) upsertTab(restored)
    }
    await refreshNoteLibrary()
  } catch {
    saveState.value = 'Failed'
  }
}

async function clearLibraryTrash() {
  try {
    const confirmed = await requestConfirmation(
      t.value.library.clearTrashTitle,
      t.value.library.clearTrashMessage,
      t.value.library.clearTrashConfirm,
      true,
    )
    if (!confirmed) return
    if (isTauriRuntime()) await clearTrash()
    else {
      tabs.value = tabs.value.filter((tab) => !tab.deleted)
    }
    await refreshNoteLibrary()
  } catch {
    await refreshNoteLibrary().catch(() => undefined)
    saveState.value = 'Failed'
  }
}

async function runLibraryNoteAction(notes: NoteTab[], action: 'rename' | 'archive' | 'delete') {
  if (action === 'rename') {
    const [tab] = notes
    if (tab) await renameTab(tab)
  } else {
    for (const tab of notes) {
      if (action === 'archive') await archiveTab(tab)
      else await deleteTab(tab)
    }
  }
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

function openSettings(initialTab: 'general' | 'ai' = 'general') {
  settingsInitialTab.value = initialTab
  settingsOpen.value = true
  statusMessage.value = t.value.status.settings
  void loadAgentToolsStatus()
  void loadAiConfig()
}

function closeSettings(returnFocusToEditor = true) {
  settingsOpen.value = false
  if (returnFocusToEditor) focusEditorAfterPageAction()
}


function openAiPanel() {
  const snapshot = editorPane.value?.captureAiSnapshot()
  if (!snapshot || !activeTab.value) return
  cancelAiInlineCommand(false)
  const savedChat = aiChatSessions.value[activeTab.value.id]
  aiPanelSession.value = {
    noteId: activeTab.value.id,
    noteTitle: activeTab.value.title,
    snapshot,
    chat: {
      messages: savedChat?.messages.map((message) => ({ ...message, sources: message.sources?.map((source) => ({ ...source })) })) ?? [],
      scope: savedChat?.scope ?? 'note',
    },
  }
}

function closeAiPanel(returnFocusToEditor = true) {
  aiPanelSession.value = null
  if (returnFocusToEditor) focusEditorAfterPageAction()
}

function openAiSettings() {
  closeAiPanel(false)
  cancelAiInlineCommand(false)
  openSettings('ai')
}

function updateAiChatState(chat: AiChatState) {
  const session = aiPanelSession.value
  if (!session) return
  session.chat = chat
  aiChatSessions.value = {
    ...aiChatSessions.value,
    [session.noteId]: {
      messages: chat.messages.map((message) => ({
        ...message,
        sources: message.sources?.map((source) => ({ ...source })),
      })),
      scope: chat.scope,
    },
  }
}

async function generateForAiPanel(
  conversation: AiConversationMessage[],
  scope: AiContextScope,
) {
  const session = aiPanelSession.value
  if (!session) throw new Error(t.value.ai.staleContext)
  const currentSnapshot = editorPane.value?.captureAiSnapshot()
  if (!currentSnapshot || session.noteId !== activeTabId.value) {
    throw new Error(t.value.ai.staleContext)
  }
  const context = currentSnapshot.contexts.find((item) => item.kind === 'note')
  if (!context) throw new Error(t.value.ai.staleContext)
  return requestAiText(context.text, conversation, {
    searchLibrary: scope === 'library',
    currentNoteId: session.noteId,
  })
}

function cancelAiInlineCommand(returnFocusToEditor = true) {
  const session = aiInlineSession.value
  if (session) editorPane.value?.cancelAiInlineCommand(session.requestId)
  aiInlineRequestId += 1
  aiInlineSession.value = null
  if (returnFocusToEditor) focusEditorAfterPageAction()
}

function displayAiRequestError(error: unknown) {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}

async function aiRenamePageById(tabId: string) {
  const tab = tabs.value.find((item) => item.id === tabId)
  if (
    aiRenamingTabId.value
    || !tab
    || !isNoteTab(tab)
    || tab.id === 'inbox'
    || tab.id === 'clipboard'
  ) return

  if (!aiReady.value) {
    statusMessage.value = t.value.ai.disabled
    openAiSettings()
    return
  }

  aiRenamingTabId.value = tabId
  statusMessage.value = t.value.tabs.aiRenaming
  try {
    if (activeTabId.value !== tabId) await selectTab(tabId)
    if (activeTabId.value !== tabId || !(await forceSave())) return

    const response = await requestAiText(
      content.value,
      [{ role: 'user', content: t.value.ai.renamePrompt }],
      { searchLibrary: false, currentNoteId: tabId },
    )
    const nextTitle = normalizeAiNoteTitle(response.content)
    if (!nextTitle) throw new Error(t.value.ai.renameEmptyTitle)

    if (activeTabId.value === tabId && !(await forceSave())) return
    const renamed = await renameNoteWithHeading(tabId, nextTitle)
    const currentTab = tabs.value.find((item) => item.id === tabId)
    if (currentTab) Object.assign(currentTab, renamed)
    await Promise.all([refreshRecentNotes(), refreshNoteLibrary()])
    if (activeTabId.value === tabId) {
      await loadActiveNote()
      focusEditorAfterPageAction()
    }
    statusMessage.value = t.value.status.aiRenamed.replace('{title}', nextTitle)
  } catch (error) {
    const detail = displayAiRequestError(error)
    statusMessage.value = detail.includes('empty or incompatible response')
      ? t.value.ai.renameEmptyTitle
      : detail
  } finally {
    aiRenamingTabId.value = null
  }
}

async function requestAiInlineResult(session: AiInlineSession) {
  if (!session.plan) return
  const requestId = session.requestId

  try {
    const response = await requestAiText(
      session.plan.requestContext,
      [{ role: 'user', content: aiInlinePrompts.value[session.command] }],
      { searchLibrary: false, currentNoteId: session.noteId, maxTokens: 800 },
    )
    const current = aiInlineSession.value
    if (!isCurrentAiRequest(current, requestId)) return
    if (current.noteId !== activeTabId.value || !editorPane.value) {
      cancelAiInlineCommand(false)
      return
    }
    const applied = editorPane.value.applyAiInlineCommand(
      requestId,
      current.plan?.action ?? 'insert',
      current.plan?.context ?? session.plan.context,
      response.content,
    )
    if (!applied) {
      showAiInlineCommandFailure(current, t.value.ai.staleContext)
      return
    }
    statusMessage.value = current.plan?.action === 'replace' ? t.value.ai.replaced : t.value.ai.inserted
    aiInlineSession.value = null
  } catch (error) {
    const current = aiInlineSession.value
    if (!isCurrentAiRequest(current, requestId)) return
    showAiInlineCommandFailure(current, displayAiRequestError(error))
  }
}

function showAiInlineCommandFailure(session: AiInlineSession, detail: string) {
  editorPane.value?.failAiInlineCommand(session.requestId, t.value.ai.inlineFailed, detail)
  statusMessage.value = detail
  window.setTimeout(() => {
    const current = aiInlineSession.value
    if (!isCurrentAiRequest(current, session.requestId)) return
    editorPane.value?.cancelAiInlineCommand(session.requestId)
    aiInlineSession.value = null
  }, 4000)
}

function runAiInlineCommand(command: AiInlineCommandName) {
  const snapshot = editorPane.value?.captureAiSnapshot()
  if (!snapshot || !activeTab.value) return
  closeAiPanel(false)
  cancelAiInlineCommand(false)
  const plan = createAiInlinePlan(command, snapshot)
  if (!plan) {
    statusMessage.value = t.value.ai.inlineEmptyContext
    return
  }
  const requestId = ++aiInlineRequestId
  const session: AiInlineSession = {
    requestId,
    noteId: activeTab.value.id,
    command,
    plan,
  }
  aiInlineSession.value = session
  editorPane.value?.startAiInlineCommand(
    requestId,
    plan.action === 'insert' && plan.context.kind === 'selection' ? plan.context.to : snapshot.cursor,
    t.value.ai.thinking,
    plan.action === 'replace' ? plan.context : undefined,
  )
  if (!aiReady.value) {
    showAiInlineCommandFailure(session, t.value.ai.disabled)
    return
  }
  void requestAiInlineResult(session)
}

function applyAiResult(
  action: 'replace' | 'insert' | 'insertBelow',
  result: string,
  contextKind: AiContextKind,
) {
  const session = aiPanelSession.value
  if (!session || session.noteId !== activeTabId.value) return false
  const context = session.snapshot.contexts.find((item) => item.kind === contextKind)
  if (!context || !editorPane.value) return false

  const applied = action === 'replace'
    ? editorPane.value.replaceAiContext(context, result)
    : action === 'insertBelow'
      ? editorPane.value.insertAiBelow(context, result)
      : editorPane.value.insertAiAtCursor(result)
  if (applied) statusMessage.value = action === 'replace' ? t.value.ai.replaced : t.value.ai.inserted
  return applied
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

function openFinancialTools() {
  if (isLoadingNote.value || reminderDialogOpen.value || confirmationDialog.value || inputDialog.value || fontDialogOpen.value || aiPanelSession.value || aiInlineSession.value) return
  closeSearch(false)
  closeSettings(false)
  closeHelp(false)
  closeReminderList(false)
  closeArchiveList(false)
  editorPane.value?.closeEditorFind()
  financialInsertTarget.value = previewMode.value === 'preview' ? null : editorPane.value?.captureFinancialTarget(activeTabId.value) ?? null
  financialNotice.value = ''
  financialToolsOpen.value = true
}

function closeFinancialTools() {
  financialToolsOpen.value = false
  financialInsertTarget.value = null
  focusEditorAfterPageAction()
}

async function copyFinancialResult(record: boolean) {
  const text = record ? financialTools.record(language.value) : financialTools.summary(language.value)
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    financialNotice.value = financeText(language.value).copied
  } catch { financialNotice.value = financeText(language.value).copyFailed }
}

function insertFinancialResult() {
  const record = financialTools.record(language.value)
  const target = financialInsertTarget.value
  if (!record || !target) return
  if (isLoadingNote.value || previewMode.value === 'preview' || content.value !== target.documentText || !editorPane.value?.insertFinancialRecord(target, activeTabId.value, record)) {
    financialNotice.value = financeText(language.value).insertFailed
    return
  }
  closeFinancialTools()
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
    :library-resize-label="t.library.resizePane"
    :library-resize-hint="t.library.resizePaneHint"
  >
    <template #title>
      <MenuBar
        :word-wrap="wordWrap"
        :always-on-top="alwaysOnTop"
        :page-actions-enabled="Boolean(activeTab && activeTab.id !== 'inbox' && activeTab.id !== 'clipboard')"
        :active-tab-archived="Boolean(activeTab?.archived)"
        :exporting-note="exportingNote"
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
        @export-note="(format, destination) => exportNote(activeTabId, format, destination)"
        @export-all="exportAllNotes"
        @open-data-folder="openWorkspaceFolder"
        @open-trash="openTrashFolder"
        @hide-window="hideMainWindow"
        @exit-app="exitApp"
        @undo="undoEditor"
        @redo="redoEditor"
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
        @previous-tab="cycleTab(-1)"
        @next-tab="cycleTab(1)"
        @toggle-note-library="toggleNoteLibrary"
        @cycle-editor-mode="cycleEditorMode"
        @toggle-immersive-mode="toggleImmersiveMode"
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
        @financial-tools="openFinancialTools"
        @process-text="processEditorText"
        @help-topic="openHelpTopic"
      />
    </template>

    <template #tabs>
      <TabBar
        :tabs="displayTabs"
        :active-tab-id="activeTabId"
        :exporting-note="exportingNote"
        :ai-renaming-tab-id="aiRenamingTabId"
        :project-directories="projectDirectories"
        :project-menu-loading="projectMenuLoading"
        :project-menu-failed="projectMenuFailed"
        :messages="t.tabs"
        @select-tab="selectTab"
        @title-double-click="handleTabTitleDoubleClick"
        @reveal-tab="revealNoteInExplorer"
        @copy-tab-path="copyNoteFilePath"
        @export-tab="exportNote"
        @rename-tab="renamePageById"
        @ai-rename-tab="aiRenamePageById"
        @delete-tab="deletePageById"
        @close-tab="closePageById"
        @archive-tab="archivePageById"
        @request-projects="refreshProjectMenu"
        @move-tab-to-project="moveTabToProject"
        @unarchive-tab="unarchiveTab(tabs.find((tab) => tab.id === $event)!)"
        @update-tab-color="updateTabColor"
        @new-tab="createLocalTab"
        @toggle-library="toggleNoteLibrary"
        @previous-tab="cycleTab(-1)"
        @next-tab="cycleTab(1)"
        @reorder-tabs="reorderTabs"
      />
    </template>

    <template #library>
      <NoteLibrary
        v-if="noteLibraryOpen"
        :notes="libraryNotes"
        :projects="projectNotes"
        :project-directories="projectDirectories"
        @new-project-note="createLibraryProjectNote"
        @move-project-notes="moveLibraryNotesToProject"
        @move-to-capture="moveLibraryNotesToCapture"
        @archive-project="archiveLibraryProject"
        :archived-notes="archivedNotes"
        :archive-directories="archiveDirectories"
        :trashed-notes="trashedNotes"
        :active-note-id="activeTabId"
        :loading="libraryLoading"
        :messages="t.library"
        @select="selectLibraryNote"
        @restore="restoreLibraryNote"
        @restore-trash="restoreTrashedLibraryNotes"
        @clear-trash="clearLibraryTrash"
        @rename="runLibraryNoteAction($event, 'rename')"
        @archive="runLibraryNoteAction($event, 'archive')"
        @delete="runLibraryNoteAction($event, 'delete')"
        @reveal="revealNoteInExplorer($event.id)"
        @new-note="createLocalTab"
        @create-directory="createLibraryDirectory"
        @archive-to="archiveLibraryNotesTo"
        @move-directory="moveLibraryDirectory"
        @rename-directory="renameLibraryDirectory"
        @delete-directory="deleteLibraryDirectory"
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
        :search-labels="t.editorFind"
        :ai-slash-labels="aiSlashLabels"
        :context-menu-labels="{
          cut: t.menu.cut,
          copy: t.menu.copy,
          openInNewTab: t.menu.openSelectionInNewTab,
          paste: t.menu.paste,
          selectAll: t.menu.selectAll,
          aiActions: t.ai.selectionActions,
          polish: t.ai.selectionPolish,
          summarize: t.ai.selectionSummarize,
          translate: t.ai.selectionTranslate,
        }"
        @vim-mode-change="activeVimMode = $event"
        @vim-tab-switch="cycleTab"
        @ai-command="runAiInlineCommand"
        @open-selection-in-new-tab="createLocalTab"
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

    <FinancialToolsPanel
      v-if="financialToolsOpen"
      v-model:category="financeCategory"
      :language="language"
      :draft="financeDraft"
      :result="financeResult"
      :error="financeError"
      :busy="financeBusy"
      :changed="financeChanged"
      :can-insert="Boolean(financialInsertTarget) && previewMode !== 'preview'"
      :notice="financialNotice"
      @update-field="(key, value) => financeDraft[key] = value"
      @calculate="financialTools.compute"
      @reset="financialTools.reset"
      @copy="copyFinancialResult"
      @insert="insertFinancialResult"
      @close="closeFinancialTools"
    />

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
      :app-version="appVersion"
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
      :agent-tools-status="agentToolsStatus"
      :agent-tools-error="agentToolsError"
      :agent-tools-busy="agentToolsBusy"
      :ai-config="aiConfig"
      :ai-error="aiError"
      :ai-testing="aiTesting"
      :ai-test-succeeded="aiTestSucceeded"
      :initial-tab="settingsInitialTab"
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
      @repair-agent-cli="repairAgentCli"
      @copy-agent-cli-path="copyAgentCliPath"
      @open-agent-guide="openAgentGuide"
      @update-ai-config="updateAiConfig"
      @save-ai-api-key="storeApiKey"
      @clear-ai-api-key="removeApiKey"
      @test-ai-connection="checkConnection"
    />

    <AiAssistantPanel
      v-if="aiPanelSession"
      :session="aiPanelSession"
      :ready="aiReady"
      :messages="t.ai"
      :generate="generateForAiPanel"
      :apply-result="applyAiResult"
      @close="closeAiPanel"
      @configure="openAiSettings"
      @update-chat="updateAiChatState"
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
      <section class="input-dialog help-panel" :class="{ 'guide-help-panel': helpTopic !== 'about' }" role="dialog" aria-modal="true" :aria-label="helpContent.title">
        <header class="input-dialog-header">
          <strong>{{ helpContent.title }}</strong>
          <button type="button" :aria-label="t.settings.close" :title="t.settings.close" @click="closeHelp()"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg></button>
        </header>
        <div v-if="helpTopic === 'shortcuts'" class="input-dialog-body help-body reference-help-body">
          <section v-for="group in shortcutHelpGroups" :key="group.title" class="reference-help-group">
            <h3>{{ group.title }}</h3>
            <table>
              <thead>
                <tr>
                  <th scope="col">{{ language === 'zh' ? '\u5feb\u6377\u952e' : 'Shortcut' }}</th>
                  <th scope="col">{{ language === 'zh' ? '\u529f\u80fd' : 'Action' }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in group.rows" :key="row.keys">
                  <td><kbd>{{ row.keys }}</kbd></td>
                  <td>{{ row.description }}</td>
                </tr>
              </tbody>
            </table>
          </section>
        </div>
        <div v-else-if="referenceHelp" class="input-dialog-body help-body reference-help-body">
          <p class="reference-help-intro">{{ referenceHelp.intro }}</p>
          <section v-for="group in referenceHelp.groups" :key="group.title" class="reference-help-group">
            <h3>{{ group.title }}</h3>
            <table>
              <thead>
                <tr>
                  <th scope="col">{{ referenceHelp.valueLabel }}</th>
                  <th scope="col">{{ referenceHelp.descriptionLabel }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in group.rows" :key="row.value">
                  <td><code>{{ row.value }}</code></td>
                  <td>{{ row.description }}</td>
                </tr>
              </tbody>
            </table>
          </section>
        </div>
        <div v-else class="input-dialog-body help-body">
          <p
            v-for="line in helpContent.lines"
            :key="line"
            :class="{ 'help-section-title': line.startsWith('## ') }"
          >
            {{ line.startsWith('## ') ? line.slice(3) : line }}
          </p>
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

