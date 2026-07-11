import { ref, watch } from 'vue'
import { searchNotes } from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'
import type { SearchResult } from '../types/note'

export function useSearchState(onError: () => void) {
  const searchOpen = ref(false)
  const searchQuery = ref('')
  const searchResults = ref<SearchResult[]>([])
  const searching = ref(false)
  const searchResultLimit = ref(100)
  const searchHasMore = ref(false)
  let searchTimer: number | null = null

  watch(searchQuery, () => {
    searchResultLimit.value = 100
    searchHasMore.value = false
    if (!searchOpen.value || !isTauriRuntime()) {
      searchResults.value = []
      return
    }
    scheduleSearch()
  })

  function scheduleSearch() {
    clearSearchTimer()
    searching.value = true
    searchTimer = window.setTimeout(() => void runSearch(), 200)
  }

  async function runSearch() {
    clearSearchTimer()
    const query = searchQuery.value.trim()
    if (!query) {
      searchResults.value = []
      searchHasMore.value = false
      searching.value = false
      return
    }
    try {
      const results = await searchNotes(query, searchResultLimit.value)
      searchResults.value = results
      searchHasMore.value = results.length === searchResultLimit.value
    } catch {
      onError()
    } finally {
      searching.value = false
    }
  }

  function loadMoreSearchResults() {
    if (searching.value || !searchQuery.value.trim()) return
    searchResultLimit.value += 100
    void runSearch()
  }

  function clearSearch() {
    searchResults.value = []
    searchHasMore.value = false
    clearSearchTimer()
  }

  function clearSearchTimer() {
    if (searchTimer) {
      window.clearTimeout(searchTimer)
      searchTimer = null
    }
  }

  return {
    searchOpen,
    searchQuery,
    searchResults,
    searching,
    searchHasMore,
    scheduleSearch,
    loadMoreSearchResults,
    clearSearch,
    disposeSearchState: clearSearchTimer,
  }
}
