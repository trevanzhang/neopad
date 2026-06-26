<script setup lang="ts">
import type { SearchResult } from '../types/note'

defineProps<{
  query: string
  results: SearchResult[]
  searching: boolean
}>()

defineEmits<{
  'update:query': [query: string]
  close: []
  select: [result: SearchResult]
}>()
</script>

<template>
  <aside class="search-panel" aria-label="Search">
    <header class="search-header">
      <input
        :value="query"
        type="search"
        placeholder="Search notes"
        autofocus
        @input="$emit('update:query', ($event.target as HTMLInputElement).value)"
      />
      <button type="button" aria-label="Close search" title="Close" @click="$emit('close')">Close</button>
    </header>

    <div class="search-results">
      <p v-if="searching" class="search-empty">Searching...</p>
      <p v-else-if="query.trim() && results.length === 0" class="search-empty">No results</p>
      <button
        v-for="result in results"
        :key="`${result.noteId}-${result.lineNumber}-${result.lineText}`"
        type="button"
        class="search-result"
        @click="$emit('select', result)"
      >
        <span class="search-result-meta">{{ result.title }} · line {{ result.lineNumber }}</span>
        <span v-for="line in result.before" :key="`before-${line}`" class="search-context">{{ line }}</span>
        <strong>{{ result.lineText }}</strong>
        <span v-for="line in result.after" :key="`after-${line}`" class="search-context">{{ line }}</span>
      </button>
    </div>
  </aside>
</template>
