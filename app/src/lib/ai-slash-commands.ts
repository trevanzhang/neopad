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
import type { AiInlineCommandName } from '../types/ai'

export type AiSlashLabels = Record<AiInlineCommandName, string>

export const aiSlashCommands: AiInlineCommandName[] = ['continue', 'polish', 'summarize', 'translate']

function isCodeContext(state: EditorState, position: number) {
  let node = syntaxTree(state).resolveInner(position, -1)
  while (node) {
    if (node.name.toLowerCase().includes('code')) return true
    if (!node.parent) break
    node = node.parent
  }
  return false
}

export function matchAiSlashCommandPrefix(value: string) {
  const match = value.match(/((?:\/{2}|、、)[a-z]*)$/i)
  if (!match) return null
  const before = value.slice(0, match.index)
  if (/[\/、]$/.test(before)) return null
  if (/(?:https?|file|ftp):$/i.test(before)) return null
  const currentToken = before.slice(Math.max(before.lastIndexOf(' '), before.lastIndexOf('\t')) + 1)
  if (/[\/、]/.test(currentToken)) return null
  return match
}

function slashMatch(state: EditorState, position: number) {
  const line = state.doc.lineAt(position)
  const before = state.doc.sliceString(line.from, position)
  return matchAiSlashCommandPrefix(before)
}

function applyCommand(command: AiInlineCommandName) {
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
  const trigger = match[1].startsWith('、、') ? '、、' : '//'

  return {
    from: context.pos - match[1].length,
    validFor: /^(?:\/{2}|、、)[a-z]*$/i,
    options: aiSlashCommands.map((command) => ({
      label: `${trigger}${command}`,
      displayLabel: command,
      detail: labels[command],
      type: 'ai-command',
      apply: applyCommand(command),
    })),
  }
}
