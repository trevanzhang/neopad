<script setup lang="ts">
import { defaultKeymap, history, historyKeymap, indentWithTab, redo, undo } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { bracketMatching } from '@codemirror/language'
import { Compartment, EditorSelection, EditorState } from '@codemirror/state'
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
}>()

const model = defineModel<string>({ required: true })
const editorRoot = ref<HTMLDivElement | null>(null)
let editorView: EditorView | null = null
const editable = new Compartment()
const wrap = new Compartment()
const appearance = new Compartment()
type ExpressionToken = { type: 'number'; value: number } | { type: 'operator'; value: string } | { type: 'paren'; value: '(' | ')' }

const extensions = [
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
      fontSize: '15px',
    },
    '.cm-scroller': {
      lineHeight: '1.45',
    },
    '.cm-content': {
      padding: '10px 12px',
      minHeight: '100%',
    },
    '.cm-line': {
      color: '#111111',
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

  editorView = new EditorView({
    state: EditorState.create({
      doc: model.value,
      extensions,
    }),
    parent: editorRoot.value,
  })
})

onBeforeUnmount(() => {
  editorView?.destroy()
  editorView = null
})

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

function insertText(text: string) {
  if (!editorView) {
    return false
  }

  editorView.dispatch(editorView.state.replaceSelection(text))
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
  undoEdit,
  redoEdit,
  cutSelection,
  copySelection,
  pasteClipboard,
  selectAllText,
  insertText,
  transformText,
  appendCurrentLineCalculation,
})
</script>

<template>
  <section class="editor-pane" :aria-label="`${title} editor`">
    <div ref="editorRoot" class="code-editor" role="textbox" aria-label="Markdown note editor" />
  </section>
</template>
