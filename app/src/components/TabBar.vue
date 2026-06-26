<script setup lang="ts">
import type { NoteTab } from '../types/note'

defineProps<{
  tabs: NoteTab[]
  activeTabId: string
}>()

defineEmits<{
  selectTab: [tabId: string]
  newTab: []
}>()
</script>

<template>
  <nav class="tab-bar" aria-label="Pages">
    <button class="tab-menu" type="button" title="Pages" aria-label="Pages">≡</button>
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="tab-item"
      :class="{ active: tab.id === activeTabId }"
      type="button"
      @click="$emit('selectTab', tab.id)"
    >
      {{ tab.title }}
    </button>
    <button class="tab-add" type="button" title="New note" aria-label="New note" @click="$emit('newTab')">
      +
    </button>
    <div class="tab-scroll-buttons" aria-hidden="true">
      <span>‹</span>
      <span>›</span>
    </div>
  </nav>
</template>
