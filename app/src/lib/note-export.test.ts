import { describe, expect, it } from 'vitest'
import { computePageSlices } from './note-export'

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
