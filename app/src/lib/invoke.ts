import { invoke } from '@tauri-apps/api/core'
import type { NoteContent, NoteTab, Reminder, SearchResult, WorkspaceInfo } from '../types/note'
import type {
  EditorMode,
  EditorModeShortcut,
  PreviewContentWidth,
  PreviewFontFamily,
  PreviewLineHeight,
  PreviewTheme,
} from '../types/editor'

export interface UiConfig {
  language: string
  vimMode: boolean
  vimUseCtrlShortcuts: boolean
  vimInsertExitKey: string
  tabBarOrientation: string
  wordWrap: boolean
  editorFontFamily: string
  editorFontSize: number
  editorBackgroundColor: string
  previewTheme: PreviewTheme
  previewFontFamily: PreviewFontFamily
  previewFontSize: number
  previewLineHeight: PreviewLineHeight
  previewContentWidth: PreviewContentWidth
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

export interface McpStatus {
  enabled: boolean
  running: boolean
  status: string
  url: string
  host: string
  port: number
  token: string
  lastError: string | null
}

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

export function writeNote(noteId: string, content: string, expectedUpdatedAt: number): Promise<NoteContent> {
  return invoke('write_note_command', { noteId, content, expectedUpdatedAt })
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

export function listReminders(): Promise<Reminder[]> {
  return invoke('list_reminders_command')
}

export function claimDueReminders(): Promise<Reminder[]> {
  return invoke('claim_due_reminders_command')
}

export function completeReminder(reminder: Reminder): Promise<void> {
  return invoke('complete_reminder_command', {
    noteId: reminder.noteId,
    lineNumber: reminder.lineNumber,
    reminderId: reminder.id,
  })
}

export function reopenReminder(reminder: Reminder): Promise<void> {
  return invoke('reopen_reminder_command', {
    noteId: reminder.noteId,
    lineNumber: reminder.lineNumber,
    reminderId: reminder.id,
  })
}

export function completeDueReminders(): Promise<number> {
  return invoke('complete_due_reminders_command')
}

export function saveMarkdownFile(suggestedFileName: string, content: string): Promise<boolean> {
  return invoke('save_markdown_file_command', { suggestedFileName, content })
}

export function exportAllNotesZip(): Promise<boolean> {
  return invoke('export_all_notes_zip_command')
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

export function getMcpStatus(): Promise<McpStatus> {
  return invoke('get_mcp_status_command')
}

export function setMcpEnabled(enabled: boolean): Promise<McpStatus> {
  return invoke('set_mcp_enabled_command', { enabled })
}

export function regenerateMcpToken(): Promise<McpStatus> {
  return invoke('regenerate_mcp_token_command')
}
