export function titleFromFileName(fileName: string) {
  const withoutExtension = fileName.replace(/\.[^/.]+$/, '')
  return withoutExtension.trim() || 'Untitled'
}

export function safeFileName(title: string) {
  return title.trim().replace(/[<>:"/\\|?*\u0000-\u001f]/g, '-').replace(/\s+/g, ' ') || 'Untitled'
}

export function downloadText(fileName: string, text: string) {
  const url = URL.createObjectURL(new Blob([text], { type: 'text/markdown;charset=utf-8' }))
  const link = document.createElement('a')
  link.href = url
  link.download = fileName
  link.click()
  URL.revokeObjectURL(url)
}

export function renderInsertTemplate(template: string, date = new Date()) {
  const parts = template.split(/\s*\+\s*/g)
  return parts.map((part) => renderInsertTemplatePart(part.trim(), date)).join('')
}

function renderInsertTemplatePart(part: string, date: Date) {
  if (part === 'crlf()') return '\n'
  if (part === 'date()') return formatDate(date)
  if (part === 'time()') return formatTime(date)

  const charsMatch = part.match(/^chars\(['"](.+)['"],\s*(\d+)\)$/)
  if (charsMatch) return charsMatch[1].repeat(Number(charsMatch[2]))

  const quotedMatch = part.match(/^['"](.*)['"]$/)
  return quotedMatch ? quotedMatch[1] : part
}

function formatDate(date: Date) {
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
}

function formatTime(date: Date) {
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`
}

export function toHalfWidth(text: string) {
  return text.replace(/[\uff01-\uff5e]/g, (char) => String.fromCharCode(char.charCodeAt(0) - 0xfee0)).replace(/\u3000/g, ' ')
}

export function toFullWidth(text: string) {
  return text.replace(/[!-~]/g, (char) => String.fromCharCode(char.charCodeAt(0) + 0xfee0)).replace(/ /g, '\u3000')
}

export function convertChinese(text: string, map: Record<string, string>) {
  return text.replace(/./g, (char) => map[char] ?? char)
}

export async function digestText(algorithm: AlgorithmIdentifier, text: string, unsupportedMessage: string) {
  if (!crypto.subtle) throw new Error(unsupportedMessage)
  const hash = await crypto.subtle.digest(algorithm, new TextEncoder().encode(text))
  return Array.from(new Uint8Array(hash))
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')
}
