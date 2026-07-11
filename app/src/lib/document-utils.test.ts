import { describe, expect, it } from 'vitest'
import {
  convertChinese,
  renderInsertTemplate,
  safeFileName,
  titleFromFileName,
  toFullWidth,
  toHalfWidth,
} from './document-utils'

describe('document utilities', () => {
  it('creates safe titles and Markdown file names', () => {
    expect(titleFromFileName('project.notes.md')).toBe('project.notes')
    expect(safeFileName('  A <bad> / name  ')).toBe('A -bad- - name')
    expect(safeFileName('   ')).toBe('Untitled')
  })

  it('renders insert templates deterministically', () => {
    const date = new Date(2026, 6, 11, 9, 5)
    expect(renderInsertTemplate("date() + ' ' + time() + crlf() + chars('-', 3)", date))
      .toBe('2026-07-11 09:05\n---')
  })

  it('converts character widths and mapped Chinese characters', () => {
    expect(toHalfWidth('ＡＢ　１')).toBe('AB 1')
    expect(toFullWidth('AB 1')).toBe('ＡＢ　１')
    expect(convertChinese('后台', { '后': '後', '台': '臺' })).toBe('後臺')
  })
})
