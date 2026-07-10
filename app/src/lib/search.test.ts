import { describe, expect, it } from 'vitest'
import { groupSearchResults, highlightSearchText } from './search'

const result = (noteId: string, title: string, lineNumber: number) => ({
  noteId,
  title,
  fileName: `${noteId}.md`,
  lineNumber,
  lineText: 'Group keyword result',
  before: [],
  after: [],
})

describe('groupSearchResults', () => {
  it('groups matching lines by note and prioritizes title matches', () => {
    const groups = groupSearchResults([
      result('first', 'Other note', 2),
      result('first', 'Other note', 5),
      result('second', 'Group planning', 1),
    ], 'group')

    expect(groups.map((group) => [group.title, group.matches.length])).toEqual([
      ['Group planning', 1],
      ['Other note', 2],
    ])
  })

  it('returns an empty array for empty results', () => {
    expect(groupSearchResults([], 'group')).toEqual([])
  })

  it('falls back to match-count ordering when no title matches', () => {
    const groups = groupSearchResults([
      result('first', 'Alpha note', 1),
      result('second', 'Beta note', 2),
      result('second', 'Beta note', 5),
    ], 'group')

    expect(groups.map((group) => group.noteId)).toEqual(['second', 'first'])
  })

  it('treats a whitespace-only query like an empty query', () => {
    const groups = groupSearchResults(
      [
        result('first', 'Alpha note', 1),
        result('second', 'Beta note', 2),
        result('second', 'Beta note', 5),
      ],
      '   ',
    )

    expect(groups.map((group) => group.noteId)).toEqual(['second', 'first'])
  })

  it('matches titles case-insensitively', () => {
    const groups = groupSearchResults(
      [
        result('first', 'Other note', 2),
        result('second', 'Group planning', 1),
      ],
      'GROUP',
    )

    expect(groups.map((group) => group.noteId)).toEqual(['second', 'first'])
  })

  it('preserves noteId, title, and full matches for each group', () => {
    const groups = groupSearchResults(
      [
        result('first', 'Alpha note', 2),
        result('first', 'Alpha note', 5),
        result('second', 'Beta note', 1),
      ],
      'alpha',
    )

    expect(groups).toEqual([
      {
        noteId: 'first',
        title: 'Alpha note',
        matches: [
          expect.objectContaining({ noteId: 'first', lineNumber: 2 }),
          expect.objectContaining({ noteId: 'first', lineNumber: 5 }),
        ],
      },
      {
        noteId: 'second',
        title: 'Beta note',
        matches: [expect.objectContaining({ noteId: 'second', lineNumber: 1 })],
      },
    ])
  })
})

describe('highlightSearchText', () => {
  it('preserves the original text while marking case-insensitive matches', () => {
    const segments = highlightSearchText('Group group', 'group')

    expect(segments).toEqual([
      { text: 'Group', matches: true },
      { text: ' ', matches: false },
      { text: 'group', matches: true },
    ])
  })

  it('returns a single non-matching segment for an empty query', () => {
    expect(highlightSearchText('hello world', '')).toEqual([
      { text: 'hello world', matches: false },
    ])
  })

  it('returns a single non-matching segment for a whitespace-only query', () => {
    expect(highlightSearchText('hello world', '   ')).toEqual([
      { text: 'hello world', matches: false },
    ])
  })

  it('returns a single non-matching segment when the query does not match', () => {
    expect(highlightSearchText('hello world', 'missing')).toEqual([
      { text: 'hello world', matches: false },
    ])
  })

  it('marks a match at the start of the text', () => {
    expect(highlightSearchText('group plan', 'group')).toEqual([
      { text: 'group', matches: true },
      { text: ' plan', matches: false },
    ])
  })

  it('marks a match at the end of the text', () => {
    expect(highlightSearchText('the group', 'group')).toEqual([
      { text: 'the ', matches: false },
      { text: 'group', matches: true },
    ])
  })

  it('handles CJK queries', () => {
    expect(highlightSearchText('项目管理笔记', '项目')).toEqual([
      { text: '项目', matches: true },
      { text: '管理笔记', matches: false },
    ])
  })
})
