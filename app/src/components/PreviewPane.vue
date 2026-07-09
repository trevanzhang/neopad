<script setup lang="ts">
import { computed } from 'vue'
import { renderMarkdown } from '../lib/markdown'
import type { PreviewContentWidth, PreviewFontFamily, PreviewLineHeight, PreviewTheme } from '../types/editor'

const props = defineProps<{
  content: string
  editorFontFamily: string
  previewTheme: PreviewTheme
  previewFontFamily: PreviewFontFamily
  previewFontSize: number
  previewLineHeight: PreviewLineHeight
  previewContentWidth: PreviewContentWidth
}>()

const rendered = computed(() => renderMarkdown(props.content))

const previewStyle = computed(() => ({
  '--np-preview-font': previewFontFamily(props.previewFontFamily, props.editorFontFamily),
  '--np-preview-font-size': `${props.previewFontSize}px`,
  '--np-preview-line-height': previewLineHeight(props.previewLineHeight),
  '--np-preview-width': previewContentWidth(props.previewContentWidth),
}))

function previewFontFamily(fontFamily: PreviewFontFamily, editorFontFamily: string) {
  if (fontFamily === 'editor') return editorFontFamily
  if (fontFamily === 'serif') return 'Georgia, "Times New Roman", serif'
  if (fontFamily === 'mono') return '"JetBrains Mono", Consolas, "Courier New", monospace'
  return '"Segoe UI", Arial, sans-serif'
}

function previewLineHeight(lineHeight: PreviewLineHeight) {
  if (lineHeight === 'compact') return '1.45'
  if (lineHeight === 'relaxed') return '1.8'
  return '1.62'
}

function previewContentWidth(contentWidth: PreviewContentWidth) {
  if (contentWidth === 'compact') return '64ch'
  if (contentWidth === 'wide') return '96ch'
  return '76ch'
}
</script>

<template>
  <section class="preview-pane" :data-preview-theme="previewTheme" :style="previewStyle" aria-label="Markdown preview">
    <article class="markdown-preview" v-html="rendered" />
  </section>
</template>
