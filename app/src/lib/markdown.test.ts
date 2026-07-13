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

  it('highlights known fenced code languages', () => {
    const rendered = renderMarkdown('```js\nconst answer = 42\n```')

    expect(rendered).toContain('language-js')
    expect(rendered).toContain('hljs-keyword')
    expect(rendered).toContain('hljs-number')
  })

  it('renders inline and display math with KaTeX', () => {
    const rendered = renderMarkdown('Inline $E = mc^2$.\n\n$$\\int_0^1 x dx$$')

    expect(rendered).toContain('class="katex"')
    expect(rendered).toContain('class="katex-display"')
  })

  it('keeps Mermaid fences available for asynchronous rendering', () => {
    const rendered = renderMarkdown('```mermaid\ngraph LR\n  A --> B\n```')

    expect(rendered).toContain('language-mermaid')
    expect(rendered).toContain('graph LR')
  })
})
