export interface AiRequestState {
  requestId: number
}

export function isCurrentAiRequest<T extends AiRequestState>(
  current: T | null | undefined,
  requestId: number,
): current is T {
  return current?.requestId === requestId
}
