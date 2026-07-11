import { afterEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { usePreferenceState } from './usePreferenceState'

describe('usePreferenceState', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('persists changes and routes native synchronization callbacks', async () => {
    const values = new Map<string, string>()
    vi.stubGlobal('window', {
      localStorage: {
        getItem: (key: string) => values.get(key) ?? null,
        setItem: (key: string, value: string) => values.set(key, value),
      },
      matchMedia: () => ({ matches: false }),
    })
    const onWindowOpacityChanged = vi.fn()
    const onPersistRequested = vi.fn()
    const state = usePreferenceState({
      onLanguageChanged: vi.fn(),
      onWindowOpacityChanged,
      onAutostartChanged: vi.fn(),
      onStartHiddenChanged: vi.fn(),
      onCloseToMinimizeChanged: vi.fn(),
      onSnapToEdgesChanged: vi.fn(),
      onToggleShortcutChanged: vi.fn(),
      onClipboardShortcutChanged: vi.fn(),
      onPersistRequested,
    })

    state.windowOpacity.value = 0.7
    state.shortcutModifiers.value = ['Ctrl', 'Alt']
    await nextTick()

    expect(values.get('neopad.windowOpacity')).toBe('0.7')
    expect(values.get('neopad.shortcutModifiers')).toBe('["Ctrl","Alt"]')
    expect(onWindowOpacityChanged).toHaveBeenCalledOnce()
    expect(onPersistRequested).toHaveBeenCalled()
  })
})
