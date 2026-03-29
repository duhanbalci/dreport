<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { PageNumberElement, TextStyle, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: PageNumberElement }>()
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
    <div class="prop-section__title">Sayfa Numarasi</div>
    <div class="prop-row">
      <label class="prop-label">Format</label>
      <select class="prop-input prop-select"
        :value="element.format ?? '{current} / {total}'"
        @change="(e) => update({ format: (e.target as HTMLSelectElement).value } as any)">
        <option value="{current} / {total}">1 / 5</option>
        <option value="{current}">1</option>
        <option value="Sayfa {current}">Sayfa 1</option>
        <option value="Sayfa {current} / {total}">Sayfa 1 / 5</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="prop-label">Boyut (pt)</label>
      <input class="prop-input" type="number" step="1" min="1"
        :value="(element.style as TextStyle).fontSize ?? 10"
        @input="(e) => updateStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 10)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Renk</label>
      <input class="prop-input prop-color" type="color"
        :value="(element.style as TextStyle).color ?? '#666666'"
        @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Hizalama</label>
      <select class="prop-input prop-select"
        :value="(element.style as TextStyle).align ?? 'center'"
        @change="(e) => updateStyle('align', (e.target as HTMLSelectElement).value)">
        <option value="left">Sol</option>
        <option value="center">Orta</option>
        <option value="right">Sag</option>
      </select>
    </div>
  </div>
</template>
