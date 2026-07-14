import { ref } from 'vue'
import {
  clearAiApiKey,
  generateAiText,
  getAiConfig,
  listAiPrompts,
  openAiPromptsFolder,
  saveAiApiKey,
  saveAiConfig,
  testAiConnection,
} from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'
import type { AiConfig, AiConversationMessage, AiPromptEntry } from '../types/ai'

const emptyConfig = (): AiConfig => ({
  enabled: false,
  baseUrl: '',
  model: '',
  apiKeyConfigured: false,
})
const AI_UI_TIMEOUT_MS = 50_000

function errorMessage(error: unknown) {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}

export function useAiAssistant() {
  const aiConfig = ref<AiConfig>(emptyConfig())
  const aiError = ref<string | null>(null)
  const aiTesting = ref(false)
  const aiTestSucceeded = ref(false)
  const aiPrompts = ref<AiPromptEntry[]>([])
  const aiPromptsLoading = ref(false)
  let persistTimer: number | null = null

  async function loadAiConfig() {
    if (!isTauriRuntime()) return
    try {
      aiConfig.value = await getAiConfig()
      aiError.value = null
    } catch (error) {
      aiError.value = errorMessage(error)
    }
  }

  function updateAiConfig(patch: Partial<Omit<AiConfig, 'apiKeyConfigured'>>) {
    aiConfig.value = { ...aiConfig.value, ...patch }
    aiTestSucceeded.value = false
    if (!isTauriRuntime()) return
    if (persistTimer) window.clearTimeout(persistTimer)
    persistTimer = window.setTimeout(() => {
      persistTimer = null
      void persistAiConfig()
    }, 300)
  }

  async function persistAiConfig() {
    try {
      await saveAiConfig({
        enabled: aiConfig.value.enabled,
        baseUrl: aiConfig.value.baseUrl,
        model: aiConfig.value.model,
      })
      aiError.value = null
      return true
    } catch (error) {
      aiError.value = errorMessage(error)
      return false
    }
  }

  async function storeApiKey(apiKey: string) {
    try {
      await saveAiApiKey(apiKey)
      aiConfig.value.apiKeyConfigured = true
      aiError.value = null
      aiTestSucceeded.value = false
      return true
    } catch (error) {
      aiError.value = errorMessage(error)
      return false
    }
  }

  async function removeApiKey() {
    try {
      await clearAiApiKey()
      aiConfig.value.apiKeyConfigured = false
      aiError.value = null
      aiTestSucceeded.value = false
      return true
    } catch (error) {
      aiError.value = errorMessage(error)
      return false
    }
  }

  async function checkConnection() {
    aiTesting.value = true
    aiTestSucceeded.value = false
    try {
      if (!(await persistAiConfig())) return false
      await testAiConnection()
      aiError.value = null
      aiTestSucceeded.value = true
      return true
    } catch (error) {
      aiError.value = errorMessage(error)
      return false
    } finally {
      aiTesting.value = false
    }
  }

  async function loadAiPrompts() {
    if (!isTauriRuntime()) return
    aiPromptsLoading.value = true
    try {
      aiPrompts.value = await listAiPrompts()
      aiError.value = null
    } catch (error) {
      aiError.value = errorMessage(error)
    } finally {
      aiPromptsLoading.value = false
    }
  }

  async function revealAiPromptsFolder() {
    try {
      await openAiPromptsFolder()
      return true
    } catch (error) {
      aiError.value = errorMessage(error)
      return false
    }
  }

  async function requestAiText(
    context: string,
    conversation: AiConversationMessage[],
    options: { searchLibrary: boolean; currentNoteId: string; prompt?: string; maxTokens?: number },
  ) {
    let timeoutId = 0
    const timeout = new Promise<never>((_resolve, reject) => {
      timeoutId = window.setTimeout(() => {
        reject(new Error('AI request timed out. Check the provider connection and try again.'))
      }, AI_UI_TIMEOUT_MS)
    })
    try {
      return await Promise.race([generateAiText(context, conversation, options), timeout])
    } finally {
      window.clearTimeout(timeoutId)
    }
  }

  function disposeAiAssistant() {
    if (persistTimer) window.clearTimeout(persistTimer)
  }

  return {
    aiConfig,
    aiError,
    aiTesting,
    aiTestSucceeded,
    aiPrompts,
    aiPromptsLoading,
    loadAiConfig,
    updateAiConfig,
    persistAiConfig,
    storeApiKey,
    removeApiKey,
    checkConnection,
    loadAiPrompts,
    revealAiPromptsFolder,
    requestAiText,
    disposeAiAssistant,
  }
}
