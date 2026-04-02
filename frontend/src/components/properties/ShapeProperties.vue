<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { ShapeElement, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ShapeElement }>()
const templateStore = useTemplateStore()
const editorStore = useEditorStore()

function update(updates: Partial<TemplateElement>) {
  const id = editorStore.selectedElementId
  if (!id) return
  templateStore.updateElement(id, updates)
}

function updateStyle(key: string, value: unknown) {
  update({ style: { ...props.element.style, [key]: value } } as Partial<TemplateElement>)
}
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Sekil</div>
    <div class="prop-row">
      <label class="prop-label">Tip</label>
      <select class="prop-input prop-select"
        :value="element.shapeType"
        @change="(e) => update({ shapeType: (e.target as HTMLSelectElement).value } as any)">
        <option value="rectangle">Dikdortgen</option>
        <option value="rounded_rectangle">Yuvarlak Dikdortgen</option>
        <option value="ellipse">Elips</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="prop-label">Arka Plan</label>
      <input class="prop-input prop-color" type="color"
        :value="element.style.backgroundColor ?? '#f0f0f0'"
        @input="(e) => updateStyle('backgroundColor', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Kenar Rengi</label>
      <input class="prop-input prop-color" type="color"
        :value="element.style.borderColor ?? '#333333'"
        @input="(e) => updateStyle('borderColor', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Kenar Kalinligi</label>
      <input class="prop-input" type="number" step="0.25" min="0"
        :value="element.style.borderWidth ?? 0.5"
        @input="(e) => updateStyle('borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)" />
    </div>
    <div v-if="element.shapeType === 'rounded_rectangle'" class="prop-row">
      <label class="prop-label">Kose Yuvarlakligi</label>
      <input class="prop-input" type="number" step="0.5" min="0"
        :value="element.style.borderRadius ?? 2"
        @input="(e) => updateStyle('borderRadius', parseFloat((e.target as HTMLInputElement).value) || 0)" />
    </div>
  </div>
</template>
