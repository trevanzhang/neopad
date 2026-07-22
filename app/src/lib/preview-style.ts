import type {
  PreviewContentWidth,
  PreviewFontFamily,
  PreviewLineHeight,
  PreviewTheme,
} from '../types/editor'
import type { MarkdownRenderTheme } from './markdown'

const darkPreviewThemes = new Set<PreviewTheme>(['oneDark', 'nord', 'solarizedDark', 'monokai', 'dracula'])

export function markdownThemeForPreview(previewTheme: PreviewTheme): MarkdownRenderTheme {
  return darkPreviewThemes.has(previewTheme) ? 'dark' : 'light'
}

export function previewFontFamilyCss(fontFamily: PreviewFontFamily, editorFontFamily: string) {
  if (fontFamily === 'editor') return editorFontFamily
  if (fontFamily === 'serif') return 'Georgia, "Times New Roman", serif'
  if (fontFamily === 'mono') return '"JetBrains Mono", Consolas, "Courier New", monospace'
  return '"Segoe UI", Arial, sans-serif'
}

export function previewLineHeightCss(lineHeight: PreviewLineHeight) {
  if (lineHeight === 'compact') return '1.45'
  if (lineHeight === 'relaxed') return '1.8'
  return '1.62'
}

export function previewContentWidthCss(contentWidth: PreviewContentWidth) {
  if (contentWidth === 'compact') return '64ch'
  if (contentWidth === 'wide') return '96ch'
  return '76ch'
}
