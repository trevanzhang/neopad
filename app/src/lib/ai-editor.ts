import type { AiContextKind, AiEditorContext, AiEditorSnapshot } from '../types/ai'

export interface TextRange {
  from: number
  to: number
}

export function paragraphRange(text: string, cursor: number): TextRange {
  if (!text) return { from: 0, to: 0 }

  const lines = text.split('\n')
  const offsets: number[] = []
  let offset = 0
  for (const line of lines) {
    offsets.push(offset)
    offset += line.length + 1
  }

  const position = Math.max(0, Math.min(cursor, text.length))
  let index = 0
  for (let candidate = 0; candidate < offsets.length; candidate += 1) {
    if (offsets[candidate] > position) break
    index = candidate
  }

  if (!lines[index].trim()) {
    let previous = index - 1
    while (previous >= 0 && !lines[previous].trim()) previous -= 1
    if (previous >= 0) {
      index = previous
    } else {
      let next = index + 1
      while (next < lines.length && !lines[next].trim()) next += 1
      if (next >= lines.length) return { from: position, to: position }
      index = next
    }
  }

  let first = index
  while (first > 0 && lines[first - 1].trim()) first -= 1
  let last = index
  while (last + 1 < lines.length && lines[last + 1].trim()) last += 1

  return {
    from: offsets[first],
    to: offsets[last] + lines[last].length,
  }
}

export function createAiEditorSnapshot(
  text: string,
  selection: TextRange,
  cursor: number,
): AiEditorSnapshot {
  const contexts: AiEditorContext[] = []
  if (selection.to > selection.from) {
    contexts.push({
      kind: 'selection',
      text: text.slice(selection.from, selection.to),
      from: selection.from,
      to: selection.to,
    })
  }

  const paragraph = paragraphRange(text, cursor)
  contexts.push({
    kind: 'paragraph',
    text: text.slice(paragraph.from, paragraph.to),
    ...paragraph,
  })
  contexts.push({ kind: 'note', text, from: 0, to: text.length })

  return {
    documentText: text,
    contexts,
    defaultKind: contexts[0]?.kind ?? ('paragraph' satisfies AiContextKind),
  }
}

export function contextMatches(text: string, context: AiEditorContext) {
  return context.from <= context.to
    && context.to <= text.length
    && text.slice(context.from, context.to) === context.text
}
