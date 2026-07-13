<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import type { NoteTab } from '../types/note'

const props = defineProps<{
  notes: NoteTab[]
  archivedNotes: NoteTab[]
  trashedNotes: NoteTab[]
  activeNoteId: string
  loading: boolean
  messages: {
    title: string
    notes: string
    archive: string
    trash: string
    emptyNotes: string
    emptyArchive: string
    emptyTrash: string
    newNote: string
    refresh: string
    restore: string
    rename: string
    archiveAction: string
    delete: string
    clearTrash: string
    revealInFileManager: string
    help: string
  }
}>()

const emit = defineEmits<{
  select: [noteId: string]
  restore: [notes: NoteTab[]]
  restoreTrash: [notes: NoteTab[]]
  rename: [notes: NoteTab[]]
  archive: [notes: NoteTab[]]
  delete: [notes: NoteTab[]]
  reveal: [note: NoteTab]
  clearTrash: []
  newNote: []
  refresh: []
}>()

const notesExpanded = ref(true)
const archiveExpanded = ref(true)
const trashExpanded = ref(true)
type LibraryGroup = 'notes' | 'archive' | 'trash'
const selectedNoteIds = ref(new Set<string>())
const selectionGroup = ref<LibraryGroup | null>(null)
const selectionAnchorId = ref<string | null>(null)
const contextMenu = ref<{ notes: NoteTab[]; group: LibraryGroup; x: number; y: number } | null>(null)
const contextMenuElement = ref<HTMLElement | null>(null)

onMounted(() => {
  window.addEventListener('pointerdown', closeContextMenu)
  window.addEventListener('keydown', closeContextMenuOnEscape)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', closeContextMenu)
  window.removeEventListener('keydown', closeContextMenuOnEscape)
})

function entriesForGroup(group: LibraryGroup) {
  if (group === 'notes') return props.notes
  if (group === 'archive') return props.archivedNotes
  return props.trashedNotes
}

function isSelected(note: NoteTab, group: LibraryGroup) {
  return selectionGroup.value === group && selectedNoteIds.value.has(note.id)
}

function selectEntry(event: MouseEvent, note: NoteTab, group: LibraryGroup) {
  const entries = entriesForGroup(group)
  const next = new Set<string>()
  if (event.shiftKey && selectionGroup.value === group && selectionAnchorId.value) {
    const start = entries.findIndex((item) => item.id === selectionAnchorId.value)
    const end = entries.findIndex((item) => item.id === note.id)
    if (start >= 0 && end >= 0) {
      for (const item of entries.slice(Math.min(start, end), Math.max(start, end) + 1)) next.add(item.id)
    }
  } else if ((event.ctrlKey || event.metaKey) && selectionGroup.value === group) {
    for (const id of selectedNoteIds.value) next.add(id)
    if (next.has(note.id)) next.delete(note.id)
    else next.add(note.id)
  } else {
    next.add(note.id)
    selectionAnchorId.value = note.id
  }
  selectedNoteIds.value = next
  selectionGroup.value = group
  if (group === 'notes' && !event.shiftKey && !(event.ctrlKey || event.metaKey)) {
    emit('select', note.id)
  }
}

function openContextMenu(event: MouseEvent, note: NoteTab, group: LibraryGroup) {
  event.preventDefault()
  if (!isSelected(note, group)) {
    selectedNoteIds.value = new Set([note.id])
    selectionGroup.value = group
    selectionAnchorId.value = note.id
  }
  const selected = entriesForGroup(group).filter((item) => selectedNoteIds.value.has(item.id))
  contextMenu.value = {
    notes: selected,
    group,
    x: Math.min(event.clientX, window.innerWidth - 176),
    y: Math.min(event.clientY, window.innerHeight - (group === 'notes' ? 136 : 88)),
  }
  void nextTick(() => contextMenuElement.value?.querySelector<HTMLButtonElement>('button')?.focus())
}

function closeContextMenu(event?: Event) {
  if (event && contextMenuElement.value?.contains(event.target as Node)) return
  contextMenu.value = null
}

function closeContextMenuOnEscape(event: KeyboardEvent) {
  if (event.key !== 'Escape' || !contextMenu.value) return
  event.preventDefault()
  contextMenu.value = null
}

function runContextAction(action: 'rename' | 'archive' | 'delete' | 'restore' | 'reveal') {
  const menu = contextMenu.value
  if (!menu) return
  contextMenu.value = null
  if (action === 'reveal') {
    const note = menu.notes[0]
    if (note) emit('reveal', note)
  } else if (action === 'rename') emit('rename', menu.notes)
  else if (action === 'archive') emit('archive', menu.notes)
  else if (action === 'delete') emit('delete', menu.notes)
  else if (menu.group === 'trash') emit('restoreTrash', menu.notes)
  else emit('restore', menu.notes)
}
</script>

