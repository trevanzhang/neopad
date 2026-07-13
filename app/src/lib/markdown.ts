import MarkdownIt from 'markdown-it'
import highlightJs from 'highlight.js/lib/common'
import markdownItKatex from '@vscode/markdown-it-katex'
import 'katex/dist/katex.min.css'

const renderer = new MarkdownIt({
  html: false,
  linkify: true,
  typographer: true,
  highlight(code, language) {
    if (language.toLowerCase() === 'mermaid') return ''
    if (language && highlightJs.getLanguage(language)) {
      try {
        return highlightJs.highlight(code, { language, ignoreIllegals: true }).value
      } catch {
        return ''
      }
    }
    return ''
  },
}).use(markdownItKatex, {
  enableFencedBlocks: true,
  throwOnError: false,
})

let mermaidSequence = 0
let mermaidQueue: Promise<void> = Promise.resolve()

export type MarkdownRenderTheme = 'light' | 'dark'

export function renderMarkdown(markdown: string): string {
  return renderer.render(markdown)
}

export async function renderMarkdownInto(
  target: HTMLElement,
  markdown: string,
  theme: MarkdownRenderTheme = 'light',
): Promise<void> {
  target.innerHTML = renderMarkdown(markdown)
  await renderMermaidBlocks(target, theme)
  await waitForRenderedAssets(target)
}

async function renderMermaidBlocks(target: HTMLElement, theme: MarkdownRenderTheme) {
  const blocks = Array.from(target.querySelectorAll<HTMLElement>('pre > code.language-mermaid'))
  if (!blocks.length) return

  const run = async () => {
    const { default: mermaid } = await import('mermaid')
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: 'strict',
      suppressErrorRendering: true,
      theme: theme === 'dark' ? 'dark' : 'neutral',
    })

    for (const block of blocks) {
      if (!block.isConnected) continue
      const source = block.textContent?.trim() ?? ''
      const pre = block.parentElement
      if (!pre || !source) continue

      try {
        const { svg } = await mermaid.render(`neopad-mermaid-${++mermaidSequence}`, source)
        if (!pre.isConnected) continue
        const diagram = document.createElement('div')
        diagram.className = 'markdown-mermaid'
        diagram.innerHTML = svg
        pre.replaceWith(diagram)
      } catch {
        if (!pre.isConnected) continue
        pre.classList.add('markdown-mermaid-error')
        pre.setAttribute('title', 'Mermaid diagram could not be rendered')
      }
    }
  }

  const queued = mermaidQueue.then(run, run)
  mermaidQueue = queued.catch(() => undefined)
  await queued
}

export async function waitForRenderedAssets(target: HTMLElement): Promise<void> {
  const fonts = document.fonts?.ready ?? Promise.resolve()
  const images = Array.from(target.querySelectorAll('img')).map(async (image) => {
    if (!image.complete) {
      await new Promise<void>((resolve) => {
        image.addEventListener('load', () => resolve(), { once: true })
        image.addEventListener('error', () => resolve(), { once: true })
      })
    }
    try {
      await image.decode()
    } catch {
      // A broken or cross-origin image should not prevent the remaining note from rendering.
    }
  })

  await Promise.all([fonts, ...images])
}
