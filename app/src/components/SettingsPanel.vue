<script setup lang="ts">
import { ref } from 'vue'
import type { AppLanguage, AppMessages } from '../lib/i18n'
import type { EditorMode, PreviewContentWidth, PreviewFontFamily, PreviewLineHeight, PreviewTheme } from '../types/editor'

type SettingsTab = 'general' | 'preview' | 'shortcuts' | 'insertText' | 'advanced' | 'mcp'
type TitleDoubleClickAction = 'none' | 'delete' | 'rename'
type McpStatus = {
  enabled: boolean
  running: boolean
  status: string
  url: string
  token: string
  lastError: string | null
}

defineProps<{
  alwaysOnTop: boolean
  vimMode: boolean
  vimUseCtrlShortcuts: boolean
  vimInsertExitKey: string
  previewMode: EditorMode
  language: AppLanguage
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
}>()

const activeTab = ref<SettingsTab>('general')
const selectedCustomIndex = ref<number | null>(null)

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
  <aside class="settings-panel settings-dialog" :aria-label="messages.title">
    <header class="settings-header">
      <strong>{{ messages.title }}</strong>
      <button type="button" class="settings-close" :aria-label="messages.close" :title="messages.close" @click="$emit('close')"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg></button>
    </header>

    <div class="settings-tabs" role="tablist">
      <button type="button" :class="{ active: activeTab === 'general' }" @click="activeTab = 'general'">
        {{ messages.general }}
      </button>
      <button type="button" :class="{ active: activeTab === 'shortcuts' }" @click="activeTab = 'shortcuts'">
        {{ messages.shortcutsTab }}
      </button>
      <button type="button" :class="{ active: activeTab === 'preview' }" @click="activeTab = 'preview'">
        {{ messages.previewTab }}
      </button>
      <button type="button" :class="{ active: activeTab === 'insertText' }" @click="activeTab = 'insertText'">
        {{ messages.insertTextTab }}
      </button>
      <button type="button" :class="{ active: activeTab === 'advanced' }" @click="activeTab = 'advanced'">
        {{ messages.advancedTab }}
      </button>
      <button type="button" :class="{ active: activeTab === 'mcp' }" @click="activeTab = 'mcp'">
        {{ messages.mcp }}
      </button>
    </div>

    <div class="settings-content">
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
        <fieldset class="settings-fieldset settings-shortcut-fieldset">
          <legend>{{ messages.switchTabs }}</legend>
          <div class="settings-form-row">
            <span>{{ messages.shortcut }}:</span>
            <kbd>Ctrl+Tab / Ctrl+Shift+Tab</kbd>
          </div>
        </fieldset>
        <fieldset class="settings-fieldset settings-shortcut-fieldset">
          <legend>{{ messages.togglePreviewThemeShortcut }}</legend>
          <div class="settings-form-row">
            <span>{{ messages.shortcut }}:</span>
            <kbd>F7</kbd>
          </div>
        </fieldset>
        <fieldset class="settings-fieldset settings-shortcut-fieldset">
          <legend>{{ messages.toggleThemeShortcut }}</legend>
          <div class="settings-form-row">
            <span>{{ messages.shortcut }}:</span>
            <kbd>F9</kbd>
          </div>
        </fieldset>
        <fieldset class="settings-fieldset settings-shortcut-fieldset">
          <legend>{{ messages.immersiveFullscreen }}</legend>
          <div class="settings-form-row">
            <span>{{ messages.shortcut }}:</span>
            <kbd>F11</kbd>
          </div>
        </fieldset>
        <fieldset class="settings-fieldset settings-shortcut-fieldset">
          <legend>{{ messages.cycleEditorMode }}</legend>
          <div class="settings-form-row">
            <span>{{ messages.shortcut }}:</span>
            <kbd>F4</kbd>
          </div>
        </fieldset>
        <fieldset class="settings-fieldset settings-shortcut-fieldset">
          <legend>{{ messages.toggleWindow }}</legend>
          <label class="settings-form-row">
            <span>{{ messages.baseKey }}:</span>
            <input
              type="text"
              maxlength="3"
              :value="shortcutBaseKey"
              @input="$emit('update:shortcutBaseKey', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <div class="settings-form-row">
            <span>{{ messages.modifiers }}:</span>
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
        </fieldset>
        <fieldset class="settings-fieldset settings-shortcut-fieldset">
          <legend>{{ messages.saveClipboard }}</legend>
          <label class="settings-form-row">
            <span>{{ messages.baseKey }}:</span>
            <input
              type="text"
              maxlength="3"
              :value="clipboardShortcutBaseKey"
              @input="$emit('update:clipboardShortcutBaseKey', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <div class="settings-form-row">
            <span>{{ messages.modifiers }}:</span>
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
        </fieldset>
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

      <template v-else>
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
    </div>

    <footer class="settings-footer">
      <button type="button" @click="$emit('close')">{{ messages.ok }}</button>
    </footer>
  </aside>
</template>
