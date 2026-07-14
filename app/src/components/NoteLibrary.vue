<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import type { AiPromptEntry, AiTrashedPromptEntry } from '../types/ai'
import type { NoteTab } from '../types/note'
import { promptTabId } from '../lib/document-tab'

const props = defineProps<{
  notes: NoteTab[]
  prompts: AiPromptEntry[]
  archivedNotes: NoteTab[]
  archiveDirectories: string[]
  trashedNotes: NoteTab[]
  trashedPrompts: AiTrashedPromptEntry[]
  promptDirectories: string[]
  activeNoteId: string
  loading: boolean
  messages: {
    title: string
    notes: string
    prompts: string
    archive: string
    trash: string
    emptyNotes: string
    emptyPrompts: string
    emptyArchive: string
    emptyTrash: string
    newNote: string
    newPrompt: string
    newFolder: string
    renameFolderTitle: string
    deleteFolderTitle: string
    deleteFolderMessage: string
    deleteFolderConfirm: string
    refresh: string
    restore: string
    rename: string
    duplicate: string
    archiveAction: string
    delete: string
    clearTrash: string
    revealInFileManager: string
    help: string
  }
}>()

const emit = defineEmits<{
  select: [noteId: string]
  selectPrompt: [promptId: string]
  restore: [notes: NoteTab[]]
  restoreTrash: [notes: NoteTab[]]
  restorePrompts: [prompts: AiTrashedPromptEntry[]]
  rename: [notes: NoteTab[]]
  renamePrompt: [prompt: AiPromptEntry]
  duplicatePrompt: [prompt: AiPromptEntry]
  archive: [notes: NoteTab[]]
  delete: [notes: NoteTab[]]
  deletePrompts: [prompts: AiPromptEntry[]]
  reveal: [note: NoteTab]
  revealPrompt: [prompt: AiPromptEntry]
  clearTrash: []
  newNote: []
  newPrompt: [directory?: string]
  createDirectory: [kind: 'archive' | 'prompt', parent?: string]
  archiveTo: [notes: NoteTab[], directory: string]
  movePrompts: [prompts: AiPromptEntry[], directory: string]
  moveDirectory: [kind: 'archive' | 'prompt', path: string, targetParent: string]
  renameDirectory: [kind: 'archive' | 'prompt', path: string]
  deleteDirectory: [kind: 'archive' | 'prompt', path: string]
  refresh: []
}>()

const notesExpanded = ref(true)
const promptsExpanded = ref(true)
const archiveExpanded = ref(true)
const trashExpanded = ref(true)
type LibraryGroup = 'notes' | 'prompts' | 'archive' | 'trash-notes' | 'trash-prompts'
type NoteContextMenu = {
  kind: 'note'
  notes: NoteTab[]
  group: 'notes' | 'archive' | 'trash-notes'
  x: number
  y: number
}
type PromptContextMenu = {
  kind: 'prompt'
  prompts: AiPromptEntry[]
  group: 'prompts'
  x: number
  y: number
}
type TrashedPromptContextMenu = {
  kind: 'trashed-prompt'
  prompts: AiTrashedPromptEntry[]
  group: 'trash-prompts'
  x: number
  y: number
}
type DirectoryContextMenu = {
  kind: 'directory'
  directoryKind: 'archive' | 'prompt'
  path: string
  x: number
  y: number
}

const selectedIds = ref(new Set<string>())
const selectionGroup = ref<LibraryGroup | null>(null)
const selectionAnchorId = ref<string | null>(null)
const contextMenu = ref<NoteContextMenu | PromptContextMenu | TrashedPromptContextMenu | DirectoryContextMenu | null>(null)
const contextMenuElement = ref<HTMLElement | null>(null)
const expandedArchiveDirectories = ref(new Set<string>())
const expandedPromptDirectories = ref(new Set<string>())
type TreeFolderRow = { kind: 'folder'; path: string; name: string; depth: number }
type TreeFileRow<T> = { kind: 'file'; item: T; depth: number }
type DragPayload =
  | { kind: 'note' | 'archive' | 'prompt'; ids: string[] }
  | { kind: 'archive-directory' | 'prompt-directory'; path: string }
