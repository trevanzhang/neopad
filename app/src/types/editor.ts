export type EditorMode = 'edit' | 'preview' | 'split'

export type EditorModeShortcut = 'F4'

export const editorModes: EditorMode[] = ['edit', 'split', 'preview']

export function isEditorMode(value: string): value is EditorMode {
  return value === 'edit' || value === 'preview' || value === 'split'
}

export function nextEditorMode(mode: EditorMode): EditorMode {
  return editorModes[(editorModes.indexOf(mode) + 1) % editorModes.length]
}
