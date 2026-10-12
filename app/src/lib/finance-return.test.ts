import { describe, expect, it, vi } from 'vitest'
import { buildFinanceRequest, cashFlowDraftRows, financeErrorText, financeFields, financeRecord, financeResultText, initialFinanceDrafts } from './finance-ui'
import { useFinancialTools } from '../composables/useFinancialTools'
import type { FinanceResponse } from '../types/finance'

describe('holding return and dated cash flows', () => {
  it('routes full quotes without unused accrued interest and converts clean quotes once', () => {
    const draft = initialFinanceDrafts().holdingReturn
    draft.holdingBuyAccrued = 'unused'
    expect(buildFinanceRequest('holdingReturn', draft)).toMatchObject({ face: 100000, buyPrice: 100, buyClean: false, buyAccrued: 0, coupons: 3000, buyFee: 100 })
    expect(financeFields('holdingReturn', draft, 'zh').some(field => field.key === 'holdingBuyAccrued')).toBe(false)
    draft.holdingBuyType = 'clean'; draft.holdingBuyAccrued = '.5'
    draft.holdingSellType = 'clean'; draft.holdingSellAccrued = '1'
    expect(buildFinanceRequest('holdingReturn', draft)).toMatchObject({ buyClean: true, buyAccrued: .5, sellAccrued: 1 })
    expect(financeFields('holdingReturn', draft, 'en').some(field => field.key === 'holdingBuyAccrued')).toBe(true)
  })
  it('retains actual dates, explicit directions, zero entries and input order for core validation', () => {
    const draft = initialFinanceDrafts().xirr
    expect(buildFinanceRequest('xirr', draft)).toEqual({ kind: 'xirr', flows: [{ date: '2026-01-01', amount: -10000 }, { date: '2027-01-01', amount: 11000 }] })
    draft.cashFlows = JSON.stringify([{ date: '2027-01-01', amount: '10000', direction: 'out' }, { date: '2026-01-01', amount: '9000', direction: 'in' }, { date: '2026-06-01', amount: '0', direction: 'in' }])
    expect(buildFinanceRequest('xirr', draft)).toMatchObject({ flows: [{ date: '2027-01-01', amount: -10000 }, { date: '2026-01-01', amount: 9000 }, { date: '2026-06-01', amount: 0 }] })
  })
  it('identifies bad cash-flow amounts by row before invoking native calculations', () => {
    const draft = initialFinanceDrafts().xirr
    for (const amount of ['', '1,000', '1e3', '-100', 'Infinity', '100元']) {
      draft.cashFlows = JSON.stringify([{ date: '2026-01-01', amount: '10000', direction: 'out' }, { date: '2027-01-01', amount, direction: 'in' }])
      try { buildFinanceRequest('xirr', draft); expect.fail('should reject') }
      catch (error) { expect(error).toMatchObject({ row: 2, field: 'cashFlowAmount' }) }
    }
    draft.cashFlows = 'bad JSON'
    expect(cashFlowDraftRows(draft)).toEqual([])
    expect(() => buildFinanceRequest('xirr', draft)).toThrow()
  })
  it('exports complete cash flows and conventions without bond units or a raw JSON blob', () => {
    const response: FinanceResponse = { engineVersion: 'finance-4', formula: 'NPV=0', cashFlows: [], datedCashFlows: [{ date: '2026-01-01', amount: -10000 }, { date: '2027-01-01', amount: 11000 }], values: [
      { key: 'xirrRate', value: .123456789, unit: 'percent' }, { key: 'xirrResidual', value: 1e-13, unit: 'ratio' }, { key: 'cashFlowEntries', value: 2, unit: 'entries' }] }
    const record = financeRecord('xirr', initialFinanceDrafts().xirr, response, 'zh')
    for (const expected of ['| 2026-01-01 | 支出 − | 10000 |', '| 2027-01-01 | 收入 + | 11000 |', '12.3456789', '同日合并', '一次变号', 'finance-4', '录入笔数: 2 笔', '| 2026-01-01 | -10000 |']) expect(record).toContain(expected)
    expect(record).not.toContain('/ 100'); expect(record).not.toContain('"direction"')
    expect(financeResultText(response, 'en')).not.toContain('residual')
  })
  it('exports all holding costs and clearly distinguishes HPR from simple annualization', () => {
    const response: FinanceResponse = { engineVersion: 'finance-4', formula: 'HPR', cashFlows: [], values: [{ key: 'holdingReturn', value: .047952047952, unit: 'percent' }] }
    const record = financeRecord('holdingReturn', initialFinanceDrafts().holdingReturn, response, 'zh')
    expect(record).toContain('持有期收益率: 4.7952047952')
    expect(record).toContain('融资成本总额: 0')
    expect(record).toContain('单利年化不是 XIRR')
    expect(record).not.toContain('买入应计利息:')
  })
  it('preserves core row errors and invalidates results on any cash-flow edit', async () => {
    const compute = vi.fn().mockRejectedValueOnce({ code: 'date', field: 'cashFlowDate', row: 2 }).mockResolvedValue({ engineVersion: 'finance-4', formula: '', cashFlows: [], values: [] })
    const tools = useFinancialTools(compute); tools.category.value = 'xirr'
    await tools.compute()
    expect(tools.error.value?.row).toBe(2)
    expect(financeErrorText(tools.error.value!, 'zh')).toContain('第 2 笔')
    await tools.compute()
    expect(tools.record('zh')).not.toBeNull()
    const rows = cashFlowDraftRows(tools.drafts.xirr); rows[1]!.amount = '12000'
    tools.drafts.xirr.cashFlows = JSON.stringify(rows)
    expect(tools.result.value).toBeNull(); expect(tools.record('en')).toBeNull()
  })
})
