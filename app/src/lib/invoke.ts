import { invoke } from '@tauri-apps/api/core'
import { Image, transformImage } from '@tauri-apps/api/image'
import type { ExternalDocument, NoteContent, NoteTab, Reminder, SearchResult, WorkspaceInfo } from '../types/note'
import type {
  EditorMode,
  EditorModeShortcut,
  PreviewContentWidth,
  PreviewFontFamily,
  PreviewLineHeight,
  PreviewTheme,
} from '../types/editor'
import type {
  AiConfig,
  AiConversationMessage,
  AiGenerateResponse,
} from '../types/ai'

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

export interface AgentToolsStatus {
  cliPath: string
  version: string
  installed: boolean
  matchesBundle: boolean
  guidePath: string
  guideAvailable: boolean
}

export interface RecoverableNoteWrite {
  recoveryFileName: string
  targetFileName: string
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

export function getAiConfig(): Promise<AiConfig> {
  return invoke('get_ai_config_command')
}

export function saveAiConfig(config: Omit<AiConfig, 'apiKeyConfigured'>): Promise<void> {
  return invoke('save_ai_config_command', { config })
}

export function saveAiApiKey(apiKey: string): Promise<void> {
  return invoke('save_ai_api_key_command', { apiKey })
}

export function clearAiApiKey(): Promise<void> {
  return invoke('clear_ai_api_key_command')
}

export function testAiConnection(): Promise<void> {
  return invoke('test_ai_connection_command')
}

export function generateAiText(
  context: string,
  messages: AiConversationMessage[],
  options: { searchLibrary: boolean; currentNoteId: string; maxTokens?: number },
): Promise<AiGenerateResponse> {
  return invoke('generate_ai_text_command', {
    request: {
      context,
      messages,
      searchLibrary: options.searchLibrary,
      currentNoteId: options.currentNoteId,
      maxTokens: options.maxTokens,
    },
  })
}

export function listNotes(): Promise<NoteTab[]> {
  return invoke('list_notes_command')
}

export function listLibraryNotes(): Promise<NoteTab[]> {
  return invoke('list_library_notes_command')
}

export function listArchivedNotes(): Promise<NoteTab[]> {
  return invoke('list_archived_notes_command')
}

export function listArchiveDirectories(): Promise<string[]> {
  return invoke('list_archive_directories_command')
}

export function createArchiveDirectory(relativePath: string): Promise<string> {
  return invoke('create_archive_directory_command', { relativePath })
}

export function moveArchiveDirectory(relativePath: string, targetParent: string): Promise<string> {
  return invoke('move_archive_directory_command', { relativePath, targetParent })
}

export function renameArchiveDirectory(relativePath: string, newName: string): Promise<string> {
  return invoke('rename_archive_directory_command', { relativePath, newName })
}

export function deleteArchiveDirectory(relativePath: string): Promise<number> {
  return invoke('delete_archive_directory_command', { relativePath })
}

export function listTrashedNotes(): Promise<NoteTab[]> {
  return invoke('list_trashed_notes_command')
}

export function readNote(noteId: string): Promise<NoteContent> {
  return invoke('read_note_command', { noteId })
}

export function createNote(title?: string): Promise<NoteContent> {
  return invoke('create_note_command', { title })
}

export function createNoteWithBody(body: string): Promise<NoteContent> {
  return invoke('create_note_with_body_command', { body })
}

export function importNeoCaptureClipboard(): Promise<NoteContent> {
  return invoke('import_neocapture_clipboard_command')
}

export function writeNote(noteId: string, content: string, expectedUpdatedAt: number): Promise<NoteContent> {
  return invoke('write_note_command', { noteId, content, expectedUpdatedAt })
}

export function renameNote(noteId: string, title: string): Promise<NoteTab> {
  return invoke('rename_note_command', { noteId, title })
}

export function renameNoteWithHeading(noteId: string, title: string): Promise<NoteTab> {
  return invoke('rename_note_with_heading_command', { noteId, title })
}

export function deleteNote(noteId: string): Promise<NoteTab> {
  return invoke('delete_note_command', { noteId })
}

export function restoreNoteFromTrash(noteId: string): Promise<NoteTab> {
  return invoke('restore_note_from_trash_command', { noteId })
}

export function clearTrash(): Promise<void> {
  return invoke('clear_trash_command')
}

export function listRecoverableNoteWrites(): Promise<RecoverableNoteWrite[]> {
  return invoke('list_recoverable_note_writes_command')
}

export function restoreRecoverableNoteWrite(recoveryFileName: string): Promise<string> {
  return invoke('restore_recoverable_note_write_command', { recoveryFileName })
}

export function closeNote(noteId: string): Promise<NoteTab> {
  return invoke('close_note_command', { noteId })
}

export function openNote(noteId: string): Promise<NoteTab> {
  return invoke('open_note_command', { noteId })
}

export function archiveNote(noteId: string): Promise<NoteTab> {
  return invoke('archive_note_command', { noteId })
}

export function archiveNoteToDirectory(noteId: string, directory: string): Promise<NoteTab> {
  return invoke('archive_note_to_directory_command', { noteId, directory })
}

export function moveArchivedNote(noteId: string, directory: string): Promise<NoteTab> {
  return invoke('move_archived_note_command', { noteId, directory })
}

export function reorderOpenNotes(orderedNoteIds: string[]): Promise<void> {
  return invoke('reorder_open_notes_command', { orderedNoteIds })
}

export function unarchiveNote(noteId: string): Promise<NoteTab> {
  return invoke('unarchive_note_command', { noteId })
}

export function listRecentNotes(): Promise<NoteTab[]> {
  return invoke('list_recent_notes_command')
}

export function openExternalMarkdown(): Promise<ExternalDocument | null> {
  return invoke('open_external_markdown_command')
}

export function openExternalMarkdownPaths(paths: string[]): Promise<ExternalDocument[]> {
  return invoke('open_external_markdown_paths_command', { paths })
}

export function takePendingExternalMarkdownPaths(): Promise<string[]> {
  return invoke('take_pending_external_markdown_paths_command')
}

export function readExternalMarkdown(path: string): Promise<ExternalDocument> {
  return invoke('read_external_markdown_command', { path })
}

export function writeExternalMarkdown(path: string, content: string, expectedRevision: string): Promise<ExternalDocument> {
  return invoke('write_external_markdown_command', { path, content, expectedRevision })
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

export function saveNoteExport(
  suggestedFileName: string,
  format: 'png' | 'pdf',
  data: Uint8Array,
): Promise<boolean> {
  return invoke('save_note_export_command', {
    suggestedFileName,
    format,
    data: Array.from(data),
  })
}

export async function copyPngToClipboard(data: Uint8Array): Promise<void> {
  const image = await Image.fromBytes(data)
  try {
    await invoke('plugin:clipboard-manager|write_image', {
      image: transformImage(image),
    })
  } finally {
    await image.close()
  }
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

export function openWorkspaceInFileManager(): Promise<void> {
  return invoke('open_workspace_in_file_manager_command')
}

export function revealNoteInFileManager(noteId: string): Promise<void> {
  return invoke('reveal_note_in_file_manager_command', { noteId })
}

export function revealExternalMarkdownInFileManager(path: string): Promise<void> {
  return invoke('reveal_external_markdown_in_file_manager_command', { path })
}

export function copyNotePathToClipboard(noteId: string): Promise<void> {
  return invoke('copy_note_path_to_clipboard_command', { noteId })
}

export function copyExternalMarkdownPathToClipboard(path: string): Promise<void> {
  return invoke('copy_external_markdown_path_to_clipboard_command', { path })
}

export function openExternalUrl(url: string): Promise<void> {
  return invoke('open_external_url_command', { url })
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

export function getAgentToolsStatus(): Promise<AgentToolsStatus> {
  return invoke('get_agent_tools_status_command')
}

export function installAgentCli(): Promise<AgentToolsStatus> {
  return invoke('install_agent_cli_command')
}

export function getAgentGuidePath(): Promise<string> {
  return invoke('get_agent_guide_path_command')
}

export function listProjectDirectories(): Promise<string[]> {
  return invoke('list_project_directories_command')
}

export function createProjectDirectory(relativePath: string): Promise<string> {
  return invoke('create_project_directory_command', { relativePath })
}

export function moveProjectDirectory(relativePath: string, targetParent: string): Promise<string> {
  return invoke('move_project_directory_command', { relativePath, targetParent })
}

export function renameProjectDirectory(relativePath: string, newName: string): Promise<string> {
  return invoke('rename_project_directory_command', { relativePath, newName })
}

export function archiveProjectDirectory(relativePath: string): Promise<string> {
  return invoke('archive_project_directory_command', { relativePath })
}

export function deleteProjectDirectory(relativePath: string): Promise<number> {
  return invoke('delete_project_directory_command', { relativePath })
}

export function moveNoteToProject(noteId: string, directory: string): Promise<NoteTab> {
  return invoke('move_note_to_project_command', { noteId, directory })
}

export function moveNoteToCapture(noteId: string): Promise<NoteTab> {
  return invoke('move_note_to_capture_command', { noteId })
}

export function createProjectNote(directory: string, title?: string): Promise<NoteContent> {
  return invoke('create_project_note_command', { directory, title })
}
