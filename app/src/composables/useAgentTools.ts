import { ref } from 'vue'
import { getAgentGuidePath, getAgentToolsStatus, installAgentCli, type AgentToolsStatus } from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'

export function useAgentTools(onCopied: () => void, onOpenGuide: (path: string) => Promise<void>) {
  const agentToolsStatus = ref<AgentToolsStatus | null>(null)
  const agentToolsError = ref<string | null>(null)
  const agentToolsBusy = ref(false)

  async function loadAgentToolsStatus() {
    if (!isTauriRuntime()) return
    try {
      agentToolsError.value = null
      agentToolsStatus.value = await getAgentToolsStatus()
    } catch (error) {
      agentToolsStatus.value = null
      agentToolsError.value = String(error)
    }
  }

  async function repairAgentCli() {
    if (!isTauriRuntime() || agentToolsBusy.value) return
    agentToolsBusy.value = true
    try {
      agentToolsError.value = null
      agentToolsStatus.value = await installAgentCli()
    } catch (error) { agentToolsError.value = String(error) }
    finally { agentToolsBusy.value = false }
  }

  async function copyAgentCliPath() {
    try {
      if (!agentToolsStatus.value) await loadAgentToolsStatus()
      const path = agentToolsStatus.value?.cliPath
      if (!path) throw new Error('NeoPad CLI path is unavailable')
      await navigator.clipboard.writeText(path)
      onCopied()
    } catch (error) { agentToolsError.value = String(error) }
  }

  async function openAgentGuide() {
    if (!isTauriRuntime() || agentToolsBusy.value) return
    agentToolsBusy.value = true
    try {
      agentToolsError.value = null
      await onOpenGuide(await getAgentGuidePath())
    } catch (error) { agentToolsError.value = String(error) }
    finally { agentToolsBusy.value = false }
  }

  return { agentToolsStatus, agentToolsError, agentToolsBusy, loadAgentToolsStatus, repairAgentCli, copyAgentCliPath, openAgentGuide }
}
