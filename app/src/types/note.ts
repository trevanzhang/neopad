export type NoteTab = {
  id: string
  title: string
  fileName: string
  createdAt: number
  updatedAt: number
  pinned: boolean
  deleted: boolean
  systemTitle: boolean
  color?: string
}

export type NoteContent = {
  id: string
  title: string
  fileName: string
  content: string
  updatedAt: number
}

export type WorkspaceInfo = {
  root: string
  notesDir: string
  metaDir: string
  trashDir: string
  backupsDir: string
  configPath: string
  tabsPath: string
}

export type SearchResult = {
  noteId: string
  title: string
  fileName: string
  lineNumber: number
  lineText: string
  before: string[]
  after: string[]
}
