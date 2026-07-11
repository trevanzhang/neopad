import { computed, ref } from 'vue'
import { describe, expect, it, vi } from 'vitest'
import { messages } from '../lib/i18n'
import type { NoteTab } from '../types/note'
import { useNoteLifecycle } from './useNoteLifecycle'

function createHarness(forceSave = vi.fn(async () => true)) {
  const now = Date.now()
  const tabs = ref<NoteTab[]>([
    { id: 'inbox', title: 'Inbox', fileName: 'inbox.md', createdAt: now, updatedAt: now,
      pinned: true, deleted: false, archived: false, open: true, systemTitle: false },
    { id: 'other', title: 'Other', fileName: 'other.md', createdAt: now, updatedAt: now,
      pinned: false, deleted: false, archived: false, open: true, systemTitle: false },
  ])
  const activeTabId = ref('inbox')
  let generation = 0
  const lifecycle = useNoteLifecycle({
    tabs,
    activeTabId,
    activeTab: computed(() => tabs.value.find((tab) => tab.id === activeTabId.value)),
    content: ref('# Inbox'),
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
    requestConfirmation: vi.fn(async () => false),
    focusEditor: vi.fn(),
    refreshRecentNotes: vi.fn(async () => undefined),
    refreshArchivedNotes: vi.fn(async () => undefined),
    upsertTab: (tab) => tabs.value.push(tab),
  })
  return { lifecycle, tabs, activeTabId }
}

describe('useNoteLifecycle', () => {
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
})
