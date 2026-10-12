export type FinanceCategory = 'interest' | 'rate' | 'tvm' | 'bond' | 'loan' | 'repo' | 'holdingReturn' | 'xirr'
export type FinanceDraft = Record<string, string>
export type FinanceRequest =
  | { kind: 'holdingReturn'; buyDate: string; sellDate: string; face: number; buyPrice: number; sellPrice: number; buyClean: boolean; sellClean: boolean; buyAccrued: number; sellAccrued: number; coupons: number; buyFee: number; sellFee: number; financingCost: number }
  | { kind: 'xirr'; flows: Array<{ date: string; amount: number }> }
  | { kind: 'loan'; target: 'payment' | 'principal' | 'rate'; method: 'annuity' | 'equalPrincipal'; principal: number; rate: number; payment: number; months: number; frequency: number }
  | { kind: 'repo'; side: 'financing' | 'lending'; amount: number; rate: number; feeRate: number; feeAtMaturity: boolean; days: number; start: string | null; end: string | null; basis: number }
  | { kind: 'simple'; principal: number; rate: number; years: number }
  | { kind: 'datedSimple'; principal: number; rate: number; start: string; end: string; basis: 'act365F' | 'act360' | 'actActIsda' }
  | { kind: 'compound'; principal: number; rate: number; periods: number; compounding: number }
  | { kind: 'rate'; rate: number; source: 'nominal' | 'effective' | 'periodic'; compounding: number; payments: number }
  | { kind: 'tvm'; target: 'presentValue' | 'futureValue' | 'payment'; rate: number; compounding: number; payments: number; periods: number; beginning: boolean; presentValue: number; futureValue: number; payment: number }
  | { kind: 'bond'; target: 'price' | 'yield'; settlement: string; maturity: string; previousCoupon?: string; nextCoupon?: string; frequency: number; endOfMonth: boolean; basis: 'actActCoupon' | 'us30_360'; coupon: number; redemption: number; yieldRate: number; price: number; dirtyInput: boolean; holding: number }

  | { kind: 'singleBond'; target: 'price' | 'yield'; profile: 'discount' | 'maturityInterest'; settlement: string; maturity: string; issue: string; basis: 'act365F' | 'act360' | 'actActIsda' | 'us30_360'; coupon: number; redemption: number; yieldRate: number; price: number; dirtyInput: boolean; holding: number }

export interface FinanceValue { key: string; value: number; unit: string }
export interface LoanRepayment { period: number; payment: number; principal: number; interest: number; balance: number }
export interface FinanceResponse {
  datedCashFlows?: Array<{ date: string; amount: number }>
  repayments?: LoanRepayment[]
  engineVersion: string
  values: FinanceValue[]
  cashFlows: Array<{ date: string; amount: number; years: number; presentValue: number }>
  schedule?: { previousCoupon: string | null; nextCoupon: string; remainingPayments: number }
  formula: string
}
export interface FinanceError { code: string; field: string; row?: number }
export interface CashFlowDraftRow { date: string; amount: string; direction: 'in' | 'out' }
export interface FinancialInsertTarget { noteId: string; documentText: string; cursor: number; documentVersion: number }
