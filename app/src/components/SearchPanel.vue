<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import type { AppMessages } from '../lib/i18n'
import type { SearchResult } from '../types/note'
import { groupSearchResults, highlightSearchText } from '../lib/search'

const props = defineProps<{
  query: string
  results: SearchResult[]
  searching: boolean
  hasMore: boolean
  messages: AppMessages['search']
}>()

const emit = defineEmits<{
  'update:query': [query: string]
  close: []
  select: [result: SearchResult]
  loadMore: []
}>()

const searchInput = ref<HTMLInputElement | null>(null)
const expandedNoteIds = ref<Set<string>>(new Set())
const groupedResults = computed(() => groupSearchResults(props.results, props.query))
const matchCount = computed(() => props.results.length)

watch(() => props.query, () => {
  expandedNoteIds.value = new Set()
})

onMounted(() => {
  void nextTick(focusSearchInput)
})

function focusSearchInput() {
  searchInput.value?.focus()
  searchInput.value?.select()
}

function toggleNote(noteId: string) {
  const next = new Set(expandedNoteIds.value)
  if (next.has(noteId)) next.delete(noteId)
  else next.add(noteId)
  expandedNoteIds.value = next
}

function visibleMatches(noteId: string, matches: SearchResult[]) {
  return expandedNoteIds.value.has(noteId) ? matches : matches.slice(0, 2)
}

function remainingMatchCount(noteId: string, matches: SearchResult[]) {
  return Math.max(0, matches.length - visibleMatches(noteId, matches).length)
}

function formatMessage(template: string, values: Record<string, number>) {
  return template.replace(/\{(\w+)\}/g, (_, key: string) => String(values[key] ?? ''))
}

defineExpose({
  focusSearchInput,
})
</script>

<template>
  <aside class="search-panel" :aria-label="messages.title">
    <header class="search-header">
      <input
        ref="searchInput"
        :value="query"
        type="search"
        :placeholder="messages.placeholder"
        @input="$emit('update:query', ($event.target as HTMLInputElement).value)"
      />
      <button type="button" :aria-label="messages.close" :title="messages.close" @click="$emit('close')">
        {{ messages.close }}
      </button>
    </header>

    <div class="search-results">
      <p v-if="searching" class="search-empty">{{ messages.searching }}</p>
      <p v-else-if="query.trim() && results.length === 0" class="search-empty">{{ messages.noResults }}</p>
      <template v-else-if="results.length > 0">
        <p class="search-summary">
          {{ formatMessage(messages.summary, { notes: groupedResults.length, matches: matchCount }) }}
        </p>
        <section v-for="group in groupedResults" :key="group.noteId" class="search-note-group">
          <button type="button" class="search-note-header" @click="toggleNote(group.noteId)">
            <strong>{{ group.title }}</strong>
            <span>{{ formatMessage(messages.matchCount, { count: group.matches.length }) }}</span>
          </button>
          <button
            v-for="result in visibleMatches(group.noteId, group.matches)"
            :key="`${result.noteId}-${result.lineNumber}-${result.lineText}`"
            type="button"
            class="search-result"
            @click="emit('select', result)"
          >
            <span class="search-result-meta">{{ messages.line }} {{ result.lineNumber }}</span>
            <span v-for="line in result.before" :key="`before-${result.lineNumber}-${line}`" class="search-context">{{ line }}</span>
            <strong>
              <template v-for="(segment, index) in highlightSearchText(result.lineText, query)" :key="index">
                <mark v-if="segment.matches">{{ segment.text }}</mark>
                <template v-else>{{ segment.text }}</template>
              </template>
            </strong>
            <span v-for="line in result.after" :key="`after-${result.lineNumber}-${line}`" class="search-context">{{ line }}</span>
          </button>
          <button
            v-if="remainingMatchCount(group.noteId, group.matches) > 0"
            type="button"
            class="search-expand"
            @click="toggleNote(group.noteId)"
          >
            {{ formatMessage(messages.showMoreInNote, { count: remainingMatchCount(group.noteId, group.matches) }) }}
          </button>
          <button
            v-else-if="group.matches.length > 2"
            type="button"
            class="search-expand"
            @click="toggleNote(group.noteId)"
          >
            {{ messages.collapse }}
          </button>
        </section>
        <button v-if="hasMore" type="button" class="search-load-more" @click="emit('loadMore')">
          {{ messages.loadMore }}
        </button>
      </template>
    </div>
  </aside>
</template>
