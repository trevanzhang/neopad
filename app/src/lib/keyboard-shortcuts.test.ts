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
    ...modifiers,
  } as unknown as KeyboardEvent
}

describe('keyboard shortcut routing', () => {
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

  it('cycles editor mode with F8', () => {
    const { spies, handler } = harness()
    handler(keyEvent('F8'))
    expect(spies.cycleEditorMode).toHaveBeenCalledOnce()
  })

  it('opens settings with Ctrl+Comma', () => {
    const { spies, handler } = harness()
    handler(keyEvent(',', { ctrlKey: true }))
    expect(spies.openSettings).toHaveBeenCalledOnce()
  })

  it('honors Escape surface precedence', () => {
    const { flags, spies, handler } = harness()
    flags.set('confirmationOpen', true)
    flags.set('inputOpen', true)
    handler(keyEvent('Escape'))
    expect(spies.cancelConfirmation).toHaveBeenCalledOnce()
    expect(spies.cancelInput).not.toHaveBeenCalled()
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
