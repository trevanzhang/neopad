export type EditorMode = 'edit' | 'preview' | 'split'

export type EditorModeShortcut = 'F4'

export type PreviewTheme = 'light' | 'oneDark' | 'nord' | 'solarizedLight' | 'solarizedDark' | 'monokai' | 'githubLight' | 'dracula'
export type PreviewFontFamily = 'editor' | 'system' | 'serif' | 'mono'
export type PreviewLineHeight = 'compact' | 'standard' | 'relaxed'
export type PreviewContentWidth = 'compact' | 'standard' | 'wide'

export const editorModes: EditorMode[] = ['edit', 'split', 'preview']

export const previewThemes: PreviewTheme[] = ['light', 'oneDark', 'nord', 'solarizedLight', 'solarizedDark', 'monokai', 'githubLight', 'dracula']
export const previewFontFamilies: PreviewFontFamily[] = ['editor', 'system', 'serif', 'mono']
export const previewLineHeights: PreviewLineHeight[] = ['compact', 'standard', 'relaxed']
export const previewContentWidths: PreviewContentWidth[] = ['compact', 'standard', 'wide']

export function isEditorMode(value: string): value is EditorMode {
  return value === 'edit' || value === 'preview' || value === 'split'
}

export function isPreviewTheme(value: string): value is PreviewTheme {
  return previewThemes.includes(value as PreviewTheme)
}

export function isPreviewFontFamily(value: string): value is PreviewFontFamily {
  return previewFontFamilies.includes(value as PreviewFontFamily)
}

export function isPreviewLineHeight(value: string): value is PreviewLineHeight {
  return previewLineHeights.includes(value as PreviewLineHeight)
}

export function isPreviewContentWidth(value: string): value is PreviewContentWidth {
  return previewContentWidths.includes(value as PreviewContentWidth)
}

export function nextEditorMode(mode: EditorMode): EditorMode {
  return editorModes[(editorModes.indexOf(mode) + 1) % editorModes.length]
}
