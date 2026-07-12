import type { Ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { hideWindow, quitApp } from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'

interface WindowLifecycleOptions {
  closeToMinimize: Ref<boolean>
  createLocalTab: () => void | Promise<void>
  saveCurrentClipboard: () => void | Promise<void>
  openExternalDocuments: (paths: string[]) => void | Promise<void>
  openPendingExternalDocuments: () => void | Promise<void>
  openSettings: () => void
  saveBeforeWindowAction: () => Promise<boolean>
  onError: () => void
}

export function useWindowLifecycle(options: WindowLifecycleOptions) {
  const unlisteners: UnlistenFn[] = []

  async function registerNativeEventListeners() {
    try {
      const listeners = await Promise.all([
        listen('neopad://new-note-requested', () => void options.createLocalTab()),
        listen('neopad://save-clipboard-requested', () => void options.saveCurrentClipboard()),
        listen('neopad://open-settings', options.openSettings),
        listen('neopad://external-markdown-open-requested', () => {
          void options.openPendingExternalDocuments()
        }),
        listen('neopad://close-requested', () => void handleCloseRequested()),
        listen('neopad://hide-requested', () => void handleHideRequested()),
        listen('neopad://quit-requested', () => void handleQuitRequested()),
        getCurrentWebview().onDragDropEvent((event) => {
          if (event.payload.type === 'drop') {
            void options.openExternalDocuments(event.payload.paths)
          }
        }),
      ])
      unlisteners.push(...listeners)
    } catch {
      options.onError()
    }
  }

  async function resetWebviewZoom() {
    try { await getCurrentWebview().setZoom(1) } catch { options.onError() }
  }

  async function hideMainWindow() {
    if (!isTauriRuntime() || !(await options.saveBeforeWindowAction())) return
    await hideWindow()
  }

  async function exitApp() {
    if (!isTauriRuntime() || !(await options.saveBeforeWindowAction())) return
    await quitApp()
  }

  async function handleCloseRequested() {
    if (options.closeToMinimize.value) await handleHideRequested()
    else await handleQuitRequested()
  }

  async function handleHideRequested() {
    if (!isTauriRuntime() || !(await options.saveBeforeWindowAction())) return
    await hideWindow()
  }

  async function handleQuitRequested() {
    if (!(await options.saveBeforeWindowAction())) return
    if (isTauriRuntime()) await quitApp()
  }

  function disposeWindowLifecycle() {
    for (const unlisten of unlisteners.splice(0)) void unlisten()
  }

  return {
    registerNativeEventListeners,
    resetWebviewZoom,
    hideMainWindow,
    exitApp,
    disposeWindowLifecycle,
  }
}
