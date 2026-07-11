import { describe, expect, it } from 'vitest'
import { getHelpContent } from './help-content'

const context = {
  appVersion: '0.4.6',
  shortcutBaseKey: 'Z',
  shortcutModifiers: ['Alt'],
  clipboardShortcutBaseKey: 'V',
  clipboardShortcutModifiers: ['Ctrl', 'Shift'],
}

describe('help content', () => {
  it('renders configured shortcuts and version', () => {
    expect(getHelpContent('shortcuts', 'en', context).lines[0]).toContain('Alt+Z')
    expect(getHelpContent('about', 'en', context).lines).toContain('Version: 0.4.6')
  })

  it('keeps localized guides available', () => {
    expect(getHelpContent('markdown', 'zh', context).title).toBe('Markdown 简明指南')
    expect(getHelpContent('expression', 'zh', context).lines[0]).toContain('Ctrl+Enter')
  })
})
