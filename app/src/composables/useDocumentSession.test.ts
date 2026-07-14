import { computed, nextTick, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { readAiPrompt, readExternalMarkdown, writeAiPrompt } from '../lib/invoke'
import type { NoteTab } from '../types/note'
import { useDocumentSession } from './useDocumentSession'

vi.mock('../lib/invoke', () => ({
  readAiPrompt: vi.fn(),
  readExternalMarkdown: vi.fn(),
  readNote: vi.fn(),
  writeAiPrompt: vi.fn(),
  writeExternalMarkdown: vi.fn(),
  writeNote: vi.fn(),
}))
vi.mock('../lib/runtime', () => ({ isTauriRuntime: vi.fn(() => true) }))

describe('useDocumentSession', () => {
  beforeEach(() => {
    vi.mocked(readAiPrompt).mockReset()
    vi.mocked(readExternalMarkdown).mockReset()
    vi.mocked(writeAiPrompt).mockReset()
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

  it('loads and saves a prompt tab through the prompt document API', async () => {
    const tab: NoteTab = {
      id: 'prompt:Review.md', title: 'Review', fileName: 'Review.md',
      createdAt: 1, updatedAt: 1, pinned: false, deleted: false, archived: false,
      open: true, systemTitle: false, kind: 'prompt', promptId: 'Review.md',
      promptRevision: 'old-revision',
    }
    const tabs = ref([tab])
    const activeTabId = ref(tab.id)
    const onPromptSaved = vi.fn()
    vi.mocked(readAiPrompt).mockResolvedValue({
      id: 'Review.md', name: 'Review', fileName: 'Review.md',
      content: 'Review carefully.', updatedAt: 2, revision: 'loaded-revision',
    })
    vi.mocked(writeAiPrompt).mockResolvedValue({
      id: 'Review.md', name: 'Review', fileName: 'Review.md',
      content: 'Review for correctness.', updatedAt: 3, revision: 'saved-revision',
    })
    const session = useDocumentSession({
      tabs,
      activeTabId,
      activeTab: computed(() => tabs.value[0]),
      statusMessage: ref(''),
      failedMessage: () => 'failed',
      rememberExternalDocument: vi.fn(),
      onPromptSaved,
    })

    expect(await session.loadActiveNote()).toBe(true)
    expect(session.content.value).toBe('Review carefully.')
    expect(tab.promptRevision).toBe('loaded-revision')

    session.content.value = 'Review for correctness.'
    await nextTick()
    expect(await session.forceSave()).toBe(true)
    expect(writeAiPrompt).toHaveBeenCalledWith('Review.md', 'Review for correctness.', 'loaded-revision')
    expect(tab.promptRevision).toBe('saved-revision')
    expect(onPromptSaved).toHaveBeenCalledOnce()
    session.disposeDocumentSession()
  })
})
