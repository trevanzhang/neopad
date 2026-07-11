<script setup lang="ts">
import { ref } from 'vue'
import type { NoteTab } from '../types/note'

defineProps<{
  notes: NoteTab[]
  archivedNotes: NoteTab[]
  activeNoteId: string
  loading: boolean
  messages: {
    title: string
    notes: string
    archive: string
    emptyNotes: string
    emptyArchive: string
    newNote: string
    refresh: string
    restore: string
  }
}>()

defineEmits<{
  select: [noteId: string]
  restore: [note: NoteTab]
  newNote: []
  refresh: []
}>()

const notesExpanded = ref(true)
const archiveExpanded = ref(true)
</script>

<template>
  <aside class="note-library" aria-label="Note library">
    <header class="note-library-header">
      <strong>{{ messages.title }}</strong>
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
            :class="{ active: note.id === activeNoteId }"
            type="button"
            :title="note.fileName"
            @click="$emit('select', note.id)"
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
          <div v-for="note in archivedNotes" :key="note.id" class="note-library-entry archived" :title="note.fileName">
            <svg class="note-library-file" aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM9 1.75v3h3M5.5 8h5M5.5 10.5h5" /></svg>
            <span>{{ note.title }}</span>
            <button type="button" :title="messages.restore" :aria-label="messages.restore" @click="$emit('restore', note)">
              <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M8 12.75v-8M5 7.75l3-3 3 3M3 3.25h10v10.5H3" /></svg>
            </button>
          </div>
        </div>
      </section>
    </div>
  </aside>
</template>
