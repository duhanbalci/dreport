<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { CheckboxElement, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: CheckboxElement }>()
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
    <div class="prop-section__title">Onay Kutusu</div>
    <div v-if="!element.binding" class="prop-row">
      <label class="prop-label">Isaretli</label>
      <input type="checkbox"
        :checked="element.checked ?? false"
        @change="(e) => update({ checked: (e.target as HTMLInputElement).checked } as any)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Boyut (mm)</label>
      <input class="prop-input" type="number" step="0.5" min="1"
        :value="element.style.size ?? 4"
        @input="(e) => updateStyle('size', parseFloat((e.target as HTMLInputElement).value) || 4)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Isaret Rengi</label>
      <input class="prop-input prop-color" type="color"
        :value="element.style.checkColor ?? '#000000'"
        @input="(e) => updateStyle('checkColor', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Kenar Rengi</label>
      <input class="prop-input prop-color" type="color"
        :value="element.style.borderColor ?? '#333333'"
        @input="(e) => updateStyle('borderColor', (e.target as HTMLInputElement).value)" />
    </div>
  </div>
</template>
