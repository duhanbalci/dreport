<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropTextStyleGroup from './shared/PropTextStyleGroup.vue'
import type { PageNumberElement, TextStyle } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: PageNumberElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const style = () => props.element.style as TextStyle

const formatOptions = [
  { value: '{current} / {total}', label: '1 / 5' },
  { value: '{current}', label: '1' },
  { value: 'Sayfa {current}', label: 'Sayfa 1' },
  { value: 'Sayfa {current} / {total}', label: 'Sayfa 1 / 5' },
]
</script>

<template>
  <PropSection title="Sayfa Numarasi">
    <PropSelect
      label="Format"
      :model-value="element.format ?? '{current} / {total}'"
      :options="formatOptions"
      data-tip="Sayfa numarasi gosterim formati"
      @update:model-value="(v) => update({ format: v } as any)"
    />
    <PropTextStyleGroup
      :font-size="style().fontSize ?? 10"
      :color="style().color ?? '#666666'"
      :align="style().align ?? 'center'"
      :show-weight="false"
      @update:font-size="(v) => updateStyle('fontSize', v)"
      @update:color="(v) => updateStyle('color', v)"
      @update:align="(v) => updateStyle('align', v)"
    />
  </PropSection>
</template>
