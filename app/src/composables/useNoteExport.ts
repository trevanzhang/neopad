import { ref, type Ref } from 'vue'
import type { AppMessages } from '../lib/i18n'
import { copyPngToClipboard, readAiPrompt, readExternalMarkdown, readNote, saveNoteExport } from '../lib/invoke'
import { isExternalTab, isPromptTab } from '../lib/document-tab'
import { isTauriRuntime } from '../lib/runtime'
import type { NoteExportFormat } from '../lib/note-export'
import type { NoteTab } from '../types/note'

interface NoteExportOptions {
  tabs: Ref<NoteTab[]>
  activeTabId: Ref<string>
  content: Ref<string>
  isLoadingNote: Ref<boolean>
  statusMessage: Ref<string>
  forceSave: () => Promise<boolean>
  text: () => AppMessages['status']
  safeFileName: (title: string) => string
}

export function useNoteExport(options: NoteExportOptions) {
  const exportingNote = ref(false)

  async function exportNote(
    tabId: string,
    format: NoteExportFormat,
    destination: 'file' | 'clipboard' = 'file',
  ) {
    if (exportingNote.value) return
    const tab = options.tabs.value.find((item) => item.id === tabId)
    if (!tab) return

    const previousStatus = options.statusMessage.value
    exportingNote.value = true
    options.statusMessage.value = options.text().exportingNote
    try {
      const source = await noteSource(tab)
      if (source === null) {
        options.statusMessage.value = previousStatus
        return
      }
      const { createNoteExportBlob } = await import('../lib/note-export')
      const blob = await createNoteExportBlob(source, format)
      if (destination === 'clipboard') {
        if (format !== 'png') throw new Error('NOTE_EXPORT_CLIPBOARD_FORMAT')
        await copyPngBlobToClipboard(blob)
        options.statusMessage.value = options.text().copiedPng
        return
      }
      const fileName = `${options.safeFileName(tab.title || 'Untitled')}.${format}`
      const saved = isTauriRuntime()
        ? await saveNoteExport(fileName, format, new Uint8Array(await blob.arrayBuffer()))
        : downloadBlob(blob, fileName)
      if (!saved) {
        options.statusMessage.value = previousStatus
        return
      }
      options.statusMessage.value = format === 'png'
        ? options.text().exportedPng
        : options.text().exportedPdf
    } catch (error) {
      options.statusMessage.value = error instanceof Error && error.message === 'NOTE_EXPORT_TOO_LONG'
        ? options.text().exportTooLong
        : options.text().exportFailed
    } finally {
      exportingNote.value = false
    }
  }

  async function noteSource(tab: NoteTab): Promise<string | null> {
    const activeAndReady = tab.id === options.activeTabId.value && !options.isLoadingNote.value
    if (activeAndReady) {
      if (!(await options.forceSave())) return null
      return options.content.value
    }
    if (!isTauriRuntime()) return options.content.value
    if (isPromptTab(tab) && tab.promptId) {
      return (await readAiPrompt(tab.promptId)).content
    }
    if (isExternalTab(tab) && tab.externalPath) {
      return (await readExternalMarkdown(tab.externalPath)).content
    }
    return (await readNote(tab.id)).content
  }

  function downloadBlob(blob: Blob, fileName: string) {
    const url = URL.createObjectURL(blob)
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = fileName
    document.body.appendChild(anchor)
    anchor.click()
    anchor.remove()
    window.setTimeout(() => URL.revokeObjectURL(url), 0)
    return true
  }

  async function copyPngBlobToClipboard(blob: Blob) {
    if (navigator.clipboard?.write && typeof ClipboardItem !== 'undefined') {
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
      return
    }
    if (isTauriRuntime()) {
      await copyPngToClipboard(new Uint8Array(await blob.arrayBuffer()))
      return
    }
    throw new Error('NOTE_EXPORT_CLIPBOARD_UNAVAILABLE')
  }

  return { exportingNote, exportNote }
}
