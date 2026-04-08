<script setup lang="ts">
import type { ChartElement, ChartType } from '../../../core/types'

defineProps<{ chart: ChartElement }>()

const emit = defineEmits<{
  update: [updates: Record<string, unknown>]
  updateStyle: [key: string, value: unknown]
}>()
</script>

<template>
  <!-- Chart type -->
  <div class="et__group">
    <button class="et__btn" :class="{ 'et__btn--active': chart.chartType === 'bar' }" data-tip="Cubuk" @click="emit('update', { chartType: 'bar' as ChartType })">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="6" width="3" height="6" rx="0.5" fill="currentColor" /><rect x="5.5" y="3" width="3" height="9" rx="0.5" fill="currentColor" /><rect x="9" y="5" width="3" height="7" rx="0.5" fill="currentColor" /></svg>
    </button>
    <button class="et__btn" :class="{ 'et__btn--active': chart.chartType === 'line' }" data-tip="Cizgi" @click="emit('update', { chartType: 'line' as ChartType })">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><polyline points="2,10 5,5 8,7 12,3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" fill="none" /><circle cx="2" cy="10" r="1.2" fill="currentColor" /><circle cx="5" cy="5" r="1.2" fill="currentColor" /><circle cx="8" cy="7" r="1.2" fill="currentColor" /><circle cx="12" cy="3" r="1.2" fill="currentColor" /></svg>
    </button>
    <button class="et__btn" :class="{ 'et__btn--active': chart.chartType === 'pie' }" data-tip="Pasta" @click="emit('update', { chartType: 'pie' as ChartType })">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M7 2a5 5 0 1 1-3.54 1.46" stroke="currentColor" stroke-width="1.3" fill="none" /><path d="M7 7V2A5 5 0 0 0 3.46 3.46Z" fill="currentColor" /></svg>
    </button>
  </div>

  <div class="et__sep" />

  <!-- Show labels -->
  <div class="et__group">
    <button class="et__btn" :class="{ 'et__btn--active': chart.labels?.show !== false }" data-tip="Etiketler" @click="emit('update', { labels: { ...chart.labels, show: chart.labels?.show === false ? true : false } })">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="8" width="3" height="4" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="5.5" y="5" width="3" height="7" rx="0.5" fill="currentColor" opacity="0.4" /><rect x="9" y="6" width="3" height="6" rx="0.5" fill="currentColor" opacity="0.4" /><text x="3.5" y="7" font-size="4" fill="currentColor" text-anchor="middle" font-weight="bold">3</text><text x="7" y="4" font-size="4" fill="currentColor" text-anchor="middle" font-weight="bold">7</text><text x="10.5" y="5" font-size="4" fill="currentColor" text-anchor="middle" font-weight="bold">5</text></svg>
    </button>
  </div>

  <div class="et__sep" />

  <!-- Show grid -->
  <div class="et__group">
    <button class="et__btn" :class="{ 'et__btn--active': chart.axis?.showGrid !== false }" data-tip="Izgara" @click="emit('update', { axis: { ...chart.axis, showGrid: chart.axis?.showGrid === false ? true : false } })">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><line x1="2" y1="3" x2="12" y2="3" stroke="currentColor" stroke-width="0.8" stroke-dasharray="2 1.5" /><line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="0.8" stroke-dasharray="2 1.5" /><line x1="2" y1="11" x2="12" y2="11" stroke="currentColor" stroke-width="0.8" stroke-dasharray="2 1.5" /></svg>
    </button>
  </div>

  <div class="et__sep" />

  <!-- Background color -->
  <div class="et__group">
    <label class="et__color-wrap" data-tip="Arka Plan">
      <input type="color" class="et__color" :value="chart.style.backgroundColor ?? '#ffffff'" @input="(e) => emit('updateStyle', 'backgroundColor', (e.target as HTMLInputElement).value)" />
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="2" width="10" height="10" rx="1.5" :fill="chart.style.backgroundColor ?? '#ffffff'" stroke="#94a3b8" stroke-width="0.8" /></svg>
    </label>
  </div>
</template>
