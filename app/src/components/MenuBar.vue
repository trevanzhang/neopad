<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import type { AppMessages } from '../lib/i18n'
import type { EditorMode } from '../types/editor'
import type { NoteTab } from '../types/note'

const props = defineProps<{
  previewMode: EditorMode
  tabBarOrientation: 'horizontal' | 'vertical'
  wordWrap: boolean
  alwaysOnTop: boolean
  pageActionsEnabled: boolean
  activeTabArchived: boolean
  recentNotes: NoteTab[]
  messages: AppMessages['menu']
}>()

const emit = defineEmits<{
  newNote: []
  renamePage: []
  deletePage: []
  closePage: []
  archivePage: []
  unarchivePage: []
  openRecent: [noteId: string]
  saveClipboard: []
  loadFile: []
  saveAsFile: []
  exportAll: []
  viewArchive: []
  openTrash: []
  hideWindow: []
  exitApp: []
  undo: []
  cut: []
  copy: []
  paste: []
  find: []
  findNext: []
  replace: []
  globalSearch: []
  selectAll: []
  search: []
  settings: []
  togglePin: []
  updateTabBarOrientation: [orientation: 'horizontal' | 'vertical']
  formatFont: []
  formatBackground: []
  toggleWordWrap: []
  toggleTheme: []
  togglePreviewTheme: []
  insertSeparator: []
  insertDateTime: []
  insertDateTimeSeparator: []
  insertReminder: []
  insertTextSettings: []
  windowOpacity: []
  reminderList: []
  processText: [action: string]
  helpTopic: [topic: 'software' | 'markdown' | 'shortcuts' | 'expression' | 'about']
  updatePreviewMode: [mode: EditorMode]
}>()

function toggleArchive() {
  if (props.activeTabArchived) emit('unarchivePage')
  else emit('archivePage')
}

function handleMenuClick(event: MouseEvent) {
  const button = (event.target as Element | null)?.closest<HTMLButtonElement>('.menu-popover button')
  if (!button || button.disabled || button.parentElement?.classList.contains('menu-subroot')) {
    return
  }

  closeAllMenus()
}

function closeMenu(event: KeyboardEvent) {
  event.preventDefault()
  event.stopPropagation()
  ;(document.activeElement as HTMLElement | null)?.blur()
}

const menuBar = ref<HTMLElement | null>(null)
let focusBeforeMenu: HTMLElement | null = null
const mnemonicRoots: Record<string, number> = { f: 0, e: 1, v: 2, p: 3, o: 4, i: 5, t: 6, h: 7 }

onMounted(() => window.addEventListener('keydown', handleMenuKeydown, { capture: true }))
onBeforeUnmount(() => window.removeEventListener('keydown', handleMenuKeydown, { capture: true }))

function rootTitles() {
  return Array.from(menuBar.value?.querySelectorAll<HTMLButtonElement>(':scope > .menu-root > .menu-title') ?? [])
}

function openRoot(index: number) {
  const titles = rootTitles()
  if (!titles.length) return
  if (!menuBar.value?.contains(document.activeElement)) {
    focusBeforeMenu = document.activeElement as HTMLElement | null
  }
  titles[(index + titles.length) % titles.length]?.focus()
}

function directMenuButtons(popover: Element) {
  return Array.from(popover.children).flatMap((child) => {
    if (child instanceof HTMLButtonElement) return [child]
    if (child.classList.contains('menu-subroot')) {
      const button = child.querySelector<HTMLButtonElement>(':scope > button')
      return button ? [button] : []
    }
    return []
  }).filter((button) => !button.disabled)
}

function focusPopoverEdge(title: HTMLButtonElement, last = false) {
  const popover = title.parentElement?.querySelector(':scope > .menu-popover')
  if (!popover) return
  const buttons = directMenuButtons(popover)
  ;(last ? buttons.at(-1) : buttons[0])?.focus()
}

function closeAllMenus() {
  const active = document.activeElement as HTMLElement | null
  active?.blur()
  focusBeforeMenu?.focus()
  focusBeforeMenu = null
}

