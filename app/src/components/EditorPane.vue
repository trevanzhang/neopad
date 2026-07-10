<script setup lang="ts">
import { defaultKeymap, history, historyKeymap, indentWithTab, redo, undo } from '@codemirror/commands'
import { markdown } from '@codemirror/lang-markdown'
import { bracketMatching } from '@codemirror/language'
import {
  closeSearchPanel,
  findNext,
  findPrevious,
  getSearchQuery,
  openSearchPanel,
  replaceAll,
  replaceNext,
  search,
  SearchQuery,
  selectMatches,
  setSearchQuery,
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
import type { Panel } from '@codemirror/view'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

const props = defineProps<{
  title: string
  wordWrap: boolean
  fontFamily: string
  fontSize: number
  backgroundColor: string
  vimMode: boolean
  vimInsertExitKey: string
}>()

const emit = defineEmits<{
  vimModeChange: [mode: string]
  vimTabSwitch: [offset: -1 | 1]
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
  markdown(),
  search({ top: true, createPanel: createNeopadSearchPanel }),
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
      position: 'relative',
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
    '.cm-panels.cm-panels-top': {
      position: 'absolute',
      top: '8px',
      right: '10px',
      left: 'auto',
      zIndex: '8',
      border: '0',
      backgroundColor: 'transparent',
    },
    '.cm-panel.cm-search.np-find-panel': {
      display: 'grid',
      gap: '5px',
      boxSizing: 'content-box',
      width: 'max-content',
      maxWidth: 'calc(100% - 20px)',
      padding: '6px 8px',
      color: 'var(--np-text)',
      backgroundColor: 'var(--np-chrome)',
      border: '1px solid var(--np-border)',
      borderRadius: '6px',
      boxShadow: 'var(--np-shadow)',
    },
    '.np-find-row': {
      display: 'grid',
      gridTemplateColumns: '210px auto auto auto auto',
      columnGap: '8px',
      rowGap: '5px',
      alignItems: 'center',
      minWidth: '0',
      width: 'max-content',
    },
    '.np-replace-row': {
      display: 'none',
      gridTemplateColumns: 'minmax(150px, 1fr) auto auto',
      gap: '5px',
      alignItems: 'center',
      minWidth: '0',
    },
    '.np-find-panel.is-replace-open .np-replace-row': {
      display: 'grid',
    },
    '.cm-panel.cm-search.np-find-panel input[type="search"], .cm-panel.cm-search.np-find-panel input[type="text"]': {
      width: '100%',
      minWidth: '0',
      height: '28px',
      padding: '0 7px',
      color: 'var(--np-text)',
      backgroundColor: 'var(--np-surface)',
      border: '1px solid var(--np-border)',
      borderRadius: '3px',
      fontSize: '13px',
    },
    '.cm-panel.cm-search.np-find-panel input[name="search"]': {
      width: '210px',
      maxWidth: '210px',
    },
    '.cm-panel.cm-search.np-find-panel button': {
      height: '28px',
      minWidth: '28px',
      margin: '0',
      padding: '0 7px',
      color: 'var(--np-text)',
      backgroundColor: 'var(--np-control)',
      backgroundImage: 'none',
      border: '1px solid var(--np-border)',
      borderRadius: '3px',
      fontSize: '12px',
      cursor: 'pointer',
    },
    '.cm-panel.cm-search.np-find-panel button:hover, .cm-panel.cm-search.np-find-panel button:focus-visible': {
      backgroundColor: 'var(--np-control-active)',
      borderColor: 'var(--np-accent)',
    },
    '.np-find-toggle[aria-pressed="true"]': {
      color: '#ffffff',
      backgroundColor: 'var(--np-accent) !important',
      borderColor: 'var(--np-accent) !important',
    },
    '.np-find-count': {
      minWidth: '34px',
      color: 'var(--np-muted)',
      textAlign: 'center',
      fontSize: '12px',
      whiteSpace: 'nowrap',
    },
    '.np-find-count.is-empty': {
      display: 'none',
    },
    '.np-find-nav, .np-find-options, .np-find-actions': {
      display: 'flex',
      flex: '0 0 auto',
      gap: '4px',
      minWidth: '0',
    },
    '.np-find-options': {
      paddingLeft: '1px',
      borderLeft: '1px solid var(--np-border)',
    },
    '.np-find-replace-toggle': {
      minWidth: '48px !important',
    },
    '.np-find-action': {
      minWidth: '48px !important',
    },
    '.np-find-close': {
      padding: '0',
      fontSize: '16px',
    },
    '@media (max-width: 680px)': {
      '.cm-panels.cm-panels-top': {
        right: '8px',
        left: '8px',
      },
      '.cm-panel.cm-search.np-find-panel': {
        width: 'auto',
      },
      '.np-find-row': {
        gridTemplateColumns: 'minmax(150px, 1fr) auto auto',
      },
      '.cm-panel.cm-search.np-find-panel input[name="search"]': {
        width: '100%',
        maxWidth: 'none',
      },
      '.np-find-nav, .np-find-options, .np-find-actions': {
        gridColumn: 'auto',
      },
      '.np-find-options': {
        gridColumn: '1 / -1',
        paddingLeft: '0',
        borderLeft: '0',
        justifyContent: 'flex-start',
      },
      '.np-find-actions': {
        gridColumn: '1 / -1',
        justifyContent: 'flex-end',
      },
      '.np-replace-row': {
        gridTemplateColumns: 'minmax(120px, 1fr) auto',
      },
      '.np-replace-row .np-find-action:last-child': {
        gridColumn: '1 / -1',
      },
    },
  })
}

