export type AppLanguage = 'en' | 'zh'

export type AppMessages = {
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
    saveAsFile: string
    exportAll: string
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
    f3: string
    ctrlR: string
    ctrlShiftF: string
    ctrlA: string
    toggleTabBarDisplay: string
    tabBarDisplay: string
    horizontal: string
    vertical: string
    f10: string
    font: string
    backgroundColor: string
    wordWrap: string
    ctrlW: string
    editMode: string
    splitMode: string
    previewMode: string
    alwaysOnTop: string
    renamePage: string
    deletePage: string
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
    search: string
    settings: string
    about: string
  }
  settings: {
    title: string
    close: string
    general: string
    language: string
    english: string
    chinese: string
    alwaysOnTop: string
    theme: string
    system: string
    light: string
    dark: string
    shortcuts: string
    toggleWindow: string
    saveClipboard: string
    hideWindow: string
    editor: string
    previewMode: string
    mcp: string
    workspace: string
    copyReadOnlyConfig: string
    copyWriteConfig: string
  }
  search: {
    title: string
    placeholder: string
    close: string
    searching: string
    noResults: string
    line: string
  }
  status: {
    saved: string
    saving: string
    failed: string
    markdown: string
    clipboard: string
    clipboardSaved: string
    loadedFromFile: string
    savedAsFile: string
    exported: string
    trashOpened: string
    fontUpdated: string
    backgroundUpdated: string
    wordWrapOn: string
    wordWrapOff: string
    inserted: string
    search: string
    settings: string
    alwaysOnTop: string
    pinned: string
    unpinned: string
    mcpReadOnlyCopied: string
    mcpWriteCopied: string
    chars: string
  }
}

