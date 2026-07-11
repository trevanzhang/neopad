import { computed, ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { readExternalMarkdown } from '../lib/invoke'
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
  beforeEach(() => vi.mocked(readExternalMarkdown).mockReset())

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
})
