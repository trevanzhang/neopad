import { ref, type Ref } from 'vue'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import {
  claimDueReminders,
  completeDueReminders,
  completeReminder,
  listReminders,
  reopenReminder,
} from '../lib/invoke'
import { isTauriRuntime } from '../lib/runtime'
import type { Reminder } from '../types/note'

interface ReminderStateOptions {
  activeTabId: Ref<string>
  forceSave: () => Promise<boolean>
  loadActiveNote: () => Promise<boolean>
  notificationTitle: () => string
  onError: () => void
}

export function useReminderState(options: ReminderStateOptions) {
  const reminderDialogOpen = ref(false)
  const reminderListOpen = ref(false)
  const reminders = ref<Reminder[]>([])
  const remindersLoading = ref(false)
  let reminderPollTimer: number | null = null
  let notificationPermissionDenied = false

  async function refreshReminders() {
    if (!isTauriRuntime()) return
    remindersLoading.value = true
    try {
      if (!(await options.forceSave())) return
      reminders.value = await listReminders()
    } catch {
      options.onError()
    } finally {
      remindersLoading.value = false
    }
  }

  async function completeReminderItem(reminder: Reminder) {
    try {
      if (!(await options.forceSave())) return
      await completeReminder(reminder)
      if (reminder.noteId === options.activeTabId.value) await options.loadActiveNote()
      await refreshReminders()
    } catch {
      options.onError()
      await refreshReminders()
    }
  }

  async function reopenReminderItem(reminder: Reminder) {
    try {
      if (!(await options.forceSave())) return
      await reopenReminder(reminder)
      if (reminder.noteId === options.activeTabId.value) await options.loadActiveNote()
      await refreshReminders()
    } catch {
      options.onError()
      await refreshReminders()
    }
  }

  async function completeAllDueReminders() {
    try {
      if (!(await options.forceSave())) return
      await completeDueReminders()
      await options.loadActiveNote()
      await refreshReminders()
    } catch {
      options.onError()
    }
  }

  async function checkDueReminders() {
    if (!isTauriRuntime() || notificationPermissionDenied) return
    try {
      const current = await listReminders()
      if (!current.some((reminder) => reminder.status === 'due')) return
      let granted = await isPermissionGranted()
      if (!granted) granted = (await requestPermission()) === 'granted'
      if (!granted) {
        notificationPermissionDenied = true
        return
      }

      const due = await claimDueReminders()
      for (const reminder of due) {
        sendNotification({ title: options.notificationTitle(), body: reminder.content })
      }
      if (reminderListOpen.value && due.length > 0) await refreshReminders()
    } catch {
      // Notification failures must not interrupt note editing.
    }
  }

  async function startReminderPolling() {
    await checkDueReminders()
    reminderPollTimer = window.setInterval(() => void checkDueReminders(), 30_000)
  }

  function disposeReminderState() {
    if (reminderPollTimer) window.clearInterval(reminderPollTimer)
    reminderPollTimer = null
  }

  return {
    reminderDialogOpen,
    reminderListOpen,
    reminders,
    remindersLoading,
    refreshReminders,
    completeReminderItem,
    reopenReminderItem,
    completeAllDueReminders,
    startReminderPolling,
    disposeReminderState,
  }
}