function createNeopadSearchPanel(view: EditorView): Panel {
  const dom = document.createElement('div')
  dom.className = 'cm-search np-find-panel'

  const findRow = document.createElement('div')
  findRow.className = 'np-find-row'

  const searchField = document.createElement('input')
  searchField.type = 'search'
  searchField.name = 'search'
  searchField.placeholder = '查找当前笔记'
  searchField.setAttribute('aria-label', '查找当前笔记')
  searchField.setAttribute('main-field', 'true')

  const countLabel = document.createElement('span')
  countLabel.className = 'np-find-count'
  countLabel.setAttribute('aria-live', 'polite')

  const nav = document.createElement('div')
  nav.className = 'np-find-nav'
  const previousButton = findButton('上一个', '↑', () => findPrevious(view))
  const nextButton = findButton('下一个', '↓', () => findNext(view))
  const allButton = findButton('选择全部匹配', '全部', () => selectMatches(view))
  allButton.classList.add('np-find-action')
  nav.append(previousButton, nextButton, allButton)

  const options = document.createElement('div')
  options.className = 'np-find-options'
  const caseButton = toggleButton('区分大小写', 'Aa')
  const regexpButton = toggleButton('使用正则表达式', '.*')
  const wordButton = toggleButton('全词匹配', '词')
  options.append(caseButton.button, regexpButton.button, wordButton.button)

  const replaceToggle = findButton('显示替换', '替换', () => {
    const nextOpen = !dom.classList.contains('is-replace-open')
    setReplaceOpen(nextOpen)
    if (nextOpen) {
      replaceField.focus()
      replaceField.select()
    }
    return true
  })
  replaceToggle.classList.add('np-find-replace-toggle')
  replaceToggle.setAttribute('aria-pressed', 'false')

  const closeButton = findButton('关闭查找', '×', () => closeSearchPanel(view))
  closeButton.classList.add('np-find-close')

  const actions = document.createElement('div')
  actions.className = 'np-find-actions'
  actions.append(replaceToggle, closeButton)

  findRow.append(searchField, countLabel, nav, options, actions)

  const replaceRow = document.createElement('div')
  replaceRow.className = 'np-replace-row'

  const replaceField = document.createElement('input')
  replaceField.type = 'text'
  replaceField.name = 'replace'
  replaceField.placeholder = '替换为'
  replaceField.setAttribute('aria-label', '替换为')

  const replaceOneButton = findButton('替换当前匹配', '替换', () => replaceNext(view))
  replaceOneButton.classList.add('np-find-action')
  const replaceAllButton = findButton('替换全部匹配', '全部替换', () => replaceAll(view))
  replaceAllButton.classList.add('np-find-action')

  replaceRow.append(replaceField, replaceOneButton, replaceAllButton)
  dom.append(findRow, replaceRow)

  const commit = () => {
    const query = new SearchQuery({
      search: searchField.value,
      replace: replaceField.value,
      caseSensitive: caseButton.isPressed(),
      regexp: regexpButton.isPressed(),
      wholeWord: wordButton.isPressed(),
    })
    if (!query.eq(getSearchQuery(view.state))) {
      view.dispatch({ effects: setSearchQuery.of(query) })
    }
    updateCount()
  }

  const syncFromState = () => {
    const query = getSearchQuery(view.state)
    searchField.value = query.search
    replaceField.value = query.replace
    caseButton.setPressed(query.caseSensitive)
    regexpButton.setPressed(query.regexp)
    wordButton.setPressed(query.wholeWord)
    updateCount()
  }

  const updateCount = () => {
    const { active, total } = getFindMatchSummary(view)
    if (!searchField.value.trim()) {
      countLabel.textContent = ''
      countLabel.classList.add('is-empty')
    } else if (total === 0) {
      countLabel.textContent = '无结果'
      countLabel.classList.remove('is-empty')
    } else {
      countLabel.textContent = `${active}/${total}`
      countLabel.classList.remove('is-empty')
    }
  }

  const setReplaceOpen = (open: boolean) => {
    dom.classList.toggle('is-replace-open', open)
    replaceToggle.setAttribute('aria-pressed', String(open))
    replaceToggle.title = open ? '隐藏替换' : '显示替换'
  }

  searchField.addEventListener('input', commit)
  replaceField.addEventListener('input', commit)
  for (const toggle of [caseButton, regexpButton, wordButton]) {
    toggle.button.addEventListener('click', commit)
  }
  dom.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      event.preventDefault()
      closeSearchPanel(view)
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'f') {
      event.preventDefault()
      event.stopPropagation()
      searchField.focus()
      searchField.select()
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'r') {
      event.preventDefault()
      event.stopPropagation()
      setReplaceOpen(true)
      replaceField.focus()
      replaceField.select()
    } else if (event.key === 'Enter' && event.target === searchField) {
      event.preventDefault()
      ;(event.shiftKey ? findPrevious : findNext)(view)
    } else if (event.key === 'Enter' && event.target === replaceField) {
      event.preventDefault()
      replaceNext(view)
    }
  })

  dom.addEventListener('focusout', () => {
    window.setTimeout(() => {
      if (!dom.contains(document.activeElement)) {
        closeSearchPanel(view)
      }
    }, 0)
  })

  syncFromState()

  return {
    dom,
    top: true,
    update(update) {
      if (update.docChanged || update.selectionSet || update.transactions.some((transaction) => transaction.effects.some((effect) => effect.is(setSearchQuery)))) {
        syncFromState()
      }
    },
  }
}

