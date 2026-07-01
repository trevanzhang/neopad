import { describe, expect, it } from 'vitest'
import { nextEditorMode } from './editor'

describe('nextEditorMode', () => {
  it('cycles edit, split, preview, then edit', () => {
    expect(nextEditorMode('edit')).toBe('split')
    expect(nextEditorMode('split')).toBe('preview')
    expect(nextEditorMode('preview')).toBe('edit')
  })
})
