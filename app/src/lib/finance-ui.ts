import type { CashFlowDraftRow, FinanceCategory, FinanceDraft, FinanceError, FinanceRequest, FinanceResponse, FinanceValue } from '../types/finance'

export const financeCategories: FinanceCategory[] = ['interest', 'rate', 'tvm', 'bond', 'loan', 'repo', 'holdingReturn', 'xirr']
type Language = 'zh' | 'en'
export interface FinanceField {
  key: string
  label: string
  type?: 'date'
  unit?: string
  options?: Array<{ value: string; label: string }>
}

const labels: Record<string, [string, string]> = {
  holdingReturn: ['债券持有期收益', 'Bond holding return'], xirr: ['真实年化收益 / 成本', 'Cash-flow annual return / cost'],
  holdingBuyDate: ['买入交收日', 'Purchase settlement'], holdingSellDate: ['卖出交收日', 'Sale settlement'], holdingFace: ['持有面值', 'Position face value'],
  holdingBuyType: ['买入价格类型', 'Purchase price type'], holdingSellType: ['卖出价格类型', 'Sale price type'], holdingBuyPrice: ['买入价格', 'Purchase price'], holdingSellPrice: ['卖出价格', 'Sale price'],
  holdingBuyAccrued: ['买入应计利息', 'Purchase accrued interest'], holdingSellAccrued: ['卖出应计利息', 'Sale accrued interest'],
  holdingCoupons: ['实收票息总额', 'Coupons actually received'], holdingBuyFee: ['买入费用总额', 'Purchase fees'], holdingSellFee: ['卖出费用总额', 'Sale fees'], holdingFinancingCost: ['融资成本总额', 'Financing cost'],
  holdingBuyFull: ['买入全价 / 百元面值', 'Purchase full price / 100 face'], holdingSellFull: ['卖出全价 / 百元面值', 'Sale full price / 100 face'],
  holdingBuyAmount: ['买入全价金额', 'Purchase full amount'], holdingSellAmount: ['卖出全价金额', 'Sale full amount'], holdingInitial: ['买入投入（含买入费用）', 'Initial investment · including purchase fees'],
  holdingPriceGain: ['全价价差损益', 'Full-price gain / loss'], holdingFees: ['买卖费用总额', 'Total purchase / sale fees'], holdingProfit: ['持有期净收益', 'Holding net profit'], holdingReturnResult: ['持有期收益率', 'Holding period return'], holdingAnnualSimple: ['单利折算年率（365天）', 'Simple annualization · 365 days'], holdingDays: ['持有天数', 'Holding days'],
  cashFlows: ['现金流', 'Cash flows'], cashFlowDate: ['现金流日期', 'Cash-flow date'], cashFlowAmount: ['现金流金额', 'Cash-flow amount'],
  xirrRate: ['有效年化收益 / 成本（XIRR）', 'Effective annual return / cost · XIRR'], cashInTotal: ['收入总额', 'Total inflows'], cashOutTotal: ['支出总额', 'Total outflows'], cashNetTotal: ['收支净额', 'Net cash flows'], cashFlowDays: ['现金流跨度', 'Cash-flow span'], cashFlowEntries: ['录入笔数', 'Input entries'], xirrResidual: ['相对净现值残差', 'Relative NPV residual'],
  loan: ['个人贷款', 'Personal loan'], repo: ['债券回购', 'Bond repo'],
  loanTarget: ['计算项目', 'Solve for'], loanPaymentTarget: ['求每期还款额', 'Periodic payment'], loanFirstTarget: ['求首期还款额', 'First payment'], loanPrincipalTarget: ['求贷款总额', 'Loan amount'], loanRateTarget: ['求年利率', 'Annual rate'],
  loanPrincipal: ['贷款总额', 'Loan amount'], loanMonths: ['贷款期限', 'Loan term'], loanRate: ['名义年利率', 'Nominal annual rate'], loanPayment: ['每期还款额', 'Periodic payment'], loanFirstPayment: ['首期还款额', 'First payment'], loanFrequency: ['偿还频率', 'Repayment frequency'], loanMethod: ['偿还方式', 'Repayment method'],
  annuity: ['等额本息', 'Equal payments'], equalPrincipal: ['等额本金', 'Equal principal'], monthly: ['每月', 'Monthly'], quarterly: ['每季', 'Quarterly'], semiannual: ['每半年', 'Semiannual'], annually: ['每年', 'Annual'],
  firstPayment: ['首期还款额', 'First payment'], lastPayment: ['末期还款额', 'Last payment'], paymentDecrease: ['每期递减额', 'Payment decrease'], totalInterest: ['利息总额', 'Total interest'], totalRepayment: ['还款总额', 'Total repayment'], repaymentCount: ['还款期数', 'Repayment count'], loanPeriodicRate: ['每期利率', 'Periodic rate'], loanResidual: ['还款额回代误差', 'Payment residual'],
  repoSide: ['回购方向', 'Repo direction'], financing: ['正回购 · 融入资金', 'Repo · borrow cash'], lending: ['逆回购 · 融出资金', 'Reverse repo · lend cash'], repoProduct: ['品种代码', 'Product code'], custom: ['自定义回购', 'Custom repo'],
  repoAmount: ['成交金额', 'Cash amount'], repoRate: ['回购年利率', 'Annual repo rate'], repoFeeRate: ['一次性手续费率', 'One-off fee rate'], repoDays: ['实际占款天数', 'Actual funding days'], repoDayMode: ['计息天数来源', 'Day input'], manualDays: ['手填占款天数', 'Enter actual days'], settlementDates: ['按实际交收日期', 'Settlement dates'], repoStart: ['首次交收日（计入）', 'Initial settlement · inclusive'], repoEnd: ['到期交收日（不计入）', 'Maturity settlement · exclusive'], repoBasis: ['年计息天数', 'Days per year'], repoFeeTiming: ['手续费扣取时点', 'Fee timing'], initialFee: ['起始扣取', 'At start'], maturityFee: ['到期扣取', 'At maturity'],
  repoInterest: ['回购利息', 'Repo interest'], repoFee: ['手续费', 'Fee'], repoNetIncome: ['逆回购净收益', 'Reverse repo net income'], repoFinancingCost: ['正回购融资成本', 'Repo financing cost'], repoNetAnnualRate: ['净收益折算年率（成交金额基准）', 'Net annual rate · cash amount basis'], repoCostAnnualRate: ['成本折算年率（成交金额基准）', 'Cost annual rate · cash amount basis'], repoInitialFlow: ['起始资金流（收入 + / 支出 −）', 'Initial cash flow · in + / out −'], repoMaturityFlow: ['到期资金流（收入 + / 支出 −）', 'Maturity cash flow · in + / out −'],
  interest: ['利息', 'Interest'], rate: ['利率换算', 'Rates'], tvm: ['现值 / 年金', 'Time value'], bond: ['债券收益', 'Bond returns'],
  method: ['计息方式', 'Calculation'], simple: ['单利（年数）', 'Simple · years'], datedSimple: ['单利（日期）', 'Simple · dates'], compound: ['复利（完整期数）', 'Compound · periods'],
  principal: ['本金', 'Principal'], years: ['期限（年）', 'Term in years'], periods: ['总期数', 'Total periods'],
  compounding: ['每年复利次数', 'Compounds per year'], payments: ['每年支付次数', 'Payments per year'],
  start: ['起日（计入）', 'Start · inclusive'], end: ['止日（不计入）', 'End · exclusive'], basis: ['计息基准', 'Day count'],
  act365F: ['Act/365F · 固定365天', 'Act/365F · fixed 365'], act360: ['Act/360 · 固定360天', 'Act/360 · fixed 360'], actActIsda: ['Act/Act ISDA · 按日历年', 'Act/Act ISDA · calendar years'],
  actActCoupon: ['Actual/Actual · 付息周期', 'Actual/Actual · coupon period'], us30_360: ['US (NASD) 30/360', 'US (NASD) 30/360'],
  source: ['输入利率类型', 'Input rate type'], nominal: ['名义年利率', 'Nominal annual'], effective: ['有效年利率', 'Effective annual'], periodic: ['支付周期利率', 'Payment-period rate'],
  target: ['计算类型', 'Calculation type'], presentValue: ['现值 PV', 'Present value · PV'], futureValue: ['终值 FV', 'Future value · FV'], payment: ['每期金额 PMT', 'Payment · PMT'],
  beginning: ['支付时点', 'Payment timing'], endPayment: ['期末 END', 'End · END'], beginningPayment: ['期初 BGN', 'Beginning · BGN'],
  out: ['支出 −', 'Outflow −'], in: ['收入 +', 'Inflow +'], direction: ['方向', 'Direction'],
  price: ['现价', 'Market price'], intrinsicValue: ['内在价值（净价）', 'Intrinsic value · clean price'], yield: ['到期收益率', 'Yield to maturity'],
  settlement: ['结算日', 'Settlement'], maturity: ['到期日', 'Maturity'], previousCoupon: ['上一付息日', 'Previous coupon'], nextCoupon: ['下一付息日', 'Next coupon'],
  frequency: ['付息频率', 'Payment frequency'], endOfMonth: ['日程规则', 'Schedule rule'], monthEnd: ['保持月底', 'End of month'], dayOfMonth: ['保持到期日号', 'Maturity day of month'],
  coupon: ['年票面利率', 'Annual coupon'], redemption: ['每百元偿还价值', 'Redemption per 100'], yieldRate: ['到期收益率 YTM', 'Yield to maturity · YTM'], dirtyInput: ['输入价格类型', 'Input price type'],
  clean: ['净价', 'Clean price'], dirty: ['全价', 'Full price'], holding: ['持仓面值', 'Holding face value'],
  profile: ['计息方式', 'Interest method'], fixed: ['固定利率', 'Fixed coupon'], discount: ['贴现债券', 'Discount bond'], issue: ['发行日', 'Issue date'], singleBasis: ['计息基准', 'Day count'], convexity: ['凸性', 'Convexity'],
  interestResult: ['利息', 'Interest'], days: ['实际天数', 'Actual days'], effectiveRate: ['有效年利率', 'Effective annual rate'], nominalRate: ['名义年利率', 'Nominal annual rate'], periodicRate: ['每期有效利率', 'Effective periodic rate'],
  cleanPrice: ['净价 / 百元面值', 'Clean price / 100 face'], dirtyPrice: ['全价 / 百元面值', 'Full price / 100 face'], accruedInterest: ['应计利息 / 百元面值', 'Accrued interest / 100 face'],
  macaulayDuration: ['麦考利久期', 'Macaulay duration'], modifiedDuration: ['修正久期', 'Modified duration'], dv01: ['收益率 DV01 / 百元面值', 'Yield DV01 / 100 face'], holdingDv01: ['持仓收益率 DV01', 'Holding yield DV01'], holdingValue: ['持仓全价金额', 'Holding full value'], residual: ['求根回代误差 / 百元', 'Price residual / 100 face'],
}

