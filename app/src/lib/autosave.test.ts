import { describe, expect, it, vi } from 'vitest'
import { AutosaveCoordinator, type SaveState } from './autosave'

interface Snapshot {
  noteId: string
  content: string
}

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

describe('AutosaveCoordinator', () => {
  it('debounces rapid edits and saves only the latest snapshot', async () => {
    vi.useFakeTimers()
    const save = vi.fn(async () => undefined)
    const coordinator = new AutosaveCoordinator({ delayMs: 500, save })

    coordinator.markChanged({ noteId: 'inbox', content: 'a' })
    coordinator.markChanged({ noteId: 'inbox', content: 'ab' })
    coordinator.markChanged({ noteId: 'inbox', content: 'abc' })
    await vi.advanceTimersByTimeAsync(500)

    expect(save).toHaveBeenCalledTimes(1)
    expect(save).toHaveBeenCalledWith({ noteId: 'inbox', content: 'abc' })
    vi.useRealTimers()
  })

  it('serializes a newer edit behind an in-flight save', async () => {
    const first = deferred<void>()
    const calls: Snapshot[] = []
    const save = vi.fn((snapshot: Snapshot) => {
      calls.push(snapshot)
      return calls.length === 1 ? first.promise : Promise.resolve()
    })
    const coordinator = new AutosaveCoordinator({ delayMs: 500, save })

    coordinator.markChanged({ noteId: 'inbox', content: 'old' })
    const flushing = coordinator.flush()
    coordinator.markChanged({ noteId: 'inbox', content: 'new' })
    first.resolve()

    expect(await flushing).toBe(true)
    expect(calls).toEqual([
      { noteId: 'inbox', content: 'old' },
      { noteId: 'inbox', content: 'new' },
    ])
  })

  it('reports failure without marking the revision as saved', async () => {
    const states: SaveState[] = []
    const save = vi.fn().mockRejectedValue(new Error('disk full'))
    const coordinator = new AutosaveCoordinator<Snapshot, void>({
      delayMs: 500,
      save,
      onStateChange: (state) => states.push(state),
    })

    coordinator.markChanged({ noteId: 'inbox', content: 'draft' })

    expect(await coordinator.flush()).toBe(false)
    expect(states.at(-1)).toBe('Failed')
  })

  it('cancels a pending save when authoritative content is loaded', async () => {
    vi.useFakeTimers()
    const save = vi.fn(async () => undefined)
    const coordinator = new AutosaveCoordinator({ delayMs: 500, save })

    coordinator.markChanged({ noteId: 'inbox', content: 'draft' })
    coordinator.markLoaded()
    await vi.advanceTimersByTimeAsync(500)

    expect(save).not.toHaveBeenCalled()
    vi.useRealTimers()
  })
})
