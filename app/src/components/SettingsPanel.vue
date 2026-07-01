<script setup lang="ts">
import { ref } from 'vue'
import type { AppLanguage, AppMessages } from '../lib/i18n'
import type { EditorMode, EditorModeShortcut } from '../types/editor'

type SettingsTab = 'general' | 'shortcuts' | 'insertText' | 'advanced'
type TitleDoubleClickAction = 'none' | 'delete' | 'rename'

defineProps<{
  alwaysOnTop: boolean
  vimMode: boolean
  vimUseCtrlShortcuts: boolean
  vimInsertExitKey: string
  previewMode: EditorMode
  editorModeShortcut: EditorModeShortcut
  language: AppLanguage
  workspacePath: string
  runAtStartup: boolean
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
  'update:editorModeShortcut': [shortcut: EditorModeShortcut]
  'update:language': [language: AppLanguage]
  'update:runAtStartup': [value: boolean]
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
  editCustomText: [index: number]
  copyMcpConfig: [allowWrite: boolean]
}>()

const activeTab = ref<SettingsTab>('general')
const selectedCustomIndex = ref<number | null>(null)

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
      <button type="button" class="settings-close" :aria-label="messages.close" :title="messages.close" @click="$emit('close')">
        X
      </button>
    </header>

    <div class="settings-tabs" role="tablist">
      <button type="button" :class="{ active: activeTab === 'general' }" @click="activeTab = 'general'">
        {{ messages.general }}
      </button>
      <button type="button" :class="{ active: activeTab === 'shortcuts' }" @click="activeTab = 'shortcuts'">
        {{ messages.shortcutsTab }}
      </button>
      <button type="button" :class="{ active: activeTab === 'insertText' }" @click="activeTab = 'insertText'">
        {{ messages.insertTextTab }}
      </button>
      <button type="button" :class="{ active: activeTab === 'advanced' }" @click="activeTab = 'advanced'">
        {{ messages.advancedTab }}
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
          <span>{{ messages.previewMode }}</span>
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
          <label class="settings-form-row">
            <span>{{ messages.shortcut }}:</span>
            <select :value="editorModeShortcut" @change="$emit('update:editorModeShortcut', ($event.target as HTMLSelectElement).value as EditorModeShortcut)">
              <option value="F7">F7</option>
              <option value="Ctrl+Shift+M">Ctrl+Shift+M</option>
              <option value="disabled">{{ messages.disabled }}</option>
            </select>
          </label>
        </fieldset>
        <fieldset class="settings-fieldset settings-shortcut-fieldset">
          <legend>{{ messages.toggleWindow }}</legend>
          <label class="settings-form-row">
            <span>{{ messages.baseKey }}:</span>
            <input
              type="text"
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

      <template v-else>
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
          <label class="settings-form-row">
            <span>{{ messages.vimInsertExitKey }}:</span>
            <input
              type="text"
              maxlength="8"
              :disabled="!vimMode"
              :value="vimInsertExitKey"
              @input="$emit('update:vimInsertExitKey', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <small class="settings-hint">{{ messages.vimModeHint }}</small>
        </fieldset>
      </template>
    </div>

    <footer class="settings-footer">
      <button type="button" @click="$emit('close')">{{ messages.ok }}</button>
    </footer>
  </aside>
</template>
