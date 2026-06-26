import { invoke } from '@tauri-apps/api/core'
import type { NoteContent, NoteTab, SearchResult, WorkspaceInfo } from '../types/note'

export function getAppVersion(): Promise<string> {
  return invoke('app_version')
}

export function getWorkspace(): Promise<WorkspaceInfo> {
  return invoke('get_workspace_command')
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

export function searchNotes(query: string, limit = 100): Promise<SearchResult[]> {
  return invoke('search_notes_command', { query, limit })
}

export function showWindow(): Promise<void> {
  return invoke('show_window_command')
}

export function hideWindow(): Promise<void> {
  return invoke('hide_window_command')
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
