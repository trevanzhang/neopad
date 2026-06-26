<script setup lang="ts">
import { computed } from 'vue'
import { renderMarkdown } from '../lib/markdown'

const props = defineProps<{
  content: string
}>()

const rendered = computed(() => renderMarkdown(props.content))
</script>

<template>
  <section class="preview-pane" aria-label="Markdown preview">
    <article class="markdown-preview" v-html="rendered" />
  </section>
</template>
