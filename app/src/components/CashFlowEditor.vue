<script setup lang="ts">
import type { CashFlowDraftRow, FinanceError } from '../types/finance'
const props = defineProps<{ rows: CashFlowDraftRow[]; language: 'zh' | 'en'; error: FinanceError | null }>()
const emit = defineEmits<{ change: [rows: CashFlowDraftRow[]] }>()
function update(index: number, field: keyof CashFlowDraftRow, value: string) {
  emit('change', props.rows.map((row, i) => i === index ? { ...row, [field]: value } : row))
}
function add() { if (props.rows.length < 200) emit('change', [...props.rows, { date: '', amount: '', direction: 'in' }]) }
function remove(index: number) { emit('change', props.rows.filter((_, i) => i !== index)) }
const label = (index: number, zh: string, en: string) => props.language === 'zh' ? `第 ${index + 1} 笔${zh}` : `Entry ${index + 1} ${en}`
</script>
<template>
  <div class="cash-flow-editor">
    <div class="flow-head"><span>{{ language === 'zh' ? '实际收付日期' : 'Actual date' }}</span><span>{{ language === 'zh' ? '方向' : 'Direction' }}</span><span>{{ language === 'zh' ? '金额 · 元' : 'Amount · currency units' }}</span><span></span></div>
    <div v-for="(row, index) in rows" :key="index" class="flow-row" :data-cash-flow-row="index + 1" :class="{ invalid: error?.row === index + 1 }">
      <input type="date" :value="row.date" :data-flow-date="index + 1" :aria-label="label(index, '日期', 'date')" :aria-invalid="error?.row === index + 1 && error?.field === 'cashFlowDate'" @input="update(index, 'date', ($event.target as HTMLInputElement).value)">
      <select :value="row.direction" :data-flow-direction="index + 1" :aria-label="label(index, '方向', 'direction')" @change="update(index, 'direction', ($event.target as HTMLSelectElement).value)"><option value="in">{{ language === 'zh' ? '收入 +' : 'In +' }}</option><option value="out">{{ language === 'zh' ? '支出 −' : 'Out −' }}</option></select>
      <input type="text" inputmode="decimal" :value="row.amount" :data-flow-amount="index + 1" :aria-label="label(index, '金额', 'amount')" :aria-invalid="error?.row === index + 1 && error?.field === 'cashFlowAmount'" autocomplete="off" @input="update(index, 'amount', ($event.target as HTMLInputElement).value)">
      <button type="button" :data-flow-remove="index + 1" :disabled="rows.length <= 2" :aria-label="label(index, '删除', 'remove')" @click="remove(index)">×</button>
    </div>
    <button type="button" data-flow-add :disabled="rows.length >= 200" @click="add">{{ language === 'zh' ? '添加一笔' : 'Add entry' }}</button>
  </div>
</template>
<style scoped>
.cash-flow-editor { min-width: 0; }
.flow-row, .flow-head { display: grid; grid-template-columns: minmax(110px, 1.3fr) minmax(68px, .7fr) minmax(65px, 1fr) 25px; gap: 6px; margin-bottom: 7px; align-items: center; }
.flow-head { color: var(--np-muted); font-size: 11px; }
input, select { min-width: 0; width: 100%; box-sizing: border-box; height: 29px; padding: 4px 5px; border: 1px solid var(--np-border); border-radius: 3px; color: var(--np-text); background: var(--np-editor); font: inherit; font-variant-numeric: tabular-nums; }
button { padding: 5px; border: 1px solid var(--np-border); border-radius: 3px; color: var(--np-text); background: var(--np-control-active); font: inherit; cursor: pointer; }
button:disabled { opacity: .5; cursor: default; }
button:focus-visible, input:focus-visible, select:focus-visible { outline: 2px solid var(--np-accent); outline-offset: 1px; }
.invalid input[aria-invalid="true"] { border-color: var(--np-danger); }
@media (max-width: 480px) { .flow-head { display: none; } .flow-row { grid-template-columns: minmax(0, 1fr) 25px; padding-bottom: 8px; border-bottom: 1px solid var(--np-border); } .flow-row input[type="date"] { grid-column: 1; } .flow-row select { grid-column: 1; } .flow-row input[type="text"] { grid-column: 1; } .flow-row button { grid-column: 2; grid-row: 1 / 4; } }
</style>
