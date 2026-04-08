<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropNumberInput from './shared/PropNumberInput.vue'
import PropColorInput from './shared/PropColorInput.vue'
import PaddingBox from './PaddingBox.vue'
import type { ContainerElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ContainerElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)

const directionOptions = [
  { value: 'column', label: 'Dikey' },
  { value: 'row', label: 'Yatay' },
]

const breakOptions = [
  { value: 'auto', label: 'Izin Ver' },
  { value: 'avoid', label: 'Bolme' },
]

const borderStyleOptions = [
  { value: 'solid', label: 'Duz' },
  { value: 'dashed', label: 'Kesikli' },
  { value: 'dotted', label: 'Noktali' },
]
</script>

<template>
  <PropSection title="Container Ayarlari">
    <PropSelect
      label="Yon"
      :model-value="element.direction"
      :options="directionOptions"
      data-tip="Cocuk elemanlarin dizilim yonu"
      @update:model-value="(v) => update({ direction: v } as any)"
    />
    <PropNumberInput
      label="Bosluk (mm)"
      :model-value="element.gap"
      :step="1"
      :min="0"
      data-tip="Cocuk elemanlar arasi bosluk (mm)"
      @update:model-value="(v) => update({ gap: v } as any)"
    />

    <div class="prop-row" data-tip="Cocuklarin cross-axis hizalamasi">
      <label class="prop-label">{{
        element.direction === 'column' ? 'Yatay Hizalama' : 'Dikey Hizalama'
      }}</label>
      <select
        class="prop-input prop-select"
        :value="element.align"
        @change="(e) => update({ align: (e.target as HTMLSelectElement).value } as any)"
      >
        <option value="start">{{ element.direction === 'column' ? 'Sol' : 'Ust' }}</option>
        <option value="center">Orta</option>
        <option value="end">{{ element.direction === 'column' ? 'Sag' : 'Alt' }}</option>
        <option value="stretch">Esnet</option>
      </select>
    </div>
    <div class="prop-row" data-tip="Cocuklarin main-axis dagilimi">
      <label class="prop-label">{{
        element.direction === 'column' ? 'Dikey Dagilim' : 'Yatay Dagilim'
      }}</label>
      <select
        class="prop-input prop-select"
        :value="element.justify"
        @change="(e) => update({ justify: (e.target as HTMLSelectElement).value } as any)"
      >
        <option value="start">{{ element.direction === 'column' ? 'Ust' : 'Sol' }}</option>
        <option value="center">Orta</option>
        <option value="end">{{ element.direction === 'column' ? 'Alt' : 'Sag' }}</option>
        <option value="space-between">Esit Aralik</option>
      </select>
    </div>

    <div class="prop-section__subtitle">Padding (mm)</div>
    <PaddingBox
      :top="element.padding.top"
      :right="element.padding.right"
      :bottom="element.padding.bottom"
      :left="element.padding.left"
      @update="(side, value) => update({ padding: { ...element.padding, [side]: value } } as any)"
    />

    <PropSelect
      label="Sayfa Bolme"
      :model-value="element.breakInside ?? 'auto'"
      :options="breakOptions"
      data-tip="Sayfa sonunda bolunmeyi kontrol eder"
      @update:model-value="(v) => update({ breakInside: v } as any)"
    />
  </PropSection>

  <PropSection title="Stil">
    <PropColorInput
      label="Arka plan"
      :model-value="element.style.backgroundColor"
      default-color="#ffffff"
      :clearable="true"
      data-tip="Container arka plan rengi"
      @update:model-value="(v) => updateStyle('backgroundColor', v)"
    />
    <PropNumberInput
      label="Kenarlik (mm)"
      :model-value="element.style.borderWidth ?? 0"
      :step="0.1"
      :min="0"
      data-tip="Kenarlik kalinligi (mm)"
      @update:model-value="(v) => updateStyle('borderWidth', v)"
    />
    <PropColorInput
      label="Kenarlik rengi"
      :model-value="element.style.borderColor"
      :clearable="true"
      data-tip="Kenarlik cizgisi rengi"
      @update:model-value="(v) => updateStyle('borderColor', v)"
    />
    <PropSelect
      label="Kenarlik stili"
      :model-value="element.style.borderStyle ?? 'solid'"
      :options="borderStyleOptions"
      data-tip="Kenarlik cizgi stili"
      @update:model-value="(v) => updateStyle('borderStyle', v)"
    />
    <PropNumberInput
      label="Radius (mm)"
      :model-value="element.style.borderRadius ?? 0"
      :step="0.5"
      :min="0"
      data-tip="Kose yuvarlakligi (mm)"
      @update:model-value="(v) => updateStyle('borderRadius', v)"
    />
  </PropSection>
</template>
