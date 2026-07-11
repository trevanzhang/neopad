import type { ComputedRef, Ref } from 'vue'
import {
  archiveNote,
  closeNote,
  createNote,
  deleteNote,
  renameNote,
  saveClipboard,
  setNoteColor,
  unarchiveNote,
  writeNote,
} from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'
import { messages, type AppLanguage } from '../lib/i18n'
import type { NoteTab } from '../types/note'
import type { TitleDoubleClickAction } from '../lib/preferences'

interface NoteLifecycleOptions {
  tabs: Ref<NoteTab[]>
  activeTabId: Ref<string>
  activeTab: ComputedRef<NoteTab | undefined>
  content: Ref<string>
  saveState: Ref<'Saved' | 'Saving' | 'Failed'>
  statusMessage: Ref<string>
  language: Ref<AppLanguage>
  titleDoubleClickAction: Ref<TitleDoubleClickAction>
  archiveListOpen: Ref<boolean>
  text: () => typeof messages.en
  forceSave: () => Promise<boolean>
  nextNoteLoadGeneration: () => number
  isCurrentNoteLoad: (generation: number) => boolean
  loadActiveNote: (generation?: number) => Promise<boolean>
  setContentFromLoad: (content: string) => void
  requestInput: (title: string, initialValue: string) => Promise<string | null>
  requestConfirmation: (title: string, message: string, confirmLabel: string, danger?: boolean) => Promise<boolean>
  focusEditor: () => void
  refreshRecentNotes: () => Promise<void>
  refreshArchivedNotes: () => Promise<void>
  refreshLibrary: () => Promise<void>
  upsertTab: (tab: NoteTab) => void
}

