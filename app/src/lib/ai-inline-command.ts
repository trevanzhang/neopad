import type {
  AiEditorContext,
  AiEditorSnapshot,
  AiInlineCommandName,
} from '../types/ai'

export type AiInlineApplyAction = 'replace' | 'insert'

export interface AiInlinePlan {
  context: AiEditorContext
  action: AiInlineApplyAction
  requestContext: string
}

const NOTE_REFERENCE_MAX_CHARS = 16_000
const NOTE_REFERENCE_HEAD_CHARS = 2_000
const OMITTED_NOTE_MARKER = '\n\n[... note context omitted ...]\n\n'

export function createAiInlinePlan(
  command: AiInlineCommandName,
  snapshot: AiEditorSnapshot,
): AiInlinePlan | null {
  const preferredKind = command === 'summarize'
    ? snapshot.contexts.some((context) => context.kind === 'selection') ? 'selection' : 'note'
    : snapshot.contexts.some((context) => context.kind === 'selection') ? 'selection' : 'paragraph'
  const context = snapshot.contexts.find((item) => item.kind === preferredKind)
  if (!context || !context.text.trim()) return null

  return {
    context,
    action: command === 'polish' || command === 'translate' ? 'replace' : 'insert',
    requestContext: createAiInlineRequestContext(command, snapshot, context),
  }
}

export function createAiInlineRequestContext(
  command: AiInlineCommandName,
  snapshot: AiEditorSnapshot,
  target: AiEditorContext,
) {
  if (command === 'summarize' && target.kind === 'note') {
    return `<operation_target kind="${target.kind}">\n${target.text}\n</operation_target>`
  }

  const focusFrom = command === 'continue' ? snapshot.cursor : target.from
  const focusTo = command === 'continue' ? snapshot.cursor : target.to
  const reference = boundedNoteReference(snapshot.documentText, focusFrom, focusTo)
  const operationTarget = command === 'continue'
    ? targetWithInsertionPoint(target, target.kind === 'selection' ? target.to : snapshot.cursor)
    : target.text

  return [
    `<note_reference truncated="${reference.truncated}">`,
    reference.text,
    '</note_reference>',
    `<operation_target kind="${target.kind}" from="${target.from}" to="${target.to}">`,
    operationTarget,
    '</operation_target>',
  ].join('\n')
}

function targetWithInsertionPoint(target: AiEditorContext, cursor: number) {
  const relativeCursor = Math.max(0, Math.min(cursor - target.from, target.text.length))
  return `${target.text.slice(0, relativeCursor)}<insertion_point />${target.text.slice(relativeCursor)}`
}

function boundedNoteReference(text: string, focusFrom: number, focusTo: number) {
  if (text.length <= NOTE_REFERENCE_MAX_CHARS) return { text, truncated: false }

  const normalizedFrom = Math.max(0, Math.min(focusFrom, text.length))
  const normalizedTo = Math.max(normalizedFrom, Math.min(focusTo, text.length))
  if (normalizedTo <= NOTE_REFERENCE_MAX_CHARS) {
    return {
      text: `${text.slice(0, NOTE_REFERENCE_MAX_CHARS)}${OMITTED_NOTE_MARKER}`,
      truncated: true,
    }
  }

  const windowBudget = NOTE_REFERENCE_MAX_CHARS
    - NOTE_REFERENCE_HEAD_CHARS
    - (OMITTED_NOTE_MARKER.length * 2)
  const focusCenter = Math.floor((normalizedFrom + normalizedTo) / 2)
  let windowFrom = Math.max(NOTE_REFERENCE_HEAD_CHARS, focusCenter - Math.floor(windowBudget / 2))
  let windowTo = Math.min(text.length, windowFrom + windowBudget)
  windowFrom = Math.max(NOTE_REFERENCE_HEAD_CHARS, windowTo - windowBudget)

  return {
    text: `${text.slice(0, NOTE_REFERENCE_HEAD_CHARS)}${OMITTED_NOTE_MARKER}`
      + `${text.slice(windowFrom, windowTo)}`
      + `${windowTo < text.length ? OMITTED_NOTE_MARKER : ''}`,
    truncated: true,
  }
}
