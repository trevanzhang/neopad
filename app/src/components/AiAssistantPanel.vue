<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import type { AppMessages } from '../lib/i18n'
import { openExternalUrl } from '../lib/invoke'
import { renderMarkdown } from '../lib/markdown'
import { isTauriRuntime } from '../lib/runtime'
import type {
  AiChatMessage,
  AiChatState,
  AiContextKind,
  AiContextScope,
  AiConversationMessage,
  AiGenerateResponse,
  AiPanelSession,
  AiPromptEntry,
} from '../types/ai'

type AssistantItem = AiChatMessage & {
  contextKind?: AiContextKind
  copied?: boolean
}

const props = defineProps<{
  session: AiPanelSession
  ready: boolean
  prompts: AiPromptEntry[]
  promptsLoading: boolean
  messages: AppMessages['ai']
  generate: (
    messages: AiConversationMessage[],
    scope: AiContextScope,
    prompt?: string,
  ) => Promise<AiGenerateResponse>
  applyResult: (action: 'replace' | 'insert' | 'insertBelow', content: string, contextKind: AiContextKind) => boolean
}>()

const emit = defineEmits<{
  close: []
  configure: []
  updateChat: [chat: AiChatState]
  refreshPrompts: []
  managePrompts: []
}>()

const panel = ref<HTMLElement | null>(null)
const promptInput = ref<HTMLTextAreaElement | null>(null)
const promptSearchInput = ref<HTMLInputElement | null>(null)
const transcriptElement = ref<HTMLElement | null>(null)
const prompt = ref('')
const transcript = ref<AssistantItem[]>(props.session.chat.messages.map((message) => ({
  ...message,
  sources: message.sources?.map((source) => ({ ...source })),
})))
const scope = ref<AiContextScope>(props.session.chat.scope)
const selectedPromptId = ref(props.session.chat.promptId)
const promptPickerOpen = ref(false)
const scopePickerOpen = ref(false)
const promptQuery = ref('')
const busy = ref(false)
const error = ref<string | null>(null)

const expanded = computed(() => transcript.value.length > 0 || busy.value)
const selectedPrompt = computed(() => props.prompts.find((item) => item.id === selectedPromptId.value))
const filteredPrompts = computed(() => {
  const query = promptQuery.value.trim().toLowerCase()
  if (!query) return props.prompts
  return props.prompts.filter((item) => `${item.relativePath} ${item.content}`.toLowerCase().includes(query))
})
function promptCategory(prompt: AiPromptEntry) {
  const index = prompt.relativePath.lastIndexOf('/')
  return index < 0 ? '' : prompt.relativePath.slice(0, index)
}
const selectionContext = computed(() => props.session.snapshot.contexts.find((item) => item.kind === 'selection'))
onMounted(() => {
  void nextTick(async () => {
    panel.value?.focus()
    promptInput.value?.focus()
    resizeComposer()
    await scrollTranscript()
  })
})

function displayError(value: unknown) {
  if (typeof value === 'string') return value
  if (value instanceof Error) return value.message
  return String(value)
}

function chatState(): AiChatState {
  return {
    messages: transcript.value.map(({ role, content, sources }) => ({
      role,
      content,
      sources: sources?.map((source) => ({ ...source })),
    })),
    scope: scope.value,
    promptId: selectedPromptId.value,
  }
}

function persistChat() {
  emit('updateChat', chatState())
}

async function submitPrompt() {
  const content = prompt.value.trim()
  if (!content || busy.value || !props.ready) return

  const contextKind = selectionContext.value ? 'selection' : 'note'
  transcript.value.push({ role: 'user', content })
  prompt.value = ''
  busy.value = true
  error.value = null
  promptPickerOpen.value = false
  scopePickerOpen.value = false
  resizeComposer()
  persistChat()
  await scrollTranscript()

  try {
    const conversation = transcript.value.map(({ role, content: messageContent }) => ({
      role,
      content: messageContent,
    }))
    const result = await props.generate(conversation, scope.value, selectedPrompt.value?.content)
    transcript.value.push({
      role: 'assistant',
      content: result.content,
      contextKind,
      sources: result.sources,
    })
    persistChat()
  } catch (requestError) {
    error.value = displayError(requestError)
  } finally {
    busy.value = false
    await scrollTranscript()
    promptInput.value?.focus()
  }
}

async function scrollTranscript() {
  await nextTick()
  transcriptElement.value?.scrollTo({ top: transcriptElement.value.scrollHeight })
}

function resizeComposer() {
  const input = promptInput.value
  if (!input) return
  input.style.height = 'auto'
  input.style.height = `${Math.min(input.scrollHeight, 132)}px`
}

function handleComposerKeydown(event: KeyboardEvent) {
  if (event.key !== 'Enter' || event.shiftKey || event.isComposing) return
  event.preventDefault()
  void submitPrompt()
}