<template>
  <aside class="note-library" aria-label="Note library">
    <header class="note-library-header">
      <strong>{{ messages.title }}</strong>
      <div class="note-library-help">
        <button type="button" :aria-label="messages.help">
          <svg aria-hidden="true" viewBox="0 0 16 16"><circle cx="8" cy="8" r="5.75" /><path d="M6.7 6.25a1.35 1.35 0 1 1 2.2 1.06c-.83.63-1.27.95-1.27 1.94M8 11.65v.1" /></svg>
        </button>
        <span role="tooltip">{{ messages.help }}</span>
      </div>
      <div class="note-library-actions">
        <button type="button" :title="messages.refresh" :aria-label="messages.refresh" :disabled="loading" @click="$emit('refresh')">
          <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M13.2 7.1A5.4 5.4 0 1 0 13 10.6M13.2 2.8v4.5H8.7" /></svg>
        </button>
        <button type="button" :title="messages.newNote" :aria-label="messages.newNote" @click="$emit('newNote')">
          <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M8 3.25v9.5M3.25 8h9.5" /></svg>
        </button>
      </div>
    </header>

    <div class="note-library-body" :aria-busy="loading">
      <section class="note-library-group">
        <button class="note-library-root" type="button" @click="notesExpanded = !notesExpanded">
          <svg class="note-library-chevron" :class="{ collapsed: !notesExpanded }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
          <svg class="note-library-folder" aria-hidden="true" viewBox="0 0 16 16"><path d="M1.5 4.25h4.7l1.3 1.5h7v5.8a1 1 0 0 1-1 1H2.5a1 1 0 0 1-1-1Z" /></svg>
          <span>{{ messages.notes }}</span>
          <small>{{ notes.length }}</small>
        </button>
        <div v-if="notesExpanded" class="note-library-entries">
          <p v-if="!loading && notes.length === 0" class="note-library-empty">{{ messages.emptyNotes }}</p>
          <button
            v-for="note in notes"
            :key="note.id"
            class="note-library-entry"
            :class="{ active: note.id === activeNoteId, selected: isSelected(note, 'notes') }"
            type="button"
            :title="note.fileName"
            @click="selectEntry($event, note, 'notes')"
            @contextmenu="openContextMenu($event, note, 'notes')"
          >
            <svg class="note-library-file" aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM9 1.75v3h3M5.5 8h5M5.5 10.5h5" /></svg>
            <span>{{ note.title }}</span>
          </button>
        </div>
      </section>

      <section class="note-library-group">
        <button class="note-library-root" type="button" @click="archiveExpanded = !archiveExpanded">
          <svg class="note-library-chevron" :class="{ collapsed: !archiveExpanded }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
          <svg class="note-library-folder archive" aria-hidden="true" viewBox="0 0 16 16"><path d="M1.5 4.25h4.7l1.3 1.5h7v5.8a1 1 0 0 1-1 1H2.5a1 1 0 0 1-1-1Z" /></svg>
          <span>{{ messages.archive }}</span>
          <small>{{ archivedNotes.length }}</small>
        </button>
        <div v-if="archiveExpanded" class="note-library-entries">
          <p v-if="!loading && archivedNotes.length === 0" class="note-library-empty">{{ messages.emptyArchive }}</p>
          <button v-for="note in archivedNotes" :key="note.id" class="note-library-entry archived" :class="{ selected: isSelected(note, 'archive') }" type="button" :title="note.fileName" @click="selectEntry($event, note, 'archive')" @contextmenu="openContextMenu($event, note, 'archive')">
            <svg class="note-library-file" aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM9 1.75v3h3M5.5 8h5M5.5 10.5h5" /></svg>
            <span>{{ note.title }}</span>
          </button>
        </div>
      </section>

      <section class="note-library-group">
        <div class="note-library-root note-library-trash-root">
          <button class="note-library-root-toggle" type="button" @click="trashExpanded = !trashExpanded">
          <svg class="note-library-chevron" :class="{ collapsed: !trashExpanded }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
          <svg class="note-library-folder trash" aria-hidden="true" viewBox="0 0 16 16"><path d="M3.25 4.5h9.5M6 4.5V3h4v1.5M4.25 4.5l.65 8.75h6.2l.65-8.75M6.5 7v3.75M9.5 7v3.75" /></svg>
          <span>{{ messages.trash }}</span>
          </button>
          <button class="note-library-clear-trash" type="button" :title="messages.clearTrash" :aria-label="messages.clearTrash" :disabled="trashedNotes.length === 0" @click="$emit('clearTrash')">
            <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M9.8 2.7 13 5.9M8.3 4.2l3.2 3.2M3.1 12.9l4.1-4.1 3.2 3.2-1 1Zm.1.1h8.6M5.4 10.6l1.1 1.1" /></svg>
          </button>
          <small>{{ trashedNotes.length }}</small>
        </div>
        <div v-if="trashExpanded" class="note-library-entries">
          <p v-if="!loading && trashedNotes.length === 0" class="note-library-empty">{{ messages.emptyTrash }}</p>
          <button v-for="note in trashedNotes" :key="note.id" class="note-library-entry trashed" :class="{ selected: isSelected(note, 'trash') }" type="button" :title="note.fileName" @click="selectEntry($event, note, 'trash')" @contextmenu="openContextMenu($event, note, 'trash')">
            <svg class="note-library-file" aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM9 1.75v3h3M5.5 8h5M5.5 10.5h5" /></svg>
            <span>{{ note.title }}</span>
          </button>
        </div>
      </section>
    </div>
    <div
      v-if="contextMenu"
      ref="contextMenuElement"
      class="note-library-context-menu"
      role="menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @contextmenu.prevent
    >
      <button
        v-if="contextMenu.notes.length === 1"
        type="button"
        role="menuitem"
        @click="runContextAction('reveal')"
      >
        {{ messages.revealInFileManager }}
      </button>
      <template v-if="contextMenu.group !== 'notes'">
        <button type="button" role="menuitem" @click="runContextAction('restore')">{{ messages.restore }}</button>
      </template>
      <template v-else>
        <button v-if="contextMenu.notes.length === 1" type="button" role="menuitem" @click="runContextAction('rename')">{{ messages.rename }}</button>
        <button type="button" role="menuitem" @click="runContextAction('archive')">{{ messages.archiveAction }}</button>
        <button class="danger" type="button" role="menuitem" @click="runContextAction('delete')">{{ messages.delete }}</button>
      </template>
    </div>
  </aside>
</template>
