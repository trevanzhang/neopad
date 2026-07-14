<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import AiSettingsSection from './AiSettingsSection.vue'
import type { AppLanguage, AppMessages } from '../lib/i18n'
import { getHelpContent } from '../lib/help-content'
import type { EditorMode, PreviewContentWidth, PreviewFontFamily, PreviewLineHeight, PreviewTheme } from '../types/editor'
import type { AiConfig } from '../types/ai'

type SettingsTab = 'general' | 'preview' | 'shortcuts' | 'insertText' | 'advanced' | 'ai' | 'mcp' | 'about'
type TitleDoubleClickAction = 'none' | 'delete' | 'rename'
type McpStatus = {
  enabled: boolean
  running: boolean
  status: string
  url: string
  token: string
  lastError: string | null
}

const props = defineProps<{
  alwaysOnTop: boolean
  vimMode: boolean
  vimUseCtrlShortcuts: boolean
  vimInsertExitKey: string
  previewMode: EditorMode
  language: AppLanguage
  appVersion: string
  workspacePath: string
  runAtStartup: boolean
  startHidden: boolean
  closeToMinimize: boolean
  snapToEdges: boolean
  transparencyEnabled: boolean
  windowOpacityPercent: number
  titleDoubleClickAction: TitleDoubleClickAction
  shortcutBaseKey: string
  shortcutModifiers: string[]
  clipboardShortcutBaseKey: string
  clipboardShortcutModifiers: string[]
  insertSeparatorTemplate: string
  insertDateTimeTemplate: string
  insertDateTimeSeparatorTemplate: string
  customInsertTexts: string[]
  previewTheme: PreviewTheme
  previewFontFamily: PreviewFontFamily
  previewFontSize: number
  previewLineHeight: PreviewLineHeight
  previewContentWidth: PreviewContentWidth
  mcpStatus: McpStatus | null
  mcpError: string | null
  aiConfig: AiConfig
  aiError: string | null
  aiTesting: boolean
  aiTestSucceeded: boolean
  initialTab?: SettingsTab
  messages: AppMessages['settings']
  menuMessages: AppMessages['menu']
}>()

const emit = defineEmits<{
  close: []
  toggleAlwaysOnTop: []
  'update:vimMode': [enabled: boolean]
  'update:vimUseCtrlShortcuts': [enabled: boolean]
  'update:vimInsertExitKey': [key: string]
  'update:previewMode': [mode: EditorMode]
  'update:language': [language: AppLanguage]
  'update:runAtStartup': [value: boolean]
  'update:startHidden': [value: boolean]
  'update:closeToMinimize': [value: boolean]
  'update:snapToEdges': [value: boolean]
  'update:transparencyEnabled': [value: boolean]
  'update:windowOpacityPercent': [value: number]
  'update:titleDoubleClickAction': [value: TitleDoubleClickAction]
  'update:shortcutBaseKey': [value: string]
  'update:shortcutModifiers': [value: string[]]
  'update:clipboardShortcutBaseKey': [value: string]
  'update:clipboardShortcutModifiers': [value: string[]]
  'update:insertSeparatorTemplate': [value: string]
  'update:insertDateTimeTemplate': [value: string]
  'update:insertDateTimeSeparatorTemplate': [value: string]
  'update:customInsertTexts': [value: string[]]
  'update:previewTheme': [value: PreviewTheme]
  'update:previewFontFamily': [value: PreviewFontFamily]
  'update:previewFontSize': [value: number]
  'update:previewLineHeight': [value: PreviewLineHeight]
  'update:previewContentWidth': [value: PreviewContentWidth]
  editCustomText: [index: number]
  'update-mcp-enabled': [enabled: boolean]
  'copy-mcp-config': []
  'regenerate-mcp-token': []
  'update-ai-config': [patch: Partial<Omit<AiConfig, 'apiKeyConfigured'>>]
  'save-ai-api-key': [apiKey: string]
  'clear-ai-api-key': []
  'test-ai-connection': []
  'manage-ai-prompts': []
  'open-ai-prompts-folder': []
}>()

