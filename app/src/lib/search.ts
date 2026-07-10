import type { SearchResult } from '../types/note'

export type SearchResultGroup = {
  noteId: string
  title: string
  matches: SearchResult[]
}

export type HighlightSegment = {
  text: string
  matches: boolean
}

export function groupSearchResults(results: SearchResult[], query: string): SearchResultGroup[] {
  const groups = new Map<string, SearchResultGroup>()

  for (const result of results) {
    const group = groups.get(result.noteId)
    if (group) {
      group.matches.push(result)
    } else {
      groups.set(result.noteId, {
        noteId: result.noteId,
        title: result.title,
        matches: [result],
      })
    }
  }

  const normalizedQuery = query.trim().toLocaleLowerCase()
  return [...groups.values()].sort((left, right) => {
    const leftTitleMatch = normalizedQuery && left.title.toLocaleLowerCase().includes(normalizedQuery) ? 1 : 0
    const rightTitleMatch = normalizedQuery && right.title.toLocaleLowerCase().includes(normalizedQuery) ? 1 : 0
    if (leftTitleMatch !== rightTitleMatch) return rightTitleMatch - leftTitleMatch
    return right.matches.length - left.matches.length
  })
}

export function highlightSearchText(text: string, query: string): HighlightSegment[] {
  const normalizedQuery = query.trim()
  if (!normalizedQuery) return [{ text, matches: false }]

  const lowerText = text.toLocaleLowerCase()
  const lowerQuery = normalizedQuery.toLocaleLowerCase()
  const segments: HighlightSegment[] = []
  let offset = 0

  while (offset < text.length) {
    const matchIndex = lowerText.indexOf(lowerQuery, offset)
    if (matchIndex < 0) break
    if (matchIndex > offset) segments.push({ text: text.slice(offset, matchIndex), matches: false })
    segments.push({ text: text.slice(matchIndex, matchIndex + normalizedQuery.length), matches: true })
    offset = matchIndex + normalizedQuery.length
  }

  if (offset < text.length) segments.push({ text: text.slice(offset), matches: false })
  return segments.length > 0 ? segments : [{ text, matches: false }]
}
