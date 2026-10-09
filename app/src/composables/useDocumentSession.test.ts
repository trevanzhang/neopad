import { computed, nextTick, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { readExternalMarkdown, readNote, writeNote } from '../lib/invoke'
import type { NoteTab } from '../types/note'
import { useDocumentSession } from './useDocumentSession'

vi.mock('../lib/invoke', () => ({
  readExternalMarkdown: vi.fn(),
  readNote: vi.fn(),
  writeExternalMarkdown: vi.fn(),
  writeNote: vi.fn(),
}))
vi.mock('../lib/runtime', () => ({ isTauriRuntime: vi.fn(() => true) }))

describe('useDocumentSession', () => {
  beforeEach(() => {
    vi.mocked(readExternalMarkdown).mockReset()
    vi.mocked(readNote).mockReset()
    vi.mocked(writeNote).mockReset()
  })

  it('adopts the revision that belongs to freshly loaded external content', async () => {
    const tab: NoteTab = {
      id: 'external:C:/notes/test.md', title: 'test', fileName: 'test.md',
      createdAt: 1, updatedAt: 1, pinned: false, deleted: false, archived: false,
      open: true, systemTitle: false, external: true, externalPath: 'C:/notes/test.md',
      externalRevision: 'old-revision',
    }
    const tabs = ref([tab])
    const activeTabId = ref(tab.id)
    const rememberExternalDocument = vi.fn()
    vi.mocked(readExternalMarkdown).mockResolvedValue({
      path: tab.externalPath!, title: tab.title, fileName: tab.fileName,
      content: '# changed externally', updatedAt: 2, revision: 'fresh-revision',
    })
    const session = useDocumentSession({
      tabs,
      activeTabId,
      activeTab: computed(() => tabs.value[0]),
      statusMessage: ref(''),
      failedMessage: () => 'failed',
      rememberExternalDocument,
    })

    expect(await session.loadActiveNote()).toBe(true)
    expect(tab.externalRevision).toBe('fresh-revision')
    expect(tab.updatedAt).toBe(2)
    expect(session.content.value).toBe('# changed externally')
    expect(rememberExternalDocument).toHaveBeenCalledWith(tab)
    session.disposeDocumentSession()
  })

  it('uses ordinary note revision checks for project documents', async () => {
    const tab: NoteTab = {
      id: 'project-review', title: 'Review', fileName: 'Review.md', projectRelativePath: 'Demo/Review.md',
      createdAt: 1, updatedAt: 1, pinned: false, deleted: false, archived: false, open: true, systemTitle: false,
    }
    const tabs = ref([tab])
    vi.mocked(readNote).mockResolvedValue({ id: tab.id, title: tab.title, fileName: tab.fileName, content: 'Original', updatedAt: 2 })
    vi.mocked(writeNote).mockResolvedValue({ id: tab.id, title: tab.title, fileName: tab.fileName, content: 'Edited', updatedAt: 3 })
    const session = useDocumentSession({ tabs, activeTabId: ref(tab.id), activeTab: computed(() => tabs.value[0]),
      statusMessage: ref(''), failedMessage: () => 'failed', rememberExternalDocument: vi.fn() })
    expect(await session.loadActiveNote()).toBe(true)
    session.content.value = 'Edited'
    await nextTick()
    expect(await session.forceSave()).toBe(true)
    expect(writeNote).toHaveBeenCalledWith(tab.id, 'Edited', 2)
    expect(tab.updatedAt).toBe(3)
    session.disposeDocumentSession()
  })
  function noteSession() {
    const tab: NoteTab = {
      id: 'inbox', title: 'Inbox', fileName: 'inbox.md', createdAt: 1, updatedAt: 1,
      pinned: true, deleted: false, archived: false, open: true, systemTitle: true,
    }
    const tabs = ref([tab])
    const activeTabId = ref(tab.id)
    return { tab, activeTabId, session: useDocumentSession({
      tabs, activeTabId, activeTab: computed(() => tabs.value.find((item) => item.id === activeTabId.value)),
      statusMessage: ref(''), failedMessage: () => 'failed', rememberExternalDocument: vi.fn(),
    }) }
  }

  it('refreshes saved content changed by a local agent without saving it back', async () => {
    const { tab, session } = noteSession()
    vi.mocked(readNote).mockResolvedValue({ id: 'inbox', title: 'Inbox', fileName: 'inbox.md', content: 'agent update', updatedAt: 2 })
    expect(await session.refreshActiveNote()).toBe(true)
    expect(session.content.value).toBe('agent update')
    expect(tab.updatedAt).toBe(2)
    await nextTick()
    expect(session.saveState.value).toBe('Saved')
    expect(await session.forceSave()).toBe(true)
    expect(writeNote).not.toHaveBeenCalled()
    session.disposeDocumentSession()
  })

  it('preserves typing that starts while an external refresh is in flight', async () => {
    const { tab, session } = noteSession()
    let finish!: (value: Awaited<ReturnType<typeof readNote>>) => void
    vi.mocked(readNote).mockReturnValue(new Promise((resolve) => { finish = resolve }))
    const refreshing = session.refreshActiveNote()
    session.content.value = 'unsaved user edits'
    await nextTick()
    finish({ id: 'inbox', title: 'Inbox', fileName: 'inbox.md', content: 'agent update', updatedAt: 2 })
    expect(await refreshing).toBe(false)
    expect(session.content.value).toBe('unsaved user edits')
    expect(tab.updatedAt).toBe(1)
    expect(session.saveState.value).toBe('Saving')
    session.disposeDocumentSession()
  })

  it('discards a refresh result after switching documents', async () => {
    const { tab, activeTabId, session } = noteSession()
    let finish!: (value: Awaited<ReturnType<typeof readNote>>) => void
    vi.mocked(readNote).mockReturnValue(new Promise((resolve) => { finish = resolve }))
    const refreshing = session.refreshActiveNote()
    activeTabId.value = 'another'
    finish({ id: 'inbox', title: 'Inbox', fileName: 'inbox.md', content: 'agent update', updatedAt: 2 })
    expect(await refreshing).toBe(false)
    expect(tab.updatedAt).toBe(1)
    expect(session.content.value).toBe('# Inbox\n\n')
    session.disposeDocumentSession()
  })

})
