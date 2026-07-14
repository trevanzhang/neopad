<script setup lang="ts">
import { defaultKeymap, history, historyKeymap, indentWithTab, redo, undo } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { bracketMatching, syntaxHighlighting } from '@codemirror/language'
import {
  closeSearchPanel,
  findNext,
  openSearchPanel,
  search,
} from '@codemirror/search'
import { Compartment, EditorSelection, EditorState } from '@codemirror/state'
import { getCM, Vim, vim } from '@replit/codemirror-vim'
import {
  drawSelection,
  dropCursor,
  EditorView,
  highlightActiveLine,
  keymap,
} from '@codemirror/view'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { evaluateExpressionLine, formatCalculationResult } from '../lib/editor-calculation'
import { editorCodeLanguages } from '../lib/editor-code-languages'
import { createNeopadSearchPanel, type EditorSearchLabels } from '../lib/editor-search-panel'
import { baseEditorTheme, editorAppearance, neopadHighlightStyle } from '../lib/editor-theme'
import { contextMatches, createAiEditorSnapshot } from '../lib/ai-editor'
import { createAiSlashExtension, type AiSlashLabels } from '../lib/ai-slash-commands'
import type { AiCommandName, AiEditorContext } from '../types/ai'

const props = defineProps<{
  title: string
  wordWrap: boolean
  fontFamily: string
  fontSize: number
  backgroundColor: string
  vimMode: boolean
  vimInsertExitKey: string
  searchLabels: EditorSearchLabels
  aiSlashLabels: AiSlashLabels
}>()

const emit = defineEmits<{
  vimModeChange: [mode: string]
  vimTabSwitch: [offset: -1 | 1]
  aiCommand: [command: AiCommandName]
}>()

const model = defineModel<string>({ required: true })
const editorRoot = ref<HTMLDivElement | null>(null)
let editorView: EditorView | null = null
const editable = new Compartment()
const wrap = new Compartment()
const appearance = new Compartment()
const vimSupport = new Compartment()
const searchSupport = new Compartment()
const aiSlashSupport = new Compartment()
let vimModeChangeHandler: ((event: { mode: string; subMode?: string }) => void) | null = null
let mappedInsertExitKey = ''
function registerNeopadVimTabMappings() {
  const neopadVim = Vim as typeof Vim & { neopadTabMappingsRegistered?: boolean }
  if (neopadVim.neopadTabMappingsRegistered) return
  neopadVim.neopadTabMappingsRegistered = true
  Vim.defineAction('neopadNextTab', (cm) => {
    cm.cm6.dom.dispatchEvent(new CustomEvent('neopad-vim-tab-switch', { bubbles: true, detail: 1 }))
  })
  Vim.defineAction('neopadPreviousTab', (cm) => {
    cm.cm6.dom.dispatchEvent(new CustomEvent('neopad-vim-tab-switch', { bubbles: true, detail: -1 }))
  })
  Vim.mapCommand('gt', 'action', 'neopadNextTab', {}, { context: 'normal' })
  Vim.mapCommand('gT', 'action', 'neopadPreviousTab', {}, { context: 'normal' })
}

const extensions = [
  vimSupport.of(props.vimMode ? vim() : []),
  history(),
  drawSelection(),
  dropCursor(),
  bracketMatching(),
  highlightActiveLine(),
  markdown({ codeLanguages: editorCodeLanguages }),
  syntaxHighlighting(neopadHighlightStyle),
  searchSupport.of(createSearchExtension()),
  aiSlashSupport.of(createAiSlashExtension(props.aiSlashLabels)),
  keymap.of([indentWithTab, ...defaultKeymap, ...historyKeymap]),
  editable.of(EditorView.editable.of(true)),
  wrap.of(props.wordWrap ? EditorView.lineWrapping : []),
  appearance.of(editorAppearance(props.backgroundColor, props.fontFamily, props.fontSize)),
  EditorView.updateListener.of((update) => {
    if (!update.docChanged) {
      return
    }

    const nextValue = update.state.doc.toString()
    if (nextValue !== model.value) {
      model.value = nextValue
    }
  }),
  baseEditorTheme(),
]

