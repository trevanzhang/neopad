<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import type { AppMessages } from '../lib/i18n'
import type { SearchResult } from '../types/note'

defineProps<{
  query: string
  results: SearchResult[]
  searching: boolean
  messages: AppMessages['search']
}>()

defineEmits<{
  'update:query': [query: string]
  close: []
  select: [result: SearchResult]
}>()

const searchInput = ref<HTMLInputElement | null>(null)

onMounted(() => {
  void nextTick(focusSearchInput)
})

function focusSearchInput() {
  searchInput.value?.focus()
  searchInput.value?.select()
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
      <button
        v-for="result in results"
        :key="`${result.noteId}-${result.lineNumber}-${result.lineText}`"
        type="button"
        class="search-result"
        @click="$emit('select', result)"
      >
        <span class="search-result-meta">{{ result.title }} - {{ messages.line }} {{ result.lineNumber }}</span>
        <span v-for="line in result.before" :key="`before-${line}`" class="search-context">{{ line }}</span>
        <strong>{{ result.lineText }}</strong>
        <span v-for="line in result.after" :key="`after-${line}`" class="search-context">{{ line }}</span>
      </button>
    </div>
  </aside>
</template>