export function cashFlowDraftRows(draft: FinanceDraft): CashFlowDraftRow[] {
  try {
    const rows = JSON.parse(draft.cashFlows ?? '[]') as unknown
    if (!Array.isArray(rows) || rows.some(row => !row || typeof row.date !== 'string' || typeof row.amount !== 'string' || !['in', 'out'].includes(row.direction))) return []
    return rows as CashFlowDraftRow[]
  } catch { return [] }
}

function buildCashAmount(input: string) {
  if (!/^[+]?(?:\d+(?:\.\d*)?|\.\d+)$/.test(input.trim())) throw { code: 'number', field: 'cashFlowAmount' } satisfies FinanceError
  const amount = Number(input)
  if (!Number.isFinite(amount) || amount > 1e15) throw { code: 'range', field: 'cashFlowAmount' } satisfies FinanceError
  return amount
}

export function financeLabel(key: string, language: Language) {
  return labels[key]?.[language === 'zh' ? 0 : 1] ?? key
}

export function financeText(language: Language) {
  const zh = language === 'zh'
  return {
    holdingScope: zh ? '同一笔面值完整买入和卖出；票息、费用与融资成本填写实收实付总额。收益率分母为买入全价金额加买入费用，未计算杠杆权益回报。票息不再投资，单利年化不是 XIRR。' : 'One position bought and sold in full. Enter actual coupon, fee and financing totals. Return uses purchase full amount plus purchase fees, not leveraged equity. No coupon reinvestment; simple annualization is not XIRR.',
    xirrScope: zh ? '按自己的资金方向：投资付款为支出，回款为收入；借款到账为收入，偿还和费用为支出。固定365天年基准，同日合并、日期排序后求 XIRR；首版支持一次变号，最多200笔。不等同于名义年利率或法定 APR。' : 'Use your cash perspective: investment payments out, proceeds in; loan proceeds in, repayments and fees out. XIRR uses 365-day years after sorting and same-day netting. One sign change, up to 200 entries; not a nominal rate or regulatory APR.',
    title: zh ? '金融计算器' : 'Financial calculator', close: zh ? '关闭' : 'Close', category: zh ? '工具' : 'Tool',
    calculate: zh ? '计算' : 'Calculate', reset: zh ? '恢复示例' : 'Reset tool', computing: zh ? '计算中…' : 'Calculating…',
    result: zh ? '计算结果' : 'Results', empty: zh ? '填写参数后点击计算，或按 Enter。' : 'Enter parameters, then calculate or press Enter.',
    changed: zh ? '参数已修改，请重新计算。' : 'Parameters changed. Calculate again.',
    copyResult: zh ? '复制结果' : 'Copy results', copyRecord: zh ? '复制完整记录' : 'Copy record', insert: zh ? '插入当前笔记' : 'Insert into note',
    copied: zh ? '已复制' : 'Copied', copyFailed: zh ? '复制失败，请重试。' : 'Copy failed. Please retry.',
    insertFailed: zh ? '目标笔记已变化，未插入。可以复制记录后粘贴。' : 'The target note changed. Copy the record to paste it safely.',
    preview: zh ? '切换到编辑或分屏模式后可插入笔记。' : 'Switch to Edit or Split mode to insert into a note.',
    details: zh ? '公式与现金流明细' : 'Formula and cash flows', amount: zh ? '金额' : 'Amount', present: zh ? '折现值' : 'Present value',
    offline: zh ? '离线计算 · 输入仅在本次启动内保留' : 'Offline · inputs retained only during this session',
    bondScope: zh ? '通用规则付息模式：固定利率、到期还本、支持负收益率；结算日的当日票息视为已支付。请核对付息日程。' : 'General regular-coupon mode: fixed rate, bullet repayment, negative yields supported. Same-day coupon is treated as paid. Verify the schedule.',
    domestic: zh ? '国内市场口径正在核验，尚未开放；本模式不代表国内报价或中债 / CFETS 估值。' : 'Domestic conventions await verification. This mode does not reproduce domestic quotes or ChinaBond / CFETS valuations.',
    signs: zh ? '收入为正，支出为负；结果保留现金流方向。利率按名义年利率输入。' : 'Inflows are positive and outflows negative. Results retain their cash-flow sign. Enter a nominal annual rate.',
    compoundScope: zh ? '按完整复利期数计算，不推断零头期利息。利率为名义年利率。' : 'Whole compounding periods only; no partial-period interest. Rate is nominal annual.',
    interestRate: zh ? '年利率' : 'Annual rate', rateValue: zh ? '输入利率' : 'Input rate',
    record: zh ? '金融计算记录' : 'Financial calculation', inputs: zh ? '输入与口径' : 'Inputs and conventions',
    engine: zh ? '引擎版本' : 'Engine version', precision: zh ? '显示舍入，记录保留完整数值精度。' : 'Display is rounded; the record keeps full numeric precision.',
    units: { amount: zh ? '元 / 基本单位' : 'currency units', percent: '%', years: zh ? '年' : 'years', yearsSquared: zh ? '年²' : 'years²', months: zh ? '月' : 'months', periods: zh ? '期' : 'periods', entries: zh ? '笔' : 'entries', days: zh ? '天' : 'days', price: zh ? '元 / 百元面值' : 'units / 100 face' } as Record<string, string>,
  }
}

