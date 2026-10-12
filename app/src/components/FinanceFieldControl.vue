<script setup lang="ts">
import type { FinanceField } from '../lib/finance-ui'
defineProps<{ field: FinanceField; value?: string; invalid?: boolean }>()
const emit = defineEmits<{ change: [key: string, value: string] }>()
</script>
<template>
  <label class="finance-field">
    <span>{{ field.label }}<small v-if="field.unit"> · {{ field.unit }}</small></span>
    <select v-if="field.options" :value="value" :data-finance-field="field.key" :aria-invalid="invalid" @change="emit('change', field.key, ($event.target as HTMLSelectElement).value)">
      <option v-for="option in field.options" :key="option.value" :value="option.value">{{ option.label }}</option>
    </select>
    <input v-else :type="field.type ?? 'text'" :inputmode="field.type ? undefined : 'decimal'" :value="value" :data-finance-field="field.key" :aria-invalid="invalid" autocomplete="off" spellcheck="false" @input="emit('change', field.key, ($event.target as HTMLInputElement).value)">
  </label>
</template>
<style scoped>
.finance-field { display: grid; grid-template-columns: minmax(75px, .85fr) minmax(0, 1.15fr); align-items: center; gap: 8px; min-width: 0; }
span { overflow-wrap: anywhere; line-height: 1.35; }
small { display: block; font-size: 10px; color: var(--np-muted); }
input, select { width: 100%; min-width: 0; height: 29px; box-sizing: border-box; padding: 4px 6px; color: var(--np-text); background: var(--np-editor); border: 1px solid var(--np-border); border-radius: 3px; font: inherit; font-variant-numeric: tabular-nums; }
[aria-invalid="true"] { border-color: var(--np-danger); }
input:focus-visible, select:focus-visible { outline: 2px solid var(--np-accent); outline-offset: 1px; }
</style>
