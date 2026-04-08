<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropTextStyleGroup from './shared/PropTextStyleGroup.vue'
import DexprEditor from '../common/DexprEditor.vue'
import type { CalculatedTextElement, TextStyle } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: CalculatedTextElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const style = () => props.element.style as TextStyle

const formatOptions = [
  { value: '', label: 'Yok' },
  { value: 'currency', label: 'Para Birimi' },
  { value: 'number', label: 'Sayi' },
  { value: 'percentage', label: 'Yuzde' },
]
</script>

<template>
  <PropSection title="Hesaplanan Metin">
    <div
      class="prop-row-stack"
      data-tip="Hesaplama ifadesi (orn: toplamlar.kdv + toplamlar.araToplam)"
    >
      <label class="prop-label">Ifade</label>
      <DexprEditor
        :model-value="element.expression"
        @update:model-value="(v) => update({ expression: v } as any)"
        placeholder="toplamlar.kdv + toplamlar.araToplam"
      />
    </div>
    <PropSelect
      label="Format"
      :model-value="element.format ?? ''"
      :options="formatOptions"
      data-tip="Sonucun gosterim formati"
      @update:model-value="(v) => update({ format: v || undefined } as any)"
    />
    <PropTextStyleGroup
      :font-size="style().fontSize ?? 11"
      :font-weight="style().fontWeight ?? 'normal'"
      :font-family="style().fontFamily"
      :color="style().color ?? '#000000'"
      :align="style().align ?? 'left'"
      @update:font-size="(v) => updateStyle('fontSize', v)"
      @update:font-weight="(v) => updateStyle('fontWeight', v)"
      @update:font-family="(v) => updateStyle('fontFamily', v)"
      @update:color="(v) => updateStyle('color', v)"
      @update:align="(v) => updateStyle('align', v)"
    />
  </PropSection>
</template>