function handleMenuKeydown(event: KeyboardEvent) {
  const mnemonic = event.altKey && !event.ctrlKey && !event.metaKey ? mnemonicRoots[event.key.toLowerCase()] : undefined
  if (mnemonic !== undefined) {
    event.preventDefault()
    event.stopPropagation()
    openRoot(mnemonic)
    return
  }

  const active = document.activeElement as HTMLButtonElement | null
  if (!active || !menuBar.value?.contains(active)) return
  const titles = rootTitles()
  const currentRoot = active.closest('.menu-root')
  const rootIndex = titles.findIndex((title) => title.parentElement === currentRoot)
  const subpopover = active.closest('.menu-subpopover')
  const parentPopover = active.closest('.menu-popover')
  const subroot = active.parentElement?.classList.contains('menu-subroot') ? active.parentElement : null

  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    event.stopImmediatePropagation()
    if (subpopover) {
      subpopover.parentElement?.querySelector<HTMLButtonElement>(':scope > button')?.focus()
    } else {
      closeAllMenus()
    }
    return
  }

  if (event.key === 'ArrowRight') {
    event.preventDefault()
    if (subroot) {
      const submenu = subroot.querySelector(':scope > .menu-subpopover')
      const first = submenu ? directMenuButtons(submenu)[0] : null
      first?.focus()
    } else if (!subpopover) {
      openRoot(rootIndex + 1)
    }
    return
  }

  if (event.key === 'ArrowLeft') {
    event.preventDefault()
    if (subpopover) {
      subpopover.parentElement?.querySelector<HTMLButtonElement>(':scope > button')?.focus()
    } else {
      openRoot(rootIndex - 1)
    }
    return
  }

  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    if (active.classList.contains('menu-title')) {
      focusPopoverEdge(active, event.key === 'ArrowUp')
      return
    }
    if (!parentPopover) return
    const buttons = directMenuButtons(parentPopover)
    const current = buttons.indexOf(active)
    const direction = event.key === 'ArrowDown' ? 1 : -1
    buttons[(current + direction + buttons.length) % buttons.length]?.focus()
    return
  }

  if ((event.key === 'Enter' || event.key === ' ') && subroot) {
    event.preventDefault()
    const submenu = subroot.querySelector(':scope > .menu-subpopover')
    const first = submenu ? directMenuButtons(submenu)[0] : null
    first?.focus()
  }
}
</script>

