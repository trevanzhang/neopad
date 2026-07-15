const TITLE_WRAPPERS: Array<[string, string]> = [
  ['"', '"'],
  ["'", "'"],
  ['`', '`'],
  ['“', '”'],
  ['‘', '’'],
  ['《', '》'],
]

export function normalizeAiNoteTitle(output: string, maxLength = 80) {
  let title = output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find(Boolean) ?? ''

  title = title
    .replace(/^#{1,6}\s*/, '')
    .replace(/^[-*]\s+/, '')
    .replace(/^(?:title|标题)\s*[:：]\s*/i, '')
    .trim()

  const wrapper = TITLE_WRAPPERS.find(([start, end]) => (
    title.startsWith(start) && title.endsWith(end) && title.length > start.length + end.length
  ))
  if (wrapper) title = title.slice(wrapper[0].length, -wrapper[1].length).trim()

  return Array.from(title.replace(/\s+/g, ' ')).slice(0, maxLength).join('').trim()
}
