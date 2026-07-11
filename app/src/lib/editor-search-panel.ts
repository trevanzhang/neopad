import {
  closeSearchPanel,
  findNext,
  findPrevious,
  getSearchQuery,
  replaceAll,
  replaceNext,
  SearchQuery,
  selectMatches,
  setSearchQuery,
} from '@codemirror/search'
import type { Panel } from '@codemirror/view'
import type { EditorView } from '@codemirror/view'
import type { EditorState } from '@codemirror/state'

export function createNeopadSearchPanel(view: EditorView): Panel {
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

  const updateCount = () => {
    const { active, total } = getFindMatchSummary(view.state)
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
  const commit = () => {
    const query = new SearchQuery({
      search: searchField.value,
      replace: replaceField.value,
      caseSensitive: caseButton.isPressed(),
      regexp: regexpButton.isPressed(),
      wholeWord: wordButton.isPressed(),
    })
    if (!query.eq(getSearchQuery(view.state))) view.dispatch({ effects: setSearchQuery.of(query) })
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
  const setReplaceOpen = (open: boolean) => {
    dom.classList.toggle('is-replace-open', open)
    replaceToggle.setAttribute('aria-pressed', String(open))
    replaceToggle.title = open ? '隐藏替换' : '显示替换'
  }

  searchField.addEventListener('input', commit)
  replaceField.addEventListener('input', commit)
  for (const toggle of [caseButton, regexpButton, wordButton]) toggle.button.addEventListener('click', commit)
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
      if (!dom.contains(document.activeElement)) closeSearchPanel(view)
    }, 0)
  })
  syncFromState()

  return {
    dom,
    top: true,
    update(update) {
      if (update.docChanged || update.selectionSet || update.transactions.some((transaction) =>
        transaction.effects.some((effect) => effect.is(setSearchQuery)))) syncFromState()
    },
  }
}

function findButton(label: string, text: string, run: () => boolean) {
  const button = document.createElement('button')
  button.type = 'button'
  button.title = label
  button.setAttribute('aria-label', label)
  button.textContent = text
  button.addEventListener('click', run)
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
    setPressed: (pressed: boolean) => button.setAttribute('aria-pressed', String(pressed)),
  }
}

export function getFindMatchSummary(state: EditorState) {
  const query = getSearchQuery(state)
  if (!query.valid) return { active: 0, total: 0 }

  const selection = state.selection.main
  const cursor = query.getCursor(state, 0, state.doc.length)
  let total = 0
  let active = 0
  let firstAfterSelection = 0
  for (let next = cursor.next(); !next.done; next = cursor.next()) {
    const match = next.value
    total += 1
    if (match.from === selection.from && match.to === selection.to) active = total
    else if (!firstAfterSelection && match.from >= selection.from) firstAfterSelection = total
  }
  return { active: total === 0 ? 0 : active || firstAfterSelection || 1, total }
}
