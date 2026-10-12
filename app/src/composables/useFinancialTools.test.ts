import { describe, expect, it, vi } from 'vitest'
import { useFinancialTools } from './useFinancialTools'
import { buildFinanceRequest, financeRecord, initialFinanceDrafts } from '../lib/finance-ui'
import type { FinanceResponse } from '../types/finance'

const response: FinanceResponse = { engineVersion: 'finance-test', formula: 'P×r×t', cashFlows: [], values: [{ key: 'interest', value: 3000, unit: 'amount' }] }
function deferred() {
  let resolve!: (value: FinanceResponse) => void
  let reject!: (value: unknown) => void
  const promise = new Promise<FinanceResponse>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}

describe('financial tool state and input conversion', () => {
  it('invalidates results synchronously and prevents copying after any input change', async () => {
    const tools = useFinancialTools(vi.fn().mockResolvedValue(response))
    tools.category.value = 'interest'
    await tools.compute()
    expect(tools.record('zh')).toContain('100000')
    tools.drafts.interest.principal = '200000'
    expect(tools.result.value).toBeNull()
    expect(tools.record('zh')).toBeNull()
    expect(tools.changed.value).toBe(true)
  })

  it('ignores stale successes and failures without clearing a newer pending calculation', async () => {
    const old = deferred(), current = deferred()
    const calculate = vi.fn().mockReturnValueOnce(old.promise).mockReturnValueOnce(current.promise)
    const tools = useFinancialTools(calculate)
    tools.category.value = 'interest'
    const first = tools.compute()
    tools.drafts.interest.rate = '4'
    const second = tools.compute()
    old.reject({ code: 'range', field: 'principal' })
    await first
    expect(tools.busy.value).toBe(true)
    expect(tools.error.value).toBeNull()
    current.resolve(response)
    await second
    expect(tools.result.value).toEqual(response)
    expect(tools.record('en')).toContain('Annual rate: 4 %')
    const later = deferred()
    const other = useFinancialTools(() => later.promise)
    other.category.value = 'interest'
    const waiting = other.compute()
    other.category.value = 'bond'
    later.resolve(response)
    await waiting
    expect(other.result.value).toBeNull()
  })

  it('keeps separate drafts and resets only the active category', () => {
    const tools = useFinancialTools(vi.fn())
    tools.drafts.interest.principal = '200'
    tools.category.value = 'bond'
    tools.drafts.bond.coupon = '9'
    tools.reset()
    expect(tools.drafts.bond.coupon).toBe('3')
    expect(tools.drafts.interest.principal).toBe('200')
  })

  it('rejects incomplete or unsafe numbers before calling the engine', async () => {
    const calculate = vi.fn()
    const tools = useFinancialTools(calculate)
    tools.category.value = 'interest'
    for (const input of ['', '1,000', '1+2', 'Infinity', 'NaN', '100元', '1e3']) {
      tools.drafts.interest.principal = input
      await tools.compute()
      expect(tools.error.value?.field).toBe('principal')
    }
    expect(calculate).not.toHaveBeenCalled()
  })

  it('converts percentages and directions once, with separate payment and compounding frequencies', () => {
    const drafts = initialFinanceDrafts()
    expect(buildFinanceRequest('interest', drafts.interest)).toMatchObject({ rate: .03, principal: 100000 })
    drafts.tvm.compounding = '2'
    expect(buildFinanceRequest('tvm', drafts.tvm)).toMatchObject({ rate: .06, compounding: 2, payments: 12, payment: -1000, futureValue: 0 })
    drafts.tvm.paymentDirection = 'in'
    expect(buildFinanceRequest('tvm', drafts.tvm)).toMatchObject({ payment: 1000 })
    drafts.bond.target = 'yield'
    drafts.bond.basis = 'us30_360'
    expect(buildFinanceRequest('bond', drafts.bond)).toMatchObject({ target: 'yield', basis: 'us30_360', price: 100, yieldRate: 0 })
  })

  it('routes single payments and metrics without sending irrelevant coupon dates', () => {
    const draft = initialFinanceDrafts().bond
    expect(buildFinanceRequest('bond', draft)).not.toHaveProperty('previousCoupon')
    draft.frequency = '0'
    expect(buildFinanceRequest('bond', draft)).toMatchObject({ kind: 'singleBond', profile: 'maturityInterest' })
    draft.profile = 'discount'
    draft.coupon = 'invalid unused input'
    draft.target = 'convexity'
    expect(buildFinanceRequest('bond', draft)).toMatchObject({ kind: 'singleBond', profile: 'discount', target: 'price', coupon: 0, yieldRate: .035 })
  })

  it('retains complete precision and conventions in the Markdown record', () => {
    const result = { ...response, schedule: { previousCoupon: '2026-10-01', nextCoupon: '2027-04-01', remainingPayments: 10 }, values: [{ key: 'yieldRate', value: .032123456789, unit: 'percent' }], cashFlows: [{ date: '2027-01-01', amount: 3, years: .5, presentValue: 2.950000001 }] }
    const record = financeRecord('bond', initialFinanceDrafts().bond, result, 'zh')
    expect(record).toContain('3.2123456789')
    expect(record).toContain('2.950000001')
    expect(record).toContain('尚未开放')
    expect(record).toContain('finance-test')
    expect(record).toContain('下一付息日: 2027-04-01')
    expect(record).toContain('剩余支付次数: 10')
  })
})
