<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { bondScope, loanScope, repoScope, cashFlowDraftRows, financeCategories, financeErrorText, financeFields, financeLabel, financeText, formatFinanceValue } from '../lib/finance-ui'
import FinanceFieldControl from './FinanceFieldControl.vue'
import LoanScheduleTable from './LoanScheduleTable.vue'
import CashFlowEditor from './CashFlowEditor.vue'
import type { FinanceCategory, FinanceDraft, FinanceError, FinanceResponse } from '../types/finance'

const props = defineProps<{
  language: 'zh' | 'en'; draft: FinanceDraft; result: FinanceResponse | null; error: FinanceError | null
  busy: boolean; changed: boolean; canInsert: boolean; notice: string
}>()
const category = defineModel<FinanceCategory>('category', { required: true })
const emit = defineEmits<{ close: []; calculate: []; reset: []; updateField: [key: string, value: string]; copy: [record: boolean]; insert: [] }>()
const panel = ref<HTMLElement | null>(null)
const t = computed(() => financeText(props.language))
const fields = computed(() => financeFields(category.value, props.draft, props.language))
const advancedKeys = ['basis', 'singleBasis', 'endOfMonth', 'redemption', 'holding']
const groups = computed(() => category.value === 'bond' ? [
  { title: props.language === 'zh' ? '债券类型' : 'Bond type', fields: fields.value.filter(field => ['profile', 'frequency'].includes(field.key)) },
  { title: props.language === 'zh' ? '参数设置' : 'Parameters', fields: fields.value.filter(field => !['target', 'profile', 'frequency', ...advancedKeys].includes(field.key)) },
] : category.value === 'loan' ? [{ title: props.language === 'zh' ? '贷款信息' : 'Loan information', fields: fields.value.filter(field => field.key !== 'loanTarget') }]
  : category.value === 'repo' ? [{ title: props.language === 'zh' ? '回购参数' : 'Repo parameters', fields: fields.value.filter(field => field.key !== 'repoSide') }]
  : category.value === 'holdingReturn' ? [
    { title: props.language === 'zh' ? '持有信息' : 'Position', fields: fields.value.filter(field => ['holdingBuyDate', 'holdingSellDate', 'holdingFace'].includes(field.key)) },
    { title: props.language === 'zh' ? '买卖价格 · 百元面值' : 'Prices · per 100 face', fields: fields.value.filter(field => /holding(Buy|Sell)(Type|Price|Accrued)/.test(field.key)) },
    { title: props.language === 'zh' ? '实收实付 · 总金额' : 'Actual receipts / costs · total amounts', fields: fields.value.filter(field => ['holdingCoupons', 'holdingBuyFee', 'holdingSellFee', 'holdingFinancingCost'].includes(field.key)) },
  ] : category.value === 'xirr' ? [] : [{ title: props.language === 'zh' ? '参数设置' : 'Parameters', fields: fields.value }])
const advanced = computed(() => category.value === 'bond' ? fields.value.filter(field => advancedKeys.includes(field.key)) : [])
const results = computed(() => props.result?.values.filter(item => !['residual', 'loanResidual', 'xirrResidual'].includes(item.key)) ?? [])
const primaryKey = computed(() => category.value === 'loan'
  ? ({ payment: 'firstPayment', principal: 'loanPrincipal', rate: 'loanRate' }[props.draft.loanTarget])
  : category.value === 'repo' ? (props.draft.repoSide === 'financing' ? 'repoFinancingCost' : 'repoNetIncome')
  : category.value === 'holdingReturn' ? 'holdingProfit' : category.value === 'xirr' ? 'xirrRate'
  : props.draft.target === 'yield' ? 'yieldRate' : props.draft.target === 'price' ? 'cleanPrice' : props.draft.target)
const primary = computed(() => props.result?.values.find(item => item.key === primaryKey.value))
const targetField = computed(() => fields.value.find(field => field.key === (category.value === 'loan' ? 'loanTarget' : category.value === 'repo' ? 'repoSide' : 'target')))
onMounted(() => nextTick(() => panel.value?.querySelector<HTMLElement>('[data-finance-field], [data-flow-date]')?.focus()))
watch(category, () => nextTick(() => {
  const body = panel.value?.querySelector<HTMLElement>('.finance-body')
  if (body) body.scrollTop = 0
}))
function trapFocus(event: KeyboardEvent) {
  if (event.key !== 'Tab' || event.ctrlKey || event.metaKey) return
  const controls = [...(panel.value?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), select:not(:disabled), summary') ?? [])].filter(control => control.getClientRects().length > 0)
  const first = controls[0], last = controls.at(-1)
  if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus() }
  else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus() }
}
</script>

