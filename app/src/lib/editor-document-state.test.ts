import { history, redoDepth, undoDepth } from '@codemirror/commands'
import type { EditorView } from '@codemirror/view'
import { describe, expect, it, vi } from 'vitest'
import { resetEditorDocument } from './editor-document-state'

describe('resetEditorDocument', () => {
  it('starts a loaded note with isolated undo and redo history', () => {
    const setState = vi.fn()
    const view = { setState } as unknown as Pick<EditorView, 'setState'>

    const state = resetEditorDocument(view, '# Loaded note', [history()])
    const edited = state.update({
      changes: { from: state.doc.length, insert: '\nchanged' },
    }).state

    expect(setState).toHaveBeenCalledWith(state)
    expect(undoDepth(state)).toBe(0)
    expect(redoDepth(state)).toBe(0)
    expect(undoDepth(edited)).toBe(1)
  })
})