onMounted(() => {
  if (!editorRoot.value) {
    return
  }

  registerNeopadVimTabMappings()
  updateInsertExitMapping(props.vimInsertExitKey)
  editorView = new EditorView({
    state: EditorState.create({
      doc: model.value,
      extensions,
    }),
    parent: editorRoot.value,
  })
  connectVimModeListener()
  editorRoot.value.addEventListener('neopad-vim-tab-switch', handleVimTabSwitch)
  editorRoot.value.addEventListener('neopad-ai-command', handleAiCommand)
})

onBeforeUnmount(() => {
  editorRoot.value?.removeEventListener('neopad-vim-tab-switch', handleVimTabSwitch)
  editorRoot.value?.removeEventListener('neopad-ai-command', handleAiCommand)
  disconnectVimModeListener()
  updateInsertExitMapping('')
  editorView?.destroy()
  editorView = null
})

function handleVimTabSwitch(event: Event) {
  const offset = (event as CustomEvent<number>).detail === -1 ? -1 : 1
  emit('vimTabSwitch', offset)
}

function handleAiCommand(event: Event) {
  emit('aiCommand', (event as CustomEvent<AiCommandName>).detail)
}

watch(
  () => props.vimMode,
  (enabled) => {
    if (!editorView) return
    disconnectVimModeListener()
    editorView.dispatch({ effects: vimSupport.reconfigure(enabled ? vim() : []) })
    connectVimModeListener()
  },
)

watch(
  () => props.vimInsertExitKey,
  (key) => updateInsertExitMapping(key),
)

watch(
  () => props.searchLabels,
  () => editorView?.dispatch({ effects: searchSupport.reconfigure(createSearchExtension()) }),
)

watch(
  () => props.aiSlashLabels,
  (labels) => editorView?.dispatch({ effects: aiSlashSupport.reconfigure(createAiSlashExtension(labels)) }),
)

function createSearchExtension() {
  return search({
    top: true,
    createPanel: (view) => createNeopadSearchPanel(view, props.searchLabels),
  })
}

function updateInsertExitMapping(key: string) {
  if (mappedInsertExitKey) {
    Vim.unmap(mappedInsertExitKey, 'insert')
  }
  mappedInsertExitKey = key
  if (mappedInsertExitKey) {
    Vim.map(mappedInsertExitKey, '<Esc>', 'insert')
  }
}

function connectVimModeListener() {
  if (!editorView || !props.vimMode) {
    emit('vimModeChange', '')
    return
  }

  const cm = getCM(editorView)
  if (!cm) return
  vimModeChangeHandler = (event) => {
    emit('vimModeChange', event.subMode ? `${event.mode}-${event.subMode}` : event.mode)
  }
  cm.on('vim-mode-change', vimModeChangeHandler)
  emit('vimModeChange', cm.state.vim?.mode ?? 'normal')
}

function disconnectVimModeListener() {
  if (editorView && vimModeChangeHandler) {
    getCM(editorView)?.off('vim-mode-change', vimModeChangeHandler)
  }
  vimModeChangeHandler = null
}

watch(model, (nextValue) => {
  if (!editorView) {
    return
  }

  const currentValue = editorView.state.doc.toString()
  if (nextValue === currentValue) {
    return
  }

  editorView.dispatch({
    changes: {
      from: 0,
      to: editorView.state.doc.length,
      insert: nextValue,
    },
  })
})

watch(
  () => props.wordWrap,
  (wordWrap) => {
    editorView?.dispatch({
      effects: wrap.reconfigure(wordWrap ? EditorView.lineWrapping : []),
    })
  },
)

watch(
  () => [props.fontFamily, props.fontSize, props.backgroundColor],
  () => {
    editorView?.dispatch({
      effects: appearance.reconfigure(editorAppearance(props.backgroundColor, props.fontFamily, props.fontSize)),
    })
  },
)