<template>
  <div class="financial-tools-overlay">
    <section ref="panel" class="financial-tools-panel" role="dialog" aria-modal="true" :aria-label="t.title" @keydown="trapFocus">
      <header class="finance-header"><strong>{{ t.title }}</strong><span class="finance-shortcut">Shift+F1</span><button type="button" class="finance-close" @click="emit('close')">{{ t.close }}</button></header>
      <div class="finance-workspace">
        <nav class="finance-categories" :aria-label="t.category">
          <span class="finance-nav-heading">{{ language === 'zh' ? '债券类' : 'Bonds' }}</span>
          <button type="button" :aria-pressed="category === 'bond'" data-finance-category="bond" @click="category = 'bond'">{{ financeLabel('bond', language) }}</button>
          <button type="button" :aria-pressed="category === 'repo'" data-finance-category="repo" @click="category = 'repo'">{{ financeLabel('repo', language) }}</button>
          <button type="button" :aria-pressed="category === 'holdingReturn'" data-finance-category="holdingReturn" @click="category = 'holdingReturn'">{{ financeLabel('holdingReturn', language) }}</button>
          <span class="finance-nav-heading">{{ language === 'zh' ? '贷款类' : 'Loans' }}</span>
          <button type="button" :aria-pressed="category === 'loan'" data-finance-category="loan" @click="category = 'loan'">{{ financeLabel('loan', language) }}</button>
          <span class="finance-nav-heading">{{ language === 'zh' ? '资金类' : 'Money' }}</span>
          <button v-for="tool in financeCategories.filter(tool => !['bond', 'repo', 'loan', 'holdingReturn'].includes(tool))" :key="tool" type="button" :aria-pressed="category === tool" :data-finance-category="tool" @click="category = tool">{{ financeLabel(tool, language) }}</button>
        </nav>
        <div class="finance-content">
          <div class="finance-body">
            <div class="finance-page">
              <form id="finance-calculation" class="finance-form" @submit.prevent="emit('calculate')">
                <fieldset v-if="['bond', 'loan', 'repo'].includes(category) && targetField" class="finance-calculation-type">
                  <legend>{{ language === 'zh' ? '计算类型' : 'Calculation' }}</legend>
                  <div class="finance-fields">
                    <FinanceFieldControl :field="targetField" :value="draft[targetField.key]" @change="(key, value) => emit('updateField', key, value)" />
                    <div class="finance-primary-result" aria-live="polite"><span>{{ t.result }}</span><output>{{ primary ? formatFinanceValue(primary, language) : '—' }}</output></div>
                  </div>
                </fieldset>
                <fieldset v-if="category === 'holdingReturn' || category === 'xirr'">
                  <legend>{{ t.result }}</legend>
                  <div class="finance-primary-result"><span>{{ financeLabel(primaryKey ?? 'result', language) }}</span><output>{{ primary ? formatFinanceValue(primary, language) : '—' }}</output></div>
                </fieldset>
                <fieldset v-for="group in groups" :key="group.title">
                  <legend>{{ group.title }}</legend>
                  <div class="finance-fields">
                    <FinanceFieldControl v-for="field in group.fields" :key="field.key" :field="field" :value="draft[field.key]" :invalid="error?.field === field.key" @change="(key, value) => emit('updateField', key, value)" />
                  </div>
                </fieldset>
                <p v-if="category === 'bond'" class="finance-hint finance-schedule-hint">{{ draft.profile === 'discount' || draft.frequency === '0' ? bondScope(draft, language) : language === 'zh' ? '规则付息日按到期日与频率自动推算。价格、票息和偿还价值均按百元面值。' : 'Regular coupon dates are derived from maturity and frequency. Prices, coupons and redemption are per 100 face.' }}</p>
                <details v-if="advanced.length" class="finance-advanced" :open="error && advancedKeys.includes(error.field) ? true : undefined">
                  <summary>{{ language === 'zh' ? '高级设置 · 计息基准 / 偿还价值 / 持仓' : 'Advanced · day count / redemption / holding' }}</summary>
                  <div class="finance-fields"><FinanceFieldControl v-for="field in advanced" :key="field.key" :field="field" :value="draft[field.key]" :invalid="error?.field === field.key" @change="(key, value) => emit('updateField', key, value)" /></div>
                </details>
                <p v-if="category === 'tvm'" class="finance-hint">{{ t.signs }}</p>
                <p v-if="category === 'loan'" class="finance-hint">{{ loanScope(language) }}</p>
                <p v-if="category === 'repo'" class="finance-hint">{{ repoScope(draft, language) }}</p>
                <p v-if="category === 'holdingReturn'" class="finance-hint">{{ t.holdingScope }}</p>
                <template v-if="category === 'xirr'">
                  <p class="finance-hint">{{ t.xirrScope }}</p>
                  <fieldset><legend>{{ language === 'zh' ? '实际现金流' : 'Actual cash flows' }}</legend><CashFlowEditor :rows="cashFlowDraftRows(draft)" :language="language" :error="error" @change="rows => emit('updateField', 'cashFlows', JSON.stringify(rows))" /></fieldset>
                </template>
                <p v-if="category === 'interest' && draft.method === 'compound'" class="finance-hint">{{ t.compoundScope }}</p>
                <p v-if="error" role="alert" class="finance-error">{{ financeErrorText(error, language) }}</p>
              </form>
              <section class="finance-results" :aria-label="t.result" aria-live="polite">
                <p v-if="!result" class="finance-hint">{{ changed ? t.changed : t.empty }}</p>
                <template v-else>
                  <strong>{{ t.result }}</strong>
                  <dl><div v-for="item in results" :key="item.key"><dt>{{ financeLabel(item.key === 'interest' ? 'interestResult' : item.key === 'holdingReturn' ? 'holdingReturnResult' : item.key, language) }}</dt><dd :data-finance-result="item.key">{{ formatFinanceValue(item, language) }}</dd></div></dl>
                  <p v-if="result.schedule" class="finance-schedule" data-finance-schedule>{{ result.schedule.previousCoupon ? `${financeLabel('previousCoupon', language)} ${result.schedule.previousCoupon} · ` : '' }}{{ financeLabel('nextCoupon', language) }} {{ result.schedule.nextCoupon }} · {{ language === 'zh' ? '剩余支付' : 'Remaining payments' }} {{ result.schedule.remainingPayments }}</p>
                  <LoanScheduleTable v-if="result.repayments" :rows="result.repayments" :language="language" />
                  <details class="finance-details"><summary>{{ t.details }}</summary><p>{{ result.formula }}</p><p>{{ t.engine }}: {{ result.engineVersion }}</p><p v-for="item in result.values.filter(value => ['residual', 'loanResidual', 'xirrResidual'].includes(value.key))" :key="item.key">{{ financeLabel(item.key, language) }}: {{ item.value }}</p>
                    <div v-if="result.datedCashFlows" class="finance-table-scroll"><p>{{ language === 'zh' ? '按日期排序、同日合并后的有效现金流' : 'Effective cash flows after sorting and same-day netting' }}</p><table><thead><tr><th>{{ language === 'zh' ? '日期' : 'Date' }}</th><th>{{ t.amount }}</th></tr></thead><tbody><tr v-for="flow in result.datedCashFlows" :key="flow.date"><td>{{ flow.date }}</td><td>{{ flow.amount }}</td></tr></tbody></table></div>
                    <div v-if="result.cashFlows.length" class="finance-table-scroll"><table><thead><tr><th>{{ language === 'zh' ? '日期' : 'Date' }}</th><th>{{ t.amount }} / 100</th><th>{{ t.present }} / 100</th></tr></thead><tbody><tr v-for="flow in result.cashFlows" :key="flow.date"><td>{{ flow.date }}</td><td>{{ flow.amount }}</td><td>{{ flow.presentValue.toFixed(6) }}</td></tr></tbody></table></div>
                  </details>
                </template>
                <details v-if="category === 'bond'" class="finance-details finance-conventions"><summary>{{ language === 'zh' ? '计算口径' : 'Conventions' }}</summary><p>{{ bondScope(draft, language) }}</p><p class="finance-domestic">{{ t.domestic }}</p></details>
                <p v-if="!canInsert" class="finance-hint">{{ t.preview }}</p>
                <p v-if="result" class="finance-hint">{{ t.precision }}</p>
              </section>
            </div>
          </div>
          <p v-if="notice" class="finance-notice" role="status">{{ notice }}</p>
          <div class="finance-actions">
            <button type="submit" form="finance-calculation" class="finance-primary" :disabled="busy">{{ busy ? t.computing : t.calculate }}</button>
            <button type="button" @click="emit('reset')">{{ t.reset }}</button>
            <span class="finance-actions-spacer"></span>
            <button type="button" :disabled="!result || busy" data-finance-copy="result" @click="emit('copy', false)">{{ t.copyResult }}</button>
            <button type="button" :disabled="!result || busy" data-finance-copy="record" @click="emit('copy', true)">{{ t.copyRecord }}</button>
            <button type="button" :disabled="!result || busy || !canInsert" data-finance-insert @click="emit('insert')">{{ t.insert }}</button>
          </div>
        </div>
      </div>
      <footer>{{ t.offline }}</footer>
    </section>
  </div>
