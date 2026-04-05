<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import type { TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: TemplateElement }>()
const templateStore = useTemplateStore()

function togglePositioning() {
  if (props.element.position.type === 'flow') {
    templateStore.updateElementPosition(props.element.id, { type: 'absolute', x: 0, y: 0 })
  } else {
    templateStore.updateElementPosition(props.element.id, { type: 'flow' })
  }
}
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Pozisyon</div>
    <div class="prop-row" data-tip="Flow: otomatik dizilim, Absolute: sabit konum">
      <label class="prop-label">Mod</label>
      <select class="prop-input prop-select" :value="element.position.type" @change="togglePositioning">
        <option value="flow">Flow</option>
        <option value="absolute">Absolute</option>
      </select>
    </div>
    <template v-if="element.position.type === 'absolute'">
      <div class="prop-row" data-tip="Yatay pozisyon — parent sol kenardan uzaklik (mm)">
        <label class="prop-label">X (mm)</label>
        <input class="prop-input" type="number" step="0.5"
          :value="element.position.x"
          @input="(e) => templateStore.updateElementPosition(element.id, { type: 'absolute', x: parseFloat((e.target as HTMLInputElement).value) || 0, y: (element.position as any).y ?? 0 })" />
      </div>
      <div class="prop-row" data-tip="Dikey pozisyon — parent ust kenardan uzaklik (mm)">
        <label class="prop-label">Y (mm)</label>
        <input class="prop-input" type="number" step="0.5"
          :value="element.position.y"
          @input="(e) => templateStore.updateElementPosition(element.id, { type: 'absolute', x: (element.position as any).x ?? 0, y: parseFloat((e.target as HTMLInputElement).value) || 0 })" />
      </div>
    </template>
  </div>
</template>