export function initialFinanceDrafts(): Record<FinanceCategory, FinanceDraft> {
  return {
    holdingReturn: { holdingBuyDate: '2026-01-01', holdingSellDate: '2027-01-01', holdingFace: '100000', holdingBuyType: 'dirty', holdingSellType: 'dirty', holdingBuyPrice: '100', holdingSellPrice: '102', holdingBuyAccrued: '0', holdingSellAccrued: '0', holdingCoupons: '3000', holdingBuyFee: '100', holdingSellFee: '100', holdingFinancingCost: '0' },
    xirr: { cashFlows: JSON.stringify([{ date: '2026-01-01', amount: '10000', direction: 'out' }, { date: '2027-01-01', amount: '11000', direction: 'in' }]) },
    interest: { method: 'simple', principal: '100000', rate: '3', years: '1', periods: '12', compounding: '12', start: '2026-10-01', end: '2026-10-31', basis: 'act365F' },
    rate: { rate: '6', source: 'nominal', compounding: '12', payments: '12' },
    tvm: { target: 'futureValue', rate: '6', compounding: '12', payments: '12', periods: '12', beginning: 'false', presentValue: '0', presentValueDirection: 'out', futureValue: '0', futureValueDirection: 'in', payment: '1000', paymentDirection: 'out' },
    loan: { loanTarget: 'payment', loanMethod: 'annuity', loanPrincipal: '1000000', loanMonths: '360', loanRate: '3.5', loanPayment: '4490.446878088235', loanFrequency: '12' },
    repo: { repoSide: 'lending', repoProduct: '204001', repoAmount: '100000', repoRate: '2', repoFeeRate: '0', repoDays: '', repoDayMode: 'manualDays', repoStart: '2026-10-12', repoEnd: '2026-10-13', repoBasis: '365', repoFeeTiming: 'false' },
    bond: { target: 'yield', profile: 'fixed', issue: '2026-10-01', singleBasis: 'act365F', settlement: '2026-10-12', maturity: '2031-10-01', frequency: '2', endOfMonth: 'false', basis: 'actActCoupon', coupon: '3', redemption: '100', yieldRate: '3.5', price: '100', dirtyInput: 'false', holding: '100' },
  }
}