function apply(action: 'replace' | 'insert' | 'insertBelow', item: AssistantItem) {
  const kind = action === 'replace'
    ? 'selection'
    : item.contextKind ?? props.session.snapshot.defaultKind
  if (!props.applyResult(action, item.content, kind)) {
    error.value = props.messages.staleContext
  }
}

async function copyResult(item: AssistantItem) {
  await navigator.clipboard.writeText(item.content)
  item.copied = true
  window.setTimeout(() => { item.copied = false }, 1600)
}

function toggleScopePicker() {
  scopePickerOpen.value = !scopePickerOpen.value
  if (scopePickerOpen.value) promptPickerOpen.value = false
}

function selectScope(value: AiContextScope) {
  scope.value = value
  scopePickerOpen.value = false
  persistChat()
  promptInput.value?.focus()
}

function handleScopeFocusOut(event: FocusEvent) {
  const nextTarget = event.relatedTarget
  if (nextTarget instanceof Node && (event.currentTarget as HTMLElement).contains(nextTarget)) return
  scopePickerOpen.value = false
}

function togglePromptPicker() {
  promptPickerOpen.value = !promptPickerOpen.value
  if (promptPickerOpen.value) {
    scopePickerOpen.value = false
    promptQuery.value = ''
    void nextTick(() => promptSearchInput.value?.focus())
  }
}

function selectPrompt(item: AiPromptEntry) {
  selectedPromptId.value = item.id
  promptPickerOpen.value = false
  persistChat()
  promptInput.value?.focus()
}

function clearSelectedPrompt() {
  selectedPromptId.value = undefined
  persistChat()
  promptInput.value?.focus()
}

function clearChat() {
  transcript.value = []
  error.value = null
  persistChat()
  void nextTick(() => promptInput.value?.focus())
}

function handlePanelEscape() {
  if (promptPickerOpen.value) {
    promptPickerOpen.value = false
  } else if (scopePickerOpen.value) {
    scopePickerOpen.value = false
  } else {
    emit('close')
  }
}

