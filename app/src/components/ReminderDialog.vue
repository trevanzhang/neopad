<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'

const props = defineProps<{
  title: string
  contentLabel: string
  dateLabel: string
  timeLabel: string
  confirmLabel: string
  cancelLabel: string
}>()

const emit = defineEmits<{
  confirm: [value: { content: string; dueText: string }]
  cancel: []
}>()

const content = ref('')
const date = ref('')
const time = ref('')
const contentInput = ref<HTMLInputElement | null>(null)

onMounted(() => {
  const initial = new Date(Date.now() + 10 * 60 * 1000)
  initial.setSeconds(0, 0)
  date.value = formatDate(initial)
  time.value = formatTime(initial)
  void nextTick(() => contentInput.value?.focus())
})

function submit() {
  const normalizedContent = content.value.trim()
  if (!normalizedContent || !date.value || !time.value) return
  emit('confirm', { content: normalizedContent, dueText: `${date.value} ${time.value}` })
}

function formatDate(value: Date) {
  const pad = (part: number) => String(part).padStart(2, '0')
  return `${value.getFullYear()}-${pad(value.getMonth() + 1)}-${pad(value.getDate())}`
}

function formatTime(value: Date) {
  const pad = (part: number) => String(part).padStart(2, '0')
  return `${pad(value.getHours())}:${pad(value.getMinutes())}`
}
</script>

<template>
  <div class="input-dialog-backdrop" role="presentation" @mousedown.self="$emit('cancel')">
    <form class="input-dialog reminder-dialog" role="dialog" aria-modal="true" :aria-label="title" @submit.prevent="submit">
      <header class="input-dialog-header">
        <strong>{{ title }}</strong>
        <button type="button" :aria-label="cancelLabel" @click="$emit('cancel')"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg></button>
      </header>
      <div class="input-dialog-body reminder-dialog-fields">
        <label>
          <span>{{ contentLabel }}</span>
          <input ref="contentInput" v-model="content" required />
        </label>
        <div class="reminder-date-time">
          <label>
            <span>{{ dateLabel }}</span>
            <input v-model="date" type="date" required />
          </label>
          <label>
            <span>{{ timeLabel }}</span>
            <input v-model="time" type="time" required />
          </label>
        </div>
      </div>
      <footer class="input-dialog-actions">
        <button type="button" @click="$emit('cancel')">{{ cancelLabel }}</button>
        <button type="submit" class="primary" :disabled="!content.trim() || !date || !time">{{ confirmLabel }}</button>
      </footer>
    </form>
  </div>
</template>