function runEditorCommand(command: (view: EditorView) => boolean) {
  if (!editorView) {
    return false
  }

  editorView.focus()
  return command(editorView)
}

function focusEditor() {
  editorView?.focus()
}

function closeEditorFind() {
  if (!editorView) return false
  const closed = closeSearchPanel(editorView)
  if (closed) editorView.focus()
  return closed
}

function isEditorFocused() {
  return editorView?.hasFocus ?? false
}

function undoEdit() {
  return runEditorCommand(undo)
}

function redoEdit() {
  return runEditorCommand(redo)
}

async function copySelection() {
  if (!editorView) {
    return false
  }

  const selectedText = editorView.state.selection.ranges
    .filter((range) => !range.empty)
    .map((range) => editorView?.state.doc.sliceString(range.from, range.to) ?? '')
    .join('\n')

  if (!selectedText) {
    return false
  }

  await navigator.clipboard.writeText(selectedText)
  editorView.focus()
  return true
}

async function cutSelection() {
  if (!editorView || !(await copySelection())) {
    return false
  }

  editorView.dispatch({
    changes: editorView.state.selection.ranges
      .filter((range) => !range.empty)
      .map((range) => ({ from: range.from, to: range.to, insert: '' })),
  })
  editorView.focus()
  return true
}

async function pasteClipboard() {
  if (!editorView) {
    return false
  }

  const text = await navigator.clipboard.readText()
  editorView.dispatch(editorView.state.replaceSelection(text))
  editorView.focus()
  return true
}

function selectAllText() {
  if (!editorView) {
    return false
  }

  editorView.dispatch({
    selection: EditorSelection.single(0, editorView.state.doc.length),
  })
  editorView.focus()
  return true
}

function openEditorFind() {
  if (!editorView) return false
  const opened = runEditorCommand(openSearchPanel)
  requestAnimationFrame(() => editorRoot.value?.querySelector<HTMLInputElement>('.cm-search input[name="search"]')?.focus())
  return opened
}

function openEditorReplace() {
  if (!editorView) return false
  const opened = runEditorCommand(openSearchPanel)
  requestAnimationFrame(() => {
    const panel = editorRoot.value?.querySelector<HTMLElement>('.np-find-panel')
    panel?.classList.add('is-replace-open')
    const toggle = panel?.querySelector<HTMLButtonElement>('.np-find-replace-toggle')
    toggle?.setAttribute('aria-pressed', 'true')
    if (toggle) {
      toggle.title = props.searchLabels.hideReplace
    }
    const replaceField = editorRoot.value?.querySelector<HTMLInputElement>('.cm-search input[name="replace"]')
    replaceField?.focus()
    replaceField?.select()
  })
  return opened
}

function findNextMatch() {
  return runEditorCommand(findNext)
}

function insertText(text: string) {
  if (!editorView) {
    return false
  }

  editorView.dispatch(editorView.state.replaceSelection(text))
  editorView.focus()
  return true
}

function insertLine(text: string) {
  if (!editorView) {
    return false
  }

  const selection = editorView.state.selection.main
  const needsLeadingBreak = selection.from > 0
    && editorView.state.doc.sliceString(selection.from - 1, selection.from) !== '\n'
  const needsTrailingBreak = selection.to < editorView.state.doc.length
    && editorView.state.doc.sliceString(selection.to, selection.to + 1) !== '\n'
  const insertion = `${needsLeadingBreak ? '\n' : ''}${text}${needsTrailingBreak ? '\n' : ''}`
  editorView.dispatch(editorView.state.replaceSelection(insertion))
  editorView.focus()
  return true
}

function captureAiSnapshot() {
  if (!editorView) return null
  const selection = editorView.state.selection.main
  return createAiEditorSnapshot(
    editorView.state.doc.toString(),
    { from: selection.from, to: selection.to },
    selection.head,
  )
}

