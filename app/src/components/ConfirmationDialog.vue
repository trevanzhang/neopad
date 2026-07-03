<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'

defineProps<{
  title: string
  message: string
  confirmLabel: string
  cancelLabel: string
}>()

defineEmits<{
  confirm: []
  cancel: []
}>()

const cancelButton = ref<HTMLButtonElement | null>(null)

onMounted(() => {
  void nextTick(() => cancelButton.value?.focus())
})
</script>

<template>
  <div class="input-dialog-backdrop" role="presentation" @mousedown.self="$emit('cancel')">
    <form class="input-dialog" role="alertdialog" aria-modal="true" :aria-label="title" @submit.prevent="$emit('confirm')">
      <header class="input-dialog-header">
        <strong>{{ title }}</strong>
        <button type="button" :aria-label="cancelLabel" :title="cancelLabel" @click="$emit('cancel')">&times;</button>
      </header>
      <div class="input-dialog-body">
        <p class="confirmation-dialog-message">{{ message }}</p>
      </div>
      <footer class="input-dialog-actions">
        <button type="submit" class="danger">{{ confirmLabel }}</button>
        <button ref="cancelButton" type="button" @click="$emit('cancel')">{{ cancelLabel }}</button>
      </footer>
    </form>
  </div>
</template>