const dragPayload = ref<DragPayload | null>(null)
const dragTarget = ref<string | null>(null)

function parentDirectory(path: string) {
  const index = path.lastIndexOf('/')
  return index < 0 ? '' : path.slice(0, index)
}

function baseName(path: string) {
  return path.slice(path.lastIndexOf('/') + 1)
}

function buildTreeRows<T>(
  directories: string[],
  items: T[],
  itemPath: (item: T) => string,
  expanded: Set<string>,
) {
  const rows: Array<TreeFolderRow | TreeFileRow<T>> = []
  const visit = (parent: string, depth: number) => {
    const children = directories
      .filter((path) => parentDirectory(path) === parent)
      .sort((left, right) => left.localeCompare(right))
    for (const path of children) {
      rows.push({ kind: 'folder', path, name: baseName(path), depth })
      if (expanded.has(path)) visit(path, depth + 1)
    }
    const files = items
      .filter((item) => parentDirectory(itemPath(item)) === parent)
      .sort((left, right) => itemPath(left).localeCompare(itemPath(right)))
    for (const item of files) rows.push({ kind: 'file', item, depth })
  }
  visit('', 0)
  return rows
}

const archiveRows = computed(() => buildTreeRows(
  props.archiveDirectories,
  props.archivedNotes,
  (note) => note.archiveRelativePath ?? note.fileName,
  expandedArchiveDirectories.value,
))
const promptRows = computed(() => buildTreeRows(
  props.promptDirectories,
  props.prompts,
  (prompt) => prompt.relativePath,
  expandedPromptDirectories.value,
))

function toggleDirectory(kind: 'archive' | 'prompt', path: string) {
  const current = kind === 'archive' ? expandedArchiveDirectories.value : expandedPromptDirectories.value
  const next = new Set(current)
  if (next.has(path)) next.delete(path)
  else next.add(path)
  if (kind === 'archive') expandedArchiveDirectories.value = next
  else expandedPromptDirectories.value = next
}

function startEntryDrag(event: DragEvent, kind: DragPayload['kind'], id: string, group: LibraryGroup) {
  if (kind === 'archive-directory' || kind === 'prompt-directory') return
  const ids = selectionGroup.value === group && selectedIds.value.has(id)
    ? [...selectedIds.value]
    : [id]
  dragPayload.value = { kind, ids }
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/neopad-library', JSON.stringify(dragPayload.value))
  }
}

function startDirectoryDrag(
  event: DragEvent,
  kind: 'archive-directory' | 'prompt-directory',
  path: string,
) {
  dragPayload.value = { kind, path }
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/neopad-library', JSON.stringify(dragPayload.value))
  }
}

function dropOnArchive(directory: string) {
  const payload = dragPayload.value
  if (!payload || payload.kind === 'prompt' || payload.kind === 'prompt-directory') return endDrag()
  if (payload.kind === 'archive-directory') {
    if (directory === payload.path || directory.startsWith(`${payload.path}/`)) return endDrag()
    emit('moveDirectory', 'archive', payload.path, directory)
    return endDrag()
  }
  if (!('ids' in payload)) return endDrag()
  const source = payload.kind === 'note' ? props.notes : props.archivedNotes
  emit('archiveTo', source.filter((item) => payload.ids.includes(item.id)), directory)
  endDrag()
}

function dropOnPrompts(directory: string) {
  const payload = dragPayload.value
  if (!payload || payload.kind === 'note' || payload.kind === 'archive' || payload.kind === 'archive-directory') return endDrag()
  if (payload.kind === 'prompt-directory') {
    if (directory === payload.path || directory.startsWith(`${payload.path}/`)) return endDrag()
    emit('moveDirectory', 'prompt', payload.path, directory)
    return endDrag()
  }
  if (!('ids' in payload)) return endDrag()
  emit('movePrompts', props.prompts.filter((item) => payload.ids.includes(item.id)), directory)
  endDrag()
}

