<script setup lang="ts">
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { bracketMatching, defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Compartment, EditorState } from '@codemirror/state'
import {
  drawSelection,
  dropCursor,
  EditorView,
  highlightActiveLine,
  keymap,
  lineNumbers,
} from '@codemirror/view'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

defineProps<{
  title: string
}>()

const model = defineModel<string>({ required: true })
const editorRoot = ref<HTMLDivElement | null>(null)
let editorView: EditorView | null = null
const editable = new Compartment()

const extensions = [
  lineNumbers(),
  history(),
  drawSelection(),
  dropCursor(),
  bracketMatching(),
  highlightActiveLine(),
  syntaxHighlighting(defaultHighlightStyle),
  markdown(),
  keymap.of([indentWithTab, ...defaultKeymap, ...historyKeymap]),
  editable.of(EditorView.editable.of(true)),
  EditorView.lineWrapping,
  EditorView.updateListener.of((update) => {
    if (!update.docChanged) {
      return
    }

    const nextValue = update.state.doc.toString()
    if (nextValue !== model.value) {
      model.value = nextValue
    }
  }),
  EditorView.theme({
    '&': {
      height: '100%',
      color: 'var(--np-text)',
      backgroundColor: 'transparent',
      fontSize: '14px',
    },
    '.cm-scroller': {
      fontFamily:
        'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace',
      lineHeight: '1.55',
    },
    '.cm-content': {
      padding: '0',
    },
    '.cm-gutters': {
      backgroundColor: 'transparent',
      color: 'var(--np-muted)',
      border: '0',
    },
    '.cm-activeLine': {
      backgroundColor: '#eef4f8',
    },
    '.cm-activeLineGutter': {
      backgroundColor: '#eef4f8',
    },
    '.cm-focused': {
      outline: '0',
    },
  }),
]

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
</script>

<template>
  <section class="editor-pane" :aria-label="`${title} editor`">
    <div ref="editorRoot" class="code-editor" role="textbox" aria-label="Markdown note editor" />
  </section>
</template>
