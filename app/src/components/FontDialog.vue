<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'

type FontPreset = {
  label: string
  value: string
}

const fontPresets: FontPreset[] = [
  { label: 'JetBrains Mono', value: '"JetBrains Mono", Consolas, "Courier New", monospace' },
  { label: 'SF Mono', value: '"SF Mono", "Cascadia Mono", Consolas, monospace' },
  { label: 'Menlo', value: 'Menlo, Consolas, "Courier New", monospace' },
  { label: 'Consolas', value: 'Consolas, "Courier New", monospace' },
  { label: 'System UI', value: '"Segoe UI", Arial, sans-serif' },
]

const props = defineProps<{
  title: string
  fieldLabel: string
  sizeLabel: string
  sampleText: string
  fontFamily: string
  fontSize: number
  confirmLabel: string
  cancelLabel: string
}>()

const emit = defineEmits<{
  confirm: [value: { fontFamily: string; fontSize: number }]
  cancel: []
}>()

const select = ref<HTMLSelectElement | null>(null)
const selectedFont = ref(resolveInitialFont(props.fontFamily))
const selectedSize = ref(clampFontSize(props.fontSize))

const previewStyle = computed(() => ({
  fontFamily: selectedFont.value,
  fontSize: `${selectedSize.value}px`,
}))

function resolveInitialFont(value: string) {
  return fontPresets.find((font) => font.value === value)?.value ?? fontPresets[4].value
}

function clampFontSize(value: number) {
  return Math.min(22, Math.max(12, Number(value) || 14))
}

function confirmFont() {
  emit('confirm', { fontFamily: selectedFont.value, fontSize: selectedSize.value })
}

onMounted(() => {
  void nextTick(() => {
    select.value?.focus()
  })
})
</script>

<template>
  <div class="input-dialog-backdrop" role="presentation" @mousedown.self="$emit('cancel')">
    <form class="input-dialog font-dialog" role="dialog" aria-modal="true" :aria-label="title" @submit.prevent="confirmFont">
      <header class="input-dialog-header">
        <strong>{{ title }}</strong>
        <button type="button" :aria-label="cancelLabel" :title="cancelLabel" @click="$emit('cancel')">&times;</button>
      </header>
      <div class="input-dialog-body font-dialog-body">
        <div class="font-dialog-controls">
          <label class="font-dialog-field">
            <span>{{ fieldLabel }}</span>
            <select ref="select" v-model="selectedFont">
              <option v-for="font in fontPresets" :key="font.label" :value="font.value">
                {{ font.label }}
              </option>
            </select>
          </label>
          <label class="font-dialog-field font-dialog-size-field">
            <span>{{ sizeLabel }}</span>
            <strong>{{ selectedSize }}px</strong>
            <input v-model.number="selectedSize" type="range" min="12" max="22" step="1" />
          </label>
        </div>
        <div class="font-dialog-preview-shell">
          <div class="font-dialog-preview" :style="previewStyle">
            {{ sampleText }}
          </div>
        </div>
      </div>
      <footer class="input-dialog-actions">
        <button type="submit" class="primary">{{ confirmLabel }}</button>
        <button type="button" @click="$emit('cancel')">{{ cancelLabel }}</button>
      </footer>
    </form>
  </div>
</template>