export function useNoteLifecycle(o: NoteLifecycleOptions) {
  const deletingTabIds = new Set<string>()
  const fail = () => { o.saveState.value = 'Failed' }
  const tabById = (id: string) => o.tabs.value.find((tab) => tab.id === id)

  async function selectTab(tabId: string) {
    if (tabId === o.activeTabId.value) return
    const generation = o.nextNoteLoadGeneration()
    if (!(await o.forceSave()) || !o.isCurrentNoteLoad(generation)) return
    o.activeTabId.value = tabId
    if (await o.loadActiveNote(generation)) o.focusEditor()
  }

  function cycleTab(offset: -1 | 1) {
    const currentIndex = o.tabs.value.findIndex((tab) => tab.id === o.activeTabId.value)
    if (currentIndex < 0 || o.tabs.value.length < 2) return
    const nextTab = o.tabs.value[(currentIndex + offset + o.tabs.value.length) % o.tabs.value.length]
    if (nextTab) void selectTab(nextTab.id)
  }

  async function renameTab(tab: NoteTab) {
    if (tab.id === 'inbox' || tab.id === 'clipboard' || tab.external) return
    const nextTitle = (await o.requestInput(o.text().settings.renameTitle, tab.title))?.trim()
    if (!nextTitle) {
      if (tab.id === o.activeTabId.value) o.focusEditor()
      return
    }
    const previousTitle = tab.title
    if (isTauriRuntime()) {
      try {
        if (tab.id === o.activeTabId.value && !(await o.forceSave())) return
        const renamed = await renameNote(tab.id, nextTitle)
        Object.assign(tab, { title: renamed.title, updatedAt: renamed.updatedAt, systemTitle: renamed.systemTitle })
        if (tab.id === o.activeTabId.value && await o.loadActiveNote()) o.focusEditor()
      } catch { fail() }
      return
    }
    tab.title = nextTitle
    const shouldUpdateDefaultHeading = tab.systemTitle && tab.id === o.activeTabId.value
    tab.systemTitle = false
    tab.updatedAt = Date.now()
    if (shouldUpdateDefaultHeading) {
      const defaultHeading = `# ${previousTitle}`
      if (o.content.value === defaultHeading || o.content.value.startsWith(`${defaultHeading}\n`)) {
        o.setContentFromLoad(`# ${nextTitle}${o.content.value.slice(defaultHeading.length)}`)
      }
    }
    if (tab.id === o.activeTabId.value) o.focusEditor()
  }

  function adjacentTabIdAfterRemoval(tabId: string) {
    const index = o.tabs.value.findIndex((item) => item.id === tabId)
    if (index < 0) return o.tabs.value[0]?.id ?? 'inbox'
    return o.tabs.value[index - 1]?.id ?? o.tabs.value[index + 1]?.id ?? 'inbox'
  }

  async function deleteTab(tab: NoteTab) {
    if (tab.id === 'inbox' || tab.id === 'clipboard' || tab.external || deletingTabIds.has(tab.id)) return
    const wasActive = o.activeTabId.value === tab.id
    const nextId = wasActive ? adjacentTabIdAfterRemoval(tab.id) : o.activeTabId.value
    const confirmed = await o.requestConfirmation(
      o.text().tabs.confirmDeleteTitle,
      o.text().tabs.confirmDeleteMessage.replace('{title}', tab.title),
      o.text().tabs.delete,
      true,
    )
    if (!confirmed) {
      if (wasActive) o.focusEditor()
      return
    }
    deletingTabIds.add(tab.id)
    if (isTauriRuntime()) {
      try {
        if (!(await o.forceSave())) { fail(); return }
        if (wasActive) { o.activeTabId.value = nextId; await o.loadActiveNote() }
        await deleteNote(tab.id)
      } catch { fail(); return } finally { deletingTabIds.delete(tab.id) }
    }
    o.tabs.value = o.tabs.value.filter((item) => item.id !== tab.id)
    await o.refreshLibrary()
    if (o.activeTabId.value === tab.id) {
      o.activeTabId.value = nextId ?? o.tabs.value[0]?.id ?? 'inbox'
      await o.loadActiveNote()
    }
    if (wasActive) o.focusEditor()
    deletingTabIds.delete(tab.id)
  }

  async function closeTab(tab: NoteTab) {
    if (tab.id === 'inbox' || tab.id === 'clipboard') return
    const wasActive = o.activeTabId.value === tab.id
    const nextId = wasActive ? adjacentTabIdAfterRemoval(tab.id) : o.activeTabId.value
    try {
      if (isTauriRuntime() && !tab.external) {
        if (!(await o.forceSave())) return
        await closeNote(tab.id)
      }
      o.tabs.value = o.tabs.value.filter((item) => item.id !== tab.id)
      if (wasActive) { o.activeTabId.value = nextId; await o.loadActiveNote(); o.focusEditor() }
      await o.refreshRecentNotes()
      await o.refreshLibrary()
    } catch { fail() }
  }

  async function archiveTab(tab: NoteTab) {
    if (tab.id === 'inbox' || tab.id === 'clipboard') return
    if (tab.external) {
      const confirmed = await o.requestConfirmation(
        o.text().tabs.archive,
        o.language.value === 'zh'
          ? `将“${tab.title}”复制到 NeoPad 存档。原始文件不会移动或删除。`
          : `Copy "${tab.title}" into the NeoPad archive? The original file will not be moved or deleted.`,
        o.text().tabs.archive,
      )
      if (!confirmed) { if (o.activeTabId.value === tab.id) o.focusEditor(); return }
      try {
        if (!(await o.forceSave())) return
        const created = await createNote(tab.title)
        const saved = await writeNote(created.id, o.content.value, created.updatedAt)
        await archiveNote(saved.id)
        await closeTab(tab)
      } catch { fail() }
      return
    }
    if (tab.archived) { await unarchiveTab(tab); return }
    const wasActive = o.activeTabId.value === tab.id
    const nextId = wasActive ? adjacentTabIdAfterRemoval(tab.id) : o.activeTabId.value
    const confirmed = await o.requestConfirmation(
      o.text().tabs.archive,
      o.text().tabs.confirmArchiveMessage.replace('{title}', tab.title),
      o.text().tabs.archive,
    )
    if (!confirmed) { if (wasActive) o.focusEditor(); return }
    try {
      if (isTauriRuntime()) { if (!(await o.forceSave())) return; await archiveNote(tab.id) }
      o.tabs.value = o.tabs.value.filter((item) => item.id !== tab.id)
      if (wasActive) { o.activeTabId.value = nextId; await o.loadActiveNote(); o.focusEditor() }
      await o.refreshRecentNotes()
      await o.refreshLibrary()
    } catch { fail() }
  }

  async function unarchiveTab(tab: NoteTab) {
    try {
      const restored = isTauriRuntime() ? await unarchiveNote(tab.id) : { ...tab, archived: false, open: true }
      o.upsertTab(restored)
      o.activeTabId.value = restored.id
      await o.loadActiveNote()
      await o.refreshRecentNotes()
      await o.refreshLibrary()
      if (o.archiveListOpen.value) await o.refreshArchivedNotes()
      else o.focusEditor()
    } catch { fail() }
  }

  async function createLocalTab() {
    const generation = o.nextNoteLoadGeneration()
    if (isTauriRuntime()) {
      if (!(await o.forceSave()) || !o.isCurrentNoteLoad(generation)) return
      try {
        const note = await createNote()
        if (!o.isCurrentNoteLoad(generation)) return
        o.upsertTab({ id: note.id, title: note.title, fileName: note.fileName,
          createdAt: note.updatedAt, updatedAt: note.updatedAt, pinned: false,
          deleted: false, archived: false, open: true, systemTitle: true })
        o.activeTabId.value = note.id
        o.setContentFromLoad(note.content)
        o.saveState.value = 'Saved'
        o.focusEditor()
        await o.refreshLibrary()
        return
      } catch {
        fail()
        return
      }
    }
    const createdAt = Date.now()
    const tab: NoteTab = { id: `draft-${createdAt}`, title: 'Untitled', fileName: `page-${createdAt}.md`,
      createdAt, updatedAt: createdAt, pinned: false, deleted: false, archived: false, open: true, systemTitle: true }
    o.tabs.value.push(tab)
    o.activeTabId.value = tab.id
    o.content.value = `# ${tab.title}\n\n`
    o.focusEditor()
    await o.refreshLibrary()
  }

  async function saveCurrentClipboard() {
    if (!isTauriRuntime()) { o.statusMessage.value = o.text().status.clipboard; return }
    const generation = o.nextNoteLoadGeneration()
    try {
      if (!(await o.forceSave()) || !o.isCurrentNoteLoad(generation)) return
      const note = await saveClipboard()
      if (!o.isCurrentNoteLoad(generation)) return
      o.upsertTab({ id: note.id, title: note.title, fileName: note.fileName,
        createdAt: note.updatedAt, updatedAt: note.updatedAt, pinned: true,
        deleted: false, archived: false, open: true, systemTitle: false })
      o.activeTabId.value = note.id
      o.setContentFromLoad(note.content)
      o.statusMessage.value = o.text().status.clipboardSaved
      o.focusEditor()
    } catch { fail() }
  }

  const runForActive = (action: (tab: NoteTab) => Promise<void>) => o.activeTab.value ? action(o.activeTab.value) : Promise.resolve()
  const runForId = (id: string, action: (tab: NoteTab) => Promise<void>) => {
    const tab = tabById(id)
    return tab ? action(tab) : Promise.resolve()
  }
  async function handleTabTitleDoubleClick(id: string) {
    if (o.titleDoubleClickAction.value === 'none') return
    const action = o.titleDoubleClickAction.value === 'rename' ? renameTab : deleteTab
    await runForId(id, action)
  }

  return {
    selectTab, cycleTab, handleTabTitleDoubleClick,
    renameActivePage: () => runForActive(renameTab),
    deleteActivePage: () => runForActive(deleteTab),
    closeActivePage: () => runForActive(closeTab),
    archiveActivePage: () => runForActive(archiveTab),
    unarchiveActivePage: () => runForActive(unarchiveTab),
    renamePageById: (id: string) => runForId(id, renameTab),
    deletePageById: (id: string) => runForId(id, deleteTab),
    closePageById: (id: string) => runForId(id, closeTab),
    archivePageById: (id: string) => runForId(id, archiveTab),
    updateTabColor: async (id: string, color: string | null) => {
      const tab = tabById(id); if (!tab) return
      if (!isTauriRuntime()) { tab.color = color ?? undefined; return }
      try { const updated = await setNoteColor(id, color); tab.color = updated.color; tab.updatedAt = updated.updatedAt } catch { fail() }
    },
    renameTab, deleteTab, closeTab, archiveTab, unarchiveTab, createLocalTab, saveCurrentClipboard,
  }
}
