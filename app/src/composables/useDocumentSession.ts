import { ref, watch, type ComputedRef, type Ref } from 'vue'
import { AutosaveCoordinator } from '../lib/autosave'
import {
  readExternalMarkdown,
  readNote,
  writeExternalMarkdown,
  writeNote,
} from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'
import type { NoteTab } from '../types/note'

interface DocumentSessionOptions {
  tabs: Ref<NoteTab[]>
  activeTabId: Ref<string>
  activeTab: ComputedRef<NoteTab | undefined>
  statusMessage: Ref<string>
  failedMessage: () => string
  rememberExternalDocument: (tab: NoteTab) => void
}

export function useDocumentSession(options: DocumentSessionOptions) {
  const content = ref('# Inbox\n\nStart typing...')
  const saveState = ref<'Saved' | 'Saving' | 'Failed'>('Saved')
  const isLoadingNote = ref(false)
  let suppressedLoadedContent: string | null = null
  let noteLoadGeneration = 0
  let loadingNoteGeneration: number | null = null

  async function saveDocument(noteId: string, nextContent: string) {
    const tab = options.tabs.value.find((item) => item.id === noteId)
    if (tab?.external && tab.externalPath) {
      if (!tab.externalRevision) throw new Error('external document revision is missing')
      const saved = await writeExternalMarkdown(tab.externalPath, nextContent, tab.externalRevision)
      tab.updatedAt = saved.updatedAt
      tab.externalRevision = saved.revision
      options.rememberExternalDocument(tab)
      return {
        id: noteId,
        title: tab.title,
        fileName: tab.fileName,
        content: saved.content,
        updatedAt: saved.updatedAt,
        revision: saved.revision,
      }
    }
    if (!tab) throw new Error(`note tab not found: ${noteId}`)
    return writeNote(noteId, nextContent, tab.updatedAt)
  }

  const autosave = new AutosaveCoordinator({
    delayMs: 500,
    save: ({ noteId, content: nextContent }: { noteId: string; content: string }) =>
      saveDocument(noteId, nextContent),
    onSaved: ({ noteId }, saved) => {
      const tab = options.tabs.value.find((item) => item.id === noteId)
      if (tab) {
        tab.updatedAt = saved.updatedAt
        if ('revision' in saved) tab.externalRevision = saved.revision
      }
    },
    onStateChange: (state) => {
      saveState.value = state
    },
    onError: (error) => {
      const message = error instanceof Error ? error.message : String(error)
      options.statusMessage.value = message || options.failedMessage()
    },
  })

  watch(content, (nextContent) => {
    if (suppressedLoadedContent === nextContent) {
      suppressedLoadedContent = null
      return
    }
    if (isLoadingNote.value || !isTauriRuntime()) return
    const tab = options.activeTab.value
    if (tab) autosave.markChanged({ noteId: tab.id, content: nextContent })
  })

  function nextNoteLoadGeneration() {
    noteLoadGeneration += 1
    return noteLoadGeneration
  }

  function isCurrentNoteLoad(generation: number) {
    return generation === noteLoadGeneration
  }

  async function loadActiveNote(generation = nextNoteLoadGeneration()) {
    const tab = options.activeTab.value
    if (!tab) return false
    const tabId = tab.id

    isLoadingNote.value = true
    loadingNoteGeneration = generation
    try {
      const note = tab.external && tab.externalPath
        ? await readExternalMarkdown(tab.externalPath)
        : await readNote(tabId)
      if (!isCurrentNoteLoad(generation) || options.activeTabId.value !== tabId) return false
      const loadedTab = options.tabs.value.find((item) => item.id === tabId)
      if (loadedTab) loadedTab.updatedAt = note.updatedAt
      setContentFromLoad(note.content)
      saveState.value = 'Saved'
      return true
    } catch {
      if (isCurrentNoteLoad(generation)) saveState.value = 'Failed'
      return false
    } finally {
      if (loadingNoteGeneration === generation) {
        isLoadingNote.value = false
        loadingNoteGeneration = null
      }
    }
  }

  function setContentFromLoad(nextContent: string) {
    if (content.value !== nextContent) suppressedLoadedContent = nextContent
    content.value = nextContent
    autosave.markLoaded()
  }

  async function forceSave() {
    if (!isTauriRuntime()) {
      saveState.value = 'Saved'
      return true
    }
    return autosave.flush()
  }

  function disposeDocumentSession() {
    autosave.dispose()
  }

  return {
    content,
    saveState,
    isLoadingNote,
    nextNoteLoadGeneration,
    isCurrentNoteLoad,
    loadActiveNote,
    setContentFromLoad,
    forceSave,
    disposeDocumentSession,
  }
}
