import { ref } from 'vue'
import { listArchivedNotes } from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'
import type { NoteTab } from '../types/note'

export function useArchiveState(forceSave: () => Promise<boolean>, onError: () => void) {
  const archiveListOpen = ref(false)
  const archivedNotes = ref<NoteTab[]>([])
  const archiveLoading = ref(false)

  async function refreshArchivedNotes() {
    if (!isTauriRuntime()) return
    archiveLoading.value = true
    try {
      if (!(await forceSave())) return
      archivedNotes.value = await listArchivedNotes()
    } catch {
      onError()
    } finally {
      archiveLoading.value = false
    }
  }

  return { archiveListOpen, archivedNotes, archiveLoading, refreshArchivedNotes }
}
