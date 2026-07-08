<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Reminder } from '../types/note'

type ReminderFilter = 'all' | 'pending' | 'due' | 'completed'

const props = defineProps<{
  reminders: Reminder[]
  loading: boolean
  messages: {
    title: string
    close: string
    refresh: string
    empty: string
    status: string
    dueAt: string
    content: string
    page: string
    pending: string
    due: string
    completed: string
    actions: string
    complete: string
    reopen: string
    completeAllDue: string
    filterLabel: string
    filterAll: string
    filterPending: string
    filterDue: string
    filterCompleted: string
  }
}>()

const emit = defineEmits<{
  close: []
  refresh: []
  select: [reminder: Reminder]
  complete: [reminder: Reminder]
  reopen: [reminder: Reminder]
  completeDue: []
}>()

const filter = ref<ReminderFilter>('all')

const filteredReminders = computed(() => {
  if (filter.value === 'all') return props.reminders
  return props.reminders.filter((reminder) => reminder.status === filter.value)
})

function toggleReminderCompletion(reminder: Reminder) {
  if (reminder.completed) emit('reopen', reminder)
  else emit('complete', reminder)
}
</script>

<template>
  <section class="reminder-list-panel" role="dialog" aria-modal="true" :aria-label="messages.title">
    <header class="reminder-list-header">
      <strong>{{ messages.title }}</strong>
      <div class="reminder-toolbar">
        <label class="reminder-filter">
          <span>{{ messages.filterLabel }}</span>
          <select v-model="filter" :disabled="loading">
            <option value="all">{{ messages.filterAll }}</option>
            <option value="pending">{{ messages.filterPending }}</option>
            <option value="due">{{ messages.filterDue }}</option>
            <option value="completed">{{ messages.filterCompleted }}</option>
          </select>
        </label>
        <button
          class="reminder-button"
          type="button"
          :disabled="loading || !reminders.some((reminder) => reminder.status === 'due')"
          @click="$emit('completeDue')"
        >
          {{ messages.completeAllDue }}
        </button>
        <button class="reminder-button" type="button" :disabled="loading" @click="$emit('refresh')">
          {{ messages.refresh }}
        </button>
        <button class="reminder-button" type="button" @click="$emit('close')">{{ messages.close }}</button>
      </div>
    </header>
    <div class="reminder-list-body">
      <p v-if="!loading && filteredReminders.length === 0" class="reminder-list-empty">{{ messages.empty }}</p>
      <table v-else class="reminder-table">
        <thead>
          <tr>
            <th>{{ messages.status }}</th>
            <th>{{ messages.dueAt }}</th>
            <th>{{ messages.content }}</th>
            <th>{{ messages.page }}</th>
            <th>{{ messages.actions }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="reminder in filteredReminders"
            :key="`${reminder.id}-${reminder.lineNumber}`"
            tabindex="0"
            @click="$emit('select', reminder)"
            @keydown.enter="$emit('select', reminder)"
          >
            <td :class="`reminder-status-${reminder.status}`">{{ messages[reminder.status] }}</td>
            <td>{{ reminder.dueText }}</td>
            <td>{{ reminder.content }}</td>
            <td>{{ reminder.title }}</td>
            <td class="reminder-actions">
              <button
                :class="['reminder-action-button', { secondary: reminder.completed }]"
                type="button"
                @click.stop="toggleReminderCompletion(reminder)"
              >
                {{ reminder.completed ? messages.reopen : messages.complete }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>
