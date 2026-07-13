import { ref, watch } from 'vue'
import { normalizeStoredShortcutKey } from '../lib/shortcut'
import {
  initialBooleanSetting,
  initialDateTimeSeparatorTemplate,
  initialJsonSetting,
  initialLanguage,
  initialNumberSetting,
  initialPreviewContentWidth,
  initialPreviewFontFamily,
  initialPreviewLineHeight,
  initialPreviewTheme,
  initialStringSetting,
  initialTabBarOrientation,
  initialTheme,
  initialTitleDoubleClickAction,
  type TabBarOrientation,
  type TitleDoubleClickAction,
} from '../lib/preferences'
import type {
  EditorMode,
  PreviewContentWidth,
  PreviewFontFamily,
  PreviewLineHeight,
  PreviewTheme,
} from '../types/editor'
import type { AppLanguage } from '../lib/i18n'
import type { AppTheme } from '../lib/invoke'

interface PreferenceCallbacks {
  onLanguageChanged: () => void
  onWindowOpacityChanged: () => void
  onAutostartChanged: () => void
  onStartHiddenChanged: () => void
  onCloseToMinimizeChanged: () => void
  onSnapToEdgesChanged: () => void
  onToggleShortcutChanged: () => void
  onClipboardShortcutChanged: () => void
  onPersistRequested: () => void
}