const activeTab = ref<SettingsTab>(props.initialTab ?? 'general')
const selectedCustomIndex = ref<number | null>(null)
const panel = ref<HTMLElement | null>(null)

onMounted(() => {
  void nextTick(() => panel.value?.focus())
})

const activeTabLabel = computed<keyof AppMessages['settings']>(() => {
  const labels: Record<SettingsTab, keyof AppMessages['settings']> = {
    general: 'general',
    preview: 'previewTab',
    shortcuts: 'shortcutsTab',
    insertText: 'insertTextTab',
    advanced: 'advancedTab',
    ai: 'aiTab',
    mcp: 'mcp',
    about: 'about',
  }
  return labels[activeTab.value]
})

const shortcutReference = computed(() => getHelpContent('shortcuts', props.language, {
  appVersion: props.appVersion,
  shortcutBaseKey: props.shortcutBaseKey,
  shortcutModifiers: props.shortcutModifiers,
  clipboardShortcutBaseKey: props.clipboardShortcutBaseKey,
  clipboardShortcutModifiers: props.clipboardShortcutModifiers,
}).lines.map((line) => {
  const dividerIndex = line.indexOf(' - ')
  return dividerIndex === -1
    ? { keys: line, description: '' }
    : { keys: line.slice(0, dividerIndex), description: line.slice(dividerIndex + 3) }
}))

function mcpAgentConfig(status: McpStatus | null) {
  return JSON.stringify(
    {
      mcpServers: {
        neopad: {
          url: status?.url || 'http://127.0.0.1:8765/mcp',
          headers: {
            Authorization: `Bearer ${status?.token || '<local-token>'}`,
          },
        },
      },
    },
    null,
    2,
  )
}

function updateModifier(modifier: string, checked: boolean, current: string[]) {
  const next = checked ? Array.from(new Set([...current, modifier])) : current.filter((item) => item !== modifier)
  emit('update:shortcutModifiers', next)
}

function updateClipboardModifier(modifier: string, checked: boolean, current: string[]) {
  const next = checked ? Array.from(new Set([...current, modifier])) : current.filter((item) => item !== modifier)
  emit('update:clipboardShortcutModifiers', next)
}

function addCustomText(current: string[]) {
  emit('update:customInsertTexts', [...current, ''])
  selectedCustomIndex.value = current.length
}

function editCustomText() {
  if (selectedCustomIndex.value === null) {
    return
  }
  emit('editCustomText', selectedCustomIndex.value)
}

function deleteCustomText(current: string[]) {
  if (selectedCustomIndex.value === null) {
    return
  }

  const next = current.filter((_, index) => index !== selectedCustomIndex.value)
  emit('update:customInsertTexts', next)
  selectedCustomIndex.value = null
}
</script>

