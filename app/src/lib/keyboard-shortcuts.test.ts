import { describe, expect, it, vi, type Mock } from 'vitest'
import {
  createKeyboardHandler,
  type KeyboardActions,
  type KeyboardState,
} from './keyboard-shortcuts'

function harness() {
  const flags = new Map<string, boolean>()
  const state = new Proxy({}, {
    get: (_target, property: string) => property === 'editableTarget'
      ? () => false
      : () => flags.get(property) ?? false,
  }) as KeyboardState
  const spies: Record<string, Mock> = {}
  const actions = new Proxy({}, {
    get: (_target, property: string) => spies[property] ??= vi.fn(),
  }) as KeyboardActions
  return { flags, spies, handler: createKeyboardHandler({ state, actions }) }
}

function keyEvent(key: string, modifiers: Partial<KeyboardEvent> = {}) {
  return {
    key,
    code: key === '-' ? 'Minus' : key === ',' ? 'Comma' : key,
    ctrlKey: false,
    altKey: false,
    shiftKey: false,
    metaKey: false,
    target: null,
    preventDefault: vi.fn(),
    stopPropagation: vi.fn(),
    stopImmediatePropagation: vi.fn(),
    ...modifiers,
  } as unknown as KeyboardEvent
}

describe('keyboard shortcut routing', () => {
  it('ignores application shortcuts during IME composition', () => {
    const { spies, handler } = harness()
    const event = keyEvent('F2', { isComposing: true })
    handler(event)
    expect(spies.cycleTab).toBeUndefined()
    expect(event.preventDefault).not.toHaveBeenCalled()
  })

  it('blocks navigation shortcuts while a modal is open', () => {
    const { flags, spies, handler } = harness()
    flags.set('modalOpen', true)
    const event = keyEvent('n', { ctrlKey: true })
    handler(event)
    expect(event.preventDefault).toHaveBeenCalledOnce()
    expect(event.stopPropagation).toHaveBeenCalledOnce()
    expect(spies.createLocalTab).toBeUndefined()
  })

  it('routes Ctrl+Shift+Tab to the previous tab', () => {
    const { spies, handler } = harness()
    handler(keyEvent('Tab', { ctrlKey: true, shiftKey: true }))
    expect(spies.cycleTab).toHaveBeenCalledWith(-1)
  })

  it('opens the note browser with F4', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F4'))
    expect(spies.toggleNoteLibrary).toHaveBeenCalledOnce()
  })

  it('switches to the previous tab with F2', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F2'))
    expect(spies.cycleTab).toHaveBeenCalledWith(-1)
  })

  it('switches to the next tab with F3', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F3'))
    expect(spies.cycleTab).toHaveBeenCalledWith(1)
  })

  it('cycles editor mode with F5', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F5'))
    expect(spies.cycleEditorMode).toHaveBeenCalledOnce()
  })

  it('toggles the preview theme with F10', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F10'))
    expect(spies.togglePreviewTheme).toHaveBeenCalledOnce()
  })

  it('opens the reminder list with F6', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F6'))
    expect(spies.openReminderList).toHaveBeenCalledOnce()
  })

  it('toggles window on top with F7', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F7'))
    expect(spies.togglePin).toHaveBeenCalledOnce()
  })

  it('renames the current page with F8', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F8'))
    expect(spies.renameActivePage).toHaveBeenCalledOnce()
  })

  it('finds the next match with Ctrl+G', () => {
    const { spies, handler } = harness()
    handler(keyEvent('g', { ctrlKey: true }))
    expect(spies.findNext).toHaveBeenCalledOnce()
  })

  it('opens settings with Ctrl+Comma', () => {
    const { spies, handler } = harness()
    handler(keyEvent(',', { ctrlKey: true }))
    expect(spies.openSettings).toHaveBeenCalledOnce()
  })

  it('opens AI chat with Ctrl+K while the editor is focused', () => {
    const { flags, spies, handler } = harness()
    flags.set('editorFocused', true)
    handler(keyEvent('k', { ctrlKey: true }))
    expect(spies.openAiPanel).toHaveBeenCalledOnce()
  })

  it('honors Escape surface precedence', () => {
    const { flags, spies, handler } = harness()
    flags.set('confirmationOpen', true)
    flags.set('inputOpen', true)
    handler(keyEvent('Escape'))
    expect(spies.cancelConfirmation).toHaveBeenCalledOnce()
    expect(spies.cancelInput).not.toHaveBeenCalled()
  })

  it('lets the AI prompt picker handle Escape before the panel', () => {
    const { flags, spies, handler } = harness()
    flags.set('aiPanelOpen', true)
    handler(keyEvent('Escape', {
      target: { closest: (selector: string) => selector === '.ai-prompt-picker' ? {} : null } as unknown as EventTarget,
    }))
    expect(spies.closeAiPanel).toBeUndefined()
  })

  it('closes settings before requesting native window hiding', () => {
    const { flags, spies, handler } = harness()
    flags.set('settingsOpen', true)
    flags.set('nativeRuntime', true)
    handler(keyEvent('Escape'))
    expect(spies.closeSettings).toHaveBeenCalledOnce()
    expect(spies.hideMainWindow).toBeUndefined()
  })

  it.each([
    ['reminder dialog', 'reminderDialogOpen', 'closeReminderDialog'],
    ['reminder list', 'reminderListOpen', 'closeReminderList'],
    ['archive list', 'archiveListOpen', 'closeArchiveList'],
    ['confirmation dialog', 'confirmationOpen', 'cancelConfirmation'],
    ['input dialog', 'inputOpen', 'cancelInput'],
    ['font dialog', 'fontDialogOpen', 'closeFontDialog'],
    ['AI inline command', 'aiInlineOpen', 'cancelAiInlineCommand'],
    ['AI panel', 'aiPanelOpen', 'closeAiPanel'],
    ['settings', 'settingsOpen', 'closeSettings'],
    ['help', 'helpOpen', 'closeHelp'],
    ['search', 'searchOpen', 'closeSearch'],
    ['editor find panel', 'findPanelOpen', 'closeEditorFind'],
  ])('closes %s before requesting native window hiding', (_label, stateKey, actionKey) => {
    const { flags, spies, handler } = harness()
    flags.set(stateKey, true)
    flags.set('nativeRuntime', true)
    handler(keyEvent('Escape'))
    expect(spies[actionKey]).toHaveBeenCalledOnce()
    expect(spies.hideMainWindow).toBeUndefined()
  })

  it('stops other window listeners after Escape closes help', () => {
    const { flags, spies, handler } = harness()
    flags.set('helpOpen', true)
    flags.set('nativeRuntime', true)
    const event = keyEvent('Escape')
    handler(event)
    expect(spies.closeHelp).toHaveBeenCalledOnce()
    expect(event.stopImmediatePropagation).toHaveBeenCalledOnce()
    expect(spies.hideMainWindow).toBeUndefined()
  })

  it('does not archive through an open settings surface', () => {
    const { flags, spies, handler } = harness()
    flags.set('settingsOpen', true)
    handler(keyEvent('F12'))
    expect(spies.archiveActivePage).toBeUndefined()
  })

  it('lets Escape fall through to native window hiding when no surface owns it', () => {
    const { flags, spies, handler } = harness()
    flags.set('nativeRuntime', true)
    handler(keyEvent('Escape'))
    expect(spies.hideMainWindow).toHaveBeenCalledOnce()
  })
})
