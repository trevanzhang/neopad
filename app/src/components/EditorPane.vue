<script setup lang="ts">
import { defaultKeymap, history, historyKeymap, indentWithTab, redo, undo } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { bracketMatching } from '@codemirror/language'
import { findNext, openSearchPanel } from '@codemirror/search'
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

const props = defineProps<{
  title: string
  wordWrap: boolean
  fontFamily: string
  backgroundColor: string
  vimMode: boolean
  vimInsertExitKey: string
}>()

const emit = defineEmits<{
  vimModeChange: [mode: string]
}>()

const model = defineModel<string>({ required: true })
const editorRoot = ref<HTMLDivElement | null>(null)
let editorView: EditorView | null = null
const editable = new Compartment()
const wrap = new Compartment()
const appearance = new Compartment()
const vimSupport = new Compartment()
let vimModeChangeHandler: ((event: { mode: string; subMode?: string }) => void) | null = null
let mappedInsertExitKey = ''
type ExpressionToken = { type: 'number'; value: number } | { type: 'operator'; value: string } | { type: 'paren'; value: '(' | ')' }

const extensions = [
  vimSupport.of(props.vimMode ? vim() : []),
  history(),
  drawSelection(),
  dropCursor(),
  bracketMatching(),
  highlightActiveLine(),
  markdown(),
  keymap.of([indentWithTab, ...defaultKeymap, ...historyKeymap]),
  editable.of(EditorView.editable.of(true)),
  wrap.of(props.wordWrap ? EditorView.lineWrapping : []),
  appearance.of(editorAppearance()),
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

function baseEditorTheme() {
  return EditorView.theme({
    '&': {
      height: '100%',
      color: 'var(--np-text)',
      fontSize: '14px',
    },
    '.cm-scroller': {
      lineHeight: '1.45',
    },
    '.cm-content': {
      padding: '10px 12px',
      minHeight: '100%',
    },
    '.cm-line': {
      color: 'var(--np-text)',
      textDecoration: 'none',
      fontWeight: '400',
    },
    '.cm-gutters': {
      backgroundColor: 'transparent',
      color: 'var(--np-muted)',
      border: '0',
    },
    '.cm-activeLine': {
      backgroundColor: 'transparent',
    },
    '.cm-focused': {
      outline: '0',
    },
    '.cm-fat-cursor': {
      backgroundColor: 'var(--np-vim-cursor) !important',
      color: 'var(--np-vim-cursor-text) !important',
    },
    '.cm-cursor, .cm-dropCursor': {
      borderLeftColor: 'var(--np-vim-cursor) !important',
    },
    '&:not(.cm-focused) .cm-fat-cursor': {
      backgroundColor: 'transparent !important',
      color: 'transparent !important',
      outline: '1px solid var(--np-vim-cursor)',
    },
    '.cm-panels': {
      color: 'var(--np-text)',
      backgroundColor: 'var(--np-chrome)',
      borderTop: '1px solid var(--np-border)',
    },
    '.cm-panel.cm-search': {
      display: 'flex',
      flexWrap: 'wrap',
      gap: '5px',
      alignItems: 'center',
      padding: '6px 8px',
    },
    '.cm-panel.cm-search br': {
      display: 'none',
    },
    '.cm-panel.cm-search input': {
      width: '140px',
      height: '26px',
      padding: '0 7px',
      color: 'var(--np-text)',
      backgroundColor: 'var(--np-surface)',
      border: '1px solid var(--np-border)',
      borderRadius: '2px',
      fontSize: '13px',
    },
    '.cm-panel.cm-search button': {
      height: '26px',
      padding: '0 8px',
      color: 'var(--np-text)',
      backgroundColor: 'var(--np-control)',
      backgroundImage: 'none',
      border: '1px solid var(--np-border)',
      borderRadius: '2px',
      fontSize: '12px',
      cursor: 'pointer',
    },
    '.cm-panel.cm-search label': {
      display: 'inline-flex',
      gap: '3px',
      alignItems: 'center',
      fontSize: '12px',
    },
    '.cm-panel.cm-search input[type="checkbox"]': {
      width: '14px',
      height: '14px',
      padding: '0',
    },
    '.cm-panel.cm-search button[name="close"]': {
      position: 'static',
      marginLeft: 'auto',
    },
  })
}

function editorAppearance() {
  return EditorView.theme({
    '&': {
      backgroundColor: props.backgroundColor,
    },
    '.cm-scroller': {
      fontFamily: props.fontFamily,
    },
  })
}

onMounted(() => {
  if (!editorRoot.value) {
    return
  }

  updateInsertExitMapping(props.vimInsertExitKey)
  editorView = new EditorView({
    state: EditorState.create({
      doc: model.value,
      extensions,
    }),
    parent: editorRoot.value,
  })
  connectVimModeListener()
})

onBeforeUnmount(() => {
  disconnectVimModeListener()
  updateInsertExitMapping('')
  editorView?.destroy()
  editorView = null
})

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
  () => [props.fontFamily, props.backgroundColor],
  () => {
    editorView?.dispatch({
      effects: appearance.reconfigure(editorAppearance()),
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
  requestAnimationFrame(() => editorRoot.value?.querySelector<HTMLInputElement>('.cm-search input[name="replace"]')?.focus())
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
  editorView.dispatch({
    changes: { from: line.from, to: line.to, insert: nextLine },
    selection: EditorSelection.single(line.from + nextLine.length),
  })
  editorView.focus()
  return true
}

function evaluateExpressionLine(lineText: string) {
  const expression = extractExpression(lineText)
  if (!expression) {
    return null
  }

  try {
    const tokens = tokenizeExpression(expression)
    if (tokens.length === 0) {
      return null
    }

    const result = evaluateTokens(tokens)
    return Number.isFinite(result) ? result : null
  } catch {
    return null
  }
}

function extractExpression(lineText: string) {
  const normalized = lineText.replace(/×/g, '*').replace(/÷/g, '/').replace(/，/g, ',')
  const start = normalized.search(/[-+.\d(]/)
  if (start === -1) {
    return ''
  }

  let expression = ''
  for (let index = start; index < normalized.length; index += 1) {
    const char = normalized[index]
    if (/[\d+\-*/%^().\s]/.test(char)) {
      expression += char
      continue
    }
    break
  }

  expression = expression.trim().replace(/[+\-*/%^.\s]+$/g, '').trim()
  while (unmatchedOpenParens(expression) > 0) {
    expression += ')'
  }
  return expression
}

function unmatchedOpenParens(expression: string) {
  let depth = 0
  for (const char of expression) {
    if (char === '(') {
      depth += 1
    } else if (char === ')') {
      depth -= 1
    }
  }
  return Math.max(0, depth)
}

function tokenizeExpression(expression: string) {
  const tokens: ExpressionToken[] = []
  let index = 0

  while (index < expression.length) {
    const char = expression[index]
    if (/\s/.test(char)) {
      index += 1
      continue
    }

    if (char === '(' || char === ')') {
      tokens.push({ type: 'paren', value: char })
      index += 1
      continue
    }

    if (/[+\-*/%^]/.test(char)) {
      const previous = tokens[tokens.length - 1]
      const unary = (char === '+' || char === '-') && (!previous || previous.type === 'operator' || (previous.type === 'paren' && previous.value === '('))
      const nextChar = expression[index + 1]
      if (unary && /[\d.]/.test(nextChar ?? '')) {
        const numberMatch = expression.slice(index).match(/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?/i)
        if (!numberMatch) {
          throw new Error('invalid number')
        }
        tokens.push({ type: 'number', value: Number(numberMatch[0]) })
        index += numberMatch[0].length
        continue
      }

      tokens.push({ type: 'operator', value: char })
      index += 1
      continue
    }

    const numberMatch = expression.slice(index).match(/^(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?/i)
    if (!numberMatch) {
      throw new Error('invalid token')
    }
    tokens.push({ type: 'number', value: Number(numberMatch[0]) })
    index += numberMatch[0].length
  }

  return tokens
}

function evaluateTokens(tokens: ExpressionToken[]) {
  const output: ExpressionToken[] = []
  const operators: ExpressionToken[] = []
  const precedence: Record<string, number> = { '+': 1, '-': 1, '*': 2, '/': 2, '%': 2, '^': 3 }

  for (const token of tokens) {
    if (token.type === 'number') {
      output.push(token)
      continue
    }

    if (token.type === 'paren') {
      if (token.value === '(') {
        operators.push(token)
      } else {
        while (operators.length && !(operators[operators.length - 1].type === 'paren' && operators[operators.length - 1].value === '(')) {
          output.push(operators.pop() as ExpressionToken)
        }
        operators.pop()
      }
      continue
    }

    while (
      operators.length &&
      operators[operators.length - 1].type === 'operator' &&
      ((token.value === '^' && precedence[operators[operators.length - 1].value] > precedence[token.value]) ||
        (token.value !== '^' && precedence[operators[operators.length - 1].value] >= precedence[token.value]))
    ) {
      output.push(operators.pop() as ExpressionToken)
    }
    operators.push(token)
  }

  while (operators.length) {
    const operator = operators.pop() as ExpressionToken
    if (operator.type === 'paren') {
      throw new Error('unbalanced parentheses')
    }
    output.push(operator)
  }

  const stack: number[] = []
  for (const token of output) {
    if (token.type === 'number') {
      stack.push(token.value)
      continue
    }
    if (token.type !== 'operator' || stack.length < 2) {
      throw new Error('invalid expression')
    }
    const right = stack.pop() as number
    const left = stack.pop() as number
    stack.push(applyOperator(left, right, token.value))
  }

  if (stack.length !== 1) {
    throw new Error('invalid expression')
  }
  return stack[0]
}

function applyOperator(left: number, right: number, operator: string) {
  if (operator === '+') return left + right
  if (operator === '-') return left - right
  if (operator === '*') return left * right
  if (operator === '/') return left / right
  if (operator === '%') return left % right
  if (operator === '^') return left ** right
  throw new Error('unknown operator')
}

function formatCalculationResult(result: number) {
  const rounded = Math.abs(result) < 1e-12 ? 0 : result
  return Number.isInteger(rounded) ? String(rounded) : String(Number(rounded.toPrecision(12)))
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
  openEditorReplace,
  findNextMatch,
  insertText,
  insertLine,
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