function dropOnNotes() {
  const payload = dragPayload.value
  if (!payload || payload.kind !== 'archive') return endDrag()
  emit('restore', props.archivedNotes.filter((item) => payload.ids.includes(item.id)))
  endDrag()
}

function dropOnTrash() {
  const payload = dragPayload.value
  if (!payload) return
  if (payload.kind === 'archive-directory' || payload.kind === 'prompt-directory') {
    emit('deleteDirectory', payload.kind === 'archive-directory' ? 'archive' : 'prompt', payload.path)
  } else if (payload.kind === 'prompt') {
    emit('deletePrompts', props.prompts.filter((item) => payload.ids.includes(item.id)))
  } else {
    if (!('ids' in payload)) return endDrag()
    const source = payload.kind === 'note' ? props.notes : props.archivedNotes
    emit('delete', source.filter((item) => payload.ids.includes(item.id)))
  }
  endDrag()
}

function endDrag() {
  dragPayload.value = null
  dragTarget.value = null
}

onMounted(() => {
  window.addEventListener('pointerdown', closeContextMenu)
  window.addEventListener('keydown', closeContextMenuOnEscape)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointerdown', closeContextMenu)
  window.removeEventListener('keydown', closeContextMenuOnEscape)
})

function noteEntries(group: 'notes' | 'archive' | 'trash-notes') {
  if (group === 'notes') return props.notes
  if (group === 'archive') return props.archivedNotes
  return props.trashedNotes
}

function promptEntries(group: 'prompts' | 'trash-prompts') {
  return group === 'prompts' ? props.prompts : props.trashedPrompts
}

function isSelected(id: string, group: LibraryGroup) {
  return selectionGroup.value === group && selectedIds.value.has(id)
}

function updateSelection(event: MouseEvent, id: string, ids: string[], group: LibraryGroup) {
  const next = new Set<string>()
  if (event.shiftKey && selectionGroup.value === group && selectionAnchorId.value) {
    const start = ids.indexOf(selectionAnchorId.value)
    const end = ids.indexOf(id)
    if (start >= 0 && end >= 0) {
      for (const selectedId of ids.slice(Math.min(start, end), Math.max(start, end) + 1)) next.add(selectedId)
    }
  } else if ((event.ctrlKey || event.metaKey) && selectionGroup.value === group) {
    for (const selectedId of selectedIds.value) next.add(selectedId)
    if (next.has(id)) next.delete(id)
    else next.add(id)
  } else {
    next.add(id)
    selectionAnchorId.value = id
  }
  selectedIds.value = next
  selectionGroup.value = group
}

function selectNote(event: MouseEvent, note: NoteTab, group: 'notes' | 'archive' | 'trash-notes') {
  updateSelection(event, note.id, noteEntries(group).map((item) => item.id), group)
  if (group === 'notes' && !event.shiftKey && !(event.ctrlKey || event.metaKey)) emit('select', note.id)
}

function selectPrompt(
  event: MouseEvent,
  prompt: AiPromptEntry | AiTrashedPromptEntry,
  group: 'prompts' | 'trash-prompts',
) {
  updateSelection(event, prompt.id, promptEntries(group).map((item) => item.id), group)
  if (group === 'prompts' && !event.shiftKey && !(event.ctrlKey || event.metaKey)) emit('selectPrompt', prompt.id)
}

function menuPosition(event: MouseEvent, estimatedHeight: number) {
  return {
    x: Math.min(event.clientX, window.innerWidth - 188),
    y: Math.min(event.clientY, window.innerHeight - estimatedHeight),
  }
}

