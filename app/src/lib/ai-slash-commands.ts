import {
  autocompletion,
  completionStatus,
  startCompletion,
  type Completion,
  type CompletionContext,
} from '@codemirror/autocomplete'
import { syntaxTree } from '@codemirror/language'
import type { EditorState, Extension } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import type { AiCommandName } from '../types/ai'

export type AiSlashLabels = Record<AiCommandName, string>

const commands: AiCommandName[] = ['rewrite', 'summarize', 'translate', 'continue', 'ask']

function isCodeContext(state: EditorState, position: number) {
  let node = syntaxTree(state).resolveInner(position, -1)
  while (node) {
    if (node.name.toLowerCase().includes('code')) return true
    if (!node.parent) break
    node = node.parent
  }
  return false
}

function slashMatch(state: EditorState, position: number) {
  const line = state.doc.lineAt(position)
  const before = state.doc.sliceString(line.from, position)
  return before.match(/(?:^|\s)(\/[a-z]*)$/i)
}

function applyCommand(command: AiCommandName) {
  return (view: EditorView, _completion: Completion, from: number, to: number) => {
    view.dispatch({ changes: { from, to, insert: '' } })
    view.dom.dispatchEvent(new CustomEvent('neopad-ai-command', {
      bubbles: true,
      detail: command,
    }))
  }
}

export function createAiSlashExtension(labels: AiSlashLabels): Extension {
  return [
    autocompletion({
      activateOnTyping: true,
      override: [(context: CompletionContext) => slashCompletions(context, labels)],
    }),
    EditorView.updateListener.of((update) => {
      if (!update.docChanged || completionStatus(update.state) !== null) return
      if (!slashMatch(update.state, update.state.selection.main.head)) return
      queueMicrotask(() => {
        if (completionStatus(update.view.state) === null) startCompletion(update.view)
      })
    }),
  ]
}

export function slashCompletions(context: CompletionContext, labels: AiSlashLabels) {
  if (isCodeContext(context.state, context.pos)) return null
  const match = slashMatch(context.state, context.pos)
  if (!match) return null

  return {
    from: context.pos - match[1].length,
    validFor: /^\/[a-z]*$/i,
    options: commands.map((command) => ({
      label: `/${command}`,
      detail: labels[command],
      type: 'keyword',
      apply: applyCommand(command),
    })),
  }
}
