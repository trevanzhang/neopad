import type { EditorSearchLabels } from './editor-search-panel'

export type AppLanguage = 'en' | 'zh'

export type AppMessages = {
  tabs: {
    inbox: string
    clipboard: string
    untitled: string
    rename: string
    delete: string
    archive: string
    close: string
    color: string
    defaultColor: string
    previous: string
    next: string
    library: string
    revealInFileManager: string
    copyFilePath: string
    exportAsPng: string
    exportAsPdf: string
    f8: string
    altDel: string
    f12: string
    ctrlW: string
    confirmDeleteTitle: string
    confirmDeleteMessage: string
    confirmArchiveMessage: string
  }
  editorFind: EditorSearchLabels
  menu: {
    file: string
    edit: string
    view: string
    page: string
    format: string
    insert: string
    tools: string
    help: string
    loadFromFile: string
    recentDocuments: string
    saveAsFile: string
    exportCurrentNote: string
    exportAsPng: string
    exportAsPdf: string
    exportAll: string
    revealArchive: string
    trash: string
    hide: string
    exit: string
    esc: string
    newPage: string
    saveClipboard: string
    undo: string
    redo: string
    cut: string
    copy: string
    paste: string
    find: string
    findNext: string
    replace: string
    globalSearch: string
    selectAll: string
    ctrlZ: string
    ctrlX: string
    ctrlC: string
    ctrlV: string
    ctrlF: string
    ctrlG: string
    f3: string
    ctrlR: string
    ctrlShiftF: string
    ctrlA: string
    ctrlN: string
    f2: string
    altDel: string
    f12: string
    ctrlO: string
    openNoteBrowser: string
    f4: string
    font: string
    backgroundColor: string
    togglePreviewTheme: string
    toggleTheme: string
    wordWrap: string
    ctrlW: string
    previousTab: string
    nextTab: string
    switchEditorMode: string
    cycleEditorMode: string
    editMode: string
    splitMode: string
    previewMode: string
    alwaysOnTop: string
    renamePage: string
    deletePage: string
    archivePage: string
    unarchivePage: string
    closePage: string
    plainText: string
    markdown: string
    date: string
    separator: string
    insertSeparator: string
    dateTime: string
    dateTimeSeparator: string
    reminder: string
    insertTextSettings: string
    ctrlDash: string
    ctrlD: string
    ctrlShiftDash: string
    ctrlE: string
    keepOnTop: string
    f6: string
    f7: string
    f9: string
    f10: string
    windowOpacity: string
    textProcessing: string
    uppercase: string
    lowercase: string
    removeExtraSpaces: string
    trimLeadingSpaces: string
    removeEmptyLines: string
    removeDuplicateEmptyLines: string
    sortLines: string
    uniqueLines: string
    toSimplifiedChinese: string
    toTraditionalChinese: string
    toHalfWidth: string
    toFullWidth: string
    addLineNumbers: string
    removeLineNumbers: string
    urlEncode: string
    urlDecode: string
    base64Encode: string
    base64Decode: string
    md5Hash: string
    sha1Hash: string
    sha256Hash: string
    reminderList: string
    f5: string
    settingsWithKey: string
    f8: string
    ctrlComma: string
    search: string
    settings: string
    f1: string
    softwareHelp: string
    markdownGuide: string
    shortcutList: string
    expressionGuide: string
    about: string
  }
  settings: {
    title: string
    close: string
    general: string
    previewTab: string
    shortcutsTab: string
    insertTextTab: string
    advancedTab: string
    about: string
    aboutDescription: string
    version: string
    author: string
    openSource: string
    license: string
    builtWith: string
    generalOptions: string
    vimMode: string
    vimUseCtrlShortcuts: string
    vimModeDescription: string
    vimSettings: string
    vimInsertExitKey: string
    vimModeHint: string
    runAtStartup: string
    startHidden: string
    closeToMinimize: string
    snapToEdges: string
    windowOpacity: string
    enableTransparency: string
    titleDoubleClick: string
    noAction: string
    deletePage: string
    renameTitle: string
    baseKey: string
    modifiers: string
    separatorText: string
    dateTimeText: string
    dateTimeSeparatorText: string
    custom: string
    add: string
    edit: string
    delete: string
    ok: string
    cancel: string
    language: string
    english: string
    chinese: string
    alwaysOnTop: string
    theme: string
    system: string
    light: string
    dark: string
    shortcuts: string
    globalShortcuts: string
    globalShortcutHint: string
    applicationShortcuts: string
    applicationShortcutHint: string
    toggleWindow: string
    saveClipboard: string
    hideWindow: string
    editor: string
    defaultMode: string
    previewAppearance: string
    editorFont: string
    editorFontSize: string
    fontSample: string
    previewTheme: string
    previewThemeLight: string
    previewThemeOneDark: string
    previewThemeNord: string
    previewThemeSolarizedLight: string
    previewThemeSolarizedDark: string
    previewThemeMonokai: string
    previewThemeGitHubLight: string
    previewThemeDracula: string
    previewFont: string
    previewFontEditor: string
    previewFontSystem: string
    previewFontSerif: string
    previewFontMono: string
    previewFontSize: string
    previewLineHeight: string
    previewLineCompact: string
    previewLineStandard: string
    previewLineRelaxed: string
    previewContentWidth: string
    previewWidthCompact: string
    previewWidthStandard: string
    previewWidthWide: string
    cycleEditorMode: string
    togglePreviewThemeShortcut: string
    toggleThemeShortcut: string
    immersiveFullscreen: string
    switchTabs: string
    shortcut: string
    disabled: string
    mcp: string
    workspace: string
    copyReadOnlyConfig: string
    copyWriteConfig: string
    mcpLocalService: string
    mcpDescription: string
    mcpStartupDescription: string
    enableMcp: string
    address: string
    status: string
    stopped: string
    startMcpService: string
    stopMcpService: string
    accessToken: string
    tokenPending: string
    copyAgentConfig: string
    regenerateToken: string
    installMethod: string
    installMethodDescription: string
  }
  search: {
    title: string
    placeholder: string
    close: string
    searching: string
    noResults: string
    line: string
    summary: string
    matchCount: string
    showMoreInNote: string
    collapse: string
    loadMore: string
  }
  reminders: {
    title: string
    createTitle: string
    contentLabel: string
    dateLabel: string
    timeLabel: string
    insert: string
    cancel: string
    close: string
    refresh: string
    empty: string
    status: string
    dueAt: string
    content: string
    page: string
    pending: string
    due: string
    completed: string
    actions: string
    complete: string
    reopen: string
    completeAllDue: string
    filterLabel: string
    filterAll: string
    filterPending: string
    filterDue: string
    filterCompleted: string
    notificationTitle: string
  }
  archive: {
    title: string
    close: string
    refresh: string
    empty: string
    restore: string
  }
  library: {
    title: string
    notes: string
    archive: string
    trash: string
    emptyNotes: string
    emptyArchive: string
    emptyTrash: string
    newNote: string
    refresh: string
    restore: string
    rename: string
    archiveAction: string
    delete: string
    clearTrash: string
    clearTrashTitle: string
    clearTrashMessage: string
    clearTrashConfirm: string
    revealInFileManager: string
    help: string
  }
  recovery: {
    title: string
    message: string
    restore: string
    restored: string
  }
  status: {
    editMode: string
    hybridMode: string
    previewMode: string
    saved: string
    saving: string
    failed: string
    markdown: string
    clipboard: string
    clipboardSaved: string
    loadedFromFile: string
    savedAsFile: string
    exported: string
    exportingNote: string
    exportedPng: string
    exportedPdf: string
    exportTooLong: string
    exportFailed: string
    archiveOpened: string
    notePathCopied: string
    trashOpened: string
    noteFileMissing: string
    fontUpdated: string
    backgroundUpdated: string
    wordWrapOn: string
    wordWrapOff: string
    inserted: string
    textProcessed: string
    opacityUpdated: string
    unsupportedHash: string
    expressionCalculated: string
    expressionNotFound: string
    search: string
    settings: string
    alwaysOnTop: string
    pinned: string
    unpinned: string
    mcpConfigCopied: string
    mcpUpdated: string
    chars: string
    switchToLight: string
    switchToDark: string
  }
}

