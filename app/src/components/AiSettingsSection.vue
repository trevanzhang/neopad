<script setup lang="ts">
import { ref } from 'vue'
import type { AppMessages } from '../lib/i18n'
import type { AiConfig } from '../types/ai'

defineProps<{
  config: AiConfig
  error: string | null
  testing: boolean
  testSucceeded: boolean
  messages: AppMessages['settings']
}>()

const emit = defineEmits<{
  updateConfig: [patch: Partial<Omit<AiConfig, 'apiKeyConfigured'>>]
  saveApiKey: [apiKey: string]
  clearApiKey: []
  testConnection: []
  managePrompts: []
  openPromptsFolder: []
}>()

const apiKeyDraft = ref('')

function saveKey() {
  const key = apiKeyDraft.value.trim()
  if (!key) return
  emit('saveApiKey', key)
}
</script>

<template>
  <fieldset class="settings-fieldset ai-settings-section">
    <legend>{{ messages.aiInline }}</legend>
    <p class="settings-description">{{ messages.aiDescription }}</p>
    <label class="settings-check-row">
      <input
        type="checkbox"
        :checked="config.enabled"
        @change="emit('updateConfig', { enabled: ($event.target as HTMLInputElement).checked })"
      />
      <span>{{ messages.enableAi }}</span>
    </label>
  </fieldset>

  <fieldset class="settings-fieldset ai-settings-section">
    <legend>{{ messages.promptLibrarySection }}</legend>
    <p class="settings-description">{{ messages.promptLibraryDescription }}</p>
    <div class="ai-test-row">
      <button type="button" @click="emit('managePrompts')">{{ messages.managePrompts }}</button>
      <button type="button" @click="emit('openPromptsFolder')">{{ messages.openPromptsFolder }}</button>
    </div>
  </fieldset>

  <fieldset class="settings-fieldset ai-settings-section">
    <label class="ai-settings-field">
      <span>{{ messages.serviceUrl }}</span>
      <input
        type="url"
        spellcheck="false"
        placeholder="https://example.com/v1"
        :value="config.baseUrl"
        @input="emit('updateConfig', { baseUrl: ($event.target as HTMLInputElement).value })"
      />
      <small>{{ messages.serviceUrlHint }}</small>
    </label>

    <label class="ai-settings-field">
      <span>{{ messages.modelName }}</span>
      <input
        type="text"
        spellcheck="false"
        placeholder="model-name"
        :value="config.model"
        @input="emit('updateConfig', { model: ($event.target as HTMLInputElement).value })"
      />
    </label>

    <div class="ai-settings-field">
      <span>{{ messages.apiKey }}</span>
      <div class="ai-secret-row">
        <input
          v-model="apiKeyDraft"
          type="password"
          autocomplete="off"
          spellcheck="false"
          :placeholder="messages.apiKeyPlaceholder"
          @keydown.enter.prevent="saveKey"
        />
        <button type="button" :disabled="!apiKeyDraft.trim()" @click="saveKey">{{ messages.saveKey }}</button>
        <button type="button" :disabled="!config.apiKeyConfigured" @click="emit('clearApiKey')">{{ messages.clearKey }}</button>
      </div>
      <small :data-configured="config.apiKeyConfigured ? 'true' : 'false'">
        {{ config.apiKeyConfigured ? messages.keyConfigured : messages.keyNotConfigured }}
      </small>
    </div>
  </fieldset>

  <fieldset class="settings-fieldset ai-settings-section">
    <div class="ai-test-row">
      <button type="button" :disabled="testing" @click="emit('testConnection')">
        {{ testing ? messages.testingConnection : messages.testConnection }}
      </button>
      <strong v-if="testSucceeded" class="ai-test-success">{{ messages.connectionOk }}</strong>
    </div>
    <p v-if="error" class="settings-error">{{ error }}</p>
    <p class="settings-description ai-privacy-note">{{ messages.aiPrivacy }}</p>
  </fieldset>
</template>
