import { EditorState, type Extension } from '@codemirror/state'
import type { EditorView } from '@codemirror/view'

/**
 * Replaces a loaded document with a fresh editor state.
 *
 * A note load is not an edit. Creating a new state keeps the previous note's
 * undo/redo history from being applied to the newly loaded note.
 */
export function resetEditorDocument(
  view: Pick<EditorView, 'setState'>,
  content: string,
  extensions: Extension,
) {
  const state = EditorState.create({ doc: content, extensions })
  view.setState(state)
  return state
}
