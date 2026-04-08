<script setup lang="ts">
import type { TextStyle, TemplateElement } from '../../../core/types'

const props = defineProps<{ element: TemplateElement }>()
const style = () => props.element.style as TextStyle

const emit = defineEmits<{
  updateStyle: [key: string, value: unknown]
}>()
</script>

<template>
  <!-- Bold -->
  <div class="et__group">
    <button class="et__btn" :class="{ 'et__btn--active': style().fontWeight === 'bold' }" data-tip="Kalin" @click="emit('updateStyle', 'fontWeight', style().fontWeight === 'bold' ? 'normal' : 'bold')">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M4 2.5h3.5a2.5 2.5 0 0 1 0 5H4V2.5z" stroke="currentColor" stroke-width="1.5" fill="none" /><path d="M4 7.5h4a2.5 2.5 0 0 1 0 5H4V7.5z" stroke="currentColor" stroke-width="1.5" fill="none" /></svg>
    </button>
  </div>

  <div class="et__sep" />

  <!-- Align -->
  <div class="et__group">
    <button class="et__btn" :class="{ 'et__btn--active': (style().align ?? 'left') === 'left' }" data-tip="Sola Hizala" @click="emit('updateStyle', 'align', 'left')">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><line x1="2" y1="3" x2="12" y2="3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /><line x1="2" y1="7" x2="9" y2="7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /><line x1="2" y1="11" x2="11" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
    </button>
    <button class="et__btn" :class="{ 'et__btn--active': style().align === 'center' }" data-tip="Ortala" @click="emit('updateStyle', 'align', 'center')">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><line x1="2" y1="3" x2="12" y2="3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /><line x1="3.5" y1="7" x2="10.5" y2="7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /><line x1="2.5" y1="11" x2="11.5" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
    </button>
    <button class="et__btn" :class="{ 'et__btn--active': style().align === 'right' }" data-tip="Saga Hizala" @click="emit('updateStyle', 'align', 'right')">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><line x1="2" y1="3" x2="12" y2="3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /><line x1="5" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /><line x1="3" y1="11" x2="12" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
    </button>
  </div>

  <div class="et__sep" />

  <!-- Font size -->
  <div class="et__group et__group--gap">
    <svg class="et__gap-icon" width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 10L6 2l4 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" fill="none" /><line x1="3.5" y1="7" x2="8.5" y2="7" stroke="currentColor" stroke-width="1" stroke-linecap="round" /></svg>
    <input type="number" class="et__num" step="1" min="1" :value="style().fontSize ?? 11" @input="(e) => emit('updateStyle', 'fontSize', parseFloat((e.target as HTMLInputElement).value) || 11)" data-tip="Yazi Boyutu (pt)" />
  </div>

  <div class="et__sep" />

  <!-- Color -->
  <div class="et__group">
    <label class="et__color-wrap" data-tip="Renk">
      <input type="color" class="et__color" :value="style().color ?? '#000000'" @input="(e) => emit('updateStyle', 'color', (e.target as HTMLInputElement).value)" />
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="11" width="10" height="2" rx="0.5" :fill="style().color ?? '#000000'" /><path d="M5 9L7 3l2 6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" fill="none" /><line x1="5.5" y1="7.5" x2="8.5" y2="7.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" /></svg>
    </label>
  </div>
</template>