</template>

<style scoped>
.financial-tools-overlay { position: absolute; inset: 0; z-index: 125; background: var(--np-overlay); padding: 8px; display: flex; }
.financial-tools-panel { width: 100%; min-width: 0; min-height: 0; display: flex; flex-direction: column; background: var(--np-surface); color: var(--np-text); border: 1px solid var(--np-border); border-radius: 6px; box-shadow: var(--np-shadow-panel); font-size: 12px; }
.finance-header { display: flex; align-items: center; gap: 10px; padding: 7px 10px; border-bottom: 1px solid var(--np-border); }
.finance-shortcut { color: var(--np-muted); font-size: 11px; }
.finance-close { margin-left: auto; }
button { cursor: pointer; background: var(--np-control-active); color: var(--np-text); border: 1px solid var(--np-border); border-radius: 3px; padding: 5px 8px; font: inherit; line-height: 1.3; }
button:disabled { cursor: default; opacity: .5; }
button:hover:not(:disabled) { background: var(--np-hover); }
button:focus-visible, summary:focus-visible { outline: 2px solid var(--np-accent); outline-offset: 1px; }
.finance-workspace { display: flex; min-height: 0; flex: 1; }
.finance-categories { width: 112px; flex-shrink: 0; padding: 8px 6px; border-right: 1px solid var(--np-border); display: flex; flex-direction: column; gap: 3px; overflow: auto; background: var(--np-bg); }
.finance-nav-heading { padding: 8px 4px 4px; color: var(--np-muted); font-size: 11px; }
.finance-categories button { text-align: left; background: transparent; border-color: transparent; padding: 6px; overflow-wrap: anywhere; }
.finance-categories button[aria-pressed="true"] { background: var(--np-accent-soft); border-color: var(--np-accent); }
.finance-content { min-width: 0; flex: 1; display: flex; flex-direction: column; }
.finance-body { overflow: auto; min-height: 0; flex: 1; padding: 10px 12px; }
.finance-page { max-width: 820px; margin: 0 auto; }
fieldset { min-width: 0; padding: 10px; margin: 0 0 10px; border: 1px solid var(--np-border); border-radius: 3px; }
legend { padding: 0 5px; color: var(--np-muted); }
.finance-fields { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 8px 16px; }
.finance-primary-result { display: grid; grid-template-columns: auto minmax(0, 1fr); align-items: center; gap: 10px; }
output { padding: 5px 7px; background: var(--np-bg); border: 1px solid var(--np-border); border-radius: 3px; min-height: 17px; font-variant-numeric: tabular-nums; font-weight: 600; overflow-wrap: anywhere; }
.finance-advanced { margin: 10px 0 16px; color: var(--np-muted); font-size: 11px; }
.finance-advanced .finance-fields { margin-top: 10px; color: var(--np-text); font-size: 12px; }
.finance-hint { color: var(--np-muted); line-height: 1.5; font-size: 11px; margin: 8px 0; overflow-wrap: anywhere; }
.finance-results { border-top: 1px solid var(--np-border); padding-top: 10px; }
dl { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 8px 16px; margin: 10px 0; }
dl > div { min-width: 0; } dt { color: var(--np-muted); font-size: 11px; } dd { font-variant-numeric: tabular-nums; font-weight: 600; margin: 3px 0 0; overflow-wrap: anywhere; }
.finance-schedule { font-size: 11px; color: var(--np-muted); line-height: 1.5; }
.finance-details { margin: 10px 0; color: var(--np-muted); font-size: 11px; overflow-wrap: anywhere; }
summary { cursor: pointer; line-height: 1.5; } .finance-table-scroll { overflow: auto; } table { border-collapse: collapse; width: 100%; white-space: nowrap; } th, td { padding: 5px; text-align: right; border-bottom: 1px solid var(--np-border); }
.finance-error { color: var(--np-danger); line-height: 1.5; overflow-wrap: anywhere; }
.finance-actions { padding: 8px 10px; display: flex; flex-wrap: wrap; gap: 5px; border-top: 1px solid var(--np-border); }
.finance-actions-spacer { flex: 1; }
.finance-primary { background: var(--np-accent); color: var(--np-accent-contrast); border-color: var(--np-accent); }
.finance-primary:hover:not(:disabled) { background: var(--np-accent); filter: brightness(1.1); }
.finance-notice { margin: 0; padding: 5px 10px; font-size: 11px; overflow-wrap: anywhere; }
footer { padding: 5px 10px; color: var(--np-muted); font-size: 10px; border-top: 1px solid var(--np-border); }
@media (max-width: 640px) { .finance-fields, dl { grid-template-columns: minmax(0, 1fr); } .finance-categories { width: 82px; font-size: 11px; } .finance-body { padding: 8px; } .finance-actions-spacer { display: none; } }
</style>