export function financeFields(category: FinanceCategory, draft: FinanceDraft, language: Language): FinanceField[] {
  const t = financeText(language)
  const field = (key: string, unit?: string): FinanceField => ({ key, label: financeLabel(key, language), unit })
  const options = (key: string, choices: Array<string | [string, string]>): FinanceField => ({ ...field(key), options: choices.map((choice) => {
    const [value, label] = typeof choice === 'string' ? [choice, financeLabel(choice, language)] : [choice[0], financeLabel(choice[1], language)]
    return { value, label }
  }) })
  const rate = { ...field('rate', '%'), label: category === 'rate' ? t.rateValue : t.interestRate }
  const money = (key: string) => field(key, t.units.amount)
  const date = (key: string): FinanceField => ({ ...field(key), type: 'date' })
  if (category === 'xirr') return []
  if (category === 'holdingReturn') return [date('holdingBuyDate'), date('holdingSellDate'), money('holdingFace'),
    options('holdingBuyType', ['dirty', 'clean']), field('holdingBuyPrice', t.units.price),
    ...(draft.holdingBuyType === 'clean' ? [field('holdingBuyAccrued', t.units.price)] : []),
    options('holdingSellType', ['dirty', 'clean']), field('holdingSellPrice', t.units.price),
    ...(draft.holdingSellType === 'clean' ? [field('holdingSellAccrued', t.units.price)] : []),
    ...['holdingCoupons', 'holdingBuyFee', 'holdingSellFee', 'holdingFinancingCost'].map(money)]
  if (category === 'interest') {
    return [options('method', ['simple', 'datedSimple', 'compound']), money('principal'), rate,
      ...(draft.method === 'simple' ? [field('years')] : draft.method === 'compound' ? [field('periods'), field('compounding')] : [date('start'), date('end'), options('basis', ['act365F', 'act360', 'actActIsda'])])]
  }
  if (category === 'rate') return [options('source', ['nominal', 'effective', 'periodic']), rate, field('compounding'), field('payments')]
  if (category === 'loan') return [
    options('loanTarget', [['payment', draft.loanMethod === 'equalPrincipal' ? 'loanFirstTarget' : 'loanPaymentTarget'], ['principal', 'loanPrincipalTarget'], ['rate', 'loanRateTarget']]),
    options('loanMethod', ['annuity', 'equalPrincipal']),
    ...(draft.loanTarget === 'principal' ? [] : [money('loanPrincipal')]), field('loanMonths', t.units.months),
    ...(draft.loanTarget === 'rate' ? [] : [field('loanRate', '%')]),
    options('loanFrequency', [['12', 'monthly'], ['4', 'quarterly'], ['2', 'semiannual'], ['1', 'annually']]),
    ...(draft.loanTarget === 'payment' ? [] : [{ ...money('loanPayment'), label: financeLabel(draft.loanMethod === 'equalPrincipal' ? 'loanFirstPayment' : 'loanPayment', language) }]),
  ]
  if (category === 'repo') return [options('repoSide', ['lending', 'financing']),
    { ...field('repoProduct'), options: [{ value: 'custom', label: financeLabel('custom', language) }, ...repoProducts.map(product => ({ value: product.code, label: `${product.code} · ${language === 'zh' ? product.market === 'sh' ? '沪' : '深' : product.market === 'sh' ? 'SH' : 'SZ'} ${product.days}${language === 'zh' ? '天' : 'd'}` }))] },
    money('repoAmount'), field('repoRate', '%'), field('repoFeeRate', '%'),
    options('repoDayMode', ['manualDays', 'settlementDates']),
    ...(draft.repoDayMode === 'settlementDates' ? [date('repoStart'), date('repoEnd')] : [field('repoDays', t.units.days)]),
    ...(draft.repoProduct === 'custom' ? [{ ...field('repoBasis'), options: ['365', '360'].map(value => ({ value, label: value })) }] : []),
    options('repoFeeTiming', [['false', 'initialFee'], ['true', 'maturityFee']]),
  ]
  if (category === 'tvm') {
    return [options('target', ['presentValue', 'futureValue', 'payment']), rate, field('compounding'), field('payments'), field('periods'), options('beginning', [['false', 'endPayment'], ['true', 'beginningPayment']]),
      ...['presentValue', 'futureValue', 'payment'].filter((key) => key !== draft.target).flatMap((key) => [money(key), { ...options(`${key}Direction`, ['out', 'in']), label: `${financeLabel(key, language)} · ${financeLabel('direction', language)}` }])]
  }
  const single = draft.profile === 'discount' || draft.frequency === '0'
  const target = options('target', ['yield', ['price', 'intrinsicValue'], 'modifiedDuration', 'convexity'])
  if (single) target.options![0]!.label = language === 'zh' ? '到期收益率（单利年化）' : 'Yield · simple annual'
  return [target, options('profile', ['fixed', 'discount']),
    ...(draft.profile === 'discount' ? [] : [{ ...options('frequency', []), options: ['0', '1', '2', '3', '4'].map(value => ({ value, label: value === '0' ? (language === 'zh' ? '到期一次性付息' : 'Interest at maturity') : (language === 'zh' ? `每年${value}付` : `${value} payments / year`) })) }]),
    ...(draft.target === 'yield' ? [field('price', t.units.price), options('dirtyInput', [['false', 'clean'], ['true', 'dirty']])] : [{ ...field('yieldRate', '%'), label: single ? (language === 'zh' ? '收益率（单利年化）' : 'Yield · simple annual') : financeLabel('yieldRate', language) }]),
    ...(draft.profile === 'discount' ? [] : [field('coupon', '%')]),
    date('settlement'), date('maturity'), ...(single && draft.profile !== 'discount' ? [date('issue')] : []),
    ...(single ? [options('singleBasis', ['act365F', 'act360', 'us30_360', 'actActIsda'])] : [options('basis', ['actActCoupon', 'us30_360']), options('endOfMonth', [['false', 'dayOfMonth'], ['true', 'monthEnd']])]),
    field('redemption', t.units.price), money('holding')]

}

