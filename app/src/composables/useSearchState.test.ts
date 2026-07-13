import { nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { searchNotes } from '../lib/invoke'
import type { SearchResult } from '../types/note'
import { useSearchState } from './useSearchState'

vi.mock('../lib/invoke', () => ({ searchNotes: vi.fn() }))
vi.mock('../lib/runtime', () => ({ isTauriRuntime: vi.fn(() => true) }))

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise
  })
  return { promise, resolve }
}

function result(noteId: string): SearchResult {
  return {
    noteId,
    title: noteId,
    fileName: `${noteId}.md`,
    lineNumber: 1,
    lineText: noteId,
    before: [],
    after: [],
  }
}

describe('useSearchState', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.stubGlobal('window', {
      setTimeout: globalThis.setTimeout,
      clearTimeout: globalThis.clearTimeout,
    })
    vi.mocked(searchNotes).mockReset()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('keeps newer results when an older request finishes last', async () => {
    const first = deferred<SearchResult[]>()
    const second = deferred<SearchResult[]>()
    vi.mocked(searchNotes)
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise)
    const state = useSearchState(vi.fn())
    state.searchOpen.value = true

    state.searchQuery.value = 'first'
    await nextTick()
    await vi.advanceTimersByTimeAsync(200)

    state.searchQuery.value = 'second'
    await nextTick()
    await vi.advanceTimersByTimeAsync(200)

    second.resolve([result('second')])
    await Promise.resolve()
    expect(state.searchResults.value.map((entry) => entry.noteId)).toEqual(['second'])

    first.resolve([result('first')])
    await Promise.resolve()
    expect(state.searchResults.value.map((entry) => entry.noteId)).toEqual(['second'])
    expect(state.searching.value).toBe(false)

    state.disposeSearchState()
  })
})
