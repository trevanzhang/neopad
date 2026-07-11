import { describe, expect, it } from 'vitest'
import { renderMarkdown } from './markdown'

describe('renderMarkdown', () => {
  it('escapes raw HTML from note content', () => {
    const rendered = renderMarkdown('<script>alert(1)</script>')

    expect(rendered).not.toContain('<script>')
    expect(rendered).toContain('&lt;script&gt;')
  })

  it('rejects dangerous link protocols and keeps HTTPS links', () => {
    const dangerous = renderMarkdown('[run](javascript:alert(1))')
    const safe = renderMarkdown('[site](https://example.com/path)')

    expect(dangerous).not.toContain('href=')
    expect(safe).toContain('href="https://example.com/path"')
  })
})
