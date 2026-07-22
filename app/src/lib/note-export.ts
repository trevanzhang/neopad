import { renderMarkdownInto } from './markdown'
import { markdownThemeForPreview } from './preview-style'
import type { PreviewTheme } from '../types/editor'

export type NoteExportFormat = 'png' | 'pdf'
export type NoteExportLayout = 'standard' | 'mobile'
export type NoteExportStyle = 'print' | 'current-theme'

export type NoteExportRenderOptions =
  | { layout: NoteExportLayout; style: 'print' }
  | {
    layout: NoteExportLayout
    style: 'current-theme'
    previewTheme: PreviewTheme
    fontFamily: string
    fontSizePx: number
    lineHeight: number
  }

interface NoteExportLayoutMetrics {
  widthPx: number
  padding: string
  fontSizePx: number
  lineHeight: number
  footerFontSizePx: number
}

const EXPORT_TIMEOUT_MS = 30_000
const MAX_CANVAS_DIMENSION = 16_384
const MAX_CANVAS_AREA = 96_000_000
const EXPORT_BRAND_TEXT = 'Powered by NeoPad'
const exportLayouts: Record<NoteExportLayout, NoteExportLayoutMetrics> = {
  standard: {
    widthPx: 794,
    padding: '56px 64px',
    fontSizePx: 15,
    lineHeight: 1.65,
    footerFontSizePx: 12,
  },
  mobile: {
    widthPx: 540,
    padding: '48px 40px',
    fontSizePx: 18,
    lineHeight: 1.75,
    footerFontSizePx: 14,
  },
}

export function getNoteExportLayoutMetrics(layout: NoteExportLayout): Readonly<NoteExportLayoutMetrics> {
  return exportLayouts[layout]
}

export async function createNoteExportBlob(
  markdown: string,
  format: NoteExportFormat,
  options: NoteExportRenderOptions = { layout: 'standard', style: 'print' },
): Promise<Blob> {
  const { layout } = options
  const metrics = getNoteExportLayoutMetrics(layout)
  const isThemed = options.style === 'current-theme'
  const background = isThemed ? 'var(--np-preview-bg)' : '#fff'
  const textColor = isThemed ? 'var(--np-preview-text)' : '#24292f'
  const fontFamily = isThemed
    ? options.fontFamily
    : '"Segoe UI",Arial,"PingFang SC","Microsoft YaHei",sans-serif'
  const fontSizePx = isThemed
    ? layout === 'mobile' ? Math.max(options.fontSizePx, metrics.fontSizePx) : options.fontSizePx
    : metrics.fontSizePx
  const lineHeight = isThemed
    ? layout === 'mobile' ? Math.max(options.lineHeight, metrics.lineHeight) : options.lineHeight
    : metrics.lineHeight
  const host = document.createElement('div')
  host.style.cssText = [
    'position:fixed',
    'left:-100000px',
    'top:0',
    'z-index:-1',
    'pointer-events:none',
  ].join(';')

  const surface = document.createElement('section')
  surface.className = 'preview-pane note-export-surface'
  surface.dataset.previewTheme = isThemed ? options.previewTheme : 'githubLight'
  surface.style.cssText = `padding:0;border:0;overflow:visible;background:${background};`

  const article = document.createElement('article')
  article.className = 'markdown-preview note-export-content'
  article.dataset.exportLayout = layout
  article.style.cssText = [
    `box-sizing:border-box`,
    `width:${metrics.widthPx}px`,
    'max-width:none',
    'min-height:1px',
    'margin:0',
    `padding:${metrics.padding}`,
    `color:${textColor}`,
    `background:${background}`,
    `font-family:${fontFamily}`,
    `font-size:${fontSizePx}px`,
    `line-height:${lineHeight}`,
  ].join(';')

  surface.appendChild(article)
  host.appendChild(surface)
  document.body.appendChild(host)

  try {
    return await withTimeout(async () => {
      const markdownTheme = isThemed ? markdownThemeForPreview(options.previewTheme) : 'light'
      await renderMarkdownInto(article, markdown, markdownTheme)
      appendBrandFooter(article, metrics, isThemed)
      await nextPaint()
      sanitizeModernColors(article)

      const backgroundColor = getComputedStyle(surface).backgroundColor || '#ffffff'
      const pageBreaks = collectBlockBreaks(article)
      const canvas = await captureArticle(article, format, backgroundColor)
      return format === 'png'
        ? canvasToBlob(canvas, 'image/png')
        : canvasToPdfBlob(canvas, article.offsetWidth, article.offsetHeight, pageBreaks, backgroundColor)
    })
  } finally {
    host.remove()
    document.querySelectorAll('iframe.html2canvas-container').forEach((element) => element.remove())
  }
}

