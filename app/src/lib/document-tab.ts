import type { DocumentKind, NoteTab } from '../types/note'

export function documentKind(tab: NoteTab | undefined): DocumentKind {
  if (tab?.kind) return tab.kind
  if (tab?.external) return 'external'
  return 'note'
}

export function isNoteTab(tab: NoteTab | undefined) {
  return documentKind(tab) === 'note'
}

export function isPromptTab(tab: NoteTab | undefined) {
  return documentKind(tab) === 'prompt'
}

export function isExternalTab(tab: NoteTab | undefined) {
  return documentKind(tab) === 'external'
}

export function promptTabId(promptId: string) {
  return `prompt:${promptId}`
}
