<script setup lang="ts">
import type { ContainerElement } from '../../../core/types'

const props = defineProps<{ container: ContainerElement }>()

const emit = defineEmits<{
  update: [updates: Record<string, unknown>]
}>()
</script>

<template>
  <!-- Direction -->
  <div class="et__group">
    <button class="et__btn" :class="{ 'et__btn--active': container.direction === 'column' }" data-tip="Dikey" @click="emit('update', { direction: 'column' })">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="3" rx="0.5" fill="currentColor" /><rect x="2" y="5.5" width="10" height="3" rx="0.5" fill="currentColor" /><rect x="2" y="10" width="10" height="3" rx="0.5" fill="currentColor" /></svg>
    </button>
    <button class="et__btn" :class="{ 'et__btn--active': container.direction === 'row' }" data-tip="Yatay" @click="emit('update', { direction: 'row' })">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="3" height="10" rx="0.5" fill="currentColor" /><rect x="5.5" y="2" width="3" height="10" rx="0.5" fill="currentColor" /><rect x="10" y="2" width="3" height="10" rx="0.5" fill="currentColor" /></svg>
    </button>
  </div>

  <div class="et__sep" />

  <!-- Align -->
  <div class="et__group">
    <template v-if="container.direction === 'column'">
      <button class="et__btn" :class="{ 'et__btn--active': container.align === 'start' }" data-tip="Sol" @click="emit('update', { align: 'start' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3.5" y="3" width="8" height="2.5" rx="0.5" fill="currentColor" /><rect x="3.5" y="8" width="5" height="2.5" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.align === 'center' }" data-tip="Orta" @click="emit('update', { align: 'center' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="6.25" y="1" width="1.5" height="12" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="3" width="8" height="2.5" rx="0.5" fill="currentColor" /><rect x="4.5" y="8" width="5" height="2.5" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.align === 'end' }" data-tip="Sag" @click="emit('update', { align: 'end' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="11.5" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="2.5" y="3" width="8" height="2.5" rx="0.5" fill="currentColor" /><rect x="5.5" y="8" width="5" height="2.5" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.align === 'stretch' }" data-tip="Esnet" @click="emit('update', { align: 'stretch' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="11.5" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3.5" y="3" width="7" height="2.5" rx="0.5" fill="currentColor" /><rect x="3.5" y="8" width="7" height="2.5" rx="0.5" fill="currentColor" /></svg>
      </button>
    </template>
    <template v-else>
      <button class="et__btn" :class="{ 'et__btn--active': container.align === 'start' }" data-tip="Ust" @click="emit('update', { align: 'start' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="3.5" width="2.5" height="8" rx="0.5" fill="currentColor" /><rect x="8" y="3.5" width="2.5" height="5" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.align === 'center' }" data-tip="Orta" @click="emit('update', { align: 'center' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="6.25" width="12" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="2" width="2.5" height="10" rx="0.5" fill="currentColor" /><rect x="8" y="3.5" width="2.5" height="7" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.align === 'end' }" data-tip="Alt" @click="emit('update', { align: 'end' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="11.5" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="2.5" width="2.5" height="8" rx="0.5" fill="currentColor" /><rect x="8" y="5.5" width="2.5" height="5" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.align === 'stretch' }" data-tip="Esnet" @click="emit('update', { align: 'stretch' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="2" y="11.5" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="3.5" width="2.5" height="7" rx="0.5" fill="currentColor" /><rect x="8" y="3.5" width="2.5" height="7" rx="0.5" fill="currentColor" /></svg>
      </button>
    </template>
  </div>

  <div class="et__sep" />

  <!-- Justify -->
  <div class="et__group">
    <template v-if="container.direction === 'column'">
      <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'start' }" data-tip="Ust" @click="emit('update', { justify: 'start' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="3.5" width="8" height="2" rx="0.5" fill="currentColor" /><rect x="3" y="6.5" width="8" height="2" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'center' }" data-tip="Orta" @click="emit('update', { justify: 'center' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="6.25" width="12" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="3" width="8" height="2" rx="0.5" fill="currentColor" /><rect x="3" y="9" width="8" height="2" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'end' }" data-tip="Alt" @click="emit('update', { justify: 'end' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="11.5" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="5.5" width="8" height="2" rx="0.5" fill="currentColor" /><rect x="3" y="8.5" width="8" height="2" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'space-between' }" data-tip="Esit Aralik" @click="emit('update', { justify: 'space-between' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="2" y="11.5" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3" y="3.5" width="8" height="2" rx="0.5" fill="currentColor" /><rect x="3" y="8.5" width="8" height="2" rx="0.5" fill="currentColor" /></svg>
      </button>
    </template>
    <template v-else>
      <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'start' }" data-tip="Sol" @click="emit('update', { justify: 'start' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3.5" y="3" width="3" height="8" rx="0.5" fill="currentColor" /><rect x="7.5" y="3" width="3" height="8" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'center' }" data-tip="Orta" @click="emit('update', { justify: 'center' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="6.25" y="1" width="1.5" height="12" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="2" y="3" width="3" height="8" rx="0.5" fill="currentColor" /><rect x="9" y="3" width="3" height="8" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'end' }" data-tip="Sag" @click="emit('update', { justify: 'end' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="11.5" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3.5" y="3" width="3" height="8" rx="0.5" fill="currentColor" /><rect x="7.5" y="3" width="3" height="8" rx="0.5" fill="currentColor" /></svg>
      </button>
      <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'space-between' }" data-tip="Esit Aralik" @click="emit('update', { justify: 'space-between' })">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="11.5" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="3.5" y="3" width="3" height="8" rx="0.5" fill="currentColor" /><rect x="7.5" y="3" width="3" height="8" rx="0.5" fill="currentColor" /></svg>
      </button>
    </template>
  </div>

  <div class="et__sep" />

  <!-- Gap -->
  <div class="et__group et__group--gap" data-tip="Bosluk (mm)">
    <svg class="et__gap-icon" width="12" height="12" viewBox="0 0 12 12" fill="none"><rect x="1" y="1" width="3.5" height="10" rx="0.5" stroke="currentColor" stroke-width="1" fill="none" /><rect x="7.5" y="1" width="3.5" height="10" rx="0.5" stroke="currentColor" stroke-width="1" fill="none" /><line x1="6" y1="3" x2="6" y2="9" stroke="currentColor" stroke-width="1" stroke-dasharray="1.5 1" /></svg>
    <input type="number" class="et__num" step="1" min="0" :value="container.gap" @input="(e) => emit('update', { gap: parseFloat((e.target as HTMLInputElement).value) || 0 })" />
  </div>
</template>
