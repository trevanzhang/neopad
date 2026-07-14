import { StateEffect, StateField, type EditorState, type Extension, type Range } from '@codemirror/state'
import { Decoration, EditorView, WidgetType, type DecorationSet } from '@codemirror/view'
import type { AiEditorContext } from '../types/ai'
import type { AiInlineApplyAction } from './ai-inline-command'

interface AiInlineStatusSpec {
  requestId: number
  position: number
  kind: 'loading' | 'error'
  label: string
  detail?: string
  source?: { from: number; to: number }
}

interface ResolvedAiInlineStatus {
  anchor: number
  source?: { from: number; to: number }
}

const setAiInlineStatus = StateEffect.define<AiInlineStatusSpec>()
const clearAiInlineStatus = StateEffect.define<number | undefined>()

class AiInlineStatusWidget extends WidgetType {
  constructor(
    readonly requestId: number,
    readonly kind: 'loading' | 'error',
    readonly label: string,
    readonly detail?: string,
  ) {
    super()
  }

  eq(other: AiInlineStatusWidget) {
    return this.requestId === other.requestId && this.kind === other.kind && this.label === other.label
      && this.detail === other.detail
  }

  toDOM() {
    const element = document.createElement('span')
    element.className = `cm-ai-inline-status is-${this.kind}`
    element.setAttribute('role', 'status')
    element.setAttribute('aria-label', this.detail ?? this.label)
    element.title = this.detail ?? this.label
    if (this.kind === 'loading') {
      for (let index = 0; index < 3; index += 1) element.append(document.createElement('i'))
    } else {
      element.textContent = this.label
    }
    return element
  }

  ignoreEvent() {
    return true
  }
}

const aiInlineStatusField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, transaction) {
    value = value.map(transaction.changes)
    for (const effect of transaction.effects) {
      if (effect.is(clearAiInlineStatus)) {
        value = Decoration.none
      } else if (effect.is(setAiInlineStatus)) {
        const spec = effect.value
        const ranges: Range<Decoration>[] = [Decoration.widget({
          widget: new AiInlineStatusWidget(spec.requestId, spec.kind, spec.label, spec.detail),
          side: 1,
        }).range(spec.position)]
        if (spec.source && spec.source.to > spec.source.from) {
          ranges.push(Decoration.mark({
            class: 'cm-ai-inline-source',
            requestId: spec.requestId,
            aiSource: true,
          }).range(spec.source.from, spec.source.to))
        }
        value = Decoration.set(ranges, true)
      }
    }
    return value
  },
  provide: (field) => EditorView.decorations.from(field),
})

export function createAiInlineStatusExtension(): Extension {
  return aiInlineStatusField
}

export function showAiInlineLoading(
  view: EditorView,
  requestId: number,
  position: number,
  label: string,
  source?: AiEditorContext,
) {
  view.dispatch({
    effects: setAiInlineStatus.of({
      requestId,
      position: Math.max(0, Math.min(position, view.state.doc.length)),
      kind: 'loading',
      label,
      source: source ? { from: source.from, to: source.to } : undefined,
    }),
  })
}

export function showAiInlineError(view: EditorView, requestId: number, label: string, detail?: string) {
  const status = resolveAiInlineStatus(view.state, requestId)
  if (!status) return false
  view.dispatch({
    effects: setAiInlineStatus.of({
      requestId,
      position: status.anchor,
      kind: 'error',
      label,
      detail,
    }),
  })
  return true
}

export function removeAiInlineStatus(view: EditorView, requestId?: number) {
  view.dispatch({ effects: clearAiInlineStatus.of(requestId) })
}

export function applyAiInlineCommandResult(
  view: EditorView,
  requestId: number,
  action: AiInlineApplyAction,
  context: AiEditorContext,
  text: string,
) {
  const status = resolveAiInlineStatus(view.state, requestId)
  if (!status) return false

  const range = action === 'replace' ? status.source : { from: status.anchor, to: status.anchor }
  if (!range) return false
  if (action === 'replace' && view.state.doc.sliceString(range.from, range.to) !== context.text) return false

  view.dispatch({
    changes: { from: range.from, to: range.to, insert: text },
    selection: { anchor: range.from + text.length },
    effects: [
      clearAiInlineStatus.of(requestId),
      EditorView.scrollIntoView(range.from + text.length, { y: 'center' }),
    ],
  })
  view.focus()
  return true
}

function resolveAiInlineStatus(state: EditorState, requestId: number): ResolvedAiInlineStatus | null {
  const decorations = state.field(aiInlineStatusField, false)
  if (!decorations) return null
  let anchor: number | undefined
  let source: { from: number; to: number } | undefined
  const iterator = decorations.iter()
  while (iterator.value) {
    const decoration = iterator.value
    const widget = decoration.spec.widget
    if (widget instanceof AiInlineStatusWidget && widget.requestId === requestId) anchor = iterator.from
    if (decoration.spec.aiSource === true && decoration.spec.requestId === requestId) {
      source = { from: iterator.from, to: iterator.to }
    }
    iterator.next()
  }
  return anchor === undefined ? null : { anchor, source }
}
