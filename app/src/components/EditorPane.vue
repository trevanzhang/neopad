<script setup lang="ts">
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { bracketMatching } from '@codemirror/language'
import { Compartment, EditorState } from '@codemirror/state'
import {
  drawSelection,
  dropCursor,
  EditorView,
  highlightActiveLine,
  keymap,
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
  history(),
  drawSelection(),
  dropCursor(),
  bracketMatching(),
  highlightActiveLine(),
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
      backgroundColor: '#ffffff',
      fontSize: '15px',
    },
    '.cm-scroller': {
      fontFamily: '"Segoe UI", Arial, sans-serif',
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
