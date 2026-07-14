import { describe, expect, it } from 'vitest'
import { getHelpContent, getReferenceHelp, getShortcutHelpGroups } from './help-content'

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

  it('groups the shortcut reference and explains Escape precedence', () => {
    const groups = getShortcutHelpGroups('en', context)
    expect(groups.map((group) => group.title)).toEqual([
      'Global and window',
      'Notes and tabs',
      'Edit, find, and insert',
      'View and tools',
    ])
    expect(groups[0]?.rows[0]).toEqual({ keys: 'Alt+Z', description: 'Show or hide the window' })
    expect(groups.flatMap((group) => group.rows).find((row) => row.keys === 'Esc')?.description).toContain('Close the current panel')
  })

  it('keeps localized guides available', () => {
    expect(getHelpContent('markdown', 'zh', context).title).toBe('Markdown 简明指南')
    expect(getHelpContent('expression', 'zh', context).lines[0]).toContain('Ctrl+Enter')
  })

  it('provides structured Markdown and expression references', () => {
    const markdown = getReferenceHelp('markdown', 'en')
    const expression = getReferenceHelp('expression', 'en')
    expect(markdown.groups.map((group) => group.title)).toContain('Code, math, and diagrams')
    expect(markdown.groups.flatMap((group) => group.rows).some((row) => row.value.includes('mermaid'))).toBe(true)
    expect(expression.groups.map((group) => group.title)).toEqual([
      'Supported operators',
      'Examples',
      'Recognition rules and limits',
    ])
    expect(expression.groups.flatMap((group) => group.rows).some((row) => row.value === '2 ^ 3 ^ 2')).toBe(true)
  })

  it('explains the three AI collaboration entry points', () => {
    const guide = getHelpContent('ai', 'en', context)
    expect(guide.title).toBe('AI Collaboration Guide')
    expect(guide.lines.join('\n')).toContain('// quick commands')
    expect(guide.lines.join('\n')).toContain('right-click')
    expect(guide.lines.join('\n')).toContain('Ctrl+K')
    expect(getHelpContent('ai', 'zh', context).title).toBe('AI \u534f\u4f5c\u6307\u5357')
  })

  it('keeps the software overview aligned with the current product', () => {
    const guide = getHelpContent('software', 'en', context)
    const text = guide.lines.join('\n')
    expect(guide.title).toBe('Software Help')
    expect(text).toContain('local-first')
    expect(text).toContain('Ctrl+K')
    expect(text).toContain('MCP')
    expect(text).toContain('no cloud sync')
    expect(getHelpContent('software', 'zh', context).title).toBe('\u8f6f\u4ef6\u8bf4\u660e')
  })
})