function handleMarkdownClick(event: MouseEvent) {
  const target = event.target
  if (!(target instanceof Element)) return
  const link = target.closest('a')
  if (!(link instanceof HTMLAnchorElement)) return
  event.preventDefault()
  const url = link.getAttribute('href') ?? ''
  if (!/^https?:\/\//i.test(url)) return
  if (isTauriRuntime()) {
    void openExternalUrl(url)
  } else {
    window.open(url, '_blank', 'noopener,noreferrer')
  }
}
</script>

<template>
  <div class="ai-panel-backdrop" role="presentation" @mousedown.self="emit('close')">
    <section
      ref="panel"
      :class="['ai-panel', { 'is-expanded': expanded }]"
      role="dialog"
      aria-modal="true"
      :aria-label="messages.title"
      tabindex="-1"
      @keydown.esc.stop.prevent="handlePanelEscape"
    >
      <header v-if="expanded" class="ai-panel-header">
        <div class="ai-panel-title">
          <span class="ai-panel-mark" aria-hidden="true">AI</span>
          <span>{{ session.noteTitle }}</span>
        </div>
        <div class="ai-panel-header-actions">
          <button type="button" @click="clearChat">{{ messages.clearChat }}</button>
          <button type="button" :aria-label="messages.close" :title="messages.close" @click="emit('close')">
            <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg>
          </button>
        </div>
      </header>

      <div v-if="!ready" class="ai-panel-empty">
        <p>{{ messages.disabled }}</p>
        <button type="button" class="primary" @click="emit('configure')">{{ messages.configure }}</button>
      </div>

      <template v-else>
        <div v-if="expanded" ref="transcriptElement" class="ai-transcript" aria-live="polite">
          <article v-for="(item, index) in transcript" :key="index" :class="['ai-message', `is-${item.role}`]">
            <div
              v-if="item.role === 'assistant'"
              class="ai-message-markdown markdown-preview"
              @click="handleMarkdownClick"
              v-html="renderMarkdown(item.content)"
            />
            <p v-else>{{ item.content }}</p>
            <div v-if="item.role === 'assistant' && item.sources?.length" class="ai-message-sources">
              <span>{{ messages.sources }}</span>
              <span v-for="source in item.sources" :key="`${source.noteId}-${source.lineNumber}`">
                {{ source.title }} · {{ source.lineNumber }}
              </span>
            </div>
            <div v-if="item.role === 'assistant'" class="ai-result-actions">
              <button type="button" @click="copyResult(item)">{{ item.copied ? messages.copied : messages.copy }}</button>
              <button type="button" @click="apply('insert', item)">{{ messages.insertAtCursor }}</button>
              <button type="button" @click="apply('insertBelow', item)">{{ messages.insertBelow }}</button>
              <button v-if="selectionContext" type="button" @click="apply('replace', item)">
                {{ messages.replaceSelection }}
              </button>
            </div>
          </article>
          <div v-if="busy" class="ai-thinking"><span /><span /><span />{{ messages.thinking }}</div>
        </div>

        <p v-if="error" class="ai-panel-error">{{ error }}</p>

        <form class="ai-composer" @submit.prevent="submitPrompt">
          <textarea
            ref="promptInput"
            v-model="prompt"
            rows="1"
            :placeholder="messages.promptPlaceholder"
            @input="resizeComposer"
            @keydown="handleComposerKeydown"
          />

          <div class="ai-composer-footer">
            <div class="ai-composer-tools">
              <button
                type="button"
                class="ai-tool-button ai-prompt-trigger"
                :class="{ active: promptPickerOpen }"
                :aria-label="messages.promptLibrary"
                :title="messages.promptLibrary"
                :aria-expanded="promptPickerOpen"
                @click="togglePromptPicker"
              >
                <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 3v10M3 8h10" /></svg>
              </button>

              <button
                v-if="selectedPrompt"
                type="button"
                class="ai-prompt-chip"
                :title="selectedPrompt.content"
                @click="clearSelectedPrompt"
              >
                {{ selectedPrompt.name }} <span aria-hidden="true">×</span>
              </button>

              <div class="ai-scope-picker" @focusout="handleScopeFocusOut">
                <button
                  type="button"
                  class="ai-scope-trigger"
                  :class="{ active: scope === 'library' }"
                  aria-haspopup="menu"
                  :aria-expanded="scopePickerOpen"
                  :aria-label="`${messages.context}: ${scope === 'library' ? messages.libraryScope : messages.noteScope}`"
                  :title="scope === 'library' ? messages.libraryScopeHint : messages.noteScopeHint"
                  @click="toggleScopePicker"
                >
                  <svg v-if="scope === 'library'" viewBox="0 0 16 16" aria-hidden="true">
                    <path d="M3.5 3.5h3v3h-3zM9.5 3.5h3v3h-3zM3.5 9.5h3v3h-3zM9.5 9.5h3v3h-3z" />
                  </svg>
                  <svg v-else viewBox="0 0 16 16" aria-hidden="true">
                    <path d="M4 2.5h5l3 3v8H4zM9 2.5v3h3" />
                  </svg>
                  <span>{{ scope === 'library' ? messages.libraryScope : messages.noteScope }}</span>
                  <svg class="ai-scope-chevron" viewBox="0 0 12 12" aria-hidden="true"><path d="m3 4.5 3 3 3-3" /></svg>
                </button>

                <div v-if="scopePickerOpen" class="ai-scope-menu" role="menu">
                  <button
                    v-for="value in (['note', 'library'] as AiContextScope[])"
                    :key="value"
                    type="button"
                    role="menuitemradio"
                    :aria-checked="scope === value"
                    :class="{ selected: scope === value }"
                    @click="selectScope(value)"
                  >
                    <span class="ai-scope-menu-check" aria-hidden="true">{{ scope === value ? '✓' : '' }}</span>
                    <span>
                      <strong>{{ value === 'library' ? messages.libraryScope : messages.noteScope }}</strong>
                      <small>{{ value === 'library' ? messages.libraryScopeHint : messages.noteScopeHint }}</small>
                    </span>
                  </button>
                </div>
              </div>
            </div>

            <button
              type="submit"
              class="ai-send-button"
              :disabled="busy || !prompt.trim()"
              :aria-label="messages.send"
              :title="messages.send"
            >
              <svg viewBox="0 0 18 18" aria-hidden="true"><path d="M9 14V4M5 8l4-4 4 4" /></svg>
            </button>
          </div>

          <div v-if="promptPickerOpen" class="ai-prompt-picker">
            <div class="ai-prompt-picker-search">
              <input ref="promptSearchInput" v-model="promptQuery" type="search" :placeholder="messages.searchPrompts" />
              <button type="button" :title="messages.refreshPrompts" @click="emit('refreshPrompts')">↻</button>
            </div>
            <div class="ai-prompt-list">
              <p v-if="promptsLoading">{{ messages.loadingPrompts }}</p>
              <button
                v-for="item in filteredPrompts"
                v-else
                :key="item.id"
                type="button"
              :class="{ selected: item.id === selectedPromptId }"
              @click="selectPrompt(item)"
            >
                <span class="ai-prompt-item-heading">
                  <strong class="ai-prompt-item-title">{{ item.name }}</strong>
                  <small v-if="promptCategory(item)" class="ai-prompt-item-category">{{ promptCategory(item) }}</small>
                </span>
                <span class="ai-prompt-item-preview">{{ item.content }}</span>
              </button>
              <p v-if="!promptsLoading && filteredPrompts.length === 0">{{ messages.noPrompts }}</p>
            </div>
            <button type="button" class="ai-open-prompts" @click="emit('managePrompts')">
              {{ messages.managePrompts }}
            </button>
          </div>
        </form>
      </template>
    </section>
  </div>
</template>