export function buildFinanceRequest(category: FinanceCategory, draft: FinanceDraft): FinanceRequest {
  const number = (key: string, integer = false, nonnegative = false) => {
    const input = draft[key]?.trim() ?? ''
    if (!/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)$/.test(input)) throw { code: 'number', field: key } satisfies FinanceError
    const result = Number(input)
    if (!Number.isFinite(result) || Math.abs(result) > 1e15) throw { code: 'range', field: key } satisfies FinanceError
    if (integer && (!Number.isInteger(result) || result < 1 || result > 100000)) throw { code: 'periods', field: key } satisfies FinanceError
    if (nonnegative && result < 0) throw { code: 'nonnegative', field: key } satisfies FinanceError
    return result
  }
  const rate = (key = 'rate') => number(key) / 100
  if (category === 'holdingReturn') return { kind: 'holdingReturn', buyDate: draft.holdingBuyDate!, sellDate: draft.holdingSellDate!,
    face: number('holdingFace', false, true), buyPrice: number('holdingBuyPrice', false, true), sellPrice: number('holdingSellPrice', false, true),
    buyClean: draft.holdingBuyType === 'clean', sellClean: draft.holdingSellType === 'clean',
    buyAccrued: draft.holdingBuyType === 'clean' ? number('holdingBuyAccrued', false, true) : 0,
    sellAccrued: draft.holdingSellType === 'clean' ? number('holdingSellAccrued', false, true) : 0,
    coupons: number('holdingCoupons', false, true), buyFee: number('holdingBuyFee', false, true), sellFee: number('holdingSellFee', false, true), financingCost: number('holdingFinancingCost', false, true) }
  if (category === 'xirr') {
    const rows = cashFlowDraftRows(draft)
    if (rows.length < 2 || rows.length > 200) throw { code: 'cashFlowCount', field: 'cashFlows' } satisfies FinanceError
    return { kind: 'xirr', flows: rows.map((row, index) => {
      try {
        const amount = buildCashAmount(row.amount)
        if (!['in', 'out'].includes(row.direction)) throw { code: 'cashFlowDirection', field: 'cashFlows' }
        return { date: row.date, amount: row.direction === 'out' ? -amount : amount }
      } catch (reason) { throw { ...(reason as FinanceError), row: index + 1 } satisfies FinanceError }
    }) }
  }
  if (category === 'interest') {
    const common = { principal: number('principal', false, true), rate: rate() }
    if (draft.method === 'compound') return { kind: 'compound', ...common, periods: number('periods', true), compounding: number('compounding', true) }
    if (draft.method === 'datedSimple') return { kind: 'datedSimple', ...common, start: draft.start!, end: draft.end!, basis: draft.basis as 'act365F' }
    return { kind: 'simple', ...common, years: number('years', false, true) }
  }
  if (category === 'rate') return { kind: 'rate', rate: rate(), source: draft.source as 'nominal', compounding: number('compounding', true), payments: number('payments', true) }
  if (category === 'loan') return { kind: 'loan', target: draft.loanTarget as 'payment', method: draft.loanMethod as 'annuity',
    principal: draft.loanTarget === 'principal' ? 0 : number('loanPrincipal', false, true),
    rate: draft.loanTarget === 'rate' ? 0 : rate('loanRate'), payment: draft.loanTarget === 'payment' ? 0 : number('loanPayment', false, true),
    months: number('loanMonths', true), frequency: number('loanFrequency', true) }
  if (category === 'repo') {
    if (draft.repoProduct !== 'custom' && !repoProducts.some(product => product.code === draft.repoProduct)) throw { code: 'repoProduct', field: 'repoProduct' } satisfies FinanceError
    return { kind: 'repo', side: draft.repoSide as 'lending', amount: number('repoAmount', false, true), rate: rate('repoRate'), feeRate: rate('repoFeeRate'),
      feeAtMaturity: draft.repoFeeTiming === 'true', days: draft.repoDayMode === 'settlementDates' ? 0 : number('repoDays', true),
      start: draft.repoDayMode === 'settlementDates' ? draft.repoStart! : null, end: draft.repoDayMode === 'settlementDates' ? draft.repoEnd! : null,
      basis: draft.repoProduct === 'custom' ? number('repoBasis', true) : 365 }
  }
  if (category === 'tvm') {
    const signed = (key: string) => draft.target === key ? 0 : number(key, false, true) * (draft[`${key}Direction`] === 'out' ? -1 : 1)
    return { kind: 'tvm', target: draft.target as 'futureValue', rate: rate(), compounding: number('compounding', true), payments: number('payments', true), periods: number('periods', true), beginning: draft.beginning === 'true', presentValue: signed('presentValue'), futureValue: signed('futureValue'), payment: signed('payment') }
  }
  const common = { target: draft.target === 'yield' ? 'yield' as const : 'price' as const,
    settlement: draft.settlement!, maturity: draft.maturity!, redemption: number('redemption', false, true),
    yieldRate: draft.target !== 'yield' ? rate('yieldRate') : 0, price: draft.target === 'yield' ? number('price', false, true) : 0,
    dirtyInput: draft.dirtyInput === 'true', holding: number('holding', false, true) }
  if (draft.profile === 'discount' || draft.frequency === '0') return { kind: 'singleBond', ...common,
    profile: draft.profile === 'discount' ? 'discount' : 'maturityInterest', issue: draft.issue!,
    basis: draft.singleBasis as 'act365F', coupon: draft.profile === 'discount' ? 0 : rate('coupon') }
  return { kind: 'bond', ...common, frequency: number('frequency', true), endOfMonth: draft.endOfMonth === 'true',
    basis: draft.basis as 'actActCoupon', coupon: rate('coupon') }

}

