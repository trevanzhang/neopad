import { computed, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { messages } from '../lib/i18n'
import { createNote, createNoteWithBody } from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'
import type { NoteTab } from '../types/note'
import { useNoteLifecycle } from './useNoteLifecycle'

vi.mock('../lib/invoke', () => ({
  archiveNote: vi.fn(),
  closeNote: vi.fn(),
  createNote: vi.fn(),
  createNoteWithBody: vi.fn(),
  deleteNote: vi.fn(),
  renameNote: vi.fn(),
  saveClipboard: vi.fn(),
  setNoteColor: vi.fn(),
  unarchiveNote: vi.fn(),
  writeNote: vi.fn(),
}))
vi.mock('../lib/runtime', () => ({ isTauriRuntime: vi.fn(() => false) }))

function createHarness(forceSave = vi.fn(async () => true)) {
  const now = Date.now()
  const tabs = ref<NoteTab[]>([
    { id: 'inbox', title: 'Inbox', fileName: 'inbox.md', createdAt: now, updatedAt: now,
      pinned: true, deleted: false, archived: false, open: true, systemTitle: false },
    { id: 'other', title: 'Other', fileName: 'other.md', createdAt: now, updatedAt: now,
      pinned: false, deleted: false, archived: false, open: true, systemTitle: false },
  ])
  const activeTabId = ref('inbox')
  const content = ref('# Inbox')
  let generation = 0
  const lifecycle = useNoteLifecycle({
    tabs,
    activeTabId,
    activeTab: computed(() => tabs.value.find((tab) => tab.id === activeTabId.value)),
    content,
    saveState: ref<'Saved' | 'Saving' | 'Failed'>('Saved'),
    statusMessage: ref(''),
    language: ref('en'),
    titleDoubleClickAction: ref('rename'),
    archiveListOpen: ref(false),
    text: () => messages.en,
    forceSave,
    nextNoteLoadGeneration: () => ++generation,
    isCurrentNoteLoad: (candidate) => candidate === generation,
    loadActiveNote: vi.fn(async () => true),
    setContentFromLoad: vi.fn(),
    requestInput: vi.fn(async () => null),
    focusEditor: vi.fn(),
    refreshRecentNotes: vi.fn(async () => undefined),
    refreshArchivedNotes: vi.fn(async () => undefined),
    refreshLibrary: vi.fn(async () => undefined),
    upsertTab: (tab) => tabs.value.push(tab),
  })
  return { lifecycle, tabs, activeTabId, content }
}

describe('useNoteLifecycle', () => {
  beforeEach(() => {
    vi.mocked(isTauriRuntime).mockReturnValue(false)
    vi.mocked(createNote).mockReset()
    vi.mocked(createNoteWithBody).mockReset()
  })

  it('does not switch tabs when the save barrier fails', async () => {
    const { lifecycle, activeTabId } = createHarness(vi.fn(async () => false))
    await lifecycle.selectTab('other')
    expect(activeTabId.value).toBe('inbox')
  })

  it('creates an isolated browser draft without touching pinned notes', async () => {
    const { lifecycle, tabs, activeTabId } = createHarness()
    await lifecycle.createLocalTab()
    expect(tabs.value).toHaveLength(3)
    expect(activeTabId.value).toMatch(/^draft-/)
    expect(tabs.value[0].id).toBe('inbox')
  })

  it('does not create an unsavable browser draft when native creation fails', async () => {
    vi.mocked(isTauriRuntime).mockReturnValue(true)
    vi.mocked(createNote).mockRejectedValue(new Error('disk full'))
    const { lifecycle, tabs, activeTabId } = createHarness()

    await lifecycle.createLocalTab()

    expect(tabs.value).toHaveLength(2)
    expect(activeTabId.value).toBe('inbox')
  })

  it('creates a browser draft with selected text in the note body', async () => {
    const { lifecycle, tabs, content } = createHarness()

    await lifecycle.createLocalTab('Selected text')

    expect(tabs.value).toHaveLength(3)
    expect(tabs.value.at(-1)?.title).toBe('Untitled')
    expect(content.value).toBe('# Untitled\n\nSelected text')
  })

  it('uses the atomic native create-with-body command for selected text', async () => {
    vi.mocked(isTauriRuntime).mockReturnValue(true)
    vi.mocked(createNoteWithBody).mockResolvedValue({
      id: 'page-selected',
      title: 'Untitled',
      fileName: 'page-selected.md',
      content: '# Untitled\n\nSelected text',
      updatedAt: 10,
    })
    const { lifecycle, activeTabId } = createHarness()

    await lifecycle.createLocalTab('Selected text')

    expect(createNoteWithBody).toHaveBeenCalledWith('Selected text')
    expect(activeTabId.value).toBe('page-selected')
  })
})
