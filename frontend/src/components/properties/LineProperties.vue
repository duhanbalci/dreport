<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import PropSection from './shared/PropSection.vue'
import PropNumberInput from './shared/PropNumberInput.vue'
import PropColorInput from './shared/PropColorInput.vue'
import type { LineElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: LineElement }>()
const { updateStyle } = usePropertyUpdate(() => props.element)
</script>

<template>
  <PropSection title="Cizgi Stili">
    <PropNumberInput
      label="Kalinlik (mm)"
      :model-value="element.style.strokeWidth ?? 0.5"
      :step="0.1"
      :min="0.1"
      data-tip="Cizgi kalinligi (mm)"
      @update:model-value="(v) => updateStyle('strokeWidth', v)"
    />
    <PropColorInput
      label="Renk"
      :model-value="element.style.strokeColor ?? '#000000'"
      data-tip="Cizgi rengi"
      @update:model-value="(v) => updateStyle('strokeColor', v)"
    />
  </PropSection>
</template>
