<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'

const props = defineProps<{
  title: string
  initialValue: string
  confirmLabel: string
  cancelLabel: string
}>()

const emit = defineEmits<{
  confirm: [value: string]
  cancel: []
}>()

const value = ref(props.initialValue)
const input = ref<HTMLInputElement | null>(null)

function confirmInput(event?: KeyboardEvent) {
  if (event?.isComposing) return
  emit('confirm', value.value)
}

onMounted(() => {
  void nextTick(() => {
    input.value?.focus()
    input.value?.select()
  })
})
</script>

<template>
  <div class="input-dialog-backdrop" role="presentation" @mousedown.self="$emit('cancel')">
    <form class="input-dialog" role="dialog" aria-modal="true" :aria-label="title" @submit.prevent="confirmInput()">
      <header class="input-dialog-header">
        <strong>{{ title }}</strong>
        <button type="button" :aria-label="cancelLabel" :title="cancelLabel" @click="$emit('cancel')">&times;</button>
      </header>
      <div class="input-dialog-body">
        <label>
          <span>{{ title }}</span>
          <input ref="input" v-model="value" type="text" @keydown.enter.prevent.stop="confirmInput($event)" />
        </label>
      </div>
      <footer class="input-dialog-actions">
        <button type="submit" class="primary">{{ confirmLabel }}</button>
        <button type="button" @click="$emit('cancel')">{{ cancelLabel }}</button>
      </footer>
    </form>
  </div>
</template>
