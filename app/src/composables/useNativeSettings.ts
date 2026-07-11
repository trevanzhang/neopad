import { ref } from 'vue'
import {
  getUiConfig,
  saveUiConfig,
  setAutostart,
  setCloseToMinimize,
  setSnapToEdges,
  setStartHidden,
  setTrayLanguage,
  setWindowOpacity,
  updateClipboardShortcut,
  updateToggleShortcut,
} from '../lib/invoke'
import {
  isEditorMode,
  isPreviewContentWidth,
  isPreviewFontFamily,
  isPreviewLineHeight,
} from '../types/editor'
import { normalizeStoredShortcutKey } from '../lib/shortcut'
import {
  defaultDateTimeSeparatorTemplate,
  legacyDateTimeSeparatorTemplate,
  normalizePreviewTheme,
} from '../lib/preferences'
import { isTauriRuntime } from '../lib/runtime'
import type { PreferenceState } from './usePreferenceState'

interface NativeSettingsOptions {
  preferences: PreferenceState
  onError: () => void
  onOpacityUpdated: () => void
}

export function useNativeSettings(options: NativeSettingsOptions) {
  const p = options.preferences
  const uiConfigLoaded = ref(false)
  let uiConfigTimer: number | null = null
  let nativeSettingsTimer: number | null = null

  async function loadNativeUiConfig() {
    if (!isTauriRuntime()) return
    try {
      const stored = await getUiConfig()
      if (!stored.initialized) {
        uiConfigLoaded.value = true
        persistUiConfig()
        return
      }
      const ui = stored.ui
      p.theme.value = stored.theme === 'dark'
        ? 'dark'
        : stored.theme === 'light'
          ? 'light'
          : window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      const storedMode = isEditorMode(stored.previewMode) ? stored.previewMode : 'edit'
      p.previewMode.value = 'edit'
      p.defaultEditorMode.value = 'edit'
      p.language.value = ui.language === 'zh' ? 'zh' : 'en'
      p.vimMode.value = ui.vimMode
      p.vimUseCtrlShortcuts.value = ui.vimUseCtrlShortcuts
      p.vimInsertExitKey.value = ui.vimInsertExitKey
      p.tabBarOrientation.value = ui.tabBarOrientation === 'vertical' ? 'vertical' : 'horizontal'
      p.wordWrap.value = ui.wordWrap
      p.editorFontFamily.value = ui.editorFontFamily
      p.editorFontSize.value = Math.min(22, Math.max(12, Number(ui.editorFontSize) || 14))
      p.editorBackgroundColor.value = ui.editorBackgroundColor
      p.previewTheme.value = normalizePreviewTheme(ui.previewTheme)
      p.previewFontFamily.value = isPreviewFontFamily(ui.previewFontFamily) ? ui.previewFontFamily : 'editor'
      p.previewFontSize.value = Math.min(22, Math.max(12, Number(ui.previewFontSize) || 14))
      p.previewLineHeight.value = isPreviewLineHeight(ui.previewLineHeight) ? ui.previewLineHeight : 'standard'
      p.previewContentWidth.value = isPreviewContentWidth(ui.previewContentWidth) ? ui.previewContentWidth : 'standard'
      p.windowOpacity.value = Math.min(1, Math.max(0.2, ui.windowOpacity))
      p.runAtStartup.value = ui.runAtStartup
      p.startHidden.value = ui.startHidden
      p.closeToMinimize.value = ui.closeToMinimize
      p.snapToEdges.value = ui.snapToEdges
      p.transparencyEnabled.value = ui.transparencyEnabled
      p.titleDoubleClickAction.value =
        ui.titleDoubleClickAction === 'none' || ui.titleDoubleClickAction === 'delete'
          ? ui.titleDoubleClickAction
          : 'rename'
      p.shortcutBaseKey.value = normalizeStoredShortcutKey(ui.shortcutBaseKey, 'Z')
      p.shortcutModifiers.value = ui.shortcutModifiers
      p.clipboardShortcutBaseKey.value = normalizeStoredShortcutKey(ui.clipboardShortcutBaseKey, 'V')
      p.clipboardShortcutModifiers.value = ui.clipboardShortcutModifiers
      p.insertSeparatorTemplate.value = ui.insertSeparatorTemplate
      p.insertDateTimeTemplate.value = ui.insertDateTimeTemplate
      p.insertDateTimeSeparatorTemplate.value = ui.insertDateTimeSeparatorTemplate === legacyDateTimeSeparatorTemplate
        ? defaultDateTimeSeparatorTemplate
        : ui.insertDateTimeSeparatorTemplate
      p.customInsertTexts.value = ui.customInsertTexts
      const shouldMigrateEditorShortcut = (ui.editorModeShortcut as string) !== 'F4'
      p.editorModeShortcut.value = 'F4'
      uiConfigLoaded.value = true
      if (storedMode !== 'edit' || shouldMigrateEditorShortcut) persistUiConfig()
    } catch {
      options.onError()
    }
  }

  function persistUiConfig() {
    if (uiConfigTimer) window.clearTimeout(uiConfigTimer)
    uiConfigTimer = window.setTimeout(async () => {
      uiConfigTimer = null
      try {
        await saveUiConfig({
          language: p.language.value,
          vimMode: p.vimMode.value,
          vimUseCtrlShortcuts: p.vimUseCtrlShortcuts.value,
          vimInsertExitKey: p.vimInsertExitKey.value,
          tabBarOrientation: p.tabBarOrientation.value,
          wordWrap: p.wordWrap.value,
          editorFontFamily: p.editorFontFamily.value,
          editorFontSize: p.editorFontSize.value,
          editorBackgroundColor: p.editorBackgroundColor.value,
          previewTheme: p.previewTheme.value,
          previewFontFamily: p.previewFontFamily.value,
          previewFontSize: p.previewFontSize.value,
          previewLineHeight: p.previewLineHeight.value,
          previewContentWidth: p.previewContentWidth.value,
          windowOpacity: p.windowOpacity.value,
          runAtStartup: p.runAtStartup.value,
          startHidden: p.startHidden.value,
          closeToMinimize: p.closeToMinimize.value,
          snapToEdges: p.snapToEdges.value,
          transparencyEnabled: p.transparencyEnabled.value,
          titleDoubleClickAction: p.titleDoubleClickAction.value,
          shortcutBaseKey: p.shortcutBaseKey.value,
          shortcutModifiers: p.shortcutModifiers.value,
          clipboardShortcutBaseKey: p.clipboardShortcutBaseKey.value,
          clipboardShortcutModifiers: p.clipboardShortcutModifiers.value,
          insertSeparatorTemplate: p.insertSeparatorTemplate.value,
          insertDateTimeTemplate: p.insertDateTimeTemplate.value,
          insertDateTimeSeparatorTemplate: p.insertDateTimeSeparatorTemplate.value,
          customInsertTexts: p.customInsertTexts.value,
          editorModeShortcut: p.editorModeShortcut.value,
        }, p.defaultEditorMode.value, p.theme.value)
      } catch {
        options.onError()
      }
    }, 150)
  }

  async function runNative(action: () => Promise<unknown>) {
    if (!isTauriRuntime()) return
    try { await action() } catch { options.onError() }
  }

  const syncTrayLanguage = () => runNative(() => setTrayLanguage(p.language.value))
  const syncAutostart = () => runNative(() => setAutostart(p.runAtStartup.value, p.startHidden.value))
  const syncStartHidden = () => uiConfigLoaded.value
    ? runNative(() => setStartHidden(p.startHidden.value))
    : Promise.resolve()
  const syncCloseToMinimize = () => runNative(() => setCloseToMinimize(p.closeToMinimize.value))
  const syncSnapToEdges = () => runNative(() => setSnapToEdges(p.snapToEdges.value))
  const syncToggleShortcut = () => runNative(() => updateToggleShortcut(p.shortcutBaseKey.value, p.shortcutModifiers.value))
  const syncClipboardShortcut = () => runNative(() => updateClipboardShortcut(p.clipboardShortcutBaseKey.value, p.clipboardShortcutModifiers.value))

  async function syncWindowOpacity() {
    if (!isTauriRuntime()) return
    try {
      await setWindowOpacity(p.transparencyEnabled.value ? p.windowOpacity.value : 1)
      options.onOpacityUpdated()
    } catch {
      options.onError()
    }
  }

  async function syncNativeSettings() {
    if (!isTauriRuntime()) return
    await Promise.allSettled([
      syncAutostart(), syncCloseToMinimize(), syncSnapToEdges(), syncWindowOpacity(),
      syncTrayLanguage(), syncToggleShortcut(), syncClipboardShortcut(),
    ])
  }

  function scheduleNativeSettingsSync() {
    nativeSettingsTimer = window.setTimeout(() => {
      nativeSettingsTimer = null
      void syncNativeSettings()
    }, 5000)
  }

  function disposeNativeSettings() {
    if (uiConfigTimer) window.clearTimeout(uiConfigTimer)
    if (nativeSettingsTimer) window.clearTimeout(nativeSettingsTimer)
  }

  return {
    uiConfigLoaded,
    loadNativeUiConfig,
    persistUiConfig,
    syncNativeSettings,
    syncTrayLanguage,
    syncAutostart,
    syncStartHidden,
    syncCloseToMinimize,
    syncSnapToEdges,
    syncWindowOpacity,
    syncToggleShortcut,
    syncClipboardShortcut,
    scheduleNativeSettingsSync,
    disposeNativeSettings,
  }
}
