import { describe, expect, it } from 'vitest'
import { contextMatches, createAiEditorSnapshot, paragraphRange } from './ai-editor'

describe('AI editor context', () => {
  it('captures a multi-line paragraph around the cursor', () => {
    const text = 'First line\nsecond line\n\nNext paragraph'
    expect(paragraphRange(text, 4)).toEqual({ from: 0, to: 22 })
  })

  it('uses the previous paragraph from a blank slash-command line', () => {
    const text = 'Rewrite this paragraph.\n\n'
    expect(paragraphRange(text, text.length)).toEqual({ from: 0, to: 23 })
  })

  it('prefers a selection and still exposes paragraph and note scopes', () => {
    const snapshot = createAiEditorSnapshot('Alpha beta', { from: 0, to: 5 }, 5)
    expect(snapshot.defaultKind).toBe('selection')
    expect(snapshot.contexts.map((context) => context.kind)).toEqual(['selection', 'paragraph', 'note'])
    expect(snapshot.contexts[0].text).toBe('Alpha')
  })

  it('detects stale replacement targets', () => {
    const snapshot = createAiEditorSnapshot('Alpha beta', { from: 0, to: 5 }, 5)
    expect(contextMatches('Alpha beta', snapshot.contexts[0])).toBe(true)
    expect(contextMatches('Changed beta', snapshot.contexts[0])).toBe(false)
  })
})
