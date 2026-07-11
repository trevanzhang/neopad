import { describe, expect, it } from 'vitest'
import {
  initialBooleanSetting,
  initialJsonSetting,
  initialNumberSetting,
  initialStringSetting,
  normalizePreviewTheme,
} from './preferences'

describe('preferences', () => {
  it('normalizes legacy preview theme names', () => {
    expect(normalizePreviewTheme('github')).toBe('githubLight')
    expect(normalizePreviewTheme('solomd')).toBe('light')
    expect(normalizePreviewTheme('unknown')).toBe('light')
  })

  it('uses safe defaults outside a browser runtime', () => {
    expect(initialBooleanSetting('missing', true)).toBe(true)
    expect(initialStringSetting('missing', 'fallback')).toBe('fallback')
    expect(initialNumberSetting('missing', 14, 12, 22)).toBe(14)
    expect(initialJsonSetting('missing', ['fallback'])).toEqual(['fallback'])
  })
})
