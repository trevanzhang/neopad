import { describe, expect, it } from 'vitest'
import { createAiEditorSnapshot } from './ai-editor'
import { createAiInlinePlan } from './ai-inline-command'

describe('createAiInlinePlan', () => {
  const text = 'First paragraph.\n\nSecond paragraph.'

  it('polishes the current paragraph and replaces it', () => {
    const snapshot = createAiEditorSnapshot(text, { from: 0, to: 0 }, text.length)
    const plan = createAiInlinePlan('polish', snapshot)
    expect(plan?.context.text).toBe('Second paragraph.')
    expect(plan?.action).toBe('replace')
  })

  it('summarizes the whole note when there is no selection', () => {
    const snapshot = createAiEditorSnapshot(text, { from: 0, to: 0 }, text.length)
    const plan = createAiInlinePlan('summarize', snapshot)
    expect(plan?.context.kind).toBe('note')
    expect(plan?.action).toBe('insert')
  })

  it('prefers a selection for transformations', () => {
    const snapshot = createAiEditorSnapshot(text, { from: 0, to: 5 }, 5)
    expect(createAiInlinePlan('translate', snapshot)?.context.kind).toBe('selection')
  })

  it('rejects commands without meaningful source text', () => {
    const snapshot = createAiEditorSnapshot('', { from: 0, to: 0 }, 0)
    expect(createAiInlinePlan('continue', snapshot)).toBeNull()
  })
})
