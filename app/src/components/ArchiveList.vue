<script setup lang="ts">
import type { NoteTab } from '../types/note'

defineProps<{
  notes: NoteTab[]
  loading: boolean
  messages: {
    title: string
    close: string
    refresh: string
    empty: string
    restore: string
  }
}>()

defineEmits<{
  close: []
  refresh: []
  restore: [note: NoteTab]
}>()
</script>

<template>
  <section class="archive-list-panel" role="dialog" aria-modal="true" :aria-label="messages.title">
    <header class="archive-list-header">
      <strong>{{ messages.title }}</strong>
      <div class="archive-toolbar">
        <button class="archive-button" type="button" :disabled="loading" @click="$emit('refresh')">
          {{ messages.refresh }}
        </button>
        <button class="archive-button" type="button" @click="$emit('close')">{{ messages.close }}</button>
      </div>
    </header>
    <div class="archive-list-body">
      <p v-if="!loading && notes.length === 0" class="archive-list-empty">{{ messages.empty }}</p>
      <ul v-else class="archive-note-list">
        <li v-for="note in notes" :key="note.id">
          <span :title="note.title">{{ note.title }}</span>
          <button class="archive-restore-button" type="button" :disabled="loading" @click="$emit('restore', note)">
            {{ messages.restore }}
          </button>
        </li>
      </ul>
    </div>
  </section>
</template>
