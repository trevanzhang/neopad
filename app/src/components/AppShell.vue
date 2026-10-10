<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { initialStringSetting } from '../lib/preferences'

const props = defineProps<{
  libraryOpen: boolean
  theme: 'light' | 'dark'
  immersive: boolean
  libraryResizeLabel: string
  libraryResizeHint: string
}>()

const defaultLibraryWidth = 168
const widthStorageKey = 'neopad.libraryWidth'
const storedWidth = Number(initialStringSetting(widthStorageKey, String(defaultLibraryWidth)))
const preferredWidth = ref(Number.isFinite(storedWidth) ? Math.min(480, Math.max(defaultLibraryWidth, storedWidth)) : defaultLibraryWidth)
const shellElement = ref<HTMLElement | null>(null)
const shellWidth = ref(typeof window === 'undefined' ? 720 : window.innerWidth)
const maxLibraryWidth = computed(() => Math.max(defaultLibraryWidth, Math.min(480, shellWidth.value - 240)))
const libraryWidth = computed(() => Math.min(maxLibraryWidth.value, preferredWidth.value))
const resizing = ref(false)
let observer: ResizeObserver | null = null
let drag: { id: number; element: HTMLElement; x: number; width: number; previousWidth: number } | null = null

function persistWidth() {
  try { window.localStorage.setItem(widthStorageKey, String(preferredWidth.value)) }
  catch { /* Keep resizing available when local storage is unavailable. */ }
}

function setWidth(width: number) {
  preferredWidth.value = Math.min(maxLibraryWidth.value, Math.max(defaultLibraryWidth, width))
}

function startResize(event: PointerEvent) {
  if (event.button !== 0 || !event.isPrimary) return
  event.preventDefault()
  const element = event.currentTarget as HTMLElement
  element.focus()
  drag = { id: event.pointerId, element, x: event.clientX, width: libraryWidth.value, previousWidth: preferredWidth.value }
  element.setPointerCapture(event.pointerId)
  resizing.value = true
}

function resize(event: PointerEvent) {
  if (drag?.id === event.pointerId) setWidth(Math.round(drag.width + event.clientX - drag.x))
}

function finishResize(event?: PointerEvent) {
  if (!drag || (event && event.pointerId !== drag.id)) return
  const finished = drag
  drag = null
  resizing.value = false
  if (finished.element.hasPointerCapture(finished.id)) finished.element.releasePointerCapture(finished.id)
  persistWidth()
}

function resetWidth() {
  preferredWidth.value = defaultLibraryWidth
  persistWidth()
}

function resizeWithKeyboard(event: KeyboardEvent) {
  if (event.key === 'Escape' && drag) {
    event.preventDefault()
    event.stopPropagation()
    preferredWidth.value = drag.previousWidth
    finishResize()
    return
  }
  if (event.ctrlKey || event.altKey || event.metaKey) return
  if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return
  event.preventDefault()
  event.stopPropagation()
  if (event.key === 'Home') resetWidth()
  else {
    setWidth(event.key === 'End' ? maxLibraryWidth.value : libraryWidth.value + (event.key === 'ArrowRight' ? 16 : -16))
    persistWidth()
  }
}

const finishOnBlur = () => finishResize()
watch(() => props.libraryOpen && !props.immersive, (visible) => { if (!visible) finishResize() })
onMounted(() => {
  if (shellElement.value) {
    shellWidth.value = shellElement.value.clientWidth
    observer = new ResizeObserver(([entry]) => { if (entry) shellWidth.value = entry.contentRect.width })
    observer.observe(shellElement.value)
  }
  window.addEventListener('blur', finishOnBlur)
})
onBeforeUnmount(() => {
  finishResize()
  observer?.disconnect()
  window.removeEventListener('blur', finishOnBlur)
})
</script>

<template>
  <main
    ref="shellElement"
    class="app-shell"
    :class="[`theme-${theme}`, { immersive, 'note-library-open': libraryOpen, 'library-resizing': resizing }]"
    :style="{ '--library-width': `${libraryWidth}px` }"
  >
    <div class="window-chrome">
      <slot name="title" />
    </div>
    <slot name="tabs" />
    <slot name="library" />
    <div
      v-if="libraryOpen && !immersive"
      class="note-library-resizer"
      role="separator"
      tabindex="0"
      aria-orientation="vertical"
      aria-controls="note-library-pane"
      :aria-label="libraryResizeLabel"
      :title="libraryResizeHint"
      :aria-valuemin="defaultLibraryWidth"
      :aria-valuemax="maxLibraryWidth"
      :aria-valuenow="libraryWidth"
      @pointerdown="startResize"
      @pointermove="resize"
      @pointerup="finishResize"
      @pointercancel="finishResize"
      @lostpointercapture="finishResize"
      @keydown="resizeWithKeyboard"
      @dblclick="resetWidth"
    />
    <slot />
    <slot name="status" />
  </main>
</template>
