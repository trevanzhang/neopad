import { isolateHistory } from '@codemirror/commands'
import { EditorSelection, type EditorState, type TransactionSpec } from '@codemirror/state'
import type { FinancialInsertTarget } from '../types/finance'

export function financialInsertTransaction(state: EditorState, target: FinancialInsertTarget, noteId: string, version: number, record: string): TransactionSpec | null {
  if (!record || noteId !== target.noteId || version !== target.documentVersion || state.doc.toString() !== target.documentText || target.cursor < 0 || target.cursor > state.doc.length) return null
  const at = target.cursor
  const before = at > 0 && state.doc.sliceString(at - 1, at) !== '\n' ? '\n\n' : ''
  const after = at < state.doc.length && state.doc.sliceString(at, at + 1) !== '\n' ? '\n' : ''
  const insert = `${before}${record}${after}`
  return {
    changes: { from: at, insert },
    selection: EditorSelection.cursor(at + insert.length),
    annotations: isolateHistory.of('full'),
    userEvent: 'input.finance',
  }
}
