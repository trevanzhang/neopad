import { ref } from 'vue'

type InputDialogState = { title: string; initialValue: string }
type ConfirmationDialogState = { title: string; message: string; confirmLabel: string; danger: boolean }

export function useDialogs() {
  const inputDialog = ref<InputDialogState | null>(null)
  const confirmationDialog = ref<ConfirmationDialogState | null>(null)
  let resolveInputDialog: ((value: string | null) => void) | null = null
  let resolveConfirmationDialog: ((confirmed: boolean) => void) | null = null

  function requestInput(title: string, initialValue: string) {
    resolveInputDialog?.(null)
    inputDialog.value = { title, initialValue }
    return new Promise<string | null>((resolve) => {
      resolveInputDialog = resolve
    })
  }

  function finishInputDialog(value: string | null) {
    const resolve = resolveInputDialog
    resolveInputDialog = null
    inputDialog.value = null
    resolve?.(value)
  }

  function requestConfirmation(title: string, message: string, confirmLabel: string, danger = false) {
    resolveConfirmationDialog?.(false)
    confirmationDialog.value = { title, message, confirmLabel, danger }
    return new Promise<boolean>((resolve) => {
      resolveConfirmationDialog = resolve
    })
  }

  function finishConfirmationDialog(confirmed: boolean) {
    const resolve = resolveConfirmationDialog
    resolveConfirmationDialog = null
    confirmationDialog.value = null
    resolve?.(confirmed)
  }

  return {
    inputDialog,
    confirmationDialog,
    requestInput,
    finishInputDialog,
    requestConfirmation,
    finishConfirmationDialog,
  }
}
