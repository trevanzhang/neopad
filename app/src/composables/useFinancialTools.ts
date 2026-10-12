import { computed, reactive, ref, watch } from 'vue'
import { calculateFinance } from '../lib/invoke'
import { buildFinanceRequest, financeRecord, financeResultText, initialFinanceDrafts } from '../lib/finance-ui'
import type { FinanceCategory, FinanceDraft, FinanceError, FinanceRequest, FinanceResponse } from '../types/finance'

export function useFinancialTools(calculate: (request: FinanceRequest) => Promise<FinanceResponse> = calculateFinance) {
  const category = ref<FinanceCategory>('bond')
  const drafts = reactive(initialFinanceDrafts())
  const draft = computed(() => drafts[category.value])
  const result = ref<FinanceResponse | null>(null)
  const error = ref<FinanceError | null>(null)
  const busy = ref(false)
  const changed = ref(false)
  let generation = 0
  let calculatedDraft: FinanceDraft | null = null
  let calculatedCategory: FinanceCategory = 'interest'

  function invalidate() {
    generation += 1
    changed.value = result.value !== null || busy.value || changed.value
    result.value = null
    error.value = null
    busy.value = false
    calculatedDraft = null
  }
  // Sync invalidation prevents copying the old result even within the same event.
  watch([category, drafts], invalidate, { deep: true, flush: 'sync' })

  async function compute() {
    invalidate()
    const requestGeneration = generation
    const snapshot = { ...draft.value }
    const tool = category.value
    busy.value = true
    try {
      const response = await calculate(buildFinanceRequest(tool, snapshot))
      if (requestGeneration !== generation) return
      result.value = response
      calculatedDraft = snapshot
      calculatedCategory = tool
      changed.value = false
    } catch (reason) {
      if (requestGeneration !== generation) return
      const candidate = reason as Partial<FinanceError> | null
      error.value = candidate && typeof candidate.code === 'string' && typeof candidate.field === 'string'
        ? { code: candidate.code, field: candidate.field, ...(Number.isInteger(candidate.row) ? { row: candidate.row } : {}) } : { code: 'unknown', field: 'result' }
    } finally {
      if (requestGeneration === generation) busy.value = false
    }
  }

  function reset() {
    Object.assign(drafts[category.value], initialFinanceDrafts()[category.value])
    invalidate()
    changed.value = false
  }

  function record(language: 'zh' | 'en') {
    return result.value && calculatedDraft ? financeRecord(calculatedCategory, calculatedDraft, result.value, language) : null
  }

  function summary(language: 'zh' | 'en') {
    return result.value ? financeResultText(result.value, language) : null
  }

  return { category, drafts, draft, result, error, busy, changed, compute, reset, invalidate, record, summary }
}
