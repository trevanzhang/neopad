import type { AppLanguage } from './i18n'
import type { AppTheme } from './invoke'
import {
  isPreviewContentWidth,
  isPreviewFontFamily,
  isPreviewLineHeight,
  isPreviewTheme,
  type PreviewContentWidth,
  type PreviewFontFamily,
  type PreviewLineHeight,
  type PreviewTheme,
} from '../types/editor'

export type TabBarOrientation = 'horizontal' | 'vertical'
export type TitleDoubleClickAction = 'none' | 'delete' | 'rename'

export const legacyDateTimeSeparatorTemplate = "crlf() + chars('-', 29) + ' ' + date() + ' ' + time()"
export const defaultDateTimeSeparatorTemplate = "crlf() + chars('-', 29) + ' ' + date() + ' ' + time() + ' ' + chars('-', 29) + crlf()"

export function initialLanguage(): AppLanguage {
  if (typeof window === 'undefined') return 'zh'
  const stored = window.localStorage.getItem('neopad.language')
  return stored === 'en' ? 'en' : 'zh'
}

export function initialTheme(): AppTheme {
  if (typeof window === 'undefined') return 'light'
  const stored = window.localStorage.getItem('neopad.theme')
  if (stored === 'light' || stored === 'dark') return stored
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function initialPreviewTheme(): PreviewTheme {
  return normalizePreviewTheme(initialStringSetting('neopad.previewTheme', 'light'))
}

export function normalizePreviewTheme(value: string): PreviewTheme {
  if (isPreviewTheme(value)) return value
  if (value === 'neopad' || value === 'paper' || value === 'solomd') return 'light'
  if (value === 'github') return 'githubLight'
  return 'light'
}

export function initialPreviewFontFamily(): PreviewFontFamily {
  const stored = initialStringSetting('neopad.previewFontFamily', 'editor')
  return isPreviewFontFamily(stored) ? stored : 'editor'
}

export function initialPreviewLineHeight(): PreviewLineHeight {
  const stored = initialStringSetting('neopad.previewLineHeight', 'standard')
  return isPreviewLineHeight(stored) ? stored : 'standard'
}

export function initialPreviewContentWidth(): PreviewContentWidth {
  const stored = initialStringSetting('neopad.previewContentWidth', 'standard')
  return isPreviewContentWidth(stored) ? stored : 'standard'
}

export function initialTabBarOrientation(): TabBarOrientation {
  if (typeof window === 'undefined') return 'horizontal'
  return window.localStorage.getItem('neopad.tabBarOrientation') === 'vertical' ? 'vertical' : 'horizontal'
}

export function initialBooleanSetting(key: string, fallback: boolean) {
  if (typeof window === 'undefined') return fallback
  const value = window.localStorage.getItem(key)
  return value === null ? fallback : value === 'true'
}

export function initialStringSetting(key: string, fallback: string) {
  if (typeof window === 'undefined') return fallback
  return window.localStorage.getItem(key) || fallback
}

export function initialNumberSetting(key: string, fallback: number, min: number, max: number) {
  if (typeof window === 'undefined') return fallback
  const value = Number(window.localStorage.getItem(key))
  return Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : fallback
}

export function initialJsonSetting<T>(key: string, fallback: T) {
  if (typeof window === 'undefined') return fallback
  try {
    const stored = window.localStorage.getItem(key)
    return stored ? JSON.parse(stored) as T : fallback
  } catch {
    return fallback
  }
}

export function initialTitleDoubleClickAction(): TitleDoubleClickAction {
  const value = initialStringSetting('neopad.titleDoubleClickAction', 'rename')
  return value === 'none' || value === 'delete' || value === 'rename' ? value : 'rename'
}

export function initialDateTimeSeparatorTemplate() {
  const value = initialStringSetting('neopad.insertDateTimeSeparatorTemplate', defaultDateTimeSeparatorTemplate)
  return value === legacyDateTimeSeparatorTemplate ? defaultDateTimeSeparatorTemplate : value
}
