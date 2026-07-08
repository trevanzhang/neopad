<script setup lang="ts">
import type { Reminder } from '../types/note'

defineProps<{
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
    completeAllDue: string
  }
}>()

defineEmits<{
  close: []
  refresh: []
  select: [reminder: Reminder]
  complete: [reminder: Reminder]
  completeDue: []
}>()
</script>

<template>
  <section class="reminder-list-panel" role="dialog" aria-modal="true" :aria-label="messages.title">
    <header class="reminder-list-header">
      <strong>{{ messages.title }}</strong>
      <div>
        <button
          type="button"
          :disabled="loading || !reminders.some((reminder) => reminder.status === 'due')"
          @click="$emit('completeDue')"
        >
          {{ messages.completeAllDue }}
        </button>
        <button type="button" :disabled="loading" @click="$emit('refresh')">{{ messages.refresh }}</button>
        <button type="button" @click="$emit('close')">{{ messages.close }}</button>
      </div>
    </header>
    <div class="reminder-list-body">
      <p v-if="!loading && reminders.length === 0" class="reminder-list-empty">{{ messages.empty }}</p>
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
            v-for="reminder in reminders"
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
                type="button"
                :disabled="reminder.completed"
                @click.stop="$emit('complete', reminder)"
              >
                {{ reminder.completed ? messages.completed : messages.complete }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>