function appendBrandFooter(article: HTMLElement, metrics: NoteExportLayoutMetrics, themed: boolean) {
  const footer = document.createElement('footer')
  footer.className = 'note-export-brand'
  footer.textContent = EXPORT_BRAND_TEXT
  footer.style.cssText = [
    'box-sizing:border-box',
    'margin-top:48px',
    'padding-top:18px',
    `border-top:1px solid ${themed ? 'var(--np-preview-border)' : '#d0d7de'}`,
    `color:${themed ? 'var(--np-preview-muted)' : '#6e7781'}`,
    `font-size:${metrics.footerFontSizePx}px`,
    'font-weight:500',
    'letter-spacing:.02em',
    'line-height:1.4',
    'text-align:center',
  ].join(';')
  article.appendChild(footer)
}

async function captureArticle(article: HTMLElement, format: NoteExportFormat, backgroundColor: string) {
  const width = Math.max(1, article.scrollWidth)
  const height = Math.max(1, article.scrollHeight)
  const desiredScale = format === 'png' ? 2 : 1.6
  const safeScale = Math.min(
    desiredScale,
    MAX_CANVAS_DIMENSION / width,
    MAX_CANVAS_DIMENSION / height,
    Math.sqrt(MAX_CANVAS_AREA / (width * height)),
  )
  if (safeScale < 0.6) {
    throw new Error('NOTE_EXPORT_TOO_LONG')
  }

  const { default: html2canvas } = await import('html2canvas')
  return html2canvas(article, {
    scale: safeScale,
    useCORS: true,
    backgroundColor,
    logging: false,
  })
}

async function canvasToBlob(canvas: HTMLCanvasElement, type: string, quality?: number) {
  return new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) resolve(blob)
      else reject(new Error('NOTE_EXPORT_CANVAS_FAILED'))
    }, type, quality)
  })
}

async function canvasToPdfBlob(
  canvas: HTMLCanvasElement,
  cssWidth: number,
  cssHeight: number,
  blockBreaks: number[],
  backgroundColor: string,
) {
  const { jsPDF } = await import('jspdf')
  const pdf = new jsPDF({ unit: 'mm', format: 'a4', orientation: 'portrait', compress: true })
  const marginMm = 15
  const printableWidthMm = 210 - marginMm * 2
  const printableHeightMm = 297 - marginMm * 2
  const cssPageHeight = cssWidth * printableHeightMm / printableWidthMm
  const slices = computePageSlices(cssHeight, cssPageHeight, blockBreaks)
  const pixelsPerCssPixel = canvas.height / cssHeight
  const [backgroundRed, backgroundGreen, backgroundBlue] = parseCssColor(backgroundColor)

  for (const [index, slice] of slices.entries()) {
    if (index > 0) pdf.addPage('a4', 'portrait')
    pdf.setFillColor(backgroundRed, backgroundGreen, backgroundBlue)
    pdf.rect(0, 0, 210, 297, 'F')
    const sourceY = Math.max(0, Math.round(slice.start * pixelsPerCssPixel))
    const sourceEnd = Math.min(canvas.height, Math.round(slice.end * pixelsPerCssPixel))
    const sourceHeight = Math.max(1, sourceEnd - sourceY)
    const pageCanvas = document.createElement('canvas')
    pageCanvas.width = canvas.width
    pageCanvas.height = sourceHeight
    const context = pageCanvas.getContext('2d')
    if (!context) throw new Error('NOTE_EXPORT_CANVAS_FAILED')
    context.fillStyle = backgroundColor
    context.fillRect(0, 0, pageCanvas.width, pageCanvas.height)
    context.drawImage(
      canvas,
      0,
      sourceY,
      canvas.width,
      sourceHeight,
      0,
      0,
      pageCanvas.width,
      pageCanvas.height,
    )
    const renderedHeightMm = sourceHeight / canvas.width * printableWidthMm
    const image = pageCanvas.toDataURL('image/jpeg', 0.94)
    pdf.addImage(image, 'JPEG', marginMm, marginMm, printableWidthMm, renderedHeightMm, undefined, 'FAST')
    pageCanvas.width = 1
    pageCanvas.height = 1
  }

  return pdf.output('blob')
}

