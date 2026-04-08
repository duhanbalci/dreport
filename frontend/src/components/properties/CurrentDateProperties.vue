<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropTextStyleGroup from './shared/PropTextStyleGroup.vue'
import type { CurrentDateElement, TextStyle } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: CurrentDateElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const style = () => props.element.style as TextStyle

const formatOptions = [
  { value: 'DD.MM.YYYY', label: '30.03.2026' },
  { value: 'DD/MM/YYYY', label: '30/03/2026' },
  { value: 'YYYY-MM-DD', label: '2026-03-30' },
  { value: 'DD.MM.YYYY HH:mm', label: '30.03.2026 14:30' },
]
</script>

<template>
  <PropSection title="Tarih">
    <PropSelect
      label="Format"
      :model-value="element.format ?? 'DD.MM.YYYY'"
      :options="formatOptions"
      data-tip="Tarih gosterim formati"
      @update:model-value="(v) => update({ format: v } as any)"
    />
    <PropTextStyleGroup
      :font-size="style().fontSize ?? 10"
      :font-weight="style().fontWeight ?? 'normal'"
      :font-family="style().fontFamily"
      :color="style().color ?? '#666666'"
      :align="style().align ?? 'left'"
      @update:font-size="(v) => updateStyle('fontSize', v)"
      @update:font-weight="(v) => updateStyle('fontWeight', v)"
      @update:font-family="(v) => updateStyle('fontFamily', v)"
      @update:color="(v) => updateStyle('color', v)"
      @update:align="(v) => updateStyle('align', v)"
    />
  </PropSection>
</template>
