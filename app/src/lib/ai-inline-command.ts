import type {
  AiEditorContext,
  AiEditorSnapshot,
  AiInlineCommandName,
} from '../types/ai'

export type AiInlineApplyAction = 'replace' | 'insert'

export interface AiInlinePlan {
  context: AiEditorContext
  action: AiInlineApplyAction
}

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
  }
}
