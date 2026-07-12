export interface KeyboardState {
  modalOpen: () => boolean
  reminderDialogOpen: () => boolean
  reminderListOpen: () => boolean
  archiveListOpen: () => boolean
  confirmationOpen: () => boolean
  inputOpen: () => boolean
  fontDialogOpen: () => boolean
  immersiveMode: () => boolean
  settingsOpen: () => boolean
  helpOpen: () => boolean
  searchOpen: () => boolean
  vimMode: () => boolean
  vimUseCtrlShortcuts: () => boolean
  vimNormalMode: () => boolean
  editorFocused: () => boolean
  editableTarget: (target: EventTarget | null) => boolean
  menuOrContextOpen: () => boolean
  tabContextOpen: () => boolean
  findPanelOpen: () => boolean
  nativeRuntime: () => boolean
}

export interface KeyboardActions {
  openShortcutHelp: () => void
  cycleTab: (offset: -1 | 1) => void
  toggleTheme: () => void
  togglePreviewTheme: () => void
  openReminderList: () => void | Promise<void>
  closeReminderList: () => void
  toggleImmersiveMode: () => void
  archiveActivePage: () => void | Promise<void>
  deleteActivePage: () => void | Promise<void>
  closeReminderDialog: () => void
  closeArchiveList: () => void
  cancelConfirmation: () => void
  cancelInput: () => void
  closeFontDialog: () => void
  exitImmersiveMode: () => void | Promise<void>
  closeSettings: () => void
  closeHelp: () => void
  closeSearch: () => void
  closeEditorFind: () => void
  cycleEditorMode: () => void
  renameActivePage: () => void | Promise<void>
  toggleMainWindowMaximize: () => void | Promise<void>
  createLocalTab: () => void | Promise<void>
  closeActivePage: () => void | Promise<void>
  triggerLoadFile: () => void
  showSearch: () => void
  openFind: () => void
  openReplace: () => void
  copy: () => void
  cut: () => void
  paste: () => void
  selectAll: () => void
  findNext: () => void
  calculateExpression: () => void
  hideMainWindow: () => void | Promise<void>
  toggleNoteLibrary: () => void
  togglePin: () => void | Promise<void>
  openSettings: () => void
  insertDateTimeSeparator: () => void
  insertSeparator: () => void
  insertDateTime: () => void
  insertReminder: () => void
  saveCurrentClipboard: () => void | Promise<void>
}

export interface KeyboardShortcutContext {
  state: KeyboardState
  actions: KeyboardActions
}

export function matchesEditorModeShortcut(event: KeyboardEvent) {
  return event.key === 'F8' && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey
}

export function matchesDeletePageShortcut(event: KeyboardEvent) {
  return event.key === 'Delete' && event.altKey && !event.ctrlKey && !event.shiftKey && !event.metaKey
}

export function isEditableElement(element: EventTarget | null) {
  if (!(element instanceof HTMLElement)) return false
  return element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement ||
    element instanceof HTMLSelectElement || element.isContentEditable
}

function plainKey(event: KeyboardEvent, key: string) {
  return event.key === key && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey
}

function consume(event: KeyboardEvent, action?: () => void) {
  event.preventDefault()
  event.stopPropagation()
  action?.()
}

