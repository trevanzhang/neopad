import { ref, watch, type ComputedRef, type Ref } from 'vue'
import { AutosaveCoordinator } from '../lib/autosave'
import {
  readAiPrompt,
  readExternalMarkdown,
  readNote,
  writeAiPrompt,
  writeExternalMarkdown,
  writeNote,
} from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'
import { isExternalTab, isPromptTab } from '../lib/document-tab'
import type { NoteTab } from '../types/note'
import type { AiPromptEntry } from '../types/ai'

interface DocumentSessionOptions {
  tabs: Ref<NoteTab[]>
  activeTabId: Ref<string>
  activeTab: ComputedRef<NoteTab | undefined>
  statusMessage: Ref<string>
  failedMessage: () => string
  rememberExternalDocument: (tab: NoteTab) => void
  onPromptSaved?: (prompt: AiPromptEntry) => void
}

export function useDocumentSession(options: DocumentSessionOptions) {
  const content = ref('# Inbox\n\n')
  const saveState = ref<'Saved' | 'Saving' | 'Failed'>('Saved')
  const isLoadingNote = ref(false)
  let suppressedLoadedContent: string | null = null
  let noteLoadGeneration = 0
  let loadingNoteGeneration: number | null = null

  async function saveDocument(noteId: string, nextContent: string) {
    const tab = options.tabs.value.find((item) => item.id === noteId)
    if (isPromptTab(tab)) {
      if (!tab?.promptId || !tab.promptRevision) throw new Error('prompt document revision is missing')
      const saved = await writeAiPrompt(tab.promptId, nextContent, tab.promptRevision)
      tab.updatedAt = saved.updatedAt
      tab.promptRevision = saved.revision
      options.onPromptSaved?.(saved)
      return {
        id: noteId,
        title: tab.title,
        fileName: tab.fileName,
        content: saved.content,
        updatedAt: saved.updatedAt,
        revision: saved.revision,
      }
    }
    if (isExternalTab(tab) && tab?.externalPath) {
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
        if ('revision' in saved) {
          if (isPromptTab(tab)) tab.promptRevision = saved.revision
          else if (isExternalTab(tab)) tab.externalRevision = saved.revision
        }
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
      const note = isPromptTab(tab) && tab.promptId
        ? await readAiPrompt(tab.promptId)
        : isExternalTab(tab) && tab.externalPath
          ? await readExternalMarkdown(tab.externalPath)
          : await readNote(tabId)
      if (!isCurrentNoteLoad(generation) || options.activeTabId.value !== tabId) return false
      const loadedTab = options.tabs.value.find((item) => item.id === tabId)
      if (loadedTab) {
        loadedTab.updatedAt = note.updatedAt
        if ('revision' in note) {
          if (isPromptTab(loadedTab)) loadedTab.promptRevision = note.revision
          else if (isExternalTab(loadedTab)) {
            loadedTab.externalRevision = note.revision
            options.rememberExternalDocument(loadedTab)
          }
        }
      }
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
