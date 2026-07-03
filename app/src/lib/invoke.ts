import { invoke } from '@tauri-apps/api/core'
import type { NoteContent, NoteTab, SearchResult, WorkspaceInfo } from '../types/note'
import type { EditorMode, EditorModeShortcut } from '../types/editor'

export interface UiConfig {
  language: string
  vimMode: boolean
  vimUseCtrlShortcuts: boolean
  vimInsertExitKey: string
  tabBarOrientation: string
  wordWrap: boolean
  editorFontFamily: string
  editorBackgroundColor: string
  windowOpacity: number
  runAtStartup: boolean
  startHidden: boolean
  closeToMinimize: boolean
  snapToEdges: boolean
  transparencyEnabled: boolean
  titleDoubleClickAction: string
  shortcutBaseKey: string
  shortcutModifiers: string[]
  clipboardShortcutBaseKey: string
  clipboardShortcutModifiers: string[]
  insertSeparatorTemplate: string
  insertDateTimeTemplate: string
  insertDateTimeSeparatorTemplate: string
  customInsertTexts: string[]
  editorModeShortcut: EditorModeShortcut
}

export type AppTheme = 'light' | 'dark'

export function getAppVersion(): Promise<string> {
  return invoke('app_version')
}

export function getWorkspace(): Promise<WorkspaceInfo> {
  return invoke('get_workspace_command')
}

export function getUiConfig(): Promise<{ initialized: boolean; ui: UiConfig; previewMode: EditorMode; theme: 'system' | AppTheme }> {
  return invoke('get_ui_config_command')
}

export function saveUiConfig(ui: UiConfig, previewMode: EditorMode, theme: AppTheme): Promise<void> {
  return invoke('save_ui_config_command', { ui, previewMode, theme })
}

export function listNotes(): Promise<NoteTab[]> {
  return invoke('list_notes_command')
}

export function readNote(noteId: string): Promise<NoteContent> {
  return invoke('read_note_command', { noteId })
}

export function createNote(title?: string): Promise<NoteContent> {
  return invoke('create_note_command', { title })
}

export function writeNote(noteId: string, content: string): Promise<NoteContent> {
  return invoke('write_note_command', { noteId, content })
}

export function renameNote(noteId: string, title: string): Promise<NoteTab> {
  return invoke('rename_note_command', { noteId, title })
}

export function deleteNote(noteId: string): Promise<NoteTab> {
  return invoke('delete_note_command', { noteId })
}

export function setNoteColor(noteId: string, color: string | null): Promise<NoteTab> {
  return invoke('set_note_color_command', { noteId, color })
}

export function searchNotes(query: string, limit = 100): Promise<SearchResult[]> {
  return invoke('search_notes_command', { query, limit })
}

export function saveMarkdownFile(suggestedFileName: string, content: string): Promise<boolean> {
  return invoke('save_markdown_file_command', { suggestedFileName, content })
}

export function showWindow(): Promise<void> {
  return invoke('show_window_command')
}

export function hideWindow(): Promise<void> {
  return invoke('hide_window_command')
}

export function setAutostart(enabled: boolean, startHidden: boolean): Promise<void> {
  return invoke('set_autostart_command', { enabled, startHidden })
}

export function setStartHidden(enabled: boolean): Promise<void> {
  return invoke('set_start_hidden_command', { enabled })
}

export function completeStartup(): Promise<void> {
  return invoke('complete_startup_command')
}

export function setCloseToMinimize(enabled: boolean): Promise<void> {
  return invoke('set_close_to_minimize_command', { enabled })
}

export function setSnapToEdges(enabled: boolean): Promise<void> {
  return invoke('set_snap_to_edges_command', { enabled })
}

export function setWindowOpacity(opacity: number): Promise<void> {
  return invoke('set_window_opacity_command', { opacity })
}

export function updateToggleShortcut(baseKey: string, modifiers: string[]): Promise<void> {
  return invoke('update_toggle_shortcut_command', { baseKey, modifiers })
}

export function updateClipboardShortcut(baseKey: string, modifiers: string[]): Promise<void> {
  return invoke('update_clipboard_shortcut_command', { baseKey, modifiers })
}

export function toggleMainWindowMaximize(): Promise<void> {
  return invoke('toggle_main_window_maximize_command')
}

export function openTrash(): Promise<void> {
  return invoke('open_trash_command')
}

export function quitApp(): Promise<void> {
  return invoke('quit_app_command')
}

export function toggleWindow(): Promise<void> {
  return invoke('toggle_window_command')
}

export function toggleAlwaysOnTop(): Promise<boolean> {
  return invoke('toggle_always_on_top_command')
}

export function saveClipboard(): Promise<NoteContent> {
  return invoke('save_clipboard_command')
}

export function getShortcutWarnings(): Promise<string[]> {
  return invoke('get_shortcut_warnings_command')
}

export function setTrayLanguage(language: 'en' | 'zh'): Promise<void> {
  return invoke('set_tray_language_command', { language })
}
