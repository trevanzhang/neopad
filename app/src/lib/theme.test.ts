import { describe, expect, it } from 'vitest'
import { editorBackgroundForTheme, isLightColor } from './theme'

describe('theme colors', () => {
  it('classifies common editor backgrounds', () => {
    expect(isLightColor('#fafafa')).toBe(true)
    expect(isLightColor('#1e2228')).toBe(false)
  })

  it('rejects configured backgrounds that conflict with the active theme', () => {
    expect(editorBackgroundForTheme('dark', '#fafafa')).toBe('#1e2228')
    expect(editorBackgroundForTheme('light', '#20252c')).toBe('#ffffff')
    expect(editorBackgroundForTheme('dark', '#20252c')).toBe('#20252c')
  })
})