function openNoteContextMenu(
  event: MouseEvent,
  note: NoteTab,
  group: 'notes' | 'archive' | 'trash-notes',
) {
  event.preventDefault()
  if (!isSelected(note.id, group)) {
    selectedIds.value = new Set([note.id])
    selectionGroup.value = group
    selectionAnchorId.value = note.id
  }
  contextMenu.value = {
    kind: 'note',
    notes: noteEntries(group).filter((item) => selectedIds.value.has(item.id)),
    group,
    ...menuPosition(event, group === 'notes' ? 144 : 96),
  }
  focusContextMenu()
}

function openPromptContextMenu(event: MouseEvent, prompt: AiPromptEntry) {
  event.preventDefault()
  if (!isSelected(prompt.id, 'prompts')) {
    selectedIds.value = new Set([prompt.id])
    selectionGroup.value = 'prompts'
    selectionAnchorId.value = prompt.id
  }
  contextMenu.value = {
    kind: 'prompt',
    prompts: props.prompts.filter((item) => selectedIds.value.has(item.id)),
    group: 'prompts',
    ...menuPosition(event, 164),
  }
  focusContextMenu()
}

function openTrashedPromptContextMenu(event: MouseEvent, prompt: AiTrashedPromptEntry) {
  event.preventDefault()
  if (!isSelected(prompt.id, 'trash-prompts')) {
    selectedIds.value = new Set([prompt.id])
    selectionGroup.value = 'trash-prompts'
    selectionAnchorId.value = prompt.id
  }
  contextMenu.value = {
    kind: 'trashed-prompt',
    prompts: props.trashedPrompts.filter((item) => selectedIds.value.has(item.id)),
    group: 'trash-prompts',
    ...menuPosition(event, 64),
  }
  focusContextMenu()
}

function openDirectoryContextMenu(
  event: MouseEvent,
  directoryKind: 'archive' | 'prompt',
  path: string,
) {
  event.preventDefault()
  contextMenu.value = {
    kind: 'directory',
    directoryKind,
    path,
    ...menuPosition(event, 72),
  }
  focusContextMenu()
}

function focusContextMenu() {
  void nextTick(() => contextMenuElement.value?.querySelector<HTMLButtonElement>('button')?.focus())
}

function closeContextMenu(event?: Event) {
  if (event && contextMenuElement.value?.contains(event.target as Node)) return
  contextMenu.value = null
}

function closeContextMenuOnEscape(event: KeyboardEvent) {
  if (event.key !== 'Escape' || !contextMenu.value) return
  event.preventDefault()
  contextMenu.value = null
}

function runNoteContextAction(action: 'rename' | 'archive' | 'delete' | 'restore' | 'reveal') {
  const menu = contextMenu.value
  if (!menu || menu.kind !== 'note') return
  contextMenu.value = null
  if (action === 'reveal') {
    const note = menu.notes[0]
    if (note) emit('reveal', note)
  } else if (action === 'rename') emit('rename', menu.notes)
  else if (action === 'archive') emit('archive', menu.notes)
  else if (action === 'delete') emit('delete', menu.notes)
  else if (menu.group === 'trash-notes') emit('restoreTrash', menu.notes)
  else emit('restore', menu.notes)
}

function runPromptContextAction(action: 'rename' | 'duplicate' | 'delete' | 'restore' | 'reveal') {
  const menu = contextMenu.value
  if (!menu || menu.kind === 'note' || menu.kind === 'directory') return
  contextMenu.value = null
  if (menu.kind === 'trashed-prompt') {
    if (action === 'restore') emit('restorePrompts', menu.prompts)
    return
  }
  if (action === 'rename') {
    const prompt = menu.prompts[0]
    if (prompt) emit('renamePrompt', prompt)
  } else if (action === 'duplicate') {
    const prompt = menu.prompts[0]
    if (prompt) emit('duplicatePrompt', prompt)
  } else if (action === 'delete') emit('deletePrompts', menu.prompts)
  else if (action === 'reveal') {
    const prompt = menu.prompts[0]
    if (prompt) emit('revealPrompt', prompt)
  }
}