export function formatFinanceValue(item: FinanceValue, language: Language, full = false) {
  const value = item.unit === 'percent' ? item.value * 100 : item.value
  const digits = item.unit === 'amount' ? 2 : ['days', 'periods', 'months', 'entries'].includes(item.unit) ? 0 : 6
  const formatted = full ? String(Object.is(value, -0) ? 0 : value) : new Intl.NumberFormat(language === 'zh' ? 'zh-CN' : 'en-US', { minimumFractionDigits: digits, maximumFractionDigits: digits }).format(Object.is(value, -0) ? 0 : value)
  return `${formatted}${financeText(language).units[item.unit] ? ` ${financeText(language).units[item.unit]}` : ''}`
}

export function financeResultText(response: FinanceResponse, language: Language, full = false) {
  return response.values.filter((value) => full || !['residual', 'loanResidual', 'xirrResidual'].includes(value.key)).map((value) => `${financeLabel(value.key === 'interest' ? 'interestResult' : value.key === 'holdingReturn' ? 'holdingReturnResult' : value.key, language)}: ${formatFinanceValue(value, language, full)}`).join('\n')
}

export function financeRecord(category: FinanceCategory, draft: FinanceDraft, response: FinanceResponse, language: Language) {
  const t = financeText(language)
  const fields = financeFields(category, draft, language)
  const fieldInputs = fields.map((field) => {
    const display = field.options?.find((option) => option.value === draft[field.key])?.label ?? draft[field.key]
    return `- ${field.label}: ${display}${field.unit ? ` ${field.unit}` : ''}`
  }).join('\n')
  const inputs = category === 'xirr' ? `| ${language === 'zh' ? '日期' : 'Date'} | ${financeLabel('direction', language)} | ${t.amount} |\n| --- | --- | ---: |\n${cashFlowDraftRows(draft).map(row => `| ${row.date} | ${financeLabel(row.direction, language)} | ${row.amount} |`).join('\n')}` : fieldInputs
  const cashFlows = response.cashFlows.length ? `\n\n| ${financeLabel('settlement', language)} / ${language === 'zh' ? '现金流日期' : 'cash-flow date'} | ${t.amount} / 100 | ${financeLabel('years', language)} | ${t.present} / 100 |\n| --- | ---: | ---: | ---: |\n${response.cashFlows.map((flow) => `| ${flow.date} | ${flow.amount} | ${flow.years} | ${flow.presentValue} |`).join('\n')}` : ''
  const datedCashFlows = response.datedCashFlows?.length ? `\n\n${language === 'zh' ? '按日期排序、同日合并后的实际现金流' : 'Actual cash flows sorted and netted by date'}\n\n| ${language === 'zh' ? '日期' : 'Date'} | ${t.amount} |\n| --- | ---: |\n${response.datedCashFlows.map(flow => `| ${flow.date} | ${flow.amount} |`).join('\n')}` : ''
  const schedule = response.schedule ? `\n- ${financeLabel('previousCoupon', language)}: ${response.schedule.previousCoupon ?? '—'}\n- ${financeLabel('nextCoupon', language)}: ${response.schedule.nextCoupon}\n- ${language === 'zh' ? '剩余支付次数' : 'Remaining payments'}: ${response.schedule.remainingPayments}` : ''
  const repayments = response.repayments?.length ? `\n\n| ${language === 'zh' ? '期数' : 'Period'} | ${language === 'zh' ? '还款额' : 'Payment'} | ${financeLabel('principal', language)} | ${financeLabel('interestResult', language)} | ${language === 'zh' ? '剩余本金' : 'Balance'} |\n| --- | ---: | ---: | ---: | ---: |\n${response.repayments.map(row => `| ${row.period} | ${row.payment} | ${row.principal} | ${row.interest} | ${row.balance} |`).join('\n')}` : ''
  return `## ${t.record} · ${financeLabel(category, language)}\n\n${inputs}${schedule}\n\n${category === 'bond' ? `${bondScope(draft, language)}\n${t.domestic}\n\n` : category === 'tvm' ? `${t.signs}\n\n` : category === 'loan' ? `${loanScope(language)}\n\n` : category === 'repo' ? `${repoScope(draft, language)}\n\n` : category === 'holdingReturn' ? `${t.holdingScope}\n\n` : category === 'xirr' ? `${t.xirrScope}\n\n` : ''}${financeResultText(response, language, true)}\n\n${response.formula}\n\n${t.engine}: ${response.engineVersion}\n${t.precision}${cashFlows}${datedCashFlows}${repayments}\n`
}

