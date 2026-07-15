import { ref } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { messages } from '../lib/i18n'
import { copyPngToClipboard, saveNoteExport } from '../lib/invoke'
import { createNoteExportBlob } from '../lib/note-export'
import type { NoteTab } from '../types/note'
import { useNoteExport } from './useNoteExport'

vi.mock('../lib/invoke', () => ({
  copyPngToClipboard: vi.fn(),
  readAiPrompt: vi.fn(),
  readExternalMarkdown: vi.fn(),
  readNote: vi.fn(),
  saveNoteExport: vi.fn(),
}))
vi.mock('../lib/note-export', () => ({ createNoteExportBlob: vi.fn() }))
vi.mock('../lib/runtime', () => ({ isTauriRuntime: vi.fn(() => true) }))

const tab: NoteTab = {
  id: 'inbox',
  title: 'Inbox',
  fileName: 'inbox.md',
  createdAt: 1,
  updatedAt: 1,
  pinned: true,
  deleted: false,
  archived: false,
  open: true,
  systemTitle: false,
}

function createHarness() {
  const statusMessage = ref('Saved')
  const exporter = useNoteExport({
    tabs: ref([tab]),
    activeTabId: ref(tab.id),
    content: ref('# Inbox'),
    isLoadingNote: ref(false),
    statusMessage,
    forceSave: vi.fn(async () => true),
    text: () => messages.en.status,
    safeFileName: (title) => title,
  })
  return { exporter, statusMessage }
}

describe('useNoteExport', () => {
  const writeClipboard = vi.fn()

  beforeEach(() => {
    writeClipboard.mockReset()
    vi.stubGlobal('navigator', { clipboard: { write: writeClipboard } })
    vi.stubGlobal('ClipboardItem', class {
      constructor(public items: Record<string, Blob>) {}
    })
    vi.mocked(copyPngToClipboard).mockReset()
    vi.mocked(saveNoteExport).mockReset()
    vi.mocked(createNoteExportBlob).mockReset()
    vi.mocked(createNoteExportBlob).mockResolvedValue(new Blob([new Uint8Array([1, 2, 3])], { type: 'image/png' }))
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('copies rendered PNG through the Web Clipboard API', async () => {
    const { exporter, statusMessage } = createHarness()

    await exporter.exportNote(tab.id, 'png', 'clipboard')

    expect(writeClipboard).toHaveBeenCalledOnce()
    expect(copyPngToClipboard).not.toHaveBeenCalled()
    expect(saveNoteExport).not.toHaveBeenCalled()
    expect(statusMessage.value).toBe('PNG copied to clipboard')
  })

  it('falls back to the native clipboard when the Web Clipboard API is unavailable', async () => {
    vi.stubGlobal('navigator', { clipboard: {} })
    const { exporter, statusMessage } = createHarness()

    await exporter.exportNote(tab.id, 'png', 'clipboard')

    expect(copyPngToClipboard).toHaveBeenCalledWith(new Uint8Array([1, 2, 3]))
    expect(writeClipboard).not.toHaveBeenCalled()
    expect(statusMessage.value).toBe('PNG copied to clipboard')
  })

  it('keeps PNG file export on the native save path', async () => {
    vi.mocked(saveNoteExport).mockResolvedValue(true)
    const { exporter, statusMessage } = createHarness()

    await exporter.exportNote(tab.id, 'png', 'file')

    expect(saveNoteExport).toHaveBeenCalledWith('Inbox.png', 'png', new Uint8Array([1, 2, 3]))
    expect(copyPngToClipboard).not.toHaveBeenCalled()
    expect(statusMessage.value).toBe('PNG exported')
  })
})
