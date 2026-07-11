import { search, SearchQuery, setSearchQuery } from '@codemirror/search'
import { EditorSelection, EditorState } from '@codemirror/state'
import { describe, expect, it } from 'vitest'
import { getFindMatchSummary } from './editor-search-panel'

function searchState(doc: string, query: string, anchor = 0, head = anchor, regexp = false) {
  const initial = EditorState.create({ doc, extensions: [search()] })
  return initial.update({
    selection: EditorSelection.single(anchor, head),
    effects: setSearchQuery.of(new SearchQuery({ search: query, regexp })),
  }).state
}

describe('editor search match summary', () => {
  it('counts matches and identifies the selected match', () => {
    expect(getFindMatchSummary(searchState('one two one', 'one', 8, 11))).toEqual({ active: 2, total: 2 })
  })

  it('chooses the first match at or after the cursor, wrapping when needed', () => {
    expect(getFindMatchSummary(searchState('one two one', 'one', 4))).toEqual({ active: 2, total: 2 })
    expect(getFindMatchSummary(searchState('one two one', 'one', 11))).toEqual({ active: 1, total: 2 })
  })

  it('reports no matches for invalid or absent results', () => {
    expect(getFindMatchSummary(searchState('plain text', '[', 0, 0, true))).toEqual({ active: 0, total: 0 })
    expect(getFindMatchSummary(searchState('plain text', 'missing', 0))).toEqual({ active: 0, total: 0 })
  })
})