export function financeErrorText(error: FinanceError, language: Language) {
  const reasons: Record<string, [string, string]> = {
    cashFlowCount: ['请录入 2–200 笔现金流。', 'Enter 2–200 cash flows.'], cashFlowDates: ['合并同日现金流后，需至少两个不同日期的非零收付款。', 'After same-day netting, use nonzero flows on at least two dates.'], cashFlowSigns: ['需要收入和支出两种方向的现金流。', 'Include both inflows and outflows.'], cashFlowPattern: ['日期排序和同日合并后多次变号，可能有多个年化结果；首版仅支持一次变号。', 'Multiple sign changes may have multiple rates; this version supports one sign change.'], cashFlowDirection: ['请选择收入或支出。', 'Choose inflow or outflow.'], xirrRange: ['年化利率超出支持范围（−99.9999% 至 100000000%）。', 'Annual rate is outside −99.9999% to 100000000%.'],
    number: ['请输入数字，单位见字段标签。', 'Enter a number in the labelled unit.'], range: ['数值超出支持范围。', 'Value is outside the supported range.'], nonnegative: ['请输入非负值。', 'Enter a nonnegative value.'], positive: ['请输入大于零的值。', 'Enter a value greater than zero.'],
    frequency: ['每年次数应为 1–366 的整数。', 'Frequency must be an integer from 1 to 366.'], bondFrequency: ['规则付息次数支持 1、2、3、4。', 'Bond frequency must be 1, 2, 3 or 4.'], periods: ['请输入支持范围内的正整数。', 'Enter a positive integer within the supported range.'],
    loanFrequency: ['偿还频率支持每月、每季、每半年或每年。', 'Repay monthly, quarterly, semiannually or annually.'], loanTerm: ['贷款期限为 1–1200 个月的整数。', 'Loan term must be 1–1200 whole months.'], termFrequency: ['期限必须包含整数个偿还周期；例如每季还款需为 3 个月的倍数。', 'Term must contain whole repayment periods; quarterly terms must be multiples of 3 months.'], loanRateRange: ['名义年利率支持 0%–1000%。', 'Nominal annual rate must be 0%–1000%.'], loanPaymentLow: ['还款额低于零利率下的应还金额，无法求得非负年利率。', 'Payment is below the zero-rate amount; no nonnegative annual rate exists.'],
    repoFeeRange: ['手续费率支持 0%–100%。', 'Fee rate must be 0%–100%.'], repoBasis: ['年计息天数支持 360 或 365。', 'Use 360 or 365 days per year.'], repoDaysRange: ['实际占款天数为 1–36600 的整数。', 'Actual funding days must be 1–36600.'], repoProduct: ['请选择支持的品种或自定义回购。', 'Select a supported product or custom repo.'],
    rateDomain: ['利率使计算无定义，请检查口径。', 'The rate is outside the mathematical domain.'], date: ['请输入有效的 YYYY-MM-DD 日期（1600–9999）。', 'Enter a valid YYYY-MM-DD date (1600–9999).'], dateOrder: ['结束日期必须晚于开始日期。', 'The end date must be later than the start date.'],
    schedule: ['付息日与到期日、频率或月底规则不一致。', 'Coupon dates do not match maturity, frequency or month-end rule.'], yieldDomain: ['价格超出当前收益率模型的求解范围。', 'Price is outside this yield model’s supported range.'],
    overflow: ['数值无法稳定计算，请缩小参数范围。', 'Calculation overflowed. Reduce the input range.'], noConvergence: ['利率求解未收敛，未返回估算结果。', 'Rate did not converge; no estimated result was returned.'], unavailable: ['请在 NeoPad 桌面应用中计算。', 'Calculate in the NeoPad desktop app.'], unknown: ['计算失败，请检查输入并重试。', 'Calculation failed. Check the inputs and retry.'],
  }
  const row = error.row ? (language === 'zh' ? `第 ${error.row} 笔 · ` : `Entry ${error.row} · `) : ''
  return `${row}${financeLabel(error.field, language)}: ${(reasons[error.code] ?? reasons.unknown)![language === 'zh' ? 0 : 1]}`
}

