import { ref } from 'vue'
import { describe, expect, it } from 'vitest'
import { isCurrentAiRequest } from './ai-request-state'

describe('isCurrentAiRequest', () => {
  it('matches a Vue-proxied session by request id instead of object identity', () => {
    const rawSession = { requestId: 7 }
    const session = ref(rawSession)

    expect(session.value).not.toBe(rawSession)
    expect(isCurrentAiRequest(session.value, 7)).toBe(true)
    expect(isCurrentAiRequest(session.value, 8)).toBe(false)
  })
})