export function parseCssColor(color: string): [number, number, number] {
  const hex = /^#([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i.exec(color.trim())
  if (hex) return [Number.parseInt(hex[1], 16), Number.parseInt(hex[2], 16), Number.parseInt(hex[3], 16)]
  const channels = color.match(/[\d.]+/g)?.slice(0, 3).map((value) => Number(value))
  return channels?.length === 3
    ? channels.map((value) => Math.max(0, Math.min(255, Math.round(value)))) as [number, number, number]
    : [255, 255, 255]
}

export function computePageSlices(
  contentHeight: number,
  idealPageHeight: number,
  blockBreaks: number[],
): Array<{ start: number; end: number }> {
  if (contentHeight <= 0 || idealPageHeight <= 0) return []
  const breaks = [...new Set(blockBreaks)]
    .filter((value) => value > 0 && value < contentHeight)
    .sort((a, b) => a - b)
  const slices: Array<{ start: number; end: number }> = []
  let start = 0

  while (start < contentHeight) {
    const idealEnd = Math.min(contentHeight, start + idealPageHeight)
    if (idealEnd === contentHeight) {
      slices.push({ start, end: contentHeight })
      break
    }
    const earliestUsefulBreak = start + idealPageHeight * 0.55
    const candidate = breaks.filter((value) => value >= earliestUsefulBreak && value <= idealEnd).at(-1)
    const end = candidate && candidate > start ? candidate : idealEnd
    slices.push({ start, end })
    start = end
  }

  return slices
}

function collectBlockBreaks(article: HTMLElement) {
  const articleTop = article.getBoundingClientRect().top
  return Array.from(article.children)
    .map((element) => (element as HTMLElement).getBoundingClientRect().top - articleTop)
    .filter((value) => value > 0)
}

function nextPaint() {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
  })
}

async function withTimeout<T>(work: () => Promise<T>): Promise<T> {
  let timeoutId = 0
  const timeout = new Promise<never>((_, reject) => {
    timeoutId = window.setTimeout(() => reject(new Error('NOTE_EXPORT_TIMEOUT')), EXPORT_TIMEOUT_MS)
  })
  try {
    return await Promise.race([work(), timeout])
  } finally {
    window.clearTimeout(timeoutId)
  }
}

const modernColorFunction = /\b(?:color|oklch|oklab|lab|lch|hwb|color-mix)\(/i
const colorProperties = [
  'color',
  'backgroundColor',
  'borderTopColor',
  'borderRightColor',
  'borderBottomColor',
  'borderLeftColor',
  'outlineColor',
  'textDecorationColor',
  'fill',
  'stroke',
] as const

function sanitizeModernColors(root: HTMLElement) {
  try {
    const canvas = document.createElement('canvas')
    canvas.width = canvas.height = 1
    const context = canvas.getContext('2d', { willReadFrequently: true })
    if (!context) return
    const cache = new Map<string, string | null>()
    const elements = [root, ...Array.from(root.querySelectorAll<HTMLElement>('*'))]

    for (const element of elements) {
      const computed = getComputedStyle(element)
      for (const property of colorProperties) {
        const value = (computed as unknown as Record<string, string>)[property]
        if (!value || !modernColorFunction.test(value)) continue
        let replacement = cache.get(value)
        if (replacement === undefined) {
          context.fillStyle = '#abcdef'
          context.fillStyle = value
          context.clearRect(0, 0, 1, 1)
          context.fillRect(0, 0, 1, 1)
          const [red, green, blue, alpha] = context.getImageData(0, 0, 1, 1).data
          replacement = context.fillStyle === '#abcdef'
            ? null
            : `rgba(${red}, ${green}, ${blue}, ${(alpha / 255).toFixed(3)})`
          cache.set(value, replacement)
        }
        if (replacement) {
          const cssName = property.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)
          element.style.setProperty(cssName, replacement, 'important')
        }
      }
    }
  } catch {
    // Color compatibility is best-effort and must never block exporting.
  }
}
