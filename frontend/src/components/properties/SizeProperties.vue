<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import type { TemplateElement, SizeValue } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: TemplateElement }>()
const templateStore = useTemplateStore()

function updateSize(axis: 'width' | 'height', sv: SizeValue) {
  templateStore.updateElementSize(props.element.id, { [axis]: sv })
}
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Boyut</div>
    <div class="prop-row">
      <label class="prop-label">Genislik</label>
      <select class="prop-input prop-select"
        :value="element.size.width.type"
        @change="(e) => {
          const t = (e.target as HTMLSelectElement).value
          if (t === 'auto') updateSize('width', { type: 'auto' })
          else if (t === 'fr') updateSize('width', { type: 'fr', value: 1 })
          else updateSize('width', { type: 'fixed', value: 50 })
        }">
        <option value="auto">Otomatik</option>
        <option value="fixed">Sabit (mm)</option>
        <option value="fr">Oran (fr)</option>
      </select>
    </div>
    <div v-if="element.size.width.type === 'fixed'" class="prop-row">
      <label class="prop-label">mm</label>
      <input class="prop-input" type="number" step="1" min="1"
        :value="(element.size.width as any).value"
        @input="(e) => updateSize('width', { type: 'fixed', value: parseFloat((e.target as HTMLInputElement).value) || 10 })" />
    </div>
    <div v-if="element.size.width.type === 'fr'" class="prop-row">
      <label class="prop-label">fr</label>
      <input class="prop-input" type="number" step="1" min="1"
        :value="(element.size.width as any).value"
        @input="(e) => updateSize('width', { type: 'fr', value: parseFloat((e.target as HTMLInputElement).value) || 1 })" />
    </div>

    <div class="prop-row">
      <label class="prop-label">Yukseklik</label>
      <select class="prop-input prop-select"
        :value="element.size.height.type"
        @change="(e) => {
          const t = (e.target as HTMLSelectElement).value
          if (t === 'auto') updateSize('height', { type: 'auto' })
          else if (t === 'fr') updateSize('height', { type: 'fr', value: 1 })
          else updateSize('height', { type: 'fixed', value: 20 })
        }">
        <option value="auto">Otomatik</option>
        <option value="fixed">Sabit (mm)</option>
        <option value="fr">Oran (fr)</option>
      </select>
    </div>
    <div v-if="element.size.height.type === 'fixed'" class="prop-row">
      <label class="prop-label">mm</label>
      <input class="prop-input" type="number" step="1" min="1"
        :value="(element.size.height as any).value"
        @input="(e) => updateSize('height', { type: 'fixed', value: parseFloat((e.target as HTMLInputElement).value) || 10 })" />
    </div>
  </div>
</template>