function findButton(label: string, text: string, run: () => boolean) {
  const button = document.createElement('button')
  button.type = 'button'
  button.title = label
  button.setAttribute('aria-label', label)
  button.textContent = text
  button.addEventListener('click', () => {
    run()
  })
  return button
}

function toggleButton(label: string, text: string) {
  const button = findButton(label, text, () => true)
  button.classList.add('np-find-toggle')
  button.setAttribute('aria-pressed', 'false')
  button.addEventListener('click', () => {
    button.setAttribute('aria-pressed', String(button.getAttribute('aria-pressed') !== 'true'))
  })
  return {
    button,
    isPressed: () => button.getAttribute('aria-pressed') === 'true',
    setPressed: (pressed: boolean) => {
      button.setAttribute('aria-pressed', String(pressed))
    },
  }
}

function getFindMatchSummary(view: EditorView) {
  const query = getSearchQuery(view.state)
  if (!query.valid) {
    return { active: 0, total: 0 }
  }

  const selection = view.state.selection.main
  const cursor = query.getCursor(view.state, 0, view.state.doc.length)
  let total = 0
  let active = 0
  let firstAfterSelection = 0

  for (let next = cursor.next(); !next.done; next = cursor.next()) {
    const match = next.value
    total += 1
    if (match.from === selection.from && match.to === selection.to) {
      active = total
    } else if (!firstAfterSelection && match.from >= selection.from) {
      firstAfterSelection = total
    }
  }

  return {
    active: total === 0 ? 0 : active || firstAfterSelection || 1,
    total,
  }
}

function editorAppearance() {
  return EditorView.theme({
    '&': {
      backgroundColor: props.backgroundColor,
    },
    '.cm-scroller': {
      fontFamily: props.fontFamily,
      fontSize: `${props.fontSize}px`,
    },
  })
}

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
})

onBeforeUnmount(() => {
  editorRoot.value?.removeEventListener('neopad-vim-tab-switch', handleVimTabSwitch)
  disconnectVimModeListener()
  updateInsertExitMapping('')
  editorView?.destroy()
  editorView = null
})

function handleVimTabSwitch(event: Event) {
  const offset = (event as CustomEvent<number>).detail === -1 ? -1 : 1
  emit('vimTabSwitch', offset)
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
      toggle.title = '隐藏替换'
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
  closeEditorFind,
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
