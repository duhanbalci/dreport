<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropNumberInput from './shared/PropNumberInput.vue'
import type { TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: TemplateElement }>()
const templateStore = useTemplateStore()

const positionOptions = [
  { value: 'flow', label: 'Flow' },
  { value: 'absolute', label: 'Absolute' },
]

function togglePositioning(value: string) {
  if (value === 'absolute') {
    templateStore.updateElementPosition(props.element.id, { type: 'absolute', x: 0, y: 0 })
  } else {
    templateStore.updateElementPosition(props.element.id, { type: 'flow' })
  }
}
</script>

<template>
  <PropSection title="Pozisyon">
    <PropSelect
      label="Mod"
      :model-value="element.position.type"
      :options="positionOptions"
      data-tip="Flow: otomatik dizilim, Absolute: sabit konum"
      @update:model-value="togglePositioning"
    />
    <template v-if="element.position.type === 'absolute'">
      <PropNumberInput
        label="X (mm)"
        :model-value="(element.position as any).x ?? 0"
        :step="0.5"
        data-tip="Yatay pozisyon — parent sol kenardan uzaklik (mm)"
        @update:model-value="
          (v) =>
            templateStore.updateElementPosition(element.id, {
              type: 'absolute',
              x: v,
              y: (element.position as any).y ?? 0,
            })
        "
      />
      <PropNumberInput
        label="Y (mm)"
        :model-value="(element.position as any).y ?? 0"
        :step="0.5"
        data-tip="Dikey pozisyon — parent ust kenardan uzaklik (mm)"
        @update:model-value="
          (v) =>
            templateStore.updateElementPosition(element.id, {
              type: 'absolute',
              x: (element.position as any).x ?? 0,
              y: v,
            })
        "
      />
    </template>
  </PropSection>
</template>