<template>
  <aside
    ref="panel"
    class="settings-panel settings-dialog"
    role="dialog"
    aria-modal="true"
    :aria-label="messages.title"
    tabindex="-1"
    @keydown.esc.stop.prevent="emit('close')"
  >
    <header class="settings-header">
      <strong>{{ messages.title }}</strong>
      <button type="button" class="settings-close" :aria-label="messages.close" :title="messages.close" @click="$emit('close')"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg></button>
    </header>

    <nav class="settings-tabs" :aria-label="messages.title">
      <button type="button" :aria-current="activeTab === 'general' ? 'page' : undefined" :class="{ active: activeTab === 'general' }" @click="activeTab = 'general'">
        <svg class="settings-tab-icon" viewBox="0 0 16 16" aria-hidden="true"><path d="M8 2.3l4.6 2.65v5.3L8 12.9l-4.6-2.65v-5.3L8 2.3zm0 3.15a2.15 2.15 0 1 0 0 4.3 2.15 2.15 0 0 0 0-4.3z" /></svg>
        <span>{{ messages.general }}</span>
      </button>
      <button type="button" :aria-current="activeTab === 'preview' ? 'page' : undefined" :class="{ active: activeTab === 'preview' }" @click="activeTab = 'preview'">
        <svg class="settings-tab-icon" viewBox="0 0 16 16" aria-hidden="true"><path d="M1.8 8s2.1-3.1 6.2-3.1S14.2 8 14.2 8s-2.1 3.1-6.2 3.1S1.8 8 1.8 8zm6.2-1.65a1.65 1.65 0 1 0 0 3.3 1.65 1.65 0 0 0 0-3.3z" /></svg>
        <span>{{ messages.previewTab }}</span>
      </button>
      <button type="button" :aria-current="activeTab === 'shortcuts' ? 'page' : undefined" :class="{ active: activeTab === 'shortcuts' }" @click="activeTab = 'shortcuts'">
        <svg class="settings-tab-icon" viewBox="0 0 16 16" aria-hidden="true"><rect x="2.1" y="4" width="11.8" height="8" rx="1.2" /><path d="M4.4 6.5h.01M6.5 6.5h.01M8.6 6.5h.01M10.7 6.5h.01M5.4 9.3h5.2" /></svg>
        <span>{{ messages.shortcutsTab }}</span>
      </button>
      <button type="button" :aria-current="activeTab === 'insertText' ? 'page' : undefined" :class="{ active: activeTab === 'insertText' }" @click="activeTab = 'insertText'">
        <svg class="settings-tab-icon" viewBox="0 0 16 16" aria-hidden="true"><path d="M3.2 1.9h6l3.6 3.6v8.6H3.2zM9.1 1.9v3.8h3.7M8 7v4M6 9h4" /></svg>
        <span>{{ messages.insertTextTab }}</span>
      </button>
      <button type="button" :aria-current="activeTab === 'advanced' ? 'page' : undefined" :class="{ active: activeTab === 'advanced' }" @click="activeTab = 'advanced'">
        <svg class="settings-tab-icon" viewBox="0 0 16 16" aria-hidden="true"><path d="M3 4h10M3 8h10M3 12h10" /><circle cx="6" cy="4" r="1.25" /><circle cx="10" cy="8" r="1.25" /><circle cx="5" cy="12" r="1.25" /></svg>
        <span>{{ messages.advancedTab }}</span>
      </button>
      <button type="button" :aria-current="activeTab === 'ai' ? 'page' : undefined" :class="{ active: activeTab === 'ai' }" @click="activeTab = 'ai'">
        <svg class="settings-tab-icon" viewBox="0 0 16 16" aria-hidden="true"><path d="M8 1.7v2.1M8 12.2v2.1M1.7 8h2.1M12.2 8h2.1M3.5 3.5L5 5M11 11l1.5 1.5M12.5 3.5L11 5M5 11l-1.5 1.5" /><circle cx="8" cy="8" r="2.55" /></svg>
        <span>{{ messages.aiTab }}</span>
      </button>
      <button type="button" :aria-current="activeTab === 'mcp' ? 'page' : undefined" :class="{ active: activeTab === 'mcp' }" @click="activeTab = 'mcp'">
        <svg class="settings-tab-icon" viewBox="0 0 16 16" aria-hidden="true"><circle cx="4" cy="4" r="1.8" /><circle cx="12" cy="4" r="1.8" /><circle cx="8" cy="12" r="1.8" /><path d="M5.5 5.1l1.4 5.1M10.5 5.1l-1.4 5.1M5.8 4h4.4" /></svg>
        <span>{{ messages.mcp }}</span>
      </button>
      <button type="button" class="settings-tab-about" :aria-current="activeTab === 'about' ? 'page' : undefined" :class="{ active: activeTab === 'about' }" @click="activeTab = 'about'">
        <svg class="settings-tab-icon" viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="5.9" /><path d="M8 7.05v3.75M8 4.8h.01" /></svg>
        <span>{{ messages.about }}</span>
      </button>
    </nav>

    <section class="settings-content">
      <header class="settings-content-header">
        <h2>{{ messages[activeTabLabel] }}</h2>
      </header>
      <div class="settings-tab-content">
      <template v-if="activeTab === 'general'">
        <fieldset class="settings-fieldset">
          <legend>{{ messages.generalOptions }}</legend>
          <label class="settings-check-row">
            <input
              :checked="runAtStartup"
              type="checkbox"
              @change="$emit('update:runAtStartup', ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ messages.runAtStartup }}</span>
          </label>
          <label class="settings-check-row">
            <input
              :checked="startHidden"
              type="checkbox"
              @change="$emit('update:startHidden', ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ messages.startHidden }}</span>
          </label>
          <label class="settings-check-row">
            <input
              :checked="closeToMinimize"
              type="checkbox"
              @change="$emit('update:closeToMinimize', ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ messages.closeToMinimize }}</span>
          </label>
          <label class="settings-check-row">
            <input
              :checked="snapToEdges"
              type="checkbox"
              @change="$emit('update:snapToEdges', ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ messages.snapToEdges }}</span>
          </label>
        </fieldset>

        <fieldset class="settings-fieldset">
          <legend>{{ messages.windowOpacity }}</legend>
          <label class="settings-check-row">
            <input
              :checked="transparencyEnabled"
              type="checkbox"
              @change="$emit('update:transparencyEnabled', ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ messages.enableTransparency }}</span>
          </label>
          <div class="settings-slider-row">
            <input
              type="range"
              min="20"
              max="100"
              :aria-label="messages.windowOpacity"
              :disabled="!transparencyEnabled"
              :value="windowOpacityPercent"
              @input="$emit('update:windowOpacityPercent', Number(($event.target as HTMLInputElement).value))"
            />
            <strong>{{ windowOpacityPercent }}%</strong>
          </div>
        </fieldset>

        <fieldset class="settings-fieldset">
          <legend>{{ messages.titleDoubleClick }}</legend>
          <div class="settings-radio-row">
            <label>
              <input
                name="title-action"
                type="radio"
                value="none"
                :checked="titleDoubleClickAction === 'none'"
                @change="$emit('update:titleDoubleClickAction', 'none')"
              />
              {{ messages.noAction }}
            </label>
            <label>
              <input
                name="title-action"
                type="radio"
                value="delete"
                :checked="titleDoubleClickAction === 'delete'"
                @change="$emit('update:titleDoubleClickAction', 'delete')"
              />
              {{ messages.deletePage }}
            </label>
            <label>
              <input
                name="title-action"
                type="radio"
                value="rename"
                :checked="titleDoubleClickAction === 'rename'"
                @change="$emit('update:titleDoubleClickAction', 'rename')"
              />
              {{ messages.renameTitle }}
            </label>
          </div>
        </fieldset>

        <label class="settings-row">
          <span>{{ messages.defaultMode }}</span>
          <select :value="previewMode" @change="$emit('update:previewMode', ($event.target as HTMLSelectElement).value as EditorMode)">
            <option value="edit">{{ menuMessages.editMode }}</option>
            <option value="split">{{ menuMessages.splitMode }}</option>
            <option value="preview">{{ menuMessages.previewMode }}</option>
          </select>
        </label>

        <label class="settings-row">
          <span>{{ messages.language }}</span>
          <select
            :value="language"
            @change="$emit('update:language', ($event.target as HTMLSelectElement).value as AppLanguage)"
          >
            <option value="en">{{ messages.english }}</option>
            <option value="zh">{{ messages.chinese }}</option>
          </select>
        </label>
      </template>

      <template v-else-if="activeTab === 'shortcuts'">
        <section class="settings-shortcut-group">
          <header class="settings-shortcut-group-header">
            <h3>{{ messages.globalShortcuts }}</h3>
            <p>{{ messages.globalShortcutHint }}</p>
          </header>
          <div class="settings-shortcut-card">
            <div class="settings-shortcut-row settings-shortcut-config-row">
              <div class="settings-shortcut-copy">
                <strong>{{ messages.toggleWindow }}</strong>
                <span>{{ shortcutReference[0]?.keys }}</span>
              </div>
              <div class="settings-shortcut-config-controls">
                <label class="settings-shortcut-key-input">
                  <span>{{ messages.baseKey }}</span>
                  <input
                    type="text"
                    maxlength="3"
                    :value="shortcutBaseKey"
                    @input="$emit('update:shortcutBaseKey', ($event.target as HTMLInputElement).value)"
                  />
                </label>
                <div class="settings-modifier-list">
                  <label v-for="modifier in ['Ctrl', 'Alt', 'Shift', 'Win']" :key="modifier">
                    <input
                      type="checkbox"
                      :checked="shortcutModifiers.includes(modifier)"
                      @change="updateModifier(modifier, ($event.target as HTMLInputElement).checked, shortcutModifiers)"
                    />
                    {{ modifier }}
                  </label>
                </div>
              </div>
            </div>
            <div class="settings-shortcut-row settings-shortcut-config-row">
              <div class="settings-shortcut-copy">
                <strong>{{ messages.saveClipboard }}</strong>
                <span>{{ shortcutReference[1]?.keys }}</span>
              </div>
              <div class="settings-shortcut-config-controls">
                <label class="settings-shortcut-key-input">
                  <span>{{ messages.baseKey }}</span>
                  <input
                    type="text"
                    maxlength="3"
                    :value="clipboardShortcutBaseKey"
                    @input="$emit('update:clipboardShortcutBaseKey', ($event.target as HTMLInputElement).value)"
                  />
                </label>
                <div class="settings-modifier-list">
                  <label v-for="modifier in ['Ctrl', 'Alt', 'Shift', 'Win']" :key="modifier">
                    <input
                      type="checkbox"
                      :checked="clipboardShortcutModifiers.includes(modifier)"
                      @change="updateClipboardModifier(modifier, ($event.target as HTMLInputElement).checked, clipboardShortcutModifiers)"
                    />
                    {{ modifier }}
                  </label>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section class="settings-shortcut-group">
          <header class="settings-shortcut-group-header">
            <h3>{{ messages.applicationShortcuts }}</h3>
            <p>{{ messages.applicationShortcutHint }}</p>
          </header>
          <div class="settings-shortcut-card">
            <div v-for="item in shortcutReference.slice(2)" :key="item.keys" class="settings-shortcut-row">
              <span>{{ item.description }}</span>
              <kbd>{{ item.keys }}</kbd>
            </div>
          </div>
        </section>
      </template>

      <template v-else-if="activeTab === 'preview'">
        <fieldset class="settings-fieldset">
          <legend>{{ messages.previewAppearance }}</legend>
          <label class="settings-row">
            <span>{{ messages.previewTheme }}</span>
            <select :value="previewTheme" @change="$emit('update:previewTheme', ($event.target as HTMLSelectElement).value as PreviewTheme)">
              <option value="light">{{ messages.previewThemeLight }}</option>
              <option value="oneDark">{{ messages.previewThemeOneDark }}</option>
              <option value="nord">{{ messages.previewThemeNord }}</option>
              <option value="solarizedLight">{{ messages.previewThemeSolarizedLight }}</option>
              <option value="solarizedDark">{{ messages.previewThemeSolarizedDark }}</option>
              <option value="monokai">{{ messages.previewThemeMonokai }}</option>
              <option value="githubLight">{{ messages.previewThemeGitHubLight }}</option>
              <option value="dracula">{{ messages.previewThemeDracula }}</option>
            </select>
          </label>
          <label class="settings-row">
            <span>{{ messages.previewFont }}</span>
            <select :value="previewFontFamily" @change="$emit('update:previewFontFamily', ($event.target as HTMLSelectElement).value as PreviewFontFamily)">
              <option value="editor">{{ messages.previewFontEditor }}</option>
              <option value="system">{{ messages.previewFontSystem }}</option>
              <option value="serif">{{ messages.previewFontSerif }}</option>
              <option value="mono">{{ messages.previewFontMono }}</option>
            </select>
          </label>
          <div class="settings-slider-row">
            <input
              type="range"
              min="12"
              max="22"
              :aria-label="messages.previewFontSize"
              :value="previewFontSize"
              @input="$emit('update:previewFontSize', Number(($event.target as HTMLInputElement).value))"
            />
            <strong>{{ previewFontSize }}px</strong>
          </div>
          <label class="settings-row">
            <span>{{ messages.previewLineHeight }}</span>
            <select :value="previewLineHeight" @change="$emit('update:previewLineHeight', ($event.target as HTMLSelectElement).value as PreviewLineHeight)">
              <option value="compact">{{ messages.previewLineCompact }}</option>
              <option value="standard">{{ messages.previewLineStandard }}</option>
              <option value="relaxed">{{ messages.previewLineRelaxed }}</option>
            </select>
          </label>
          <label class="settings-row">
            <span>{{ messages.previewContentWidth }}</span>
            <select :value="previewContentWidth" @change="$emit('update:previewContentWidth', ($event.target as HTMLSelectElement).value as PreviewContentWidth)">
              <option value="compact">{{ messages.previewWidthCompact }}</option>
              <option value="standard">{{ messages.previewWidthStandard }}</option>
              <option value="wide">{{ messages.previewWidthWide }}</option>
            </select>
          </label>
        </fieldset>
      </template>

      <template v-else-if="activeTab === 'insertText'">
        <label class="settings-template-row">
          <span>{{ messages.separatorText }}:</span>
          <input
            type="text"
            :value="insertSeparatorTemplate"
            @input="$emit('update:insertSeparatorTemplate', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="settings-template-row">
          <span>{{ messages.dateTimeText }}:</span>
          <input
            type="text"
            :value="insertDateTimeTemplate"
            @input="$emit('update:insertDateTimeTemplate', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="settings-template-row">
          <span>{{ messages.dateTimeSeparatorText }}:</span>
          <input
            type="text"
            :value="insertDateTimeSeparatorTemplate"
            @input="$emit('update:insertDateTimeSeparatorTemplate', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <fieldset class="settings-custom-fieldset">
          <legend>{{ messages.custom }}</legend>
          <select
            size="7"
            :value="selectedCustomIndex ?? ''"
            @change="selectedCustomIndex = Number(($event.target as HTMLSelectElement).value)"
          >
            <option v-for="(item, index) in customInsertTexts" :key="index" :value="index">
              {{ item || '(empty)' }}
            </option>
          </select>
          <div class="settings-custom-actions">
            <button type="button" @click="addCustomText(customInsertTexts)">{{ messages.add }}</button>
            <button type="button" @click="editCustomText">{{ messages.edit }}</button>
            <button type="button" @click="deleteCustomText(customInsertTexts)">{{ messages.delete }}</button>
          </div>
        </fieldset>
      </template>

      <template v-else-if="activeTab === 'advanced'">
        <fieldset class="settings-fieldset">
          <legend>{{ messages.vimSettings }}</legend>
          <p class="settings-description">{{ messages.vimModeDescription }}</p>
          <label class="settings-check-row">
            <input
              :checked="vimMode"
              type="checkbox"
              @change="$emit('update:vimMode', ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ messages.vimMode }}</span>
          </label>
          <label class="settings-check-row">
            <input
              :checked="vimUseCtrlShortcuts"
              :disabled="!vimMode"
              type="checkbox"
              @change="$emit('update:vimUseCtrlShortcuts', ($event.target as HTMLInputElement).checked)"
            />
            <span>{{ messages.vimUseCtrlShortcuts }}</span>
          </label>
          <label class="settings-template-row vim-exit-row">
            <span>{{ messages.vimInsertExitKey }}:</span>
            <input
              type="text"
              maxlength="8"
              :disabled="!vimMode"
              :value="vimInsertExitKey"
              @input="$emit('update:vimInsertExitKey', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <small class="settings-hint vim-exit-hint">{{ messages.vimModeHint }}</small>
        </fieldset>
      </template>

      <template v-else-if="activeTab === 'ai'">
        <AiSettingsSection
          :config="aiConfig"
          :error="aiError"
          :testing="aiTesting"
          :test-succeeded="aiTestSucceeded"
          :messages="messages"
          @update-config="$emit('update-ai-config', $event)"
          @save-api-key="$emit('save-ai-api-key', $event)"
          @clear-api-key="$emit('clear-ai-api-key')"
          @test-connection="$emit('test-ai-connection')"
          @manage-prompts="$emit('manage-ai-prompts')"
          @open-prompts-folder="$emit('open-ai-prompts-folder')"
        />
      </template>

      <template v-else-if="activeTab === 'mcp'">
        <fieldset class="settings-fieldset">
          <legend>{{ messages.mcpLocalService }}</legend>
          <p class="settings-description">{{ messages.mcpDescription }}</p>
          <p class="settings-description">{{ messages.mcpStartupDescription }}</p>
          <div class="mcp-service-actions">
            <button
              type="button"
              class="mcp-primary-button"
              @click="$emit('update-mcp-enabled', !mcpStatus?.enabled)"
            >
              {{ mcpStatus?.enabled ? messages.stopMcpService : messages.startMcpService }}
            </button>
            <strong class="mcp-status" :data-running="mcpStatus?.running ? 'true' : 'false'">
              {{ mcpStatus?.status || messages.stopped }}
            </strong>
          </div>
          <div class="settings-form-row">
            <span>{{ messages.address }}:</span>
            <code>{{ mcpStatus?.url || 'http://127.0.0.1:8765/mcp' }}</code>
          </div>
          <div class="settings-form-row">
            <span>{{ messages.accessToken }}:</span>
            <code class="settings-secret">{{ mcpStatus?.token || messages.tokenPending }}</code>
          </div>
          <p v-if="mcpError || mcpStatus?.lastError" class="settings-error">{{ mcpError || mcpStatus?.lastError }}</p>
          <div class="settings-actions">
            <button type="button" :disabled="!mcpStatus?.token" @click="$emit('copy-mcp-config')">
              {{ messages.copyAgentConfig }}
            </button>
            <button type="button" @click="$emit('regenerate-mcp-token')">
              {{ messages.regenerateToken }}
            </button>
          </div>
        </fieldset>

        <fieldset class="settings-fieldset">
          <legend>{{ messages.installMethod }}</legend>
          <p class="settings-description">{{ messages.installMethodDescription }}</p>
          <pre class="settings-code-block">{{ mcpAgentConfig(mcpStatus) }}</pre>
        </fieldset>
      </template>
      <template v-else>
        <section class="settings-about">
          <div class="settings-about-mark" aria-hidden="true">
            <svg viewBox="0 0 24 24"><path d="M6 3.5h9.2L19 7.3v13.2H6zM15 3.5v4h4M9 11.2h6M9 14.5h6M9 17.8h4" /></svg>
          </div>
          <div>
            <h3>NeoPad</h3>
            <p>{{ messages.aboutDescription }}</p>
          </div>
          <dl class="settings-about-details">
            <div><dt>{{ messages.version }}</dt><dd>{{ appVersion ? `v${appVersion}` : '—' }}</dd></div>
            <div><dt>{{ messages.author }}</dt><dd>TrevanZhang</dd></div>
            <div><dt>{{ messages.openSource }}</dt><dd>github.com/trevanzhang/neopad</dd></div>
            <div><dt>{{ messages.license }}</dt><dd>MIT License</dd></div>
            <div><dt>{{ messages.builtWith }}</dt><dd>Tauri 2 · Vue 3 · TypeScript · Rust</dd></div>
          </dl>
        </section>
      </template>
      </div>
    </section>
  </aside>
</template>
