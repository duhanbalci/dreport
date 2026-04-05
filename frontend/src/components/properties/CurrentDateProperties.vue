<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { CurrentDateElement, TextStyle, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: CurrentDateElement }>()
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
    <div class="prop-section__title">Tarih</div>
    <div class="prop-row" data-tip="Tarih gosterim formati">
      <label class="prop-label">Format</label>
      <select class="prop-input prop-select"
        :value="element.format ?? 'DD.MM.YYYY'"
        @change="(e) => update({ format: (e.target as HTMLSelectElement).value } as any)">
        <option value="DD.MM.YYYY">30.03.2026</option>
        <option value="DD/MM/YYYY">30/03/2026</option>
        <option value="YYYY-MM-DD">2026-03-30</option>
        <option value="DD.MM.YYYY HH:mm">30.03.2026 14:30</option>
      </select>
    </div>
    <div class="prop-row" data-tip="Yazi tipi boyutu (point)">
      <label class="prop-label">Boyut (pt)</label>
      <input class="prop-input" type="number" step="1" min="1"
        :value="(element.style as TextStyle).fontSize ?? 10"
        @input="(e) => updateStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 10)" />
    </div>
    <div class="prop-row" data-tip="Metin rengi">
      <label class="prop-label">Renk</label>
      <input class="prop-input prop-color" type="color"
        :value="(element.style as TextStyle).color ?? '#666666'"
        @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row" data-tip="Metnin yatay hizalamasi">
      <label class="prop-label">Hizalama</label>
      <select class="prop-input prop-select"
        :value="(element.style as TextStyle).align ?? 'left'"
        @change="(e) => updateStyle('align', (e.target as HTMLSelectElement).value)">
        <option value="left">Sol</option>
        <option value="center">Orta</option>
        <option value="right">Sag</option>
      </select>
    </div>
  </div>
</template>
