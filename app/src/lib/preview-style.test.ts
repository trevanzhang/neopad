import { describe, expect, it } from 'vitest'
import {
  markdownThemeForPreview,
  previewContentWidthCss,
  previewFontFamilyCss,
  previewLineHeightCss,
} from './preview-style'

describe('preview style helpers', () => {
  it('keeps preview and export typography mappings shared', () => {
    expect(previewFontFamilyCss('editor', 'Consolas')).toBe('Consolas')
    expect(previewFontFamilyCss('serif', 'Consolas')).toBe('Georgia, "Times New Roman", serif')
    expect(previewLineHeightCss('relaxed')).toBe('1.8')
    expect(previewContentWidthCss('wide')).toBe('96ch')
  })

  it('selects the matching Mermaid light or dark renderer', () => {
    expect(markdownThemeForPreview('dracula')).toBe('dark')
    expect(markdownThemeForPreview('solarizedLight')).toBe('light')
  })
})
