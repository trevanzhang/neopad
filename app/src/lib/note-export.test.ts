import { describe, expect, it } from 'vitest'
import { computePageSlices, getNoteExportLayoutMetrics, parseCssColor } from './note-export'

describe('note export layouts', () => {
  it('uses a 1080px-class high-density canvas for mobile sharing', () => {
    const standard = getNoteExportLayoutMetrics('standard')
    const mobile = getNoteExportLayoutMetrics('mobile')

    expect(mobile.widthPx).toBe(540)
    expect(mobile.widthPx).toBeLessThan(standard.widthPx)
    expect(mobile.fontSizePx).toBeGreaterThan(standard.fontSizePx)
    expect(mobile.lineHeight).toBeGreaterThan(standard.lineHeight)
  })
})

describe('parseCssColor', () => {
  it('converts computed RGB and theme hex colors for PDF page backgrounds', () => {
    expect(parseCssColor('rgb(40, 44, 52)')).toEqual([40, 44, 52])
    expect(parseCssColor('#fdf6e3')).toEqual([253, 246, 227])
  })

  it('falls back to white for an unsupported color value', () => {
    expect(parseCssColor('transparent')).toEqual([255, 255, 255])
  })
})

describe('computePageSlices', () => {
  it('uses nearby block boundaries to avoid splitting normal content', () => {
    expect(computePageSlices(2500, 1000, [620, 940, 1510, 1980])).toEqual([
      { start: 0, end: 940 },
      { start: 940, end: 1510 },
      { start: 1510, end: 2500 },
    ])
  })

  it('falls back to the ideal height for a block taller than a page', () => {
    expect(computePageSlices(2200, 1000, [])).toEqual([
      { start: 0, end: 1000 },
      { start: 1000, end: 2000 },
      { start: 2000, end: 2200 },
    ])
  })
})