<template>
  <nav ref="menuBar" class="menu-bar" aria-label="Application menu" @click="handleMenuClick" @keydown.esc="closeMenu">
    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.file }}</button>
      <div class="menu-popover">
        <button type="button" class="menu-command" @click="$emit('loadFile')">
          <span>{{ messages.loadFromFile }}</span>
          <span class="menu-shortcut">{{ messages.ctrlO }}</span>
        </button>
        <div class="menu-subroot">
          <button type="button" class="menu-command">
            <span>{{ messages.recentDocuments }}</span>
            <span class="menu-arrow">&rsaquo;</span>
          </button>
          <div class="menu-popover menu-subpopover">
            <button v-for="note in recentNotes" :key="note.id" type="button" @click="$emit('openRecent', note.id)">
              {{ note.title }}
            </button>
          </div>
        </div>
        <button type="button" @click="$emit('saveAsFile')">{{ messages.saveAsFile }}</button>
        <div class="menu-separator" role="separator" />
        <button type="button" @click="$emit('exportAll')">{{ messages.exportAll }}</button>
        <div class="menu-separator" role="separator" />
        <button type="button" @click="$emit('viewArchive')">{{ messages.viewArchive }}</button>
        <button type="button" @click="$emit('openTrash')">{{ messages.trash }}</button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" @click="$emit('hideWindow')">
          <span>{{ messages.hide }}</span>
          <span class="menu-shortcut">{{ messages.esc }}</span>
        </button>
        <button type="button" @click="$emit('exitApp')">{{ messages.exit }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.edit }}</button>
      <div class="menu-popover">
        <button type="button" class="menu-command menu-muted" @click="$emit('undo')">
          <span>{{ messages.undo }}</span>
          <span class="menu-shortcut">{{ messages.ctrlZ }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" @click="$emit('cut')">
          <span>{{ messages.cut }}</span>
          <span class="menu-shortcut">{{ messages.ctrlX }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('copy')">
          <span>{{ messages.copy }}</span>
          <span class="menu-shortcut">{{ messages.ctrlC }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('paste')">
          <span>{{ messages.paste }}</span>
          <span class="menu-shortcut">{{ messages.ctrlV }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" @click="$emit('find')">
          <span>{{ messages.find }}</span>
          <span class="menu-shortcut">{{ messages.ctrlF }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('findNext')">
          <span>{{ messages.findNext }}</span>
          <span class="menu-shortcut">{{ messages.f3 }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('replace')">
          <span>{{ messages.replace }}</span>
          <span class="menu-shortcut">{{ messages.ctrlR }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('globalSearch')">
          <span>{{ messages.globalSearch }}</span>
          <span class="menu-shortcut">{{ messages.ctrlShiftF }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command menu-muted" @click="$emit('selectAll')">
          <span>{{ messages.selectAll }}</span>
          <span class="menu-shortcut">{{ messages.ctrlA }}</span>
        </button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.view }}</button>
      <div class="menu-popover menu-view-popover">
        <div class="menu-subroot">
          <button type="button" class="menu-command">
            <span>{{ messages.editorMode }}</span>
            <span class="menu-arrow">&rsaquo;</span>
          </button>
          <div class="menu-popover menu-subpopover menu-view-subpopover">
            <button type="button" :class="{ checked: previewMode === 'edit' }" @click="$emit('updatePreviewMode', 'edit')">
              {{ messages.editMode }}
            </button>
            <button type="button" :class="{ checked: previewMode === 'split' }" @click="$emit('updatePreviewMode', 'split')">
              {{ messages.splitMode }}
            </button>
            <button type="button" :class="{ checked: previewMode === 'preview' }" @click="$emit('updatePreviewMode', 'preview')">
              {{ messages.previewMode }}
            </button>
          </div>
        </div>
        <div class="menu-separator" role="separator" />
        <div class="menu-subroot">
          <button type="button" class="menu-command">
            <span>{{ messages.tabBarDisplay }}</span>
            <span class="menu-arrow">&rsaquo;</span>
          </button>
          <div class="menu-popover menu-subpopover menu-view-subpopover">
            <button
              type="button"
              :class="{ checked: tabBarOrientation === 'horizontal' }"
              @click="$emit('updateTabBarOrientation', 'horizontal')"
            >
              {{ messages.horizontal }}
            </button>
            <button
              type="button"
              :class="{ checked: tabBarOrientation === 'vertical' }"
              @click="$emit('updateTabBarOrientation', 'vertical')"
            >
              {{ messages.vertical }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.page }}</button>
      <div class="menu-popover">
        <button type="button" class="menu-command" @click="$emit('newNote')">
          <span>{{ messages.newPage }}</span>
          <span class="menu-shortcut">{{ messages.ctrlN }}</span>
        </button>
        <button type="button" class="menu-command" :disabled="!pageActionsEnabled" @click="$emit('renamePage')">
          <span>{{ messages.renamePage }}</span>
          <span class="menu-shortcut">{{ messages.f2 }}</span>
        </button>
        <button type="button" class="menu-command" :disabled="!pageActionsEnabled" @click="$emit('deletePage')">
          <span>{{ messages.deletePage }}</span>
          <span class="menu-shortcut">{{ messages.altDel }}</span>
        </button>
        <button type="button" class="menu-command" :disabled="!pageActionsEnabled" @click="toggleArchive">
          <span>{{ activeTabArchived ? messages.unarchivePage : messages.archivePage }}</span>
          <span class="menu-shortcut">{{ messages.f12 }}</span>
        </button>
        <button type="button" class="menu-command" :disabled="!pageActionsEnabled" @click="$emit('closePage')">
          <span>{{ messages.closePage }}</span>
          <span class="menu-shortcut">{{ messages.ctrlW }}</span>
        </button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.format }}</button>
      <div class="menu-popover">
        <button type="button" @click="$emit('formatFont')">{{ messages.font }}</button>
        <button type="button" @click="$emit('formatBackground')">{{ messages.backgroundColor }}</button>
        <button type="button" class="menu-command" @click="$emit('togglePreviewTheme')">
          <span>{{ messages.togglePreviewTheme }}</span>
          <span class="menu-shortcut">{{ messages.f7 }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('toggleTheme')">
          <span>{{ messages.toggleTheme }}</span>
          <span class="menu-shortcut">{{ messages.f9 }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" :class="{ checked: wordWrap }" @click="$emit('toggleWordWrap')">{{ messages.wordWrap }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.tools }}</button>
      <div class="menu-popover">
        <button type="button" class="menu-command" :class="{ checked: alwaysOnTop }" @click="$emit('togglePin')">
          <span>{{ messages.keepOnTop }}</span>
          <span class="menu-shortcut">{{ messages.f6 }}</span>
        </button>
        <button type="button" @click="$emit('windowOpacity')">{{ messages.windowOpacity }}</button>
        <div class="menu-separator" role="separator" />
        <div class="menu-subroot">
          <button type="button" class="menu-command">
            <span>{{ messages.textProcessing }}</span>
            <span class="menu-arrow">&rsaquo;</span>
          </button>
          <div class="menu-popover menu-subpopover menu-tall-popover">
            <button type="button" @click="$emit('processText', 'uppercase')">{{ messages.uppercase }}</button>
            <button type="button" @click="$emit('processText', 'lowercase')">{{ messages.lowercase }}</button>
            <div class="menu-separator" role="separator" />
            <button type="button" @click="$emit('processText', 'removeExtraSpaces')">{{ messages.removeExtraSpaces }}</button>
            <button type="button" @click="$emit('processText', 'trimLeadingSpaces')">{{ messages.trimLeadingSpaces }}</button>
            <button type="button" @click="$emit('processText', 'removeEmptyLines')">{{ messages.removeEmptyLines }}</button>
            <button type="button" @click="$emit('processText', 'removeDuplicateEmptyLines')">{{ messages.removeDuplicateEmptyLines }}</button>
            <div class="menu-separator" role="separator" />
            <button type="button" @click="$emit('processText', 'sortLines')">{{ messages.sortLines }}</button>
            <button type="button" @click="$emit('processText', 'uniqueLines')">{{ messages.uniqueLines }}</button>
            <div class="menu-separator" role="separator" />
            <button type="button" @click="$emit('processText', 'toSimplified')">{{ messages.toSimplifiedChinese }}</button>
            <button type="button" @click="$emit('processText', 'toTraditional')">{{ messages.toTraditionalChinese }}</button>
            <button type="button" @click="$emit('processText', 'toHalfWidth')">{{ messages.toHalfWidth }}</button>
            <button type="button" @click="$emit('processText', 'toFullWidth')">{{ messages.toFullWidth }}</button>
            <div class="menu-separator" role="separator" />
            <button type="button" @click="$emit('processText', 'addLineNumbers')">{{ messages.addLineNumbers }}</button>
            <button type="button" @click="$emit('processText', 'removeLineNumbers')">{{ messages.removeLineNumbers }}</button>
            <div class="menu-separator" role="separator" />
            <button type="button" @click="$emit('processText', 'urlEncode')">{{ messages.urlEncode }}</button>
            <button type="button" @click="$emit('processText', 'urlDecode')">{{ messages.urlDecode }}</button>
            <button type="button" @click="$emit('processText', 'base64Encode')">{{ messages.base64Encode }}</button>
            <button type="button" @click="$emit('processText', 'base64Decode')">{{ messages.base64Decode }}</button>
            <div class="menu-separator" role="separator" />
            <button type="button" @click="$emit('processText', 'md5Hash')">{{ messages.md5Hash }}</button>
            <button type="button" @click="$emit('processText', 'sha1Hash')">{{ messages.sha1Hash }}</button>
            <button type="button" @click="$emit('processText', 'sha256Hash')">{{ messages.sha256Hash }}</button>
          </div>
        </div>
        <button type="button" class="menu-command" @click="$emit('reminderList')">
          <span>{{ messages.reminderList }}</span>
          <span class="menu-shortcut">{{ messages.f5 }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" @click="$emit('settings')">
          <span>{{ messages.settingsWithKey }}</span>
          <span class="menu-shortcut">{{ messages.f8 }}</span>
        </button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.insert }}</button>
      <div class="menu-popover">
        <button type="button" class="menu-command" @click="$emit('insertSeparator')">
          <span>{{ messages.insertSeparator }}</span>
          <span class="menu-shortcut">{{ messages.ctrlDash }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('insertDateTime')">
          <span>{{ messages.dateTime }}</span>
          <span class="menu-shortcut">{{ messages.ctrlD }}</span>
        </button>
        <button type="button" class="menu-command" @click="$emit('insertDateTimeSeparator')">
          <span>{{ messages.dateTimeSeparator }}</span>
          <span class="menu-shortcut">{{ messages.ctrlShiftDash }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" class="menu-command" @click="$emit('insertReminder')">
          <span>{{ messages.reminder }}</span>
          <span class="menu-shortcut">{{ messages.ctrlE }}</span>
        </button>
        <div class="menu-separator" role="separator" />
        <button type="button" @click="$emit('insertTextSettings')">{{ messages.insertTextSettings }}</button>
      </div>
    </div>

    <div class="menu-root">
      <button type="button" class="menu-title">{{ messages.help }}</button>
      <div class="menu-popover">
        <button type="button" @click="$emit('helpTopic', 'software')">{{ messages.softwareHelp }}</button>
        <button type="button" class="menu-command" @click="$emit('helpTopic', 'shortcuts')">
          <span>{{ messages.shortcutList }}</span>
          <span class="menu-shortcut">{{ messages.f1 }}</span>
        </button>
        <button type="button" @click="$emit('helpTopic', 'expression')">{{ messages.expressionGuide }}</button>
        <button type="button" @click="$emit('helpTopic', 'markdown')">{{ messages.markdownGuide }}</button>
        <div class="menu-separator" role="separator" />
        <button type="button" @click="$emit('helpTopic', 'about')">{{ messages.about }}</button>
      </div>
    </div>
  </nav>
</template>
