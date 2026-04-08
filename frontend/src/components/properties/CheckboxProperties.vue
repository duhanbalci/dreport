<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import PropSection from './shared/PropSection.vue'
import PropNumberInput from './shared/PropNumberInput.vue'
import PropColorInput from './shared/PropColorInput.vue'
import PropCheckbox from './shared/PropCheckbox.vue'
import type { CheckboxElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: CheckboxElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
</script>

<template>
  <PropSection title="Onay Kutusu">
    <PropCheckbox
      v-if="!element.binding"
      label="Isaretli"
      :model-value="element.checked ?? false"
      data-tip="Onay kutusunun varsayilan durumu"
      @update:model-value="(v) => update({ checked: v } as any)"
    />
    <PropNumberInput
      label="Boyut (mm)"
      :model-value="element.style.size ?? 4"
      :step="0.5"
      :min="1"
      data-tip="Onay kutusu boyutu (mm)"
      @update:model-value="(v) => updateStyle('size', v)"
    />
    <PropColorInput
      label="Isaret Rengi"
      :model-value="element.style.checkColor ?? '#000000'"
      data-tip="Isaret (tik) rengi"
      @update:model-value="(v) => updateStyle('checkColor', v)"
    />
    <PropColorInput
      label="Kenar Rengi"
      :model-value="element.style.borderColor ?? '#333333'"
      data-tip="Kutu kenarlik rengi"
      @update:model-value="(v) => updateStyle('borderColor', v)"
    />
  </PropSection>
</template>
