import { ref } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { claimDueReminders, listReminders } from '../lib/invoke'
import type { Reminder } from '../types/note'
import { useReminderState } from './useReminderState'

vi.mock('@tauri-apps/plugin-notification', () => ({
  isPermissionGranted: vi.fn(),
  requestPermission: vi.fn(),
  sendNotification: vi.fn(),
}))

vi.mock('../lib/invoke', () => ({
  claimDueReminders: vi.fn(),
  completeDueReminders: vi.fn(),
  completeReminder: vi.fn(),
  listReminders: vi.fn(),
  reopenReminder: vi.fn(),
}))

vi.mock('../lib/runtime', () => ({ isTauriRuntime: vi.fn(() => true) }))

const dueReminder: Reminder = {
  id: 'inbox\u001f1000\u001fcheck',
  noteId: 'inbox',
  title: 'Inbox',
  fileName: 'inbox.md',
  lineNumber: 2,
  dueAt: 1000,
  dueText: '2026-01-01 09:00',
  content: 'check',
  completed: false,
  status: 'due',
}

describe('useReminderState', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.mocked(isPermissionGranted).mockReset()
    vi.mocked(requestPermission).mockReset()
    vi.mocked(sendNotification).mockReset()
    vi.mocked(claimDueReminders).mockReset()
    vi.mocked(listReminders).mockReset()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('claims due reminders directly without listing first when notification permission is granted', async () => {
    vi.mocked(isPermissionGranted).mockResolvedValue(true)
    vi.mocked(claimDueReminders).mockResolvedValue([dueReminder])

    const state = useReminderState({
      activeTabId: ref('inbox'),
      forceSave: vi.fn(async () => true),
      loadActiveNote: vi.fn(async () => true),
      notificationTitle: () => 'NeoPad',
      onError: vi.fn(),
    })

    await state.startReminderPolling()

    expect(listReminders).not.toHaveBeenCalled()
    expect(claimDueReminders).toHaveBeenCalledTimes(1)
    expect(sendNotification).toHaveBeenCalledWith({ title: 'NeoPad', body: 'check' })
    state.disposeReminderState()
  })

  it('does not request notification permission when no reminder is due', async () => {
    vi.mocked(isPermissionGranted).mockResolvedValue(false)
    vi.mocked(listReminders).mockResolvedValue([])

    const state = useReminderState({
      activeTabId: ref('inbox'),
      forceSave: vi.fn(async () => true),
      loadActiveNote: vi.fn(async () => true),
      notificationTitle: () => 'NeoPad',
      onError: vi.fn(),
    })

    await state.startReminderPolling()

    expect(listReminders).toHaveBeenCalledTimes(1)
    expect(requestPermission).not.toHaveBeenCalled()
    expect(claimDueReminders).not.toHaveBeenCalled()
    state.disposeReminderState()
  })
})
