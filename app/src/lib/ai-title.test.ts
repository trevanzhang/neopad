import { describe, expect, it } from 'vitest'
import { normalizeAiNoteTitle } from './ai-title'

describe('normalizeAiNoteTitle', () => {
  it('extracts one plain title from common AI formatting', () => {
    expect(normalizeAiNoteTitle('## Title: "Project launch risks"\nExtra explanation'))
      .toBe('Project launch risks')
    expect(normalizeAiNoteTitle('标题：《季度复盘》')).toBe('季度复盘')
  })

  it('limits the title by Unicode characters', () => {
    expect(normalizeAiNoteTitle('项目风险与下一步行动', 6)).toBe('项目风险与下')
  })

  it('returns an empty string for empty output', () => {
    expect(normalizeAiNoteTitle('\n  \n')).toBe('')
  })
})
