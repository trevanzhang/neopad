import { describe, expect, it, vi } from 'vitest'
import { buildFinanceRequest, financeFields, financeRecord, financeResultText, initialFinanceDrafts, repoProducts } from './finance-ui'
import { useFinancialTools } from '../composables/useFinancialTools'
import type { FinanceResponse } from '../types/finance'

describe('loan and repo requests and records', () => {
  it('converts loan rates once and omits the unknown amount from validation and records', () => {
    const draft = initialFinanceDrafts().loan
    draft.loanPayment = 'unused'
    expect(buildFinanceRequest('loan', draft)).toMatchObject({ kind: 'loan', rate: .035, payment: 0, months: 360, frequency: 12 })
    draft.loanTarget = 'principal'
    draft.loanPrincipal = 'unused'
    draft.loanPayment = '1120'
    expect(buildFinanceRequest('loan', draft)).toMatchObject({ principal: 0, payment: 1120 })
    expect(financeFields('loan', draft, 'zh').some(field => field.key === 'loanPrincipal')).toBe(false)
    draft.loanTarget = 'rate'
    draft.loanPrincipal = '12000'
    draft.loanRate = 'unused'
    draft.loanMethod = 'equalPrincipal'
    expect(buildFinanceRequest('loan', draft)).toMatchObject({ rate: 0, method: 'equalPrincipal' })
    expect(financeFields('loan', draft, 'zh').find(field => field.key === 'loanPayment')?.label).toBe('首期还款额')
  })

  it('requires actual days and fixes exchange year basis without assuming nominal tenor', () => {
    const draft = initialFinanceDrafts().repo
    expect(() => buildFinanceRequest('repo', draft)).toThrow()
    draft.repoDays = '3'
    draft.repoFeeRate = '.001'
    draft.repoBasis = '360'
    expect(buildFinanceRequest('repo', draft)).toMatchObject({ days: 3, basis: 365, feeRate: .00001, side: 'lending' })
    draft.repoProduct = '131810'
    expect(buildFinanceRequest('repo', draft)).toMatchObject({ days: 3, basis: 365 })
    draft.repoProduct = 'custom'
    draft.repoSide = 'financing'
    draft.repoFeeTiming = 'true'
    expect(buildFinanceRequest('repo', draft)).toMatchObject({ basis: 360, side: 'financing', feeAtMaturity: true })
    draft.repoProduct = 'invalid'
    expect(() => buildFinanceRequest('repo', draft)).toThrow()
    expect(repoProducts).toHaveLength(18)
    expect(new Set(repoProducts.map(product => product.code)).size).toBe(18)
  })

  it('routes actual settlement dates without validating an unused days field', () => {
    const draft = initialFinanceDrafts().repo
    draft.repoDayMode = 'settlementDates'
    draft.repoStart = '2026-10-16'
    draft.repoEnd = '2026-10-19'
    expect(buildFinanceRequest('repo', draft)).toMatchObject({ days: 0, start: '2026-10-16', end: '2026-10-19' })
    expect(financeFields('repo', draft, 'en').some(field => field.key === 'repoDays')).toBe(false)
  })

  it('exports every repayment at full precision in currency units', () => {
    const response: FinanceResponse = { engineVersion: 'finance-3', formula: 'fixed rate', cashFlows: [],
      values: [{ key: 'firstPayment', value: 1120.123456789, unit: 'amount' }, { key: 'loanResidual', value: 1e-12, unit: 'amount' }],
      repayments: Array.from({ length: 120 }, (_, index) => ({ period: index + 1, payment: 1120.123456789,
        principal: 1000, interest: 120.123456789, balance: (119 - index) * 1000 })) }
    const record = financeRecord('loan', initialFinanceDrafts().loan, response, 'zh')
    expect(record).toContain('| 120 | 1120.123456789 | 1000 | 120.123456789 | 0 |')
    expect(record).not.toContain('/ 100')
    expect(record).toContain('固定利率、期末还款')
    expect(record).toContain('finance-3')
    expect(financeResultText(response, 'en')).toContain('1,120.12')
    expect(financeResultText(response, 'en')).not.toContain('residual')
  })

  it('retains repo direction, fees, year basis and negative net income in the record', () => {
    const draft = initialFinanceDrafts().repo
    draft.repoDays = '3'
    draft.repoFeeRate = '1'
    const response: FinanceResponse = { engineVersion: 'finance-3', formula: 'actual days', cashFlows: [],
      values: [{ key: 'repoNetIncome', value: -983.5616438356164, unit: 'amount' }, { key: 'repoDays', value: 3, unit: 'days' }] }
    const record = financeRecord('repo', draft, response, 'zh')
    for (const text of ['逆回购 · 融出资金', '204001 · 沪 1天', '实际占款天数: 3', '一次性手续费率: 1 %',
      '年计息天数 365', String(response.values[0]!.value), '不是 IRR', '起始扣取']) expect(record).toContain(text)
  })

  it('keeps loan and repo drafts separate and invalidates results when repo direction changes', async () => {
    const calculate = vi.fn().mockResolvedValue({ engineVersion: 'finance-3', formula: '', cashFlows: [], values: [] })
    const tools = useFinancialTools(calculate)
    tools.category.value = 'repo'
    tools.drafts.repo.repoDays = '3'
    await tools.compute()
    expect(tools.result.value).not.toBeNull()
    tools.drafts.repo.repoSide = 'financing'
    expect(tools.result.value).toBeNull()
    tools.category.value = 'loan'
    tools.drafts.loan.loanPrincipal = '12000'
    tools.reset()
    expect(tools.drafts.repo.repoDays).toBe('3')
    expect(tools.drafts.loan.loanPrincipal).toBe('1000000')
  })
})
