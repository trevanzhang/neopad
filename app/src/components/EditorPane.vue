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
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { evaluateExpressionLine, formatCalculationResult } from '../lib/editor-calculation'
import { editorCodeLanguages } from '../lib/editor-code-languages'
import { createNeopadSearchPanel, type EditorSearchLabels } from '../lib/editor-search-panel'
import { baseEditorTheme, editorAppearance, neopadHighlightStyle } from '../lib/editor-theme'
import { contextMatches, createAiEditorSnapshot } from '../lib/ai-editor'
import {
  applyAiInlineCommandResult,
  createAiInlineStatusExtension,
  removeAiInlineStatus,
  showAiInlineError,
  showAiInlineLoading,
} from '../lib/ai-inline-status'
import type { AiInlineApplyAction } from '../lib/ai-inline-command'
import { createAiSlashExtension, type AiSlashLabels } from '../lib/ai-slash-commands'
import type { AiEditorContext, AiInlineCommandName } from '../types/ai'

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
  contextMenuLabels: {
    cut: string
    copy: string
    paste: string
    selectAll: string
    aiActions: string
    polish: string
    summarize: string
    translate: string
  }
}>()

const emit = defineEmits<{
  vimModeChange: [mode: string]
  vimTabSwitch: [offset: -1 | 1]
  aiCommand: [command: AiInlineCommandName]
}>()

const model = defineModel<string>({ required: true })
const editorRoot = ref<HTMLDivElement | null>(null)
const contextMenuElement = ref<HTMLElement | null>(null)
const aiSubmenuTrigger = ref<HTMLButtonElement | null>(null)
const aiSubmenuOpen = ref(false)
const contextMenu = ref<{ x: number; y: number; openSubmenuLeft: boolean } | null>(null)
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
  createAiInlineStatusExtension(),
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
  window.addEventListener('pointerdown', closeContextMenu)
  window.addEventListener('keydown', handleContextMenuKeydown, { capture: true })
})

onBeforeUnmount(() => {
  editorRoot.value?.removeEventListener('neopad-vim-tab-switch', handleVimTabSwitch)
  editorRoot.value?.removeEventListener('neopad-ai-command', handleAiCommand)
  window.removeEventListener('pointerdown', closeContextMenu)
  window.removeEventListener('keydown', handleContextMenuKeydown, { capture: true })
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
  emit('aiCommand', (event as CustomEvent<AiInlineCommandName>).detail)
}

function openContextMenu(event: MouseEvent) {
  if (!editorView) return
  const selection = editorView.state.selection.main
  const position = editorView.posAtCoords({ x: event.clientX, y: event.clientY })
  if (selection.empty || position === null || position < selection.from || position > selection.to) {
    contextMenu.value = null
    return
  }

  event.preventDefault()
  event.stopPropagation()
  aiSubmenuOpen.value = false
  const x = Math.max(4, Math.min(event.clientX, window.innerWidth - 212))
  contextMenu.value = {
    x,
    y: Math.max(4, Math.min(event.clientY, window.innerHeight - 240)),
    openSubmenuLeft: x + 384 > window.innerWidth && x >= 180,
  }
  void nextTick(() => contextMenuElement.value?.querySelector<HTMLButtonElement>('button')?.focus())
}

function closeContextMenu(event?: Event) {
  if (event && contextMenuElement.value?.contains(event.target as Node)) return
  contextMenu.value = null
  aiSubmenuOpen.value = false
}

function handleContextMenuKeydown(event: KeyboardEvent) {
  if (!contextMenu.value || !contextMenuElement.value) return
  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopImmediatePropagation()
    contextMenu.value = null
    aiSubmenuOpen.value = false
    editorView?.focus()
    return
  }
  if (event.key === 'ArrowRight' && document.activeElement === aiSubmenuTrigger.value) {
    event.preventDefault()
    openAiSubmenu(true)
    return
  }
  const activeElement = document.activeElement as HTMLElement | null
  const submenu = activeElement?.closest<HTMLElement>('.editor-context-submenu')
  if (event.key === 'ArrowLeft' && submenu) {
    event.preventDefault()
    aiSubmenuOpen.value = false
    void nextTick(() => aiSubmenuTrigger.value?.focus())
    return
  }
  if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return

  const activeMenu = activeElement?.closest<HTMLElement>('[role="menu"]') ?? contextMenuElement.value
  const selector = activeMenu.classList.contains('editor-context-submenu')
    ? ':scope > button:not(:disabled)'
    : ':scope > button:not(:disabled), :scope > .editor-context-ai-menu > button:not(:disabled)'
  const items = Array.from(activeMenu.querySelectorAll<HTMLButtonElement>(selector))
  if (!items.length) return
  event.preventDefault()
  const current = items.indexOf(document.activeElement as HTMLButtonElement)
  const next = event.key === 'Home'
    ? 0
    : event.key === 'End'
      ? items.length - 1
      : event.key === 'ArrowDown'
        ? (current + 1 + items.length) % items.length
        : (current - 1 + items.length) % items.length
  items[next]?.focus()
}

