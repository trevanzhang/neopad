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

defineExpose({
  undoEdit,
  redoEdit,
  cutSelection,
  copySelection,
  pasteClipboard,
  selectAllText,
  insertText,
  transformText,
})
</script>

<template>
  <section class="editor-pane" :aria-label="`${title} editor`">
    <div ref="editorRoot" class="code-editor" role="textbox" aria-label="Markdown note editor" />
  </section>
</template>
