export type AiContextKind = 'selection' | 'paragraph' | 'note'
export type AiInlineCommandName = 'continue' | 'polish' | 'summarize' | 'translate'
export type AiContextScope = 'note' | 'library'

export interface AiConfig {
  enabled: boolean
  baseUrl: string
  model: string
  apiKeyConfigured: boolean
}

export interface AiConversationMessage {
  role: 'user' | 'assistant'
  content: string
}

export interface AiChatMessage extends AiConversationMessage {
  sources?: AiContextSource[]
}

export interface AiPromptEntry {
  id: string
  name: string
  fileName: string
  content: string
}

export interface AiContextSource {
  noteId: string
  title: string
  fileName: string
  lineNumber: number
}

export interface AiGenerateResponse {
  content: string
  sources: AiContextSource[]
}

export interface AiChatState {
  messages: AiChatMessage[]
  scope: AiContextScope
  promptId?: string
}

export interface AiEditorContext {
  kind: AiContextKind
  text: string
  from: number
  to: number
}

export interface AiEditorSnapshot {
  documentText: string
  cursor: number
  contexts: AiEditorContext[]
  defaultKind: AiContextKind
}

export interface AiPanelSession {
  noteId: string
  noteTitle: string
  snapshot: AiEditorSnapshot
  chat: AiChatState
}