export function usePreferenceState(callbacks: PreferenceCallbacks) {
  const previewMode = ref<EditorMode>('edit')
  const defaultEditorMode = ref<EditorMode>('edit')
  const editorModeShortcut = ref('F5' as const)
  const theme = ref<AppTheme>(initialTheme())
  const language = ref<AppLanguage>(initialLanguage())
  const vimMode = ref(initialBooleanSetting('neopad.vimMode', false))
  const vimUseCtrlShortcuts = ref(initialBooleanSetting('neopad.vimUseCtrlShortcuts', true))
  const vimInsertExitKey = ref(initialStringSetting('neopad.vimInsertExitKey', 'jj'))
  const tabBarOrientation = ref<TabBarOrientation>(initialTabBarOrientation())
  const wordWrap = ref(initialBooleanSetting('neopad.wordWrap', true))
  const editorFontFamily = ref(initialStringSetting('neopad.editorFontFamily', '"Segoe UI", Arial, sans-serif'))
  const editorFontSize = ref(initialNumberSetting('neopad.editorFontSize', 14, 12, 22))
  const editorBackgroundColor = ref(initialStringSetting('neopad.editorBackgroundColor', '#ffffff'))
  const previewTheme = ref<PreviewTheme>(initialPreviewTheme())
  const previewFontFamily = ref<PreviewFontFamily>(initialPreviewFontFamily())
  const previewFontSize = ref(initialNumberSetting('neopad.previewFontSize', 14, 12, 22))
  const previewLineHeight = ref<PreviewLineHeight>(initialPreviewLineHeight())
  const previewContentWidth = ref<PreviewContentWidth>(initialPreviewContentWidth())
  const windowOpacity = ref(Number(initialStringSetting('neopad.windowOpacity', '1')))
  const runAtStartup = ref(initialBooleanSetting('neopad.runAtStartup', false))
  const startHidden = ref(initialBooleanSetting('neopad.startHidden', false))
  const closeToMinimize = ref(initialBooleanSetting('neopad.closeToMinimize', true))
  const snapToEdges = ref(initialBooleanSetting('neopad.snapToEdges', false))
  const transparencyEnabled = ref(initialBooleanSetting('neopad.transparencyEnabled', true))
  const titleDoubleClickAction = ref<TitleDoubleClickAction>(initialTitleDoubleClickAction())
  const shortcutBaseKey = ref(normalizeStoredShortcutKey(initialStringSetting('neopad.shortcutBaseKey', 'Z'), 'Z'))
  const shortcutModifiers = ref<string[]>(initialJsonSetting('neopad.shortcutModifiers', ['Alt']))
  const clipboardShortcutBaseKey = ref(normalizeStoredShortcutKey(initialStringSetting('neopad.clipboardShortcutBaseKey', 'V'), 'V'))
  const clipboardShortcutModifiers = ref<string[]>(initialJsonSetting('neopad.clipboardShortcutModifiers', ['Ctrl', 'Shift']))
  const insertSeparatorTemplate = ref(initialStringSetting('neopad.insertSeparatorTemplate', "crlf() + chars('-', 80) + crlf()"))
  const insertDateTimeTemplate = ref(initialStringSetting('neopad.insertDateTimeTemplate', "date() + ' ' + time()"))
  const insertDateTimeSeparatorTemplate = ref(initialDateTimeSeparatorTemplate())
  const customInsertTexts = ref<string[]>(initialJsonSetting('neopad.customInsertTexts', []))

  const refsByKey: Record<string, { readonly value: unknown }> = {
    'neopad.language': language, 'neopad.tabBarOrientation': tabBarOrientation,
    'neopad.wordWrap': wordWrap, 'neopad.editorFontFamily': editorFontFamily,
    'neopad.editorFontSize': editorFontSize, 'neopad.editorBackgroundColor': editorBackgroundColor,
    'neopad.previewTheme': previewTheme, 'neopad.previewFontFamily': previewFontFamily,
    'neopad.previewFontSize': previewFontSize, 'neopad.previewLineHeight': previewLineHeight,
    'neopad.previewContentWidth': previewContentWidth, 'neopad.theme': theme,
    'neopad.windowOpacity': windowOpacity, 'neopad.runAtStartup': runAtStartup,
    'neopad.startHidden': startHidden, 'neopad.closeToMinimize': closeToMinimize,
    'neopad.snapToEdges': snapToEdges, 'neopad.transparencyEnabled': transparencyEnabled,
    'neopad.titleDoubleClickAction': titleDoubleClickAction, 'neopad.shortcutBaseKey': shortcutBaseKey,
    'neopad.shortcutModifiers': shortcutModifiers, 'neopad.clipboardShortcutBaseKey': clipboardShortcutBaseKey,
    'neopad.clipboardShortcutModifiers': clipboardShortcutModifiers,
    'neopad.insertSeparatorTemplate': insertSeparatorTemplate,
    'neopad.insertDateTimeTemplate': insertDateTimeTemplate,
    'neopad.insertDateTimeSeparatorTemplate': insertDateTimeSeparatorTemplate,
    'neopad.customInsertTexts': customInsertTexts, 'neopad.vimMode': vimMode,
    'neopad.vimUseCtrlShortcuts': vimUseCtrlShortcuts, 'neopad.vimInsertExitKey': vimInsertExitKey,
  }
  for (const [key, preference] of Object.entries(refsByKey)) {
    watch(() => preference.value, () => {
      const value = preference.value
      window.localStorage.setItem(key, Array.isArray(value) ? JSON.stringify(value) : String(value))
    }, { deep: Array.isArray(preference.value) })
  }
  watch(language, callbacks.onLanguageChanged)
  watch(windowOpacity, callbacks.onWindowOpacityChanged)
  watch(transparencyEnabled, callbacks.onWindowOpacityChanged)
  watch(runAtStartup, callbacks.onAutostartChanged)
  watch(startHidden, () => {
    callbacks.onAutostartChanged()
    callbacks.onStartHiddenChanged()
  })
  watch(closeToMinimize, callbacks.onCloseToMinimizeChanged)
  watch(snapToEdges, callbacks.onSnapToEdgesChanged)
  watch(shortcutBaseKey, callbacks.onToggleShortcutChanged)
  watch(shortcutModifiers, callbacks.onToggleShortcutChanged, { deep: true })
  watch(clipboardShortcutBaseKey, callbacks.onClipboardShortcutChanged)
  watch(clipboardShortcutModifiers, callbacks.onClipboardShortcutChanged, { deep: true })

  watch([
    language, vimMode, vimUseCtrlShortcuts, vimInsertExitKey, tabBarOrientation, wordWrap,
    editorFontFamily, editorFontSize, editorBackgroundColor, previewTheme, previewFontFamily,
    previewFontSize, previewLineHeight, previewContentWidth, theme, windowOpacity, runAtStartup,
    startHidden, closeToMinimize, snapToEdges, transparencyEnabled, titleDoubleClickAction,
    shortcutBaseKey, shortcutModifiers, clipboardShortcutBaseKey, clipboardShortcutModifiers,
    insertSeparatorTemplate, insertDateTimeTemplate, insertDateTimeSeparatorTemplate,
    customInsertTexts, defaultEditorMode,
  ], callbacks.onPersistRequested, { deep: true })

  return {
    previewMode, defaultEditorMode, editorModeShortcut, theme, language, vimMode,
    vimUseCtrlShortcuts, vimInsertExitKey, tabBarOrientation, wordWrap, editorFontFamily,
    editorFontSize, editorBackgroundColor, previewTheme, previewFontFamily, previewFontSize,
    previewLineHeight, previewContentWidth, windowOpacity, runAtStartup, startHidden,
    closeToMinimize, snapToEdges, transparencyEnabled, titleDoubleClickAction,
    shortcutBaseKey, shortcutModifiers, clipboardShortcutBaseKey, clipboardShortcutModifiers,
    insertSeparatorTemplate, insertDateTimeTemplate, insertDateTimeSeparatorTemplate,
    customInsertTexts,
  }
}

export type PreferenceState = ReturnType<typeof usePreferenceState>
