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
    expect(plan?.requestContext).toContain(`<note_reference truncated="false">\n${text}`)
    expect(plan?.requestContext).toContain('<operation_target kind="paragraph"')
  })

  it('summarizes the whole note when there is no selection', () => {
    const snapshot = createAiEditorSnapshot(text, { from: 0, to: 0 }, text.length)
    const plan = createAiInlinePlan('summarize', snapshot)
    expect(plan?.context.kind).toBe('note')
    expect(plan?.action).toBe('insert')
    expect(plan?.requestContext).toBe(`<operation_target kind="note">\n${text}\n</operation_target>`)
  })

  it('prefers a selection for transformations', () => {
    const snapshot = createAiEditorSnapshot(text, { from: 0, to: 5 }, 5)
    expect(createAiInlinePlan('translate', snapshot)?.context.kind).toBe('selection')
  })

  it('uses the full note as reference when summarizing a selection', () => {
    const snapshot = createAiEditorSnapshot(text, { from: 0, to: 5 }, 5)
    const plan = createAiInlinePlan('summarize', snapshot)
    expect(plan?.context.kind).toBe('selection')
    expect(plan?.requestContext).toContain(`<note_reference truncated="false">\n${text}`)
    expect(plan?.requestContext).toContain('<operation_target kind="selection"')
  })

  it('places a continuation marker after a reverse selection', () => {
    const snapshot = createAiEditorSnapshot('First paragraph\n\nSecond paragraph', { from: 0, to: 15 }, 0)
    const plan = createAiInlinePlan('continue', snapshot)
    expect(plan?.requestContext).toContain('First paragraph<insertion_point />')
  })

  it('rejects commands without meaningful source text', () => {
    const snapshot = createAiEditorSnapshot('', { from: 0, to: 0 }, 0)
    expect(createAiInlinePlan('continue', snapshot)).toBeNull()
  })

  it('keeps the note opening and target vicinity when reference context is long', () => {
    const target = 'TARGET PARAGRAPH'
    const longText = `# Note title\n${'a'.repeat(18_000)}\n\n${target}\n\n${'z'.repeat(18_000)}`
    const from = longText.indexOf(target)
    const snapshot = createAiEditorSnapshot(longText, { from, to: from + target.length }, from + target.length)
    const plan = createAiInlinePlan('translate', snapshot)

    expect(plan?.requestContext).toContain('<note_reference truncated="true">')
    expect(plan?.requestContext).toContain('# Note title')
    expect(plan?.requestContext).toContain(target)
    expect(plan?.requestContext).toContain('[... note context omitted ...]')
    expect(plan?.requestContext.length).toBeLessThan(17_000)
  })

  it('marks the captured insertion point for continuation', () => {
    const snapshot = createAiEditorSnapshot(text, { from: 0, to: 0 }, text.length)
    const plan = createAiInlinePlan('continue', snapshot)
    expect(plan?.requestContext).toContain('Second paragraph.<insertion_point />')
  })
})
