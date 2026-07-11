<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'

defineProps<{
  title: string
  message: string
  confirmLabel: string
  cancelLabel: string
  danger?: boolean
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const closeButton = ref<HTMLButtonElement | null>(null)
const confirmButton = ref<HTMLButtonElement | null>(null)
const cancelButton = ref<HTMLButtonElement | null>(null)

onMounted(() => {
  void nextTick(() => confirmButton.value?.focus())
})

function handleTab(event: KeyboardEvent) {
  const focusableButtons = [closeButton.value, confirmButton.value, cancelButton.value].filter(
    (button): button is HTMLButtonElement => Boolean(button),
  )
  if (focusableButtons.length === 0) return

  const currentIndex = focusableButtons.findIndex((button) => button === document.activeElement)
  const nextIndex = event.shiftKey
    ? (currentIndex - 1 + focusableButtons.length) % focusableButtons.length
    : (currentIndex + 1) % focusableButtons.length
  event.preventDefault()
  focusableButtons[nextIndex]?.focus()
}
</script>

<template>
  <div class="input-dialog-backdrop" role="presentation" @mousedown.self="emit('cancel')">
    <form
      class="input-dialog"
      role="alertdialog"
      aria-modal="true"
      :aria-label="title"
      @submit.prevent="emit('confirm')"
      @keydown.esc.prevent.stop="emit('cancel')"
      @keydown.tab="handleTab"
    >
      <header class="input-dialog-header">
        <strong>{{ title }}</strong>
        <button
          ref="closeButton"
          type="button"
          :aria-label="cancelLabel"
          :title="cancelLabel"
          @click="emit('cancel')"
        ><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg></button>
      </header>
      <div class="input-dialog-body">
        <p class="confirmation-dialog-message">{{ message }}</p>
      </div>
      <footer class="input-dialog-actions">
        <button ref="confirmButton" type="submit" :class="{ primary: !danger, danger }">{{ confirmLabel }}</button>
        <button ref="cancelButton" type="button" @click="emit('cancel')">{{ cancelLabel }}</button>
      </footer>
    </form>
  </div>
</template>
