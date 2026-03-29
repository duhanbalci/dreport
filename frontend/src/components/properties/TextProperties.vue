<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { StaticTextElement, TextStyle, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: TemplateElement }>()
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
    <div class="prop-section__title">Metin Stili</div>

    <div v-if="element.type === 'static_text'" class="prop-row">
      <label class="prop-label">Metin</label>
      <input class="prop-input" type="text"
        :value="(element as StaticTextElement).content"
        @input="(e) => update({ content: (e.target as HTMLInputElement).value } as any)" />
    </div>

    <div class="prop-row">
      <label class="prop-label">Boyut (pt)</label>
      <input class="prop-input" type="number" step="1" min="1"
        :value="(element.style as TextStyle).fontSize ?? 11"
        @input="(e) => updateStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 11)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Kalinlik</label>
      <select class="prop-input prop-select"
        :value="(element.style as TextStyle).fontWeight ?? 'normal'"
        @change="(e) => updateStyle('fontWeight', (e.target as HTMLSelectElement).value)">
        <option value="normal">Normal</option>
        <option value="bold">Kalin</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="prop-label">Renk</label>
      <input class="prop-input prop-color" type="color"
        :value="(element.style as TextStyle).color ?? '#000000'"
        @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row">
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