function openAiSubmenu(focusFirst = false) {
  aiSubmenuOpen.value = true
  if (focusFirst) {
    void nextTick(() => contextMenuElement.value
      ?.querySelector<HTMLButtonElement>('.editor-context-submenu > button')
      ?.focus())
  }
}

async function runContextAction(action: 'cut' | 'copy' | 'paste' | 'select-all' | AiInlineCommandName) {
  contextMenu.value = null
  aiSubmenuOpen.value = false
  if (action === 'cut') await cutSelection()
  else if (action === 'copy') await copySelection()
  else if (action === 'paste') await pasteClipboard()
  else if (action === 'select-all') selectAllText()
  else {
    editorView?.focus()
    emit('aiCommand', action)
  }
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

function startAiInlineCommand(
  requestId: number,
  position: number,
  label: string,
  source?: AiEditorContext,
) {
  if (!editorView) return false
  showAiInlineLoading(editorView, requestId, position, label, source)
  return true
}

function failAiInlineCommand(requestId: number, label: string, detail?: string) {
  return editorView ? showAiInlineError(editorView, requestId, label, detail) : false
}

function cancelAiInlineCommand(requestId?: number) {
  if (!editorView) return false
  removeAiInlineStatus(editorView, requestId)
  return true
}

function applyAiInlineCommand(
  requestId: number,
  action: AiInlineApplyAction,
  context: AiEditorContext,
  text: string,
) {
  return editorView
    ? applyAiInlineCommandResult(editorView, requestId, action, context, text)
    : false
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
  startAiInlineCommand,
  failAiInlineCommand,
  cancelAiInlineCommand,
  applyAiInlineCommand,
  insertAiBelow,
  goToLine,
  transformText,
  appendCurrentLineCalculation,
})
</script>

<template>
  <section class="editor-pane" :aria-label="`${title} editor`">
    <div
      ref="editorRoot"
      class="code-editor"
      role="textbox"
      aria-label="Markdown note editor"
      @contextmenu="openContextMenu"
    />
    <div
      v-if="contextMenu"
      ref="contextMenuElement"
      class="editor-context-menu"
      role="menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @contextmenu.prevent
    >
      <button type="button" role="menuitem" @click="runContextAction('copy')">
        <span>{{ contextMenuLabels.copy }}</span>
        <span class="editor-context-shortcut">Ctrl+C</span>
      </button>
      <button type="button" role="menuitem" @click="runContextAction('cut')">
        <span>{{ contextMenuLabels.cut }}</span>
        <span class="editor-context-shortcut">Ctrl+X</span>
      </button>
      <button type="button" role="menuitem" @click="runContextAction('paste')">
        <span>{{ contextMenuLabels.paste }}</span>
        <span class="editor-context-shortcut">Ctrl+V</span>
      </button>
      <button type="button" role="menuitem" @click="runContextAction('select-all')">
        <span>{{ contextMenuLabels.selectAll }}</span>
        <span class="editor-context-shortcut">Ctrl+A</span>
      </button>
      <div class="menu-separator" role="separator" />
      <div
        class="editor-context-ai-menu"
        @pointerenter="openAiSubmenu()"
        @pointerleave="aiSubmenuOpen = false"
      >
        <button
          ref="aiSubmenuTrigger"
          type="button"
          role="menuitem"
          aria-haspopup="menu"
          :aria-expanded="aiSubmenuOpen"
          @click="openAiSubmenu(true)"
        >
          <span>{{ contextMenuLabels.aiActions }}</span>
          <span class="editor-context-arrow" aria-hidden="true">›</span>
        </button>
        <div
          v-if="aiSubmenuOpen"
          class="editor-context-submenu"
          :class="{ left: contextMenu.openSubmenuLeft }"
          role="menu"
        >
          <button type="button" role="menuitem" @click="runContextAction('polish')">
            {{ contextMenuLabels.polish }}
          </button>
          <button type="button" role="menuitem" @click="runContextAction('summarize')">
            {{ contextMenuLabels.summarize }}
          </button>
          <button type="button" role="menuitem" @click="runContextAction('translate')">
            {{ contextMenuLabels.translate }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
