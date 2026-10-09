<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import type { NoteTab } from '../types/note'

const props = defineProps<{
  notes: NoteTab[]
  projects: NoteTab[]
  archivedNotes: NoteTab[]
  archiveDirectories: string[]
  trashedNotes: NoteTab[]
  projectDirectories: string[]
  activeNoteId: string
  loading: boolean
  messages: {
    title: string
    notes: string
    projects: string
    archive: string
    trash: string
    emptyNotes: string
    emptyProjects: string
    emptyArchive: string
    emptyTrash: string
    newNote: string
    newProject: string
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
  restore: [notes: NoteTab[]]
  restoreTrash: [notes: NoteTab[]]
  rename: [notes: NoteTab[]]
  archive: [notes: NoteTab[]]
  delete: [notes: NoteTab[]]
  reveal: [note: NoteTab]
  clearTrash: []
  newNote: []
  newProjectNote: [directory?: string]
  archiveProject: [path: string]
  moveToCapture: [notes: NoteTab[]]
  createDirectory: [kind: 'archive' | 'project', parent?: string]
  archiveTo: [notes: NoteTab[], directory: string]
  moveProjectNotes: [notes: NoteTab[], directory: string]
  moveDirectory: [kind: 'archive' | 'project', path: string, targetParent: string]
  renameDirectory: [kind: 'archive' | 'project', path: string]
  deleteDirectory: [kind: 'archive' | 'project', path: string]
  refresh: []
}>()

const notesExpanded = ref(true)
const projectsExpanded = ref(true)
const archiveExpanded = ref(true)
const trashExpanded = ref(true)
type LibraryGroup = 'notes' | 'projects' | 'archive' | 'trash-notes'
type NoteContextMenu = {
  kind: 'note'
  notes: NoteTab[]
  group: 'notes' | 'projects' | 'archive' | 'trash-notes'
  x: number
  y: number
}
type DirectoryContextMenu = {
  kind: 'directory'
  directoryKind: 'archive' | 'project'
  path: string
  x: number
  y: number
}

const selectedIds = ref(new Set<string>())
const selectionGroup = ref<LibraryGroup | null>(null)
const selectionAnchorId = ref<string | null>(null)
const contextMenu = ref<NoteContextMenu | DirectoryContextMenu | null>(null)
const contextMenuElement = ref<HTMLElement | null>(null)
const expandedArchiveDirectories = ref(new Set<string>())
const expandedProjectDirectories = ref(new Set<string>())
type TreeFolderRow = { kind: 'folder'; path: string; name: string; depth: number }
type TreeFileRow<T> = { kind: 'file'; item: T; depth: number }
type DragPayload =
  | { kind: 'note' | 'archive' | 'project'; ids: string[] }
  | { kind: 'archive-directory' | 'project-directory'; path: string }
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
const projectRows = computed(() => buildTreeRows(
  props.projectDirectories,
  props.projects,
  (note) => note.projectRelativePath ?? note.fileName,
  expandedProjectDirectories.value,
))

function toggleDirectory(kind: 'archive' | 'project', path: string) {
  const current = kind === 'archive' ? expandedArchiveDirectories.value : expandedProjectDirectories.value
  const next = new Set(current)
  if (next.has(path)) next.delete(path)
  else next.add(path)
  if (kind === 'archive') expandedArchiveDirectories.value = next
  else expandedProjectDirectories.value = next
}

function startEntryDrag(event: DragEvent, kind: DragPayload['kind'], id: string, group: LibraryGroup) {
  if (kind === 'archive-directory' || kind === 'project-directory') return
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
  kind: 'archive-directory' | 'project-directory',
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
  if (!payload) return
  if (payload.kind === 'project-directory') emit('archiveProject', payload.path)
  else if (payload.kind === 'archive-directory') {
    if (directory !== payload.path && !directory.startsWith(`${payload.path}/`)) emit('moveDirectory', 'archive', payload.path, directory)
  } else if ('ids' in payload) emit('archiveTo', [...props.notes, ...props.projects, ...props.archivedNotes].filter(note => payload.ids.includes(note.id)), directory)
  endDrag()
}
function dropOnProjects(directory: string) {
  const payload = dragPayload.value
  if (!payload) return
  if (payload.kind === 'project-directory') {
    if (directory !== payload.path && !directory.startsWith(`${payload.path}/`)) emit('moveDirectory', 'project', payload.path, directory)
  } else if ('ids' in payload && directory) emit('moveProjectNotes', [...props.notes, ...props.projects, ...props.archivedNotes].filter(note => payload.ids.includes(note.id)), directory)
  endDrag()
}
function dropOnNotes() {
  const payload = dragPayload.value
  if (payload && 'ids' in payload) emit('moveToCapture', [...props.projects, ...props.archivedNotes].filter(note => payload.ids.includes(note.id)))
  endDrag()
}
function dropOnTrash() {
  const payload = dragPayload.value
  if (!payload) return
  if (payload.kind === 'archive-directory' || payload.kind === 'project-directory') emit('deleteDirectory', payload.kind === 'archive-directory' ? 'archive' : 'project', payload.path)
  else if ('ids' in payload) emit('delete', [...props.notes, ...props.projects, ...props.archivedNotes].filter(note => payload.ids.includes(note.id)))
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

function noteEntries(group: 'notes' | 'projects' | 'archive' | 'trash-notes') {
  if (group === 'notes') return props.notes
  if (group === 'projects') return props.projects
  if (group === 'archive') return props.archivedNotes
  return props.trashedNotes
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

function selectNote(event: MouseEvent, note: NoteTab, group: 'notes' | 'projects' | 'archive' | 'trash-notes') {
  updateSelection(event, note.id, noteEntries(group).map((item) => item.id), group)
  if (group !== 'trash-notes' && !event.shiftKey && !(event.ctrlKey || event.metaKey)) {
    emit('select', note.id)
  }
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
  group: 'notes' | 'projects' | 'archive' | 'trash-notes',
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
    ...menuPosition(event, group === 'notes' || group === 'projects' ? 144 : 96),
  }
  focusContextMenu()
}

function openDirectoryContextMenu(
  event: MouseEvent,
  directoryKind: 'archive' | 'project',
  path: string,
) {
  event.preventDefault()
  contextMenu.value = {
    kind: 'directory',
    directoryKind,
    path,
    ...menuPosition(event, directoryKind === 'project' ? 180 : 120),
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

function runDirectoryContextAction(action: 'rename' | 'delete' | 'archive') {
  const menu = contextMenu.value
  if (!menu || menu.kind !== 'directory') return
  contextMenu.value = null
  if (action === 'rename') emit('renameDirectory', menu.directoryKind, menu.path)
  else if (action === 'archive') emit('archiveProject', menu.path)
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
        <button type="button" class="note-library-new-project" :title="messages.newProject" :aria-label="messages.newProject" @click="$emit('newProjectNote')">
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
            <svg class="note-library-archive-box" aria-hidden="true" viewBox="0 0 16 16"><path d="M1.75 5.25h12.5v8.5H1.75zM1.25 2.5h13.5v2.75H1.25zM6.25 8.75h3.5" /></svg>
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
                'top-level': row.depth === 0,
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

      <section class="note-library-group project-library-group">
        <div
          class="note-library-root note-library-project-root"
          :class="{ 'drop-target': dragTarget === 'project-root' }"
          @dragover.prevent="dragTarget = 'project-root'"
          @dragleave="dragTarget = null"
          @drop.prevent="dropOnProjects('')"
        >
          <button class="note-library-root-toggle" type="button" @click="projectsExpanded = !projectsExpanded">
            <svg class="note-library-chevron" :class="{ collapsed: !projectsExpanded }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
            <span class="note-library-project-folder" aria-hidden="true">▱</span>
            <span>{{ messages.projects }}</span>
          </button>
          <button class="note-library-create-project" type="button" :title="messages.newProject" :aria-label="messages.newProject" @click="$emit('newProjectNote')">
            <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M8 3.25v9.5M3.25 8h9.5" /></svg>
          </button>
          <button class="note-library-create-folder" type="button" :title="messages.newFolder" :aria-label="messages.newFolder" @click="$emit('createDirectory', 'project')">
            <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M2 4.25h4.2l1.3 1.5H14v6.5H2ZM8 7.25v3.5M6.25 9h3.5" /></svg>
          </button>
          <small>{{ projects.length }}</small>
        </div>
        <div v-if="projectsExpanded" class="note-library-entries">
          <p
            v-if="!loading && projects.length === 0 && projectDirectories.length === 0"
            class="note-library-empty"
          >
            {{ messages.emptyProjects }}
          </p>
          <template v-for="row in projectRows" :key="row.kind === 'folder' ? `folder:${row.path}` : row.item.id">
            <button
              v-if="row.kind === 'folder'"
              class="note-library-directory project"
              :class="{
                'top-level': row.depth === 0,
                'drop-target': dragTarget === `project:${row.path}`,
                dragging: dragPayload?.kind === 'project-directory' && dragPayload.path === row.path,
              }"
              :style="{ '--tree-depth': row.depth }"
              type="button"
              draggable="true"
              @click="toggleDirectory('project', row.path)"
              @dblclick.stop="$emit('newProjectNote', row.path)"
              @contextmenu="openDirectoryContextMenu($event, 'project', row.path)"
              @dragstart="startDirectoryDrag($event, 'project-directory', row.path)"
              @dragover.prevent="dragTarget = `project:${row.path}`"
              @dragleave="dragTarget = null"
              @drop.prevent.stop="dropOnProjects(row.path)"
              @dragend="endDrag"
            >
              <svg class="note-library-chevron" :class="{ collapsed: !expandedProjectDirectories.has(row.path) }" aria-hidden="true" viewBox="0 0 16 16"><path d="m4.5 6 3.5 4 3.5-4" /></svg>
              <span class="note-library-project-folder" aria-hidden="true">▱</span>
              <span>{{ row.name }}</span>
            </button>
            <button
              v-else
              class="note-library-entry project tree-entry"
              :class="{ active: row.item.id === activeNoteId, selected: isSelected(row.item.id, 'projects') }"
              :style="{ '--tree-depth': row.depth }"
              type="button"
              draggable="true"
              :title="row.item.projectRelativePath"
              @click="selectNote($event, row.item, 'projects')"
              @contextmenu="openNoteContextMenu($event, row.item, 'projects')"
              @dragstart="startEntryDrag($event, 'project', row.item.id, 'projects')"
              @dragend="endDrag"
            >
            <span class="note-library-project-file" aria-hidden="true">▱</span>
              <span>{{ row.item.title }}</span>
            </button>
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
          <button class="note-library-clear-trash" type="button" :title="messages.clearTrash" :aria-label="messages.clearTrash" :disabled="trashedNotes.length === 0" @click="$emit('clearTrash')">
            <svg aria-hidden="true" viewBox="0 0 16 16"><path d="M9.8 2.7 13 5.9M8.3 4.2l3.2 3.2M3.1 12.9l4.1-4.1 3.2 3.2-1 1Zm.1.1h8.6M5.4 10.6l1.1 1.1" /></svg>
          </button>
          <small>{{ trashedNotes.length }}</small>
        </div>
        <div v-if="trashExpanded" class="note-library-entries">
          <p v-if="!loading && trashedNotes.length === 0" class="note-library-empty">{{ messages.emptyTrash }}</p>
          <button v-for="note in trashedNotes" :key="note.id" class="note-library-entry trashed" :class="{ selected: isSelected(note.id, 'trash-notes') }" type="button" :title="note.fileName" @click="selectNote($event, note, 'trash-notes')" @contextmenu="openNoteContextMenu($event, note, 'trash-notes')">
            <svg class="note-library-file" aria-hidden="true" viewBox="0 0 16 16"><path d="M4 1.75h5l3 3v9.5H4a1 1 0 0 1-1-1v-10.5a1 1 0 0 1 1-1ZM9 1.75v3h3M5.5 8h5M5.5 10.5h5" /></svg>
            <span>{{ note.title }}</span>
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
        <button v-if="contextMenu.group === 'archive' || contextMenu.group === 'trash-notes'" type="button" role="menuitem" @click="runNoteContextAction('restore')">{{ messages.restore }}</button>
        <template v-else>
          <button v-if="contextMenu.notes.length === 1" type="button" role="menuitem" @click="runNoteContextAction('rename')">{{ messages.rename }}</button>
          <button type="button" role="menuitem" @click="runNoteContextAction('archive')">{{ messages.archiveAction }}</button>
          <button class="danger" type="button" role="menuitem" @click="runNoteContextAction('delete')">{{ messages.delete }}</button>
        </template>
      </template>
      <template v-else>
        <button type="button" role="menuitem" @click="$emit('createDirectory', contextMenu.directoryKind, contextMenu.path); closeContextMenu()">{{ messages.newFolder }}</button>
        <button v-if="contextMenu.directoryKind === 'project'" type="button" role="menuitem" @click="$emit('newProjectNote', contextMenu.path); closeContextMenu()">{{ messages.newProject }}</button>
        <button type="button" role="menuitem" @click="runDirectoryContextAction('rename')">{{ messages.rename }}</button>
        <button v-if="contextMenu.directoryKind === 'project'" type="button" role="menuitem" @click="runDirectoryContextAction('archive')">{{ messages.archiveAction }}</button>
        <button class="danger" type="button" role="menuitem" @click="runDirectoryContextAction('delete')">{{ messages.delete }}</button>
      </template>
    </div>
  </aside>
</template>
