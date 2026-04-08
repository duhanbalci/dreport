<script setup lang="ts">
import type { TableStyle } from '../../../core/types'

defineProps<{ tableStyle: TableStyle }>()

const emit = defineEmits<{
  updateStyle: [key: string, value: unknown]
}>()
</script>

<template>
  <!-- Font size -->
  <div class="et__group et__group--gap" data-tip="Yazi Boyutu">
    <svg class="et__gap-icon" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 10L6 2l4 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" fill="none" /><line x1="3.5" y1="7" x2="8.5" y2="7" stroke="currentColor" stroke-width="1" stroke-linecap="round" /></svg>
    <input type="number" class="et__num" step="1" min="6" :value="tableStyle.fontSize ?? 10" @input="(e) => emit('updateStyle', 'fontSize', parseFloat((e.target as HTMLInputElement).value) || 10)" />
  </div>

  <div class="et__sep" />

  <!-- Header bg color -->
  <div class="et__group">
    <label class="et__color-wrap" data-tip="Header Rengi">
      <input type="color" class="et__color" :value="tableStyle.headerBg ?? '#f0f0f0'" @input="(e) => emit('updateStyle', 'headerBg', (e.target as HTMLInputElement).value)" />
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="2" width="10" height="4" rx="1" :fill="tableStyle.headerBg ?? '#f0f0f0'" stroke="#94a3b8" stroke-width="0.5" /><rect x="2" y="7" width="10" height="2" rx="0.5" fill="none" stroke="#94a3b8" stroke-width="0.5" /><rect x="2" y="10" width="10" height="2" rx="0.5" fill="none" stroke="#94a3b8" stroke-width="0.5" /></svg>
    </label>
  </div>

  <!-- Zebra color -->
  <div class="et__group">
    <label class="et__color-wrap" data-tip="Zebra Rengi">
      <input type="color" class="et__color" :value="tableStyle.zebraOdd ?? '#fafafa'" @input="(e) => emit('updateStyle', 'zebraOdd', (e.target as HTMLInputElement).value)" />
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="2" width="10" height="2.5" rx="0.5" fill="none" stroke="#94a3b8" stroke-width="0.5" /><rect x="2" y="5.5" width="10" height="2.5" rx="0.5" :fill="tableStyle.zebraOdd ?? '#fafafa'" stroke="#94a3b8" stroke-width="0.5" /><rect x="2" y="9" width="10" height="2.5" rx="0.5" fill="none" stroke="#94a3b8" stroke-width="0.5" /></svg>
    </label>
  </div>

  <div class="et__sep" />

  <!-- Border color -->
  <div class="et__group">
    <label class="et__color-wrap" data-tip="Kenarlik Rengi">
      <input type="color" class="et__color" :value="tableStyle.borderColor ?? '#cccccc'" @input="(e) => emit('updateStyle', 'borderColor', (e.target as HTMLInputElement).value)" />
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="2" width="10" height="10" rx="1" fill="none" :stroke="tableStyle.borderColor ?? '#cccccc'" stroke-width="1.5" /><line x1="2" y1="6" x2="12" y2="6" :stroke="tableStyle.borderColor ?? '#cccccc'" stroke-width="0.8" /><line x1="7" y1="2" x2="7" y2="12" :stroke="tableStyle.borderColor ?? '#cccccc'" stroke-width="0.8" /></svg>
    </label>
  </div>

  <!-- Border width -->
  <div class="et__group et__group--gap" data-tip="Kenarlik (mm)">
    <svg class="et__gap-icon" width="12" height="12" viewBox="0 0 12 12" fill="none"><rect x="1" y="1" width="10" height="10" rx="1" fill="none" stroke="currentColor" stroke-width="1.5" /></svg>
    <input type="number" class="et__num" step="0.1" min="0" :value="tableStyle.borderWidth ?? 0.5" @input="(e) => emit('updateStyle', 'borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)" />
  </div>
</template>