function runDirectoryContextAction(action: 'rename' | 'delete') {
  const menu = contextMenu.value
  if (!menu || menu.kind !== 'directory') return
  contextMenu.value = null
  if (action === 'rename') emit('renameDirectory', menu.directoryKind, menu.path)
  else emit('deleteDirectory', menu.directoryKind, menu.path)
}
</script>

<template>
  <aside class="note-library" :aria-label="messages.title">
    <header class="note-library-header">
      <strong>{{ messages.title }}</strong>
      <div class="note-library-help">
        <button type="button" :aria-label="messages.help">
          <svg aria-hidden="true" viewBox="0 0 16 16"><circle cx="8" cy="8" r="5.75" /><path d="M6.7 6.25a1.35 1.35 0 1 1 2.2 1.06c-.83.63-1.27.95-1.27 1.94M8 11.65v.1" /></svg>
        </button>
        <span role="tooltip">{{ messages.help }}</span>
      </div>
      <div class="note-library-actions">
        <button type="button" :title="messages.refresh" :aria-label="messages.refresh" :disabled="loading" @click="$emit('refresh')">
          <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M13.2 7.1A5.4 5.4 0 1 0 13 10.6M13.2 2.8v4.5H8.7" /></svg>
        </button>
        <button type="button" :title="messages.newNote" :aria-label="messages.newNote" @click="$emit('newNote')">
          <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM8 7v4M6 9h4" /></svg>
        </button>
        <button type="button" class="note-library-new-prompt" :title="messages.newPrompt" :aria-label="messages.newPrompt" @click="$emit('newPrompt')">
          <span aria-hidden="true">P+</span>
        </button>
      </div>
    </header>

    <div class="note-library-body" :aria-busy="loading">
      <section class="note-library-group">
        <button
          class="note-library-root"
          :class="{ 'drop-target': dragTarget === 'notes-root' }"
          type="button"
          @click="notesExpanded = !notesExpanded"
          @dragover.prevent="dragTarget = 'notes-root'"
          @dragleave="dragTarget = null"
          @drop.prevent="dropOnNotes"
        >
          <svg class="note-library-chevron" :class="{ collapsed: !notesExpanded }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
          <svg class="note-library-folder" aria-hidden="true" viewBox="0 0 16 16"><path d="M1.5 4.25h4.7l1.3 1.5h7v5.8a1 1 0 0 1-1 1H2.5a1 1 0 0 1-1-1Z" /></svg>
          <span>{{ messages.notes }}</span>
          <small>{{ notes.length }}</small>
        </button>
        <div v-if="notesExpanded" class="note-library-entries">
          <p v-if="!loading && notes.length === 0" class="note-library-empty">{{ messages.emptyNotes }}</p>
          <button
            v-for="note in notes"
            :key="note.id"
            class="note-library-entry"
            :class="{ active: note.id === activeNoteId, selected: isSelected(note.id, 'notes') }"
            type="button"
            draggable="true"
            :title="note.fileName"
            @click="selectNote($event, note, 'notes')"
            @contextmenu="openNoteContextMenu($event, note, 'notes')"
            @dragstart="startEntryDrag($event, 'note', note.id, 'notes')"
            @dragend="endDrag"
          >
            <svg class="note-library-file" aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM9 1.75v3h3M5.5 8h5M5.5 10.5h5" /></svg>
            <span>{{ note.title }}</span>
          </button>
        </div>
      </section>

      <section class="note-library-group">
        <div
          class="note-library-root note-library-archive-root"
          :class="{ 'drop-target': dragTarget === 'archive-root' }"
          @dragover.prevent="dragTarget = 'archive-root'"
          @dragleave="dragTarget = null"
          @drop.prevent="dropOnArchive('')"
        >
          <button class="note-library-root-toggle" type="button" @click="archiveExpanded = !archiveExpanded">
            <svg class="note-library-chevron" :class="{ collapsed: !archiveExpanded }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
            <svg class="note-library-folder archive" aria-hidden="true" viewBox="0 0 16 16"><path d="M1.5 4.25h4.7l1.3 1.5h7v5.8a1 1 0 0 1-1 1H2.5a1 1 0 0 1-1-1Z" /></svg>
            <span>{{ messages.archive }}</span>
          </button>
          <button class="note-library-create-folder" type="button" :title="messages.newFolder" :aria-label="messages.newFolder" @click="$emit('createDirectory', 'archive')">
            <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M2 4.25h4.2l1.3 1.5H14v6.5H2ZM8 7.25v3.5M6.25 9h3.5" /></svg>
          </button>
          <small>{{ archivedNotes.length }}</small>
        </div>
        <div v-if="archiveExpanded" class="note-library-entries">
          <p
            v-if="!loading && archivedNotes.length === 0 && archiveDirectories.length === 0"
            class="note-library-empty"
          >
            {{ messages.emptyArchive }}
          </p>
          <template v-for="row in archiveRows" :key="row.kind === 'folder' ? `folder:${row.path}` : row.item.id">
            <button
              v-if="row.kind === 'folder'"
              class="note-library-directory"
              :class="{
                'drop-target': dragTarget === `archive:${row.path}`,
                dragging: dragPayload?.kind === 'archive-directory' && dragPayload.path === row.path,
              }"
              :style="{ '--tree-depth': row.depth }"
              type="button"
              draggable="true"
              @click="toggleDirectory('archive', row.path)"
              @dblclick.stop="$emit('createDirectory', 'archive', row.path)"
              @contextmenu="openDirectoryContextMenu($event, 'archive', row.path)"
              @dragstart="startDirectoryDrag($event, 'archive-directory', row.path)"
              @dragover.prevent="dragTarget = `archive:${row.path}`"
              @dragleave="dragTarget = null"
              @drop.prevent.stop="dropOnArchive(row.path)"
              @dragend="endDrag"
            >
              <svg class="note-library-chevron" :class="{ collapsed: !expandedArchiveDirectories.has(row.path) }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
              <svg class="note-library-folder archive" aria-hidden="true" viewBox="0 0 16 16"><path d="M1.5 4.25h4.7l1.3 1.5h7v5.8a1 1 0 0 1-1 1H2.5a1 1 0 0 1-1-1Z" /></svg>
              <span>{{ row.name }}</span>
            </button>
            <button
              v-else
              class="note-library-entry archived tree-entry"
              :class="{ selected: isSelected(row.item.id, 'archive') }"
              :style="{ '--tree-depth': row.depth }"
              type="button"
              draggable="true"
              :title="row.item.archiveRelativePath ?? row.item.fileName"
              @click="selectNote($event, row.item, 'archive')"
              @contextmenu="openNoteContextMenu($event, row.item, 'archive')"
              @dragstart="startEntryDrag($event, 'archive', row.item.id, 'archive')"
              @dragend="endDrag"
            >
              <svg class="note-library-file" aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM9 1.75v3h3M5.5 8h5M5.5 10.5h5" /></svg>
              <span>{{ row.item.title }}</span>
            </button>
          </template>
        </div>
      </section>

      <section class="note-library-group prompt-library-group">
        <div
          class="note-library-root note-library-prompt-root"
          :class="{ 'drop-target': dragTarget === 'prompt-root' }"
          @dragover.prevent="dragTarget = 'prompt-root'"
          @dragleave="dragTarget = null"
          @drop.prevent="dropOnPrompts('')"
        >
          <button class="note-library-root-toggle" type="button" @click="promptsExpanded = !promptsExpanded">
            <svg class="note-library-chevron" :class="{ collapsed: !promptsExpanded }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
            <span class="note-library-prompt-folder" aria-hidden="true">P</span>
            <span>{{ messages.prompts }}</span>
          </button>
          <button class="note-library-create-prompt" type="button" :title="messages.newPrompt" :aria-label="messages.newPrompt" @click="$emit('newPrompt')">
            <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M8 3.25v9.5M3.25 8h9.5" /></svg>
          </button>
          <button class="note-library-create-folder" type="button" :title="messages.newFolder" :aria-label="messages.newFolder" @click="$emit('createDirectory', 'prompt')">
            <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M2 4.25h4.2l1.3 1.5H14v6.5H2ZM8 7.25v3.5M6.25 9h3.5" /></svg>
          </button>
          <small>{{ prompts.length }}</small>
        </div>
        <div v-if="promptsExpanded" class="note-library-entries">
          <p
            v-if="!loading && prompts.length === 0 && promptDirectories.length === 0"
            class="note-library-empty"
          >
            {{ messages.emptyPrompts }}
          </p>
          <template v-for="row in promptRows" :key="row.kind === 'folder' ? `folder:${row.path}` : row.item.id">
            <button
              v-if="row.kind === 'folder'"
              class="note-library-directory prompt"
              :class="{
                'drop-target': dragTarget === `prompt:${row.path}`,
                dragging: dragPayload?.kind === 'prompt-directory' && dragPayload.path === row.path,
              }"
              :style="{ '--tree-depth': row.depth }"
              type="button"
              draggable="true"
              @click="toggleDirectory('prompt', row.path)"
              @dblclick.stop="$emit('newPrompt', row.path)"
              @contextmenu="openDirectoryContextMenu($event, 'prompt', row.path)"
              @dragstart="startDirectoryDrag($event, 'prompt-directory', row.path)"
              @dragover.prevent="dragTarget = `prompt:${row.path}`"
              @dragleave="dragTarget = null"
              @drop.prevent.stop="dropOnPrompts(row.path)"
              @dragend="endDrag"
            >
              <svg class="note-library-chevron" :class="{ collapsed: !expandedPromptDirectories.has(row.path) }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
              <span class="note-library-prompt-folder" aria-hidden="true">P</span>
              <span>{{ row.name }}</span>
            </button>
            <template v-else v-for="prompt in [row.item]" :key="row.item.id">
            <button
              class="note-library-entry prompt tree-entry"
              :class="{ active: promptTabId(row.item.id) === activeNoteId, selected: isSelected(row.item.id, 'prompts') }"
              :style="{ '--tree-depth': row.depth }"
              type="button"
              draggable="true"
              :title="row.item.relativePath"
              @click="selectPrompt($event, row.item, 'prompts')"
              @contextmenu="openPromptContextMenu($event, row.item)"
              @dragstart="startEntryDrag($event, 'prompt', row.item.id, 'prompts')"
              @dragend="endDrag"
            >
            <span class="note-library-prompt-file" aria-hidden="true">P</span>
              <span>{{ row.item.name }}</span>
            <small v-if="!prompt.content.trim()" class="note-library-draft-mark">·</small>
          </button>
            </template>
          </template>
        </div>
      </section>

      <section class="note-library-group">
        <div
          class="note-library-root note-library-trash-root"
          :class="{ 'drop-target': dragTarget === 'trash-root' }"
          @dragover.prevent="dragTarget = 'trash-root'"
          @dragleave="dragTarget = null"
          @drop.prevent="dropOnTrash"
        >
          <button class="note-library-root-toggle" type="button" @click="trashExpanded = !trashExpanded">
            <svg class="note-library-chevron" :class="{ collapsed: !trashExpanded }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
            <svg class="note-library-folder trash" aria-hidden="true" viewBox="0 0 16 16"><path d="M3.25 4.5h9.5M6 4.5V3h4v1.5M4.25 4.5l.65 8.75h6.2l.65-8.75M6.5 7v3.75M9.5 7v3.75" /></svg>
            <span>{{ messages.trash }}</span>
          </button>
          <button class="note-library-clear-trash" type="button" :title="messages.clearTrash" :aria-label="messages.clearTrash" :disabled="trashedNotes.length + trashedPrompts.length === 0" @click="$emit('clearTrash')">
            <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M9.8 2.7 13 5.9M8.3 4.2l3.2 3.2M3.1 12.9l4.1-4.1 3.2 3.2-1 1Zm.1.1h8.6M5.4 10.6l1.1 1.1" /></svg>
          </button>
          <small>{{ trashedNotes.length + trashedPrompts.length }}</small>
        </div>
        <div v-if="trashExpanded" class="note-library-entries">
          <p v-if="!loading && trashedNotes.length + trashedPrompts.length === 0" class="note-library-empty">{{ messages.emptyTrash }}</p>
          <button v-for="note in trashedNotes" :key="note.id" class="note-library-entry trashed" :class="{ selected: isSelected(note.id, 'trash-notes') }" type="button" :title="note.fileName" @click="selectNote($event, note, 'trash-notes')" @contextmenu="openNoteContextMenu($event, note, 'trash-notes')">
            <svg class="note-library-file" aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM9 1.75v3h3M5.5 8h5M5.5 10.5h5" /></svg>
            <span>{{ note.title }}</span>
          </button>
          <button v-for="prompt in trashedPrompts" :key="prompt.id" class="note-library-entry trashed prompt" :class="{ selected: isSelected(prompt.id, 'trash-prompts') }" type="button" :title="prompt.originalFileName" @click="selectPrompt($event, prompt, 'trash-prompts')" @contextmenu="openTrashedPromptContextMenu($event, prompt)">
            <span class="note-library-prompt-file" aria-hidden="true">P</span>
            <span>{{ prompt.name }}</span>
          </button>
        </div>
      </section>
    </div>

    <div
      v-if="contextMenu"
      ref="contextMenuElement"
      class="note-library-context-menu"
      role="menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @contextmenu.prevent
    >
      <template v-if="contextMenu.kind === 'note'">
        <button v-if="contextMenu.notes.length === 1" type="button" role="menuitem" @click="runNoteContextAction('reveal')">{{ messages.revealInFileManager }}</button>
        <template v-if="contextMenu.group !== 'notes'">
          <button type="button" role="menuitem" @click="runNoteContextAction('restore')">{{ messages.restore }}</button>
        </template>
        <template v-else>
          <button v-if="contextMenu.notes.length === 1" type="button" role="menuitem" @click="runNoteContextAction('rename')">{{ messages.rename }}</button>
          <button type="button" role="menuitem" @click="runNoteContextAction('archive')">{{ messages.archiveAction }}</button>
          <button class="danger" type="button" role="menuitem" @click="runNoteContextAction('delete')">{{ messages.delete }}</button>
        </template>
      </template>
      <template v-else-if="contextMenu.kind === 'prompt'">
        <button v-if="contextMenu.prompts.length === 1" type="button" role="menuitem" @click="runPromptContextAction('reveal')">{{ messages.revealInFileManager }}</button>
        <button v-if="contextMenu.prompts.length === 1" type="button" role="menuitem" @click="runPromptContextAction('rename')">{{ messages.rename }}</button>
        <button v-if="contextMenu.prompts.length === 1" type="button" role="menuitem" @click="runPromptContextAction('duplicate')">{{ messages.duplicate }}</button>
        <button class="danger" type="button" role="menuitem" @click="runPromptContextAction('delete')">{{ messages.delete }}</button>
      </template>
      <template v-else-if="contextMenu.kind === 'directory'">
        <button type="button" role="menuitem" @click="runDirectoryContextAction('rename')">{{ messages.rename }}</button>
        <button class="danger" type="button" role="menuitem" @click="runDirectoryContextAction('delete')">{{ messages.delete }}</button>
      </template>
      <template v-else>
        <button type="button" role="menuitem" @click="runPromptContextAction('restore')">{{ messages.restore }}</button>
      </template>
    </div>
  </aside>
</template>
