<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropNumberInput from './shared/PropNumberInput.vue'
import PropColorInput from './shared/PropColorInput.vue'
import type { ShapeElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ShapeElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)

const shapeOptions = [
  { value: 'rectangle', label: 'Dikdortgen' },
  { value: 'rounded_rectangle', label: 'Yuvarlak Dikdortgen' },
  { value: 'ellipse', label: 'Elips' },
]

const borderStyleOptions = [
  { value: 'solid', label: 'Duz' },
  { value: 'dashed', label: 'Kesikli' },
  { value: 'dotted', label: 'Noktali' },
]
</script>

<template>
  <PropSection title="Sekil">
    <PropSelect
      label="Tip"
      :model-value="element.shapeType"
      :options="shapeOptions"
      data-tip="Sekil tipi"
      @update:model-value="(v) => update({ shapeType: v } as any)"
    />
    <PropColorInput
      label="Arka Plan"
      :model-value="element.style.backgroundColor ?? '#f0f0f0'"
      data-tip="Sekil arka plan rengi"
      @update:model-value="(v) => updateStyle('backgroundColor', v)"
    />
    <PropColorInput
      label="Kenar Rengi"
      :model-value="element.style.borderColor ?? '#333333'"
      data-tip="Kenarlik cizgisi rengi"
      @update:model-value="(v) => updateStyle('borderColor', v)"
    />
    <PropNumberInput
      label="Kenar Kalinligi"
      :model-value="element.style.borderWidth ?? 0.5"
      :step="0.25"
      :min="0"
      data-tip="Kenarlik cizgi kalinligi (mm)"
      @update:model-value="(v) => updateStyle('borderWidth', v)"
    />
    <PropSelect
      label="Kenar Stili"
      :model-value="element.style.borderStyle ?? 'solid'"
      :options="borderStyleOptions"
      data-tip="Kenarlik cizgi stili"
      @update:model-value="(v) => updateStyle('borderStyle', v)"
    />
    <PropNumberInput
      v-if="element.shapeType === 'rounded_rectangle'"
      label="Kose Yuvarlakligi"
      :model-value="element.style.borderRadius ?? 2"
      :step="0.5"
      :min="0"
      data-tip="Kose yuvarlakligi (mm)"
      @update:model-value="(v) => updateStyle('borderRadius', v)"
    />
  </PropSection>
</template>
