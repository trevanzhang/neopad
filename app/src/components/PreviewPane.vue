<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { openExternalUrl } from '../lib/invoke'
import { renderMarkdownInto } from '../lib/markdown'
import {
  markdownThemeForPreview,
  previewContentWidthCss,
  previewFontFamilyCss,
  previewLineHeightCss,
} from '../lib/preview-style'
import { isTauriRuntime } from '../lib/runtime'
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

const previewContent = ref<HTMLElement | null>(null)

const previewStyle = computed(() => ({
  '--np-preview-font': previewFontFamilyCss(props.previewFontFamily, props.editorFontFamily),
  '--np-preview-font-size': `${props.previewFontSize}px`,
  '--np-preview-line-height': previewLineHeightCss(props.previewLineHeight),
  '--np-preview-width': previewContentWidthCss(props.previewContentWidth),
}))

async function updateRenderedPreview() {
  if (!previewContent.value) return
  try {
    await renderMarkdownInto(previewContent.value, props.content, markdownThemeForPreview(props.previewTheme))
  } catch {
    // Keep the synchronously rendered Markdown visible if an optional renderer fails to load.
  }
}

onMounted(() => void updateRenderedPreview())
watch(
  () => [props.content, props.previewTheme] as const,
  () => void nextTick().then(updateRenderedPreview),
  { flush: 'post' },
)

function handlePreviewClick(event: MouseEvent) {
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
  <section class="preview-pane" :data-preview-theme="previewTheme" :style="previewStyle" aria-label="Markdown preview">
    <article ref="previewContent" class="markdown-preview" @click="handlePreviewClick" />
  </section>
</template>