export function bondScope(draft: FinanceDraft, language: Language) {
  if (draft.profile === 'discount' || draft.frequency === '0') return language === 'zh'
    ? '到期单次偿付，按单利年化投资收益率折现；贴现债券无票息，非银行贴现率。'
    : 'Single repayment discounted using a simple annual investment yield. Discount bonds have no coupon; this is not a bank discount rate.'
  return financeText(language).bondScope
}

export const repoProducts = [
  ...[1, 2, 3, 4, 7, 14, 28, 91, 182].map(days => ({ code: `204${String(days).padStart(3, '0')}`, market: 'sh', days })),
  ...[['131810', 1], ['131811', 2], ['131800', 3], ['131809', 4], ['131801', 7], ['131802', 14], ['131803', 28], ['131805', 91], ['131806', 182]].map(([code, days]) => ({ code: String(code), market: 'sz', days: Number(days) })),
]
export function loanScope(language: Language) {
  return language === 'zh' ? '固定利率、期末还款，期利率 = 名义年利率 ÷ 每年偿还次数。明细按未舍入数值计算，显示到分；不含手续费、提前还款或利率调整。' : 'Fixed rate, end-of-period payments. Periodic rate = nominal annual rate / payments per year. Schedule uses unrounded values, displayed to cents; excludes fees, prepayment and rate changes.'
}
export function repoScope(draft: FinanceDraft, language: Language) {
  const basis = draft.repoProduct === 'custom' ? draft.repoBasis : '365'
  return language === 'zh' ? `实际占款天数单独填写，或按实际交收日期计算；品种名义期限不代替占款天数。年计息天数 ${basis}；手续费按成交金额一次计收，不设置最低收费。折算年率以成交金额为基准，不是 IRR。` : `Enter actual funding days or actual settlement dates; product tenor does not determine funding days. Year basis ${basis}; fee charged once on cash amount with no minimum. Annualized net rate uses cash amount, not IRR.`
}