export const messages: Record<AppLanguage, AppMessages> = {
  en: {
    tabs: {
      inbox: 'Inbox',
      clipboard: 'Clipboard',
      untitled: 'Untitled',
      rename: 'Rename',
      delete: 'Delete',
      archive: 'Archive',
      close: 'Close',
      color: 'Tab color',
      defaultColor: 'Default',
      previous: 'Previous tab',
      next: 'Next tab',
      library: 'Open note browser',
      revealInFileManager: 'Show in Explorer',
      copyFilePath: 'Copy Page File Path',
      exportAsPng: 'Export as PNG...',
      exportAsPdf: 'Export as PDF...',
      f8: 'F8',
      altDel: 'Alt+Del',
      f12: 'F12',
      ctrlW: 'Ctrl+W',
      confirmDeleteTitle: 'Delete tab?',
      confirmDeleteMessage: 'Delete "{title}"? The note will be moved to Trash.',
      confirmArchiveMessage: 'Archive "{title}"? The note will be moved to Archive.',
    },
    editorFind: {
      findPlaceholder: 'Find in current note',
      previous: 'Previous match',
      next: 'Next match',
      selectAllMatches: 'Select all matches',
      selectAll: 'All',
      caseSensitive: 'Match case',
      regexp: 'Use regular expression',
      wholeWord: 'Match whole word',
      showReplace: 'Show replace',
      hideReplace: 'Hide replace',
      replace: 'Replace',
      close: 'Close find',
      replacePlaceholder: 'Replace with',
      replaceCurrent: 'Replace current match',
      replaceAllMatches: 'Replace all matches',
      replaceAll: 'Replace all',
      noResults: 'No results',
    },
    menu: {
      file: 'File(F)',
      edit: 'Edit(E)',
      view: 'View(V)',
      page: 'Page(P)',
      format: 'Format(O)',
      insert: 'Insert(I)',
      tools: 'Tools(T)',
      help: 'Help(H)',
      loadFromFile: 'Load from File(L)',
      recentDocuments: 'Recent Documents',
      saveAsFile: 'Save As File(S)',
      exportCurrentNote: 'Export Current Note',
      exportAsPng: 'As PNG Image...',
      exportAsPdf: 'As PDF Document...',
      exportAll: 'Export All(E)...',
      revealArchive: 'Open Archive Folder(A)',
      trash: 'Trash(R)...',
      hide: 'Hide(H)',
      exit: 'Exit(X)',
      esc: 'Esc',
      newPage: 'New Page',
      saveClipboard: 'Save Clipboard',
      undo: 'Undo',
      redo: 'Redo',
      cut: 'Cut(X)',
      copy: 'Copy(C)',
      paste: 'Paste(P)',
      find: 'Find(F)...',
      findNext: 'Find Next(N)',
      replace: 'Replace(R)...',
      globalSearch: 'Global Search(G)',
      selectAll: 'Select All(A)',
      ctrlZ: 'Ctrl+Z',
      ctrlX: 'Ctrl+X',
      ctrlC: 'Ctrl+C',
      ctrlV: 'Ctrl+V',
      ctrlF: 'Ctrl+F',
      ctrlG: 'Ctrl+G',
      f3: 'F3',
      ctrlR: 'Ctrl+R',
      ctrlShiftF: 'Ctrl+Shift+F',
      ctrlA: 'Ctrl+A',
      ctrlN: 'Ctrl+N',
      f2: 'F2',
      altDel: 'Alt+Del',
      f12: 'F12',
      ctrlO: 'Ctrl+O',
      openNoteBrowser: 'Open Note Browser',
      f4: 'F4',
      font: 'Font(F)...',
      backgroundColor: 'Background Color(B)...',
      togglePreviewTheme: 'Toggle Preview Theme',
      toggleTheme: 'Toggle Day/Night Mode',
      wordWrap: 'Word Wrap(W)',
      ctrlW: 'Ctrl+W',
      previousTab: 'Previous Tab',
      nextTab: 'Next Tab',
      switchEditorMode: 'Switch Editor Mode',
      cycleEditorMode: 'Cycle Editor Mode(C)',
      editMode: 'Edit',
      splitMode: 'Hybrid',
      previewMode: 'Preview',
      alwaysOnTop: 'Always on Top',
      renamePage: 'Rename Page',
      deletePage: 'Delete Page',
      archivePage: 'Archive Page',
      unarchivePage: 'Restore from Archive',
      closePage: 'Close Tab',
      plainText: 'Plain Text',
      markdown: 'Markdown',
      date: 'Date',
      separator: 'Separator',
      insertSeparator: 'Separator(L)',
      dateTime: 'Date Time(D)',
      dateTimeSeparator: 'Date Time Separator(B)',
      reminder: 'Reminder(R)',
      insertTextSettings: 'Insert Text Settings(C)...',
      ctrlDash: 'Ctrl+-',
      ctrlD: 'Ctrl+D',
      ctrlShiftDash: 'Ctrl+Shift+-',
      ctrlE: 'Ctrl+E',
      keepOnTop: 'Keep Window on Top(T)',
      f6: 'F6',
      f7: 'F7',
      f9: 'F9',
      f10: 'F10',
      windowOpacity: 'Window Opacity(W)...',
      textProcessing: 'Text Processing(P)',
      uppercase: 'To Uppercase(U)',
      lowercase: 'To Lowercase(L)',
      removeExtraSpaces: 'Remove Extra Spaces(S)',
      trimLeadingSpaces: 'Trim Leading Spaces(T)',
      removeEmptyLines: 'Remove Empty Lines(R)',
      removeDuplicateEmptyLines: 'Remove Duplicate Empty Lines(M)',
      sortLines: 'Sort Lines(O)',
      uniqueLines: 'Remove Duplicates(D)',
      toSimplifiedChinese: 'To Simplified Chinese(J)',
      toTraditionalChinese: 'To Traditional Chinese(F)',
      toHalfWidth: 'To Half Width(H)',
      toFullWidth: 'To Full Width(W)',
      addLineNumbers: 'Add Line Numbers(N)',
      removeLineNumbers: 'Remove Line Numbers(I)',
      urlEncode: 'URL Encode(E)',
      urlDecode: 'URL Decode(C)',
      base64Encode: 'Base64 Encode(B)',
      base64Decode: 'Base64 Decode(A)',
      md5Hash: 'MD5 Hash',
      sha1Hash: 'SHA1 Hash',
      sha256Hash: 'SHA256 Hash',
      reminderList: 'Reminder List(R)...',
      f5: 'F5',
      settingsWithKey: 'Settings(O)...',
      f8: 'F8',
      ctrlComma: 'Ctrl+,',
      search: 'Search',
      settings: 'Settings',
      f1: 'F1',
      softwareHelp: 'Software Help(R)...',
      markdownGuide: 'Markdown Quick Guide(M)...',
      shortcutList: 'Shortcut List(S)...',
      expressionGuide: 'Expression Guide(E)...',
      about: 'About NeoPad',
    },
    settings: {
      title: 'Settings',
      close: 'Close',
      general: 'General',
      previewTab: 'Preview',
      shortcutsTab: 'Shortcuts',
      insertTextTab: 'Insert Text',
      advancedTab: 'Vim',
      about: 'About',
      aboutDescription: 'A lightweight, local-first Markdown desktop note pad.',
      version: 'Version',
      author: 'Author',
      openSource: 'Open source',
      license: 'License',
      builtWith: 'Built with',
      generalOptions: 'General Options',
      vimMode: 'Enable Vim key bindings',
      vimUseCtrlShortcuts: 'Keep NeoPad Ctrl shortcuts enabled',
      vimModeDescription: 'Vim mode adds Normal, Insert, and Visual editing. NeoPad shortcuts can take priority over conflicting Vim Ctrl mappings.',
      vimSettings: 'Vim editing',
      vimInsertExitKey: 'Insert mode exit sequence',
      vimModeHint: 'The custom sequence exits Insert mode without replacing Esc. Leave it empty to disable the extra mapping.',
      runAtStartup: 'Run automatically at system startup',
      startHidden: 'Keep the window hidden after launch',
      closeToMinimize: 'Minimize when clicking the main window close button',
      snapToEdges: 'Snap main window to screen edges',
      windowOpacity: 'Window Opacity',
      enableTransparency: 'Enable window transparency',
      titleDoubleClick: 'When double-clicking a page title',
      noAction: 'No action',
      deletePage: 'Delete page',
      renameTitle: 'Rename title',
      baseKey: 'Base key',
      modifiers: 'Modifiers',
      separatorText: 'Separator',
      dateTimeText: 'Date time',
      dateTimeSeparatorText: 'Date time separator',
      custom: 'Custom',
      add: 'Add',
      edit: 'Edit',
      delete: 'Delete',
      ok: 'OK',
      cancel: 'Cancel',
      language: 'Language',
      english: 'English',
      chinese: 'Chinese',
      alwaysOnTop: 'Always on top',
      theme: 'Theme',
      system: 'System',
      light: 'Light',
      dark: 'Dark',
      shortcuts: 'Shortcuts',
      globalShortcuts: 'Global shortcuts',
      globalShortcutHint: 'Available even when NeoPad is not focused.',
      applicationShortcuts: 'Application shortcuts',
      applicationShortcutHint: 'Available while the NeoPad window is active.',
      toggleWindow: 'Toggle window',
      saveClipboard: 'Save clipboard',
      hideWindow: 'Hide window',
      editor: 'Editor',
      defaultMode: 'Default mode',
      previewAppearance: 'Preview Appearance',
      editorFont: 'Font Family',
      editorFontSize: 'Size',
      fontSample: 'NeoPad Markdown 123',
      previewTheme: 'Preview theme',
      previewThemeLight: 'Light (Default)',
      previewThemeOneDark: 'Dark (One Dark)',
      previewThemeNord: 'Nord',
      previewThemeSolarizedLight: 'Solarized Light',
      previewThemeSolarizedDark: 'Solarized Dark',
      previewThemeMonokai: 'Monokai',
      previewThemeGitHubLight: 'GitHub Light',
      previewThemeDracula: 'Dracula',
      previewFont: 'Preview font',
      previewFontEditor: 'Follow editor',
      previewFontSystem: 'System sans',
      previewFontSerif: 'Serif',
      previewFontMono: 'Monospace',
      previewFontSize: 'Font size',
      previewLineHeight: 'Line height',
      previewLineCompact: 'Compact',
      previewLineStandard: 'Standard',
      previewLineRelaxed: 'Relaxed',
      previewContentWidth: 'Content width',
      previewWidthCompact: 'Compact',
      previewWidthStandard: 'Standard',
      previewWidthWide: 'Wide',
      cycleEditorMode: 'Cycle editor mode',
      togglePreviewThemeShortcut: 'Toggle preview theme',
      toggleThemeShortcut: 'Toggle day/night mode',
      immersiveFullscreen: 'Immersive fullscreen',
      switchTabs: 'Switch tabs',
      shortcut: 'Shortcut',
      disabled: 'Disabled',
      mcp: 'MCP',
      workspace: 'Workspace',
      copyReadOnlyConfig: 'Copy read-only config',
      copyWriteConfig: 'Copy write config',
      mcpLocalService: 'MCP Local HTTP Service',
      mcpDescription: 'Off by default. When enabled, local agents with the access token can read and write NeoPad notes.',
      mcpStartupDescription: 'To run MCP after sign-in, enable NeoPad startup and start hidden in General, then leave this service enabled.',
      enableMcp: 'Enable MCP',
      address: 'Address',
      status: 'Status',
      stopped: 'Stopped',
      startMcpService: 'Start MCP Service',
      stopMcpService: 'Stop MCP Service',
      accessToken: 'Token',
      tokenPending: 'Generated after first use',
      copyAgentConfig: 'Copy Agent Config',
      regenerateToken: 'Regenerate Token',
      installMethod: 'Agent setup',
      installMethodDescription: 'Use this configuration in an MCP client that supports Streamable HTTP.',
    },
    search: {
      title: 'Search',
      placeholder: 'Search notes',
      close: 'Close',
      searching: 'Searching...',
      noResults: 'No results',
      line: 'line',
      summary: '{notes} notes, {matches} matching lines',
      matchCount: '{count} matches',
      showMoreInNote: 'Show {count} more matches',
      collapse: 'Show fewer matches',
      loadMore: 'Load more results',
    },
    reminders: {
      title: 'Reminder List',
      createTitle: 'Create Reminder',
      contentLabel: 'Reminder',
      dateLabel: 'Date',
      timeLabel: 'Time',
      insert: 'Insert',
      cancel: 'Cancel',
      close: 'Close',
      refresh: 'Refresh',
      empty: 'No reminders',
      status: 'Status',
      dueAt: 'Due at',
      content: 'Reminder',
      page: 'Page',
      pending: 'Pending',
      due: 'Due',
      completed: 'Completed',
      actions: 'Actions',
      complete: 'Mark Completed',
      reopen: 'Mark Open',
      completeAllDue: 'Clear Due',
      filterLabel: 'Show',
      filterAll: 'All',
      filterPending: 'Pending only',
      filterDue: 'Due only',
      filterCompleted: 'Completed only',
      notificationTitle: 'NeoPad Reminder',
    },
    archive: {
      title: 'Archive',
      close: 'Close',
      refresh: 'Refresh',
      empty: 'No archived pages',
      restore: 'Restore',
    },
    library: {
      title: 'FILES',
      notes: 'Notes',
      archive: 'Archive',
      trash: 'Trash',
      emptyNotes: 'No notes yet',
      emptyArchive: 'No archived notes',
      emptyTrash: 'Trash is empty',
      newNote: 'New note',
      refresh: 'Refresh library',
      restore: 'Restore note',
      rename: 'Rename',
      archiveAction: 'Archive',
      delete: 'Delete',
      clearTrash: 'Empty Trash',
      clearTrashTitle: 'Empty Trash?',
      clearTrashMessage: 'Move all notes in NeoPad Trash to the system Recycle Bin or Trash? You can restore them there.',
      clearTrashConfirm: 'Empty Trash',
      revealInFileManager: 'Show in Explorer',
      help: 'Browse local notes, archived notes, and Trash. Ctrl-click selects individual notes; Shift-click selects a range. Right-click to manage the selection.',
    },
    recovery: {
      title: 'Recover unsaved note?',
      message: 'NeoPad found an unfinished save for {fileName}. Restore the preserved content?',
      restore: 'Restore',
      restored: 'Recovered unsaved content',
    },
    status: {
      editMode: 'Edit mode',
      hybridMode: 'Hybrid mode',
      previewMode: 'Preview mode',
      saved: 'Saved',
      saving: 'Saving',
      failed: 'Failed',
      markdown: 'Markdown',
      clipboard: 'Clipboard',
      clipboardSaved: 'Clipboard saved',
      loadedFromFile: 'Loaded from file',
      savedAsFile: 'Saved as file',
      exported: 'Exported',
      exportingNote: 'Exporting note...',
      exportedPng: 'PNG exported',
      exportedPdf: 'PDF exported',
      exportTooLong: 'This note is too long to export safely',
      exportFailed: 'Note export failed',
      archiveOpened: 'Archive opened in Explorer',
      notePathCopied: 'Page file path copied',
      trashOpened: 'Trash opened',
      noteFileMissing: 'Note file does not exist',
      fontUpdated: 'Font updated',
      backgroundUpdated: 'Background color updated',
      wordWrapOn: 'Word wrap on',
      wordWrapOff: 'Word wrap off',
      inserted: 'Inserted',
      textProcessed: 'Text processed',
      opacityUpdated: 'Window opacity updated',
      unsupportedHash: 'Hash algorithm is not available',
      expressionCalculated: 'Expression calculated',
      expressionNotFound: 'No expression found',
      search: 'Search',
      settings: 'Settings',
      alwaysOnTop: 'Always on top',
      pinned: 'Pinned',
      unpinned: 'Unpinned',
      mcpConfigCopied: 'MCP agent config copied',
      mcpUpdated: 'MCP settings updated',
      chars: 'chars',
      switchToLight: 'Switch to light theme',
      switchToDark: 'Switch to dark theme',
    },
  },
  zh: {
    tabs: {
      inbox: '\u9ed8\u8ba4',
      clipboard: '\u526a\u8d34\u677f',
      untitled: '\u672a\u547d\u540d',
      rename: '\u91cd\u547d\u540d',
      delete: '\u5220\u9664',
      archive: '\u5f52\u6863',
      close: '\u5173\u95ed',
      color: '\u4fee\u6539\u6807\u7b7e\u9875\u989c\u8272',
      defaultColor: '\u9ed8\u8ba4\u989c\u8272',
      previous: '\u4e0a\u4e00\u4e2a\u6807\u7b7e\u9875',
      next: '\u4e0b\u4e00\u4e2a\u6807\u7b7e\u9875',
      library: '\u6253\u5f00\u7b14\u8bb0\u6d4f\u89c8\u5668',
      revealInFileManager: '\u5728\u8d44\u6e90\u7ba1\u7406\u5668\u4e2d\u663e\u793a',
      copyFilePath: '\u590d\u5236\u5f53\u524d\u9875\u9762\u6587\u4ef6\u5730\u5740',
      exportAsPng: '\u5bfc\u51fa\u4e3a PNG...',
      exportAsPdf: '\u5bfc\u51fa\u4e3a PDF...',
      f8: 'F8',
      altDel: 'Alt+Del',
      f12: 'F12',
      ctrlW: 'Ctrl+W',
      confirmDeleteTitle: '\u5220\u9664\u6807\u7b7e\u9875\uff1f',
      confirmDeleteMessage: '\u786e\u5b9a\u5220\u9664\u201c{title}\u201d\u5417\uff1f\u7b14\u8bb0\u5c06\u79fb\u81f3\u56de\u6536\u7ad9\u3002',
      confirmArchiveMessage: '\u786e\u5b9a\u5f52\u6863\u201c{title}\u201d\u5417\uff1f\u7b14\u8bb0\u5c06\u79fb\u81f3\u5b58\u6863\u3002',
    },
    editorFind: {
      findPlaceholder: '\u67e5\u627e\u5f53\u524d\u7b14\u8bb0',
      previous: '\u4e0a\u4e00\u4e2a',
      next: '\u4e0b\u4e00\u4e2a',
      selectAllMatches: '\u9009\u62e9\u5168\u90e8\u5339\u914d',
      selectAll: '\u5168\u90e8',
      caseSensitive: '\u533a\u5206\u5927\u5c0f\u5199',
      regexp: '\u4f7f\u7528\u6b63\u5219\u8868\u8fbe\u5f0f',
      wholeWord: '\u5168\u8bcd\u5339\u914d',
      showReplace: '\u663e\u793a\u66ff\u6362',
      hideReplace: '\u9690\u85cf\u66ff\u6362',
      replace: '\u66ff\u6362',
      close: '\u5173\u95ed\u67e5\u627e',
      replacePlaceholder: '\u66ff\u6362\u4e3a',
      replaceCurrent: '\u66ff\u6362\u5f53\u524d\u5339\u914d',
      replaceAllMatches: '\u66ff\u6362\u5168\u90e8\u5339\u914d',
      replaceAll: '\u5168\u90e8\u66ff\u6362',
      noResults: '\u65e0\u7ed3\u679c',
    },
    menu: {
      file: '\u6587\u4ef6(F)',
      edit: '\u7f16\u8f91(E)',
      view: '\u89c6\u56fe(V)',
      page: '\u9875\u9762(P)',
      format: '\u683c\u5f0f(O)',
      insert: '\u63d2\u5165(I)',
      tools: '\u5de5\u5177(T)',
      help: '\u5e2e\u52a9(H)',
      loadFromFile: '\u4ece\u6587\u4ef6\u8f7d\u5165(L)',
      recentDocuments: '\u6700\u8fd1\u6253\u5f00\u7684\u6587\u6863',
      saveAsFile: '\u53e6\u5b58\u5230\u6587\u4ef6(S)',
      exportCurrentNote: '\u5bfc\u51fa\u5f53\u524d\u7b14\u8bb0',
      exportAsPng: '\u5bfc\u51fa\u4e3a PNG \u56fe\u7247...',
      exportAsPdf: '\u5bfc\u51fa\u4e3a PDF \u6587\u4ef6...',
      exportAll: '\u5168\u90e8\u5bfc\u51fa(E)...',
      revealArchive: '\u672c\u5730\u67e5\u770b\u5b58\u6863(A)',
      trash: '\u56de\u6536\u7ad9(R)...',
      hide: '\u9690\u85cf(H)',
      exit: '\u9000\u51fa(X)',
      esc: 'Esc',
      newPage: '\u65b0\u5efa\u9875\u9762',
      saveClipboard: '\u4fdd\u5b58\u526a\u8d34\u677f',
      undo: '\u64a4\u9500',
      redo: '\u91cd\u505a',
      cut: '\u526a\u5207(X)',
      copy: '\u590d\u5236(C)',
      paste: '\u7c98\u8d34(P)',
      find: '\u67e5\u627e(F)...',
      findNext: '\u67e5\u627e\u4e0b\u4e00\u4e2a(N)',
      replace: '\u66ff\u6362(R)...',
      globalSearch: '\u5168\u5c40\u641c\u7d22(G)',
      selectAll: '\u5168\u9009(A)',
      ctrlZ: 'Ctrl+Z',
      ctrlX: 'Ctrl+X',
      ctrlC: 'Ctrl+C',
      ctrlV: 'Ctrl+V',
      ctrlF: 'Ctrl+F',
      ctrlG: 'Ctrl+G',
      f3: 'F3',
      ctrlR: 'Ctrl+R',
      ctrlShiftF: 'Ctrl+Shift+F',
      ctrlA: 'Ctrl+A',
      ctrlN: 'Ctrl+N',
      f2: 'F2',
      altDel: 'Alt+Del',
      f12: 'F12',
      ctrlO: 'Ctrl+O',
      openNoteBrowser: '\u6253\u5f00\u7b14\u8bb0\u6d4f\u89c8\u5668',
      f4: 'F4',
      font: '\u5b57\u4f53(F)...',
      backgroundColor: '\u80cc\u666f\u8272(B)...',
      togglePreviewTheme: '\u5207\u6362\u9884\u89c8\u4e3b\u9898',
      toggleTheme: '\u5207\u6362\u65e5\u95f4/\u591c\u95f4\u6a21\u5f0f',
      wordWrap: '\u81ea\u52a8\u6362\u884c(W)',
      ctrlW: 'Ctrl+W',
      previousTab: '\u4e0a\u4e00\u4e2a\u6807\u7b7e\u9875',
      nextTab: '\u4e0b\u4e00\u4e2a\u6807\u7b7e\u9875',
      switchEditorMode: '\u5207\u6362\u7f16\u8f91\u5668\u6a21\u5f0f',
      cycleEditorMode: '\u5faa\u73af\u5207\u6362\u7f16\u8f91\u5668\u6a21\u5f0f(C)',
      editMode: '\u7f16\u8f91',
      splitMode: '\u6df7\u5408',
      previewMode: '\u9884\u89c8',
      alwaysOnTop: '\u7a97\u53e3\u7f6e\u9876',
      renamePage: '\u91cd\u547d\u540d\u9875\u9762',
      deletePage: '\u5220\u9664\u9875\u9762',
      archivePage: '\u5f52\u6863\u9875\u9762',
      unarchivePage: '\u4ece\u5f52\u6863\u4e2d\u6062\u590d',
      closePage: '\u5173\u95ed\u6807\u7b7e\u9875',
      plainText: '\u7eaf\u6587\u672c',
      markdown: 'Markdown',
      date: '\u65e5\u671f',
      separator: '\u5206\u9694\u7ebf',
      insertSeparator: '\u5206\u9694\u884c(L)',
      dateTime: '\u65e5\u671f\u65f6\u95f4(D)',
      dateTimeSeparator: '\u65e5\u671f\u65f6\u95f4\u5206\u9694\u884c(B)',
      reminder: '\u63d0\u9192(R)',
      insertTextSettings: '\u63d2\u5165\u6587\u672c\u8bbe\u7f6e(C)...',
      ctrlDash: 'Ctrl+-',
      ctrlD: 'Ctrl+D',
      ctrlShiftDash: 'Ctrl+Shift+-',
      ctrlE: 'Ctrl+E',
      keepOnTop: '\u7a97\u53e3\u7f6e\u9876(T)',
      f6: 'F6',
      f7: 'F7',
      f9: 'F9',
      f10: 'F10',
      windowOpacity: '\u7a97\u53e3\u900f\u660e\u5ea6(W)...',
      textProcessing: '\u6587\u672c\u5904\u7406(P)',
      uppercase: '\u8f6c\u5927\u5199(U)',
      lowercase: '\u8f6c\u5c0f\u5199(L)',
      removeExtraSpaces: '\u53bb\u9664\u591a\u4f59\u7a7a\u683c(S)',
      trimLeadingSpaces: '\u9996\u5c3e\u53bb\u7a7a\u683c(T)',
      removeEmptyLines: '\u53bb\u7a7a\u884c(R)',
      removeDuplicateEmptyLines: '\u53bb\u91cd\u590d\u7a7a\u884c(M)',
      sortLines: '\u6309\u884c\u6392\u5e8f(O)',
      uniqueLines: '\u53bb\u91cd(D)',
      toSimplifiedChinese: '\u8f6c\u7b80\u4f53\u4e2d\u6587(J)',
      toTraditionalChinese: '\u8f6c\u7e41\u4f53\u4e2d\u6587(F)',
      toHalfWidth: '\u8f6c\u534a\u89d2(H)',
      toFullWidth: '\u8f6c\u5168\u89d2(W)',
      addLineNumbers: '\u6dfb\u52a0\u884c\u53f7(N)',
      removeLineNumbers: '\u79fb\u9664\u884c\u53f7(I)',
      urlEncode: 'URL \u7f16\u7801(E)',
      urlDecode: 'URL \u89e3\u7801(C)',
      base64Encode: 'Base64 \u7f16\u7801(B)',
      base64Decode: 'Base64 \u89e3\u7801(A)',
      md5Hash: 'MD5 Hash',
      sha1Hash: 'SHA1 Hash',
      sha256Hash: 'SHA256 Hash',
      reminderList: '\u63d0\u9192\u5217\u8868(R)...',
      f5: 'F5',
      settingsWithKey: '\u8bbe\u7f6e(O)...',
      f8: 'F8',
      ctrlComma: 'Ctrl+,',
      search: '\u641c\u7d22',
      settings: '\u8bbe\u7f6e',
      f1: 'F1',
      softwareHelp: '\u8f6f\u4ef6\u8bf4\u660e(R)...',
      markdownGuide: 'Markdown \u7b80\u660e\u6307\u5357(M)...',
      shortcutList: '\u5feb\u6377\u952e\u5217\u8868(S)...',
      expressionGuide: '\u8868\u8fbe\u5f0f\u8ba1\u7b97\u6307\u5357(E)...',
      about: '\u5173\u4e8e NeoPad',
    },
    settings: {
      title: '\u8bbe\u7f6e',
      close: '\u5173\u95ed',
      general: '\u5e38\u89c4',
      previewTab: '\u9884\u89c8',
      shortcutsTab: '\u5feb\u6377\u952e',
      insertTextTab: '\u63d2\u5165\u6587\u672c',
      advancedTab: 'Vim \u6a21\u5f0f',
      about: '\u5173\u4e8e',
      aboutDescription: '\u8f7b\u91cf\u3001\u672c\u5730\u4f18\u5148\u7684 Markdown \u684c\u9762\u4fbf\u7b7a\u3002',
      version: '\u7248\u672c',
      author: '\u4f5c\u8005',
      openSource: '\u5f00\u6e90\u9879\u76ee',
      license: '\u5f00\u6e90\u8bb8\u53ef',
      builtWith: '\u6280\u672f\u6808',
      generalOptions: '\u5e38\u89c4\u9009\u9879',
      vimMode: '\u542f\u7528 Vim \u952e\u4f4d',
      vimUseCtrlShortcuts: '\u4fdd\u7559 NeoPad \u7684 Ctrl \u5feb\u6377\u952e',
      vimModeDescription: 'Vim \u6a21\u5f0f\u63d0\u4f9b Normal\u3001Insert \u548c Visual \u7f16\u8f91\u3002\u5f00\u542f\u4e0b\u65b9\u9009\u9879\u540e\uff0cNeoPad \u5feb\u6377\u952e\u4f18\u5148\u4e8e\u51b2\u7a81\u7684 Vim Ctrl \u6620\u5c04\u3002',
      vimSettings: 'Vim \u7f16\u8f91',
      vimInsertExitKey: 'Insert \u6a21\u5f0f\u9000\u51fa\u5e8f\u5217',
      vimModeHint: '\u81ea\u5b9a\u4e49\u5e8f\u5217\u53ea\u4f5c\u4e3a\u9000\u51fa Insert \u6a21\u5f0f\u7684\u9644\u52a0\u6620\u5c04\uff0c\u4e0d\u4f1a\u66ff\u6362 Esc\uff1b\u7559\u7a7a\u53ef\u5173\u95ed\u3002',
      runAtStartup: '\u7cfb\u7edf\u542f\u52a8\u65f6\u81ea\u52a8\u8fd0\u884c',
      startHidden: '\u542f\u52a8\u540e\u4fdd\u6301\u9690\u85cf',
      closeToMinimize: '\u70b9\u51fb\u4e3b\u7a97\u53e3\u7684\u5173\u95ed\u6309\u94ae\u65f6\u6700\u5c0f\u5316',
      snapToEdges: '\u4e3b\u7a97\u53e3\u5438\u9644\u5c4f\u5e55\u8fb9\u7f18',
      windowOpacity: '\u7a97\u53e3\u900f\u660e\u5ea6',
      enableTransparency: '\u542f\u7528\u7a97\u53e3\u534a\u900f\u660e',
      titleDoubleClick: '\u53cc\u51fb\u9875\u9762\u6807\u9898\u65f6',
      noAction: '\u65e0\u52a8\u4f5c',
      deletePage: '\u5220\u9664\u9875\u9762',
      renameTitle: '\u4fee\u6539\u6807\u9898',
      baseKey: '\u57fa\u672c\u952e',
      modifiers: '\u7ec4\u5408\u952e',
      separatorText: '\u5206\u9694\u884c',
      dateTimeText: '\u65e5\u671f\u65f6\u95f4',
      dateTimeSeparatorText: '\u65e5\u671f\u65f6\u95f4\u5206\u9694\u884c',
      custom: '\u81ea\u5b9a\u4e49',
      add: '\u6dfb\u52a0',
      edit: '\u4fee\u6539',
      delete: '\u5220\u9664',
      ok: '\u786e\u5b9a',
      cancel: '\u53d6\u6d88',
      language: '\u754c\u9762\u8bed\u8a00',
      english: 'English',
      chinese: '\u4e2d\u6587',
      alwaysOnTop: '\u7a97\u53e3\u7f6e\u9876',
      theme: '\u4e3b\u9898',
      system: '\u8ddf\u968f\u7cfb\u7edf',
      light: '\u6d45\u8272',
      dark: '\u6df1\u8272',
      shortcuts: '\u5feb\u6377\u952e',
      globalShortcuts: '\u5168\u5c40\u5feb\u6377\u952e',
      globalShortcutHint: 'NeoPad \u4e0d\u5728\u524d\u53f0\u65f6\u4ecd\u53ef\u4f7f\u7528\u3002',
      applicationShortcuts: '\u5e94\u7528\u5185\u5feb\u6377\u952e',
      applicationShortcutHint: 'NeoPad \u7a97\u53e3\u5904\u4e8e\u6d3b\u52a8\u72b6\u6001\u65f6\u53ef\u4f7f\u7528\u3002',
      toggleWindow: '\u663e\u793a/\u9690\u85cf\u7a97\u53e3',
      saveClipboard: '\u4fdd\u5b58\u526a\u8d34\u677f',
      hideWindow: '\u9690\u85cf\u7a97\u53e3',
      editor: '\u7f16\u8f91\u5668',
      defaultMode: '\u9ed8\u8ba4\u6a21\u5f0f',
      previewAppearance: '\u9884\u89c8\u5916\u89c2',
      editorFont: '\u5b57\u4f53',
      editorFontSize: '\u5b57\u53f7',
      fontSample: 'NeoPad Markdown 123 \u4f60\u597d',
      previewTheme: '\u9884\u89c8\u4e3b\u9898',
      previewThemeLight: 'Light (\u9ed8\u8ba4)',
      previewThemeOneDark: 'Dark (One Dark)',
      previewThemeNord: 'Nord',
      previewThemeSolarizedLight: 'Solarized Light',
      previewThemeSolarizedDark: 'Solarized Dark',
      previewThemeMonokai: 'Monokai',
      previewThemeGitHubLight: 'GitHub Light',
      previewThemeDracula: 'Dracula',
      previewFont: '\u9884\u89c8\u5b57\u4f53',
      previewFontEditor: '\u8ddf\u968f\u7f16\u8f91\u5668',
      previewFontSystem: '\u7cfb\u7edf\u65e0\u886c\u7ebf',
      previewFontSerif: '\u886c\u7ebf',
      previewFontMono: '\u7b49\u5bbd',
      previewFontSize: '\u5b57\u53f7',
      previewLineHeight: '\u884c\u9ad8',
      previewLineCompact: '\u7d27\u51d1',
      previewLineStandard: '\u6807\u51c6',
      previewLineRelaxed: '\u5bbd\u677e',
      previewContentWidth: '\u5185\u5bb9\u5bbd\u5ea6',
      previewWidthCompact: '\u7d27\u51d1',
      previewWidthStandard: '\u6807\u51c6',
      previewWidthWide: '\u5bbd\u5c4f',
      cycleEditorMode: '\u5faa\u73af\u5207\u6362\u7f16\u8f91\u5668\u6a21\u5f0f',
      togglePreviewThemeShortcut: '\u5207\u6362\u9884\u89c8\u4e3b\u9898',
      toggleThemeShortcut: '\u5207\u6362\u65e5\u95f4/\u591c\u95f4\u6a21\u5f0f',
      immersiveFullscreen: '\u6c89\u6d78\u5f0f\u5168\u5c4f',
      switchTabs: '\u5207\u6362\u6807\u7b7e\u9875',
      shortcut: '\u5feb\u6377\u952e',
      disabled: '\u7981\u7528',
      mcp: 'MCP',
      workspace: '\u5de5\u4f5c\u533a',
      copyReadOnlyConfig: '\u590d\u5236\u53ea\u8bfb\u914d\u7f6e',
      copyWriteConfig: '\u590d\u5236\u5199\u5165\u914d\u7f6e',
      mcpLocalService: 'MCP \u672c\u5730 HTTP \u670d\u52a1',
      mcpDescription: '\u9ed8\u8ba4\u5173\u95ed\u3002\u542f\u7528\u540e\uff0c\u6301\u6709\u8bbf\u95ee token \u7684\u672c\u5730\u4ee3\u7406\u53ef\u4ee5\u8bfb\u5199 NeoPad \u7b14\u8bb0\u3002',
      mcpStartupDescription: '\u5982\u9700\u5f00\u673a\u540e\u81ea\u52a8\u5728\u540e\u53f0\u8fd0\u884c MCP\uff0c\u8bf7\u5728\u201c\u5e38\u89c4\u201d\u4e2d\u542f\u7528 NeoPad \u5f00\u673a\u81ea\u542f\u548c\u542f\u52a8\u540e\u9690\u85cf\uff0c\u5e76\u4fdd\u6301\u672c\u670d\u52a1\u542f\u7528\u3002',
      enableMcp: '\u542f\u7528 MCP',
      address: '\u5730\u5740',
      status: '\u72b6\u6001',
      stopped: '\u5df2\u505c\u6b62',
      startMcpService: '\u542f\u52a8 MCP \u670d\u52a1',
      stopMcpService: '\u505c\u6b62 MCP \u670d\u52a1',
      accessToken: '\u5bc6\u94a5',
      tokenPending: '\u9996\u6b21\u4f7f\u7528\u65f6\u751f\u6210',
      copyAgentConfig: '\u590d\u5236\u4ee3\u7406\u914d\u7f6e',
      regenerateToken: '\u91cd\u751f Token',
      installMethod: '\u5b89\u88c5\u65b9\u6cd5',
      installMethodDescription: '\u5c06\u4e0b\u65b9\u914d\u7f6e\u586b\u5165\u652f\u6301 Streamable HTTP \u7684 MCP \u5ba2\u6237\u7aef\u3002',
    },
    search: {
      title: '\u641c\u7d22',
      placeholder: '\u641c\u7d22\u7b14\u8bb0',
      close: '\u5173\u95ed',
      searching: '\u641c\u7d22\u4e2d...',
      noResults: '\u6ca1\u6709\u7ed3\u679c',
      line: '\u884c',
      summary: '\u5171 {notes} \u7bc7\u7b14\u8bb0\uff0c{matches} \u4e2a\u5339\u914d\u884c',
      matchCount: '{count} \u4e2a\u5339\u914d',
      showMoreInNote: '\u5c55\u5f00\u5176\u4f59 {count} \u4e2a\u5339\u914d',
      collapse: '\u6536\u8d77\u5339\u914d',
      loadMore: '\u52a0\u8f7d\u66f4\u591a\u7ed3\u679c',
    },
    reminders: {
      title: '\u63d0\u9192\u5217\u8868',
      createTitle: '\u521b\u5efa\u63d0\u9192',
      contentLabel: '\u63d0\u9192\u4e8b\u9879',
      dateLabel: '\u65e5\u671f',
      timeLabel: '\u65f6\u95f4',
      insert: '\u63d2\u5165',
      cancel: '\u53d6\u6d88',
      close: '\u5173\u95ed',
      refresh: '\u5237\u65b0',
      empty: '\u6682\u65e0\u63d0\u9192',
      status: '\u72b6\u6001',
      dueAt: '\u5230\u671f\u65f6\u95f4',
      content: '\u63d0\u9192\u5185\u5bb9',
      page: '\u6240\u5728\u9875\u9762',
      pending: '\u5f85\u63d0\u9192',
      due: '\u5df2\u5230\u671f',
      completed: '\u5df2\u5b8c\u6210',
      actions: '\u64cd\u4f5c',
      complete: '\u6807\u8bb0\u5df2\u5b8c\u6210',
      reopen: '\u6807\u8bb0\u672a\u5b8c\u6210',
      completeAllDue: '\u6e05\u7406\u5df2\u5230\u671f',
      filterLabel: '\u7b5b\u9009',
      filterAll: '\u5168\u90e8',
      filterPending: '\u53ea\u770b\u672a\u5230\u671f',
      filterDue: '\u53ea\u770b\u5df2\u5230\u671f',
      filterCompleted: '\u53ea\u770b\u5df2\u5b8c\u6210',
      notificationTitle: 'NeoPad \u63d0\u9192',
    },
    archive: {
      title: '\u5b58\u6863',
      close: '\u5173\u95ed',
      refresh: '\u5237\u65b0',
      empty: '\u6682\u65e0\u5b58\u6863\u9875\u9762',
      restore: '\u6062\u590d',
    },
    library: {
      title: '\u6587\u4ef6',
      notes: '\u7b14\u8bb0',
      archive: '\u5b58\u6863',
      trash: '\u56de\u6536\u7ad9',
      emptyNotes: '\u6682\u65e0\u7b14\u8bb0',
      emptyArchive: '\u6682\u65e0\u5b58\u6863\u7b14\u8bb0',
      emptyTrash: '\u56de\u6536\u7ad9\u4e3a\u7a7a',
      newNote: '\u65b0\u5efa\u7b14\u8bb0',
      refresh: '\u5237\u65b0\u7b14\u8bb0\u5e93',
      restore: '\u6062\u590d\u7b14\u8bb0',
      rename: '\u91cd\u547d\u540d',
      archiveAction: '\u5b58\u6863',
      delete: '\u5220\u9664',
      clearTrash: '\u6e05\u7a7a\u56de\u6536\u7ad9',
      clearTrashTitle: '\u6e05\u7a7a\u56de\u6536\u7ad9\uff1f',
      clearTrashMessage: '\u8981\u5c06 NeoPad \u56de\u6536\u7ad9\u4e2d\u7684\u5168\u90e8\u7b14\u8bb0\u79fb\u81f3\u7cfb\u7edf\u56de\u6536\u7ad9\u5417\uff1f\u4ecd\u53ef\u4ece\u7cfb\u7edf\u56de\u6536\u7ad9\u6062\u590d\u3002',
      clearTrashConfirm: '\u6e05\u7a7a\u56de\u6536\u7ad9',
      revealInFileManager: '\u5728\u8d44\u6e90\u7ba1\u7406\u5668\u4e2d\u663e\u793a',
      help: '\u6d4f\u89c8\u672c\u5730\u7b14\u8bb0\u3001\u5b58\u6863\u548c\u56de\u6536\u7ad9\u3002Ctrl+\u70b9\u51fb\u53ef\u591a\u9009\uff0cShift+\u70b9\u51fb\u53ef\u8fde\u7eed\u9009\u4e2d\uff1b\u53f3\u952e\u7ba1\u7406\u5f53\u524d\u9009\u533a\u3002',
    },
    recovery: {
      title: '\u6062\u590d\u672a\u4fdd\u5b58\u7684\u7b14\u8bb0\uff1f',
      message: 'NeoPad \u53d1\u73b0 {fileName} \u6709\u4e00\u6b21\u672a\u5b8c\u6210\u7684\u4fdd\u5b58\u3002\u8981\u6062\u590d\u5df2\u4fdd\u7559\u7684\u5185\u5bb9\u5417\uff1f',
      restore: '\u6062\u590d',
      restored: '\u5df2\u6062\u590d\u672a\u4fdd\u5b58\u5185\u5bb9',
    },
    status: {
      editMode: '\u7f16\u8f91\u6a21\u5f0f',
      hybridMode: '\u6df7\u5408\u6a21\u5f0f',
      previewMode: '\u9884\u89c8\u6a21\u5f0f',
      saved: '\u5df2\u4fdd\u5b58',
      saving: '\u4fdd\u5b58\u4e2d',
      failed: '\u5931\u8d25',
      markdown: 'Markdown',
      clipboard: '\u526a\u8d34\u677f',
      clipboardSaved: '\u526a\u8d34\u677f\u5df2\u4fdd\u5b58',
      loadedFromFile: '\u5df2\u4ece\u6587\u4ef6\u8f7d\u5165',
      savedAsFile: '\u5df2\u53e6\u5b58\u5230\u6587\u4ef6',
      exported: '\u5df2\u5168\u90e8\u5bfc\u51fa',
      exportingNote: '\u6b63\u5728\u5bfc\u51fa\u7b14\u8bb0...',
      exportedPng: '\u5df2\u5bfc\u51fa PNG',
      exportedPdf: '\u5df2\u5bfc\u51fa PDF',
      exportTooLong: '\u7b14\u8bb0\u8fc7\u957f\uff0c\u65e0\u6cd5\u5b89\u5168\u5bfc\u51fa',
      exportFailed: '\u7b14\u8bb0\u5bfc\u51fa\u5931\u8d25',
      archiveOpened: '\u5df2\u5728\u8d44\u6e90\u7ba1\u7406\u5668\u4e2d\u6253\u5f00\u5b58\u6863',
      notePathCopied: '\u5df2\u590d\u5236\u5f53\u524d\u9875\u9762\u6587\u4ef6\u5730\u5740',
      trashOpened: '\u5df2\u6253\u5f00\u56de\u6536\u7ad9',
      noteFileMissing: '\u7b14\u8bb0\u6587\u4ef6\u4e0d\u5b58\u5728',
      fontUpdated: '\u5df2\u66f4\u65b0\u5b57\u4f53',
      backgroundUpdated: '\u5df2\u66f4\u65b0\u80cc\u666f\u8272',
      wordWrapOn: '\u5df2\u5f00\u542f\u81ea\u52a8\u6362\u884c',
      wordWrapOff: '\u5df2\u5173\u95ed\u81ea\u52a8\u6362\u884c',
      inserted: '\u5df2\u63d2\u5165',
      textProcessed: '\u5df2\u5904\u7406\u6587\u672c',
      opacityUpdated: '\u5df2\u66f4\u65b0\u7a97\u53e3\u900f\u660e\u5ea6',
      unsupportedHash: '\u5f53\u524d\u73af\u5883\u4e0d\u652f\u6301\u8be5 Hash \u7b97\u6cd5',
      expressionCalculated: '\u5df2\u8ba1\u7b97\u8868\u8fbe\u5f0f',
      expressionNotFound: '\u672a\u627e\u5230\u53ef\u8ba1\u7b97\u8868\u8fbe\u5f0f',
      search: '\u641c\u7d22',
      settings: '\u8bbe\u7f6e',
      alwaysOnTop: '\u7a97\u53e3\u7f6e\u9876',
      pinned: '\u5df2\u7f6e\u9876',
      unpinned: '\u5df2\u53d6\u6d88\u7f6e\u9876',
      mcpConfigCopied: '\u5df2\u590d\u5236 MCP \u4ee3\u7406\u914d\u7f6e',
      mcpUpdated: 'MCP \u8bbe\u7f6e\u5df2\u66f4\u65b0',
      chars: '\u5b57\u7b26',
      switchToLight: '\u5207\u6362\u81f3\u65e5\u95f4\u6a21\u5f0f',
      switchToDark: '\u5207\u6362\u81f3\u591c\u95f4\u6a21\u5f0f',
    },
  },
}
