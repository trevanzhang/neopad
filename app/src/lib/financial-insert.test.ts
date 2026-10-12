import { history, undo, redo } from '@codemirror/commands'
import { EditorSelection, EditorState, type Transaction } from '@codemirror/state'
import { describe, expect, it } from 'vitest'
import { financialInsertTransaction } from './financial-insert'
import type { FinancialInsertTarget } from '../types/finance'

describe('financial record insertion', () => {
  const target: FinancialInsertTarget = { noteId: 'one', documentText: 'keep this text', cursor: 4, documentVersion: 7 }
  const makeState = () => EditorState.create({ doc: target.documentText, selection: EditorSelection.single(0, 4), extensions: [history()] })

  it('inserts at the captured cursor without deleting selected text, and is one undo/redo step', () => {
    let state = makeState()
    const transaction = financialInsertTransaction(state, target, 'one', 7, '## Calculation\nresult\n')!
    state = state.update(transaction).state
    expect(state.doc.toString()).toContain('keep\n\n## Calculation\nresult\n\n this text')
    const dispatch = (tr: Transaction) => { state = tr.state }
    expect(undo({ state, dispatch })).toBe(true)
    expect(state.doc.toString()).toBe(target.documentText)
    expect(redo({ state, dispatch })).toBe(true)
    expect(state.doc.toString()).toContain('## Calculation')
  })

  it('rejects a switched note, changed text, or changed revision even after undo restores the text', () => {
    const state = makeState()
    expect(financialInsertTransaction(state, target, 'two', 7, 'record')).toBeNull()
    expect(financialInsertTransaction(state, target, 'one', 9, 'record')).toBeNull()
    expect(financialInsertTransaction(state.update({ changes: { from: 0, insert: 'x' } }).state, target, 'one', 7, 'record')).toBeNull()
    expect(financialInsertTransaction(state, { ...target, cursor: 100 }, 'one', 7, 'record')).toBeNull()
  })

  it('isolates insertion from adjacent typing history', () => {
    let state = makeState()
    state = state.update({ changes: { from: state.doc.length, insert: '!' }, userEvent: 'input.type' }).state
    const updated = { ...target, documentText: state.doc.toString(), documentVersion: 8 }
    state = state.update(financialInsertTransaction(state, updated, 'one', 8, 'record')!).state
    state = state.update({ changes: { from: state.selection.main.head, insert: '?' }, userEvent: 'input.type' }).state
    const dispatch = (tr: Transaction) => { state = tr.state }
    undo({ state, dispatch })
    expect(state.doc.toString()).toContain('record')
    undo({ state, dispatch })
    expect(state.doc.toString()).toBe('keep this text!')
    undo({ state, dispatch })
    expect(state.doc.toString()).toBe('keep this text')
  })
})
