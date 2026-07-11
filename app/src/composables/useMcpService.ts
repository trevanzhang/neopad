import { ref } from 'vue'
import { getMcpStatus, regenerateMcpToken, setMcpEnabled, type McpStatus } from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'

interface McpServiceOptions {
  stoppedLabel: () => string
  onUpdated: () => void
  onCopied: () => void
  onError: () => void
}

export function useMcpService(options: McpServiceOptions) {
  const mcpStatus = ref<McpStatus | null>(null)
  const mcpUiError = ref<string | null>(null)

  async function loadMcpStatus() {
    if (!isTauriRuntime()) {
      mcpStatus.value = {
        enabled: false,
        running: false,
        status: options.stoppedLabel(),
        url: 'http://127.0.0.1:8765/mcp',
        host: '127.0.0.1',
        port: 8765,
        token: '',
        lastError: null,
      }
      return
    }

    try {
      mcpUiError.value = null
      mcpStatus.value = await getMcpStatus()
    } catch (error) {
      mcpUiError.value = error instanceof Error ? error.message : String(error)
      options.onError()
    }
  }

  async function updateMcpEnabled(enabled: boolean) {
    try {
      mcpUiError.value = null
      mcpStatus.value = await setMcpEnabled(enabled)
      options.onUpdated()
    } catch (error) {
      mcpUiError.value = error instanceof Error ? error.message : String(error)
      await loadMcpStatus()
      options.onError()
    }
  }

  async function refreshMcpToken() {
    try {
      mcpUiError.value = null
      mcpStatus.value = await regenerateMcpToken()
      options.onUpdated()
    } catch (error) {
      mcpUiError.value = error instanceof Error ? error.message : String(error)
      options.onError()
    }
  }

  async function copyMcpConfig() {
    if (!mcpStatus.value) await loadMcpStatus()
    const status = mcpStatus.value
    if (!status) {
      options.onError()
      return
    }
    const config = {
      mcpServers: {
        neopad: {
          url: status.url,
          headers: { Authorization: `Bearer ${status.token}` },
        },
      },
    }
    try {
      await navigator.clipboard.writeText(JSON.stringify(config, null, 2))
      options.onCopied()
    } catch {
      options.onError()
    }
  }

  return {
    mcpStatus,
    mcpUiError,
    loadMcpStatus,
    updateMcpEnabled,
    refreshMcpToken,
    copyMcpConfig,
  }
}
