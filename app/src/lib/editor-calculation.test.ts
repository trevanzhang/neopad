import { describe, expect, it } from 'vitest'
import { evaluateExpressionLine, formatCalculationResult } from './editor-calculation'

describe('editor calculation', () => {
  it.each([
    ['1 + 2 * 3', 7],
    ['2 ^ 3 ^ 2', 512],
    ['预算：(12.5 + 7.5) × 3', 60],
    ['10 ÷ 4', 2.5],
    ['(-3 + 5) * 2', 4],
    ['1 + (2 * 3', 7],
  ])('evaluates %s', (source, result) => {
    expect(evaluateExpressionLine(source)).toBe(result)
  })

  it.each(['', '没有算式', '1 / 0', '2 ** 3'])('rejects invalid input %s', (source) => {
    expect(evaluateExpressionLine(source)).toBeNull()
  })

  it('keeps the existing forgiving trailing-operator behavior', () => {
    expect(evaluateExpressionLine('1 +')).toBe(1)
  })

  it('formats stable compact results', () => {
    expect(formatCalculationResult(2)).toBe('2')
    expect(formatCalculationResult(1 / 3)).toBe('0.333333333333')
    expect(formatCalculationResult(-1e-14)).toBe('0')
  })
})
