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
    newPage: string
    saveClipboard: string
    undo: string
    redo: string
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
      newPage: 'New Page',
      saveClipboard: 'Save Clipboard',
      undo: 'Undo',
      redo: 'Redo',
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
      file: '文件(F)',
      edit: '编辑(E)',
      view: '视图(V)',
      page: '页面(P)',
      format: '格式(O)',
      insert: '插入(I)',
      tools: '工具(T)',
      help: '帮助(H)',
      newPage: '新建页面',
      saveClipboard: '保存剪贴板',
      undo: '撤销',
      redo: '重做',
      editMode: '编辑',
      splitMode: '分屏',
      previewMode: '预览',
      alwaysOnTop: '窗口置顶',
      renamePage: '重命名页面',
      deletePage: '删除页面',
      plainText: '纯文本',
      markdown: 'Markdown',
      date: '日期',
      separator: '分隔线',
      search: '搜索',
      settings: '设置',
      about: '关于 NeoPad',
    },
    settings: {
      title: '设置',
      close: '关闭',
      general: '常规',
      language: '界面语言',
      english: 'English',
      chinese: '中文',
      alwaysOnTop: '窗口置顶',
      theme: '主题',
      system: '跟随系统',
      light: '浅色',
      dark: '深色',
      shortcuts: '快捷键',
      toggleWindow: '显示/隐藏窗口',
      saveClipboard: '保存剪贴板',
      hideWindow: '隐藏窗口',
      editor: '编辑器',
      previewMode: '预览模式',
      mcp: 'MCP',
      workspace: '工作区',
      copyReadOnlyConfig: '复制只读配置',
      copyWriteConfig: '复制写入配置',
    },
    search: {
      title: '搜索',
      placeholder: '搜索笔记',
      close: '关闭',
      searching: '搜索中...',
      noResults: '没有结果',
      line: '第',
    },
    status: {
      saved: '已保存',
      saving: '保存中',
      failed: '失败',
      markdown: 'Markdown',
      clipboard: '剪贴板',
      clipboardSaved: '剪贴板已保存',
      search: '搜索',
      settings: '设置',
      alwaysOnTop: '窗口置顶',
      pinned: '已置顶',
      unpinned: '已取消置顶',
      mcpReadOnlyCopied: '已复制 MCP 只读配置',
      mcpWriteCopied: '已复制 MCP 写入配置',
      chars: '字符',
    },
  },
}
