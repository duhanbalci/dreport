<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import PropSection from './shared/PropSection.vue'
import PropNumberInput from './shared/PropNumberInput.vue'
import type { TemplateElement, SizeValue } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: TemplateElement }>()
const templateStore = useTemplateStore()

const sizeOptions = [
  { value: 'auto', label: 'Otomatik' },
  { value: 'fixed', label: 'Sabit (mm)' },
  { value: 'fr', label: 'Oran (fr)' },
]

function updateSize(axis: 'width' | 'height', sv: SizeValue) {
  templateStore.updateElementSize(props.element.id, { [axis]: sv })
}

function onTypeChange(axis: 'width' | 'height', type: string) {
  if (type === 'auto') updateSize(axis, { type: 'auto' })
  else if (type === 'fr') updateSize(axis, { type: 'fr', value: 1 })
  else updateSize(axis, { type: 'fixed', value: axis === 'width' ? 50 : 20 })
}
</script>

<template>
  <PropSection title="Boyut">
    <div class="prop-row" data-tip="Genislik boyutlandirma modu">
      <label class="prop-label">Genislik</label>
      <select
        class="prop-input prop-select"
        :value="element.size.width.type"
        @change="(e) => onTypeChange('width', (e.target as HTMLSelectElement).value)"
      >
        <option v-for="opt in sizeOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </div>
    <PropNumberInput
      v-if="element.size.width.type === 'fixed'"
      label="mm"
      :model-value="(element.size.width as any).value"
      :step="1"
      :min="1"
      data-tip="Sabit genislik degeri (mm)"
      @update:model-value="(v) => updateSize('width', { type: 'fixed', value: v })"
    />
    <PropNumberInput
      v-if="element.size.width.type === 'fr'"
      label="fr"
      :model-value="(element.size.width as any).value"
      :step="1"
      :min="1"
      data-tip="Kalan alani oransal doldurma degeri"
      @update:model-value="(v) => updateSize('width', { type: 'fr', value: v })"
    />

    <div class="prop-row" data-tip="Yukseklik boyutlandirma modu">
      <label class="prop-label">Yukseklik</label>
      <select
        class="prop-input prop-select"
        :value="element.size.height.type"
        @change="(e) => onTypeChange('height', (e.target as HTMLSelectElement).value)"
      >
        <option v-for="opt in sizeOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </div>
    <PropNumberInput
      v-if="element.size.height.type === 'fixed'"
      label="mm"
      :model-value="(element.size.height as any).value"
      :step="1"
      :min="1"
      data-tip="Sabit yukseklik degeri (mm)"
      @update:model-value="(v) => updateSize('height', { type: 'fixed', value: v })"
    />
  </PropSection>
</template>