export const messages: Record<AppLanguage, AppMessages> = {
  en: {
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
      saveAsFile: 'Save As File(S)',
      exportAll: 'Export All(E)...',
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
      f3: 'F3',
      ctrlR: 'Ctrl+R',
      ctrlShiftF: 'Ctrl+Shift+F',
      ctrlA: 'Ctrl+A',
      toggleTabBarDisplay: 'Toggle Tab Bar Display(S)',
      tabBarDisplay: 'Tab Bar Display(T)',
      horizontal: 'Horizontal(H)',
      vertical: 'Vertical(V)',
      f10: 'F10',
      font: 'Font(F)...',
      backgroundColor: 'Background Color(B)...',
      wordWrap: 'Word Wrap(W)',
      ctrlW: 'Ctrl+W',
      editMode: 'Edit',
      splitMode: 'Split',
      previewMode: 'Preview',
      alwaysOnTop: 'Always on Top',
      renamePage: 'Rename Page',
      deletePage: 'Delete Page',
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
      search: 'Search',
      settings: 'Settings',
      about: 'About NeoPad',
    },
    settings: {
      title: 'Settings',
      close: 'Close',
      general: 'General',
      language: 'Language',
      english: 'English',
      chinese: 'Chinese',
      alwaysOnTop: 'Always on top',
      theme: 'Theme',
      system: 'System',
      light: 'Light',
      dark: 'Dark',
      shortcuts: 'Shortcuts',
      toggleWindow: 'Toggle window',
      saveClipboard: 'Save clipboard',
      hideWindow: 'Hide window',
      editor: 'Editor',
      previewMode: 'Preview mode',
      mcp: 'MCP',
      workspace: 'Workspace',
      copyReadOnlyConfig: 'Copy read-only config',
      copyWriteConfig: 'Copy write config',
    },
    search: {
      title: 'Search',
      placeholder: 'Search notes',
      close: 'Close',
      searching: 'Searching...',
      noResults: 'No results',
      line: 'line',
    },
    status: {
      saved: 'Saved',
      saving: 'Saving',
      failed: 'Failed',
      markdown: 'Markdown',
      clipboard: 'Clipboard',
      clipboardSaved: 'Clipboard saved',
      loadedFromFile: 'Loaded from file',
      savedAsFile: 'Saved as file',
      exported: 'Exported',
      trashOpened: 'Trash opened',
      fontUpdated: 'Font updated',
      backgroundUpdated: 'Background color updated',
      wordWrapOn: 'Word wrap on',
      wordWrapOff: 'Word wrap off',
      inserted: 'Inserted',
      search: 'Search',
      settings: 'Settings',
      alwaysOnTop: 'Always on top',
      pinned: 'Pinned',
      unpinned: 'Unpinned',
      mcpReadOnlyCopied: 'MCP read-only config copied',
      mcpWriteCopied: 'MCP write config copied',
      chars: 'chars',
    },
  },
  zh: {
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
      saveAsFile: '\u53e6\u5b58\u5230\u6587\u4ef6(S)',
      exportAll: '\u5168\u90e8\u5bfc\u51fa(E)...',
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
      f3: 'F3',
      ctrlR: 'Ctrl+R',
      ctrlShiftF: 'Ctrl+Shift+F',
      ctrlA: 'Ctrl+A',
      toggleTabBarDisplay: '\u5207\u6362\u6807\u7b7e\u680f\u663e\u793a\u65b9\u5f0f(S)',
      tabBarDisplay: '\u6807\u7b7e\u680f\u663e\u793a\u65b9\u5f0f(T)',
      horizontal: '\u6a2a\u5411\u65b9\u5f0f(H)',
      vertical: '\u7eb5\u5411\u65b9\u5f0f(V)',
      f10: 'F10',
      font: '\u5b57\u4f53(F)...',
      backgroundColor: '\u80cc\u666f\u8272(B)...',
      wordWrap: '\u81ea\u52a8\u6362\u884c(W)',
      ctrlW: 'Ctrl+W',
      editMode: '\u7f16\u8f91',
      splitMode: '\u5206\u5c4f',
      previewMode: '\u9884\u89c8',
      alwaysOnTop: '\u7a97\u53e3\u7f6e\u9876',
      renamePage: '\u91cd\u547d\u540d\u9875\u9762',
      deletePage: '\u5220\u9664\u9875\u9762',
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
      search: '\u641c\u7d22',
      settings: '\u8bbe\u7f6e',
      about: '\u5173\u4e8e NeoPad',
    },
    settings: {
      title: '\u8bbe\u7f6e',
      close: '\u5173\u95ed',
      general: '\u5e38\u89c4',
      language: '\u754c\u9762\u8bed\u8a00',
      english: 'English',
      chinese: '\u4e2d\u6587',
      alwaysOnTop: '\u7a97\u53e3\u7f6e\u9876',
      theme: '\u4e3b\u9898',
      system: '\u8ddf\u968f\u7cfb\u7edf',
      light: '\u6d45\u8272',
      dark: '\u6df1\u8272',
      shortcuts: '\u5feb\u6377\u952e',
      toggleWindow: '\u663e\u793a/\u9690\u85cf\u7a97\u53e3',
      saveClipboard: '\u4fdd\u5b58\u526a\u8d34\u677f',
      hideWindow: '\u9690\u85cf\u7a97\u53e3',
      editor: '\u7f16\u8f91\u5668',
      previewMode: '\u9884\u89c8\u6a21\u5f0f',
      mcp: 'MCP',
      workspace: '\u5de5\u4f5c\u533a',
      copyReadOnlyConfig: '\u590d\u5236\u53ea\u8bfb\u914d\u7f6e',
      copyWriteConfig: '\u590d\u5236\u5199\u5165\u914d\u7f6e',
    },
    search: {
      title: '\u641c\u7d22',
      placeholder: '\u641c\u7d22\u7b14\u8bb0',
      close: '\u5173\u95ed',
      searching: '\u641c\u7d22\u4e2d...',
      noResults: '\u6ca1\u6709\u7ed3\u679c',
      line: '\u884c',
    },
    status: {
      saved: '\u5df2\u4fdd\u5b58',
      saving: '\u4fdd\u5b58\u4e2d',
      failed: '\u5931\u8d25',
      markdown: 'Markdown',
      clipboard: '\u526a\u8d34\u677f',
      clipboardSaved: '\u526a\u8d34\u677f\u5df2\u4fdd\u5b58',
      loadedFromFile: '\u5df2\u4ece\u6587\u4ef6\u8f7d\u5165',
      savedAsFile: '\u5df2\u53e6\u5b58\u5230\u6587\u4ef6',
      exported: '\u5df2\u5168\u90e8\u5bfc\u51fa',
      trashOpened: '\u5df2\u6253\u5f00\u56de\u6536\u7ad9',
      fontUpdated: '\u5df2\u66f4\u65b0\u5b57\u4f53',
      backgroundUpdated: '\u5df2\u66f4\u65b0\u80cc\u666f\u8272',
      wordWrapOn: '\u5df2\u5f00\u542f\u81ea\u52a8\u6362\u884c',
      wordWrapOff: '\u5df2\u5173\u95ed\u81ea\u52a8\u6362\u884c',
      inserted: '\u5df2\u63d2\u5165',
      search: '\u641c\u7d22',
      settings: '\u8bbe\u7f6e',
      alwaysOnTop: '\u7a97\u53e3\u7f6e\u9876',
      pinned: '\u5df2\u7f6e\u9876',
      unpinned: '\u5df2\u53d6\u6d88\u7f6e\u9876',
      mcpReadOnlyCopied: '\u5df2\u590d\u5236 MCP \u53ea\u8bfb\u914d\u7f6e',
      mcpWriteCopied: '\u5df2\u590d\u5236 MCP \u5199\u5165\u914d\u7f6e',
      chars: '\u5b57\u7b26',
    },
  },
}
