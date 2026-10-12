<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { LoanRepayment } from '../types/finance'
const props = defineProps<{ rows: LoanRepayment[]; language: 'zh' | 'en' }>()
const limit = ref(50)
const displayed = computed(() => props.rows.slice(0, limit.value))
watch(() => props.rows, () => { limit.value = 50 })
const amount = (value: number) => new Intl.NumberFormat(props.language === 'zh' ? 'zh-CN' : 'en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(value)
</script>
<template>
  <details class="loan-schedule" data-loan-schedule>
    <summary>{{ language === 'zh' ? '还款明细' : 'Repayment schedule' }} · {{ rows.length }} {{ language === 'zh' ? '期' : 'periods' }}</summary>
    <div class="loan-table-scroll">
      <table><thead><tr><th>{{ language === 'zh' ? '期数' : 'Period' }}</th><th>{{ language === 'zh' ? '还款额' : 'Payment' }}</th><th>{{ language === 'zh' ? '本金' : 'Principal' }}</th><th>{{ language === 'zh' ? '利息' : 'Interest' }}</th><th>{{ language === 'zh' ? '剩余本金' : 'Balance' }}</th></tr></thead>
        <tbody><tr v-for="row in displayed" :key="row.period" :data-loan-period="row.period"><td>{{ row.period }}</td><td>{{ amount(row.payment) }}</td><td>{{ amount(row.principal) }}</td><td>{{ amount(row.interest) }}</td><td>{{ amount(row.balance) }}</td></tr></tbody>
      </table>
    </div>
    <button v-if="limit < rows.length" type="button" @click="limit += 50">{{ language === 'zh' ? '再显示 50 期' : 'Show 50 more periods' }}</button>
    <p>{{ language === 'zh' ? '完整记录包含所有期数的未舍入明细。' : 'The complete record contains every period at full precision.' }}</p>
  </details>
</template>
<style scoped>
.loan-schedule { margin: 10px 0; color: var(--np-muted); font-size: 11px; }
summary { cursor: pointer; line-height: 1.5; }
.loan-table-scroll { overflow: auto; margin-top: 8px; }
table { border-collapse: collapse; width: 100%; white-space: nowrap; font-variant-numeric: tabular-nums; }
th, td { padding: 5px; text-align: right; border-bottom: 1px solid var(--np-border); }
button { cursor: pointer; margin-top: 8px; padding: 5px 8px; background: var(--np-control-active); color: var(--np-text); border: 1px solid var(--np-border); border-radius: 3px; font: inherit; }
button:focus-visible, summary:focus-visible { outline: 2px solid var(--np-accent); outline-offset: 1px; }
</style>
