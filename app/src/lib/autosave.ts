export type SaveState = 'Saved' | 'Saving' | 'Failed'

export interface AutosaveOptions<TSnapshot, TResult> {
  delayMs: number
  save: (snapshot: TSnapshot) => Promise<TResult>
  onSaved?: (snapshot: TSnapshot, result: TResult) => void
  onStateChange?: (state: SaveState) => void
  onError?: (error: unknown) => void
}

export class AutosaveCoordinator<TSnapshot, TResult> {
  private readonly options: AutosaveOptions<TSnapshot, TResult>
  private timer: ReturnType<typeof setTimeout> | null = null
  private inFlight: Promise<boolean> | null = null
  private latestSnapshot: TSnapshot | null = null
  private revision = 0
  private savedRevision = 0

  constructor(options: AutosaveOptions<TSnapshot, TResult>) {
    this.options = options
  }

  markLoaded() {
    this.clearTimer()
    this.latestSnapshot = null
    this.revision += 1
    this.savedRevision = this.revision
    this.options.onStateChange?.('Saved')
  }

  markChanged(snapshot: TSnapshot) {
    this.latestSnapshot = snapshot
    this.revision += 1
    this.options.onStateChange?.('Saving')
    this.clearTimer()
    this.timer = setTimeout(() => {
      void this.flush()
    }, this.options.delayMs)
  }

  async flush(): Promise<boolean> {
    this.clearTimer()

    while (this.savedRevision < this.revision) {
      if (this.inFlight) {
        const succeeded = await this.inFlight
        if (!succeeded) return false
        continue
      }

      const snapshot = this.latestSnapshot
      if (!snapshot) return true
      const revision = this.revision
      this.options.onStateChange?.('Saving')
      this.inFlight = this.options.save(snapshot)
        .then((result) => {
          this.savedRevision = Math.max(this.savedRevision, revision)
          this.options.onSaved?.(snapshot, result)
          if (this.savedRevision === this.revision) {
            this.options.onStateChange?.('Saved')
          }
          return true
        })
        .catch((error: unknown) => {
          this.options.onStateChange?.('Failed')
          this.options.onError?.(error)
          return false
        })
        .finally(() => {
          this.inFlight = null
        })

      if (!(await this.inFlight)) return false
    }

    return true
  }

  dispose() {
    this.clearTimer()
  }

  private clearTimer() {
    if (this.timer) {
      clearTimeout(this.timer)
      this.timer = null
    }
  }
}