function replaceAiContext(context: AiEditorContext, text: string) {
  if (!editorView || !contextMatches(editorView.state.doc.toString(), context)) return false
  editorView.dispatch({
    changes: { from: context.from, to: context.to, insert: text },
    selection: EditorSelection.single(context.from, context.from + text.length),
    effects: EditorView.scrollIntoView(context.from, { y: 'center' }),
  })
  editorView.focus()
  return true
}

function insertAiAtCursor(text: string) {
  if (!editorView) return false
  const position = editorView.state.selection.main.head
  editorView.dispatch({
    changes: { from: position, insert: text },
    selection: EditorSelection.cursor(position + text.length),
    effects: EditorView.scrollIntoView(position + text.length, { y: 'center' }),
  })
  editorView.focus()
  return true
}

function insertAiBelow(context: AiEditorContext, text: string) {
  if (!editorView || !contextMatches(editorView.state.doc.toString(), context)) return false
  const needsBreak = context.to > 0 && !text.startsWith('\n')
  const insertion = `${needsBreak ? '\n\n' : ''}${text}`
  editorView.dispatch({
    changes: { from: context.to, insert: insertion },
    selection: EditorSelection.cursor(context.to + insertion.length),
    effects: EditorView.scrollIntoView(context.to + insertion.length, { y: 'center' }),
  })
  editorView.focus()
  return true
}

function goToLine(lineNumber: number) {
  if (!editorView || lineNumber < 1 || lineNumber > editorView.state.doc.lines) {
    return false
  }

  const line = editorView.state.doc.line(lineNumber)
  editorView.dispatch({
    selection: EditorSelection.cursor(line.from),
    effects: EditorView.scrollIntoView(line.from, { y: 'center' }),
  })
  editorView.focus()
  return true
}

async function transformText(transform: (text: string) => string | Promise<string>) {
  if (!editorView) {
    return false
  }

  const selection = editorView.state.selection.main
  const from = selection.empty ? 0 : selection.from
  const to = selection.empty ? editorView.state.doc.length : selection.to
  const currentText = editorView.state.doc.sliceString(from, to)
  const nextText = await transform(currentText)

  editorView.dispatch({
    changes: { from, to, insert: nextText },
    selection: EditorSelection.single(from, from + nextText.length),
  })
  editorView.focus()
  return true
}

function appendCurrentLineCalculation() {
  if (!editorView) {
    return false
  }

  const cursor = editorView.state.selection.main.head
  const line = editorView.state.doc.lineAt(cursor)
  const lineText = line.text.trimEnd()
  const sourceText = lineText.replace(/\s*=\s*[-+]?\d+(?:\.\d+)?(?:e[-+]?\d+)?\s*$/i, '')
  const result = evaluateExpressionLine(sourceText)

  if (result === null) {
    editorView.focus()
    return false
  }

  const nextLine = `${sourceText} = ${formatCalculationResult(result)}`
  const hasFollowingLine = line.to < editorView.state.doc.length
  const insert = hasFollowingLine ? nextLine : `${nextLine}\n`
  const nextCursor = line.from + nextLine.length + 1
  editorView.dispatch({
    changes: { from: line.from, to: line.to, insert },
    selection: EditorSelection.cursor(nextCursor),
    effects: EditorView.scrollIntoView(nextCursor, { y: 'center' }),
  })
  editorView.focus()
  return true
}

defineExpose({
  focusEditor,
  isEditorFocused,
  undoEdit,
  redoEdit,
  cutSelection,
  copySelection,
  pasteClipboard,
  selectAllText,
  openEditorFind,
  closeEditorFind,
  openEditorReplace,
  findNextMatch,
  insertText,
  insertLine,
  captureAiSnapshot,
  replaceAiContext,
  insertAiAtCursor,
  insertAiBelow,
  goToLine,
  transformText,
  appendCurrentLineCalculation,
})
</script>

<template>
  <section class="editor-pane" :aria-label="`${title} editor`">
    <div ref="editorRoot" class="code-editor" role="textbox" aria-label="Markdown note editor" />
  </section>
</template>
