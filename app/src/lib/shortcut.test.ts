import { describe, expect, it } from 'vitest'
import { formatShortcutLabel, normalizeShortcutInput, normalizeStoredShortcutKey } from './shortcut'

describe('shortcut key normalization', () => {
  it('repairs legacy multi-character letter keys', () => {
    expect(normalizeStoredShortcutKey('Zx', 'Z')).toBe('Z')
    expect(formatShortcutLabel('Zx', ['Alt'])).toBe('Alt+Z')
  })

  it('preserves valid function keys while editing', () => {
    expect(normalizeShortcutInput('f')).toBe('F')
    expect(normalizeShortcutInput('f10')).toBe('F10')
    expect(normalizeStoredShortcutKey('F12', 'Z')).toBe('F12')
  })
})
