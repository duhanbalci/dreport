<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { LineElement, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: LineElement }>()
const templateStore = useTemplateStore()
const editorStore = useEditorStore()

function updateStyle(key: string, value: unknown) {
  const id = editorStore.selectedElementId
  if (!id) return
  templateStore.updateElement(id, {
    style: { ...props.element.style, [key]: value },
  } as Partial<TemplateElement>)
}
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Cizgi Stili</div>
    <div class="prop-row" data-tip="Cizgi kalinligi (mm)">
      <label class="prop-label">Kalinlik (mm)</label>
      <input
        class="prop-input"
        type="number"
        step="0.1"
        min="0.1"
        :value="element.style.strokeWidth ?? 0.5"
        @input="
          (e) => updateStyle('strokeWidth', parseFloat((e.target as HTMLInputElement).value) || 0.5)
        "
      />
    </div>
    <div class="prop-row" data-tip="Cizgi rengi">
      <label class="prop-label">Renk</label>
      <input
        class="prop-input prop-color"
        type="color"
        :value="element.style.strokeColor ?? '#000000'"
        @input="(e) => updateStyle('strokeColor', (e.target as HTMLInputElement).value)"
      />
    </div>
  </div>
</template>