export function createKeyboardHandler({ state, actions }: KeyboardShortcutContext) {
  const overlaysClear = () =>
    !state.reminderListOpen() && !state.archiveListOpen() && !state.settingsOpen() &&
    !state.helpOpen() && !state.searchOpen()
  const workspaceClear = () => overlaysClear() && !state.menuOrContextOpen()

  return function handleKeydown(event: KeyboardEvent) {
    if (state.modalOpen() && event.key !== 'Escape') {
      const key = event.key.toLowerCase()
      const blockedCtrl = event.ctrlKey && !event.altKey && !event.metaKey && ['tab', 'n', 'w', 'o', ','].includes(key)
      const blockedFunction = plainKey(event, event.key) && ['F4', 'F5', 'F7', 'F8', 'F9', 'F11', 'F12'].includes(event.key)
      if (blockedCtrl || blockedFunction || matchesDeletePageShortcut(event) || matchesEditorModeShortcut(event)) consume(event)
      return
    }

    if (plainKey(event, 'F1') && (!state.editableTarget(event.target) || state.editorFocused()) && !state.menuOrContextOpen()) {
      consume(event, actions.openShortcutHelp); return
    }
    if (event.key === 'Tab' && event.ctrlKey && !event.altKey && !event.metaKey) {
      consume(event, () => actions.cycleTab(event.shiftKey ? -1 : 1)); return
    }
    if (plainKey(event, 'F9')) { consume(event, actions.toggleTheme); return }
    if (plainKey(event, 'F7')) { consume(event, actions.togglePreviewTheme); return }
    if (plainKey(event, 'F5')) {
      consume(event, () => state.reminderListOpen() ? actions.closeReminderList() : void actions.openReminderList()); return
    }
    if (plainKey(event, 'F11')) { consume(event, actions.toggleImmersiveMode); return }
    if (plainKey(event, 'F12') && workspaceClear()) { consume(event, () => void actions.archiveActivePage()); return }
    if (matchesDeletePageShortcut(event) && overlaysClear() && !state.tabContextOpen()) {
      consume(event, () => void actions.deleteActivePage()); return
    }

    if (event.key === 'Escape') {
      const escapeActions: Array<[boolean, () => void]> = [
        [state.reminderDialogOpen(), actions.closeReminderDialog],
        [state.reminderListOpen(), actions.closeReminderList],
        [state.archiveListOpen(), actions.closeArchiveList],
        [state.confirmationOpen(), actions.cancelConfirmation],
        [state.inputOpen(), actions.cancelInput],
        [state.fontDialogOpen(), actions.closeFontDialog],
        [state.immersiveMode(), () => void actions.exitImmersiveMode()],
        [state.settingsOpen(), actions.closeSettings],
        [state.helpOpen(), actions.closeHelp],
        [state.searchOpen(), actions.closeSearch],
        [state.findPanelOpen(), actions.closeEditorFind],
      ]
      const match = escapeActions.find(([open]) => open)
      if (match) { consume(event, match[1]); return }
      if (state.menuOrContextOpen()) return
    }

    if (matchesEditorModeShortcut(event)) { consume(event, actions.cycleEditorMode); return }
    if (plainKey(event, 'F2') && workspaceClear()) { consume(event, () => void actions.renameActivePage()); return }
    if (event.key === 'Enter' && event.altKey && !event.ctrlKey && !event.shiftKey && !event.metaKey) {
      consume(event, () => { if (state.nativeRuntime()) void actions.toggleMainWindowMaximize() }); return
    }

    if (event.code === 'Comma' && (event.ctrlKey || event.metaKey) && !event.altKey && !event.shiftKey) {
      consume(event, actions.openSettings); return
    }

    if (event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
      const key = event.key.toLowerCase()
      if (key === 'n') { consume(event, () => void actions.createLocalTab()); return }
      if (key === 'w') {
        if (state.tabContextOpen()) return
        consume(event, () => void actions.closeActivePage()); return
      }
      if (key === 'o') { consume(event, actions.triggerLoadFile); return }
    }

    if (state.vimMode() && state.editorFocused()) {
      const appFunctionKey = ['F3', 'F4', 'F6'].includes(event.key)
      const hideFromNormalMode = event.key === 'Escape' && state.vimNormalMode()
      const preservedCtrl = state.vimUseCtrlShortcuts() && (event.ctrlKey || event.metaKey)
      if (!appFunctionKey && !hideFromNormalMode && !preservedCtrl) return
    }

    if (state.vimMode() && state.vimUseCtrlShortcuts() && state.editorFocused() && event.ctrlKey && !event.altKey) {
      const key = event.key.toLowerCase()
      if (key === 'f') { consume(event, event.shiftKey ? actions.showSearch : actions.openFind); return }
      if (key === 'r' && !event.shiftKey) { consume(event, actions.openReplace); return }
      if (key === 'c') { event.preventDefault(); actions.copy(); return }
      if (key === 'x') { event.preventDefault(); actions.cut(); return }
      if (key === 'v' && !event.shiftKey) { event.preventDefault(); actions.paste(); return }
      if (key === 'a') { event.preventDefault(); actions.selectAll(); return }
    }

    if (!state.vimMode() && event.key.toLowerCase() === 'f' && event.ctrlKey && !event.altKey && !event.metaKey) {
      consume(event, event.shiftKey ? actions.showSearch : actions.openFind); return
    }
    if (!state.vimMode() && event.key.toLowerCase() === 'r' && event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
      consume(event, actions.openReplace); return
    }
    if (plainKey(event, 'F3')) { consume(event, actions.findNext); return }
    if (event.key === 'Enter' && event.ctrlKey) { consume(event, actions.calculateExpression); return }
    if (event.key === 'Escape' && state.nativeRuntime()) { event.preventDefault(); void actions.hideMainWindow() }
    if (event.key === 'F4') { event.preventDefault(); actions.toggleNoteLibrary() }
    if (event.key === 'F6') { event.preventDefault(); void actions.togglePin() }
    if (event.code === 'Minus' && event.ctrlKey && event.shiftKey) { event.preventDefault(); actions.insertDateTimeSeparator() }
    else if (event.code === 'Minus' && event.ctrlKey) { event.preventDefault(); actions.insertSeparator() }
    if (event.key.toLowerCase() === 'd' && event.ctrlKey) { event.preventDefault(); actions.insertDateTime() }
    if (event.key.toLowerCase() === 'e' && event.ctrlKey) { event.preventDefault(); actions.insertReminder() }
    if (event.key.toLowerCase() === 'v' && event.ctrlKey && event.shiftKey) { event.preventDefault(); void actions.saveCurrentClipboard() }
  }
}
