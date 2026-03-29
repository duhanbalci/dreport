<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import PaddingBox from './PaddingBox.vue'
import type { ContainerElement, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ContainerElement }>()
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
    <div class="prop-section__title">Container Ayarlari</div>
    <div class="prop-row">
      <label class="prop-label">Yon</label>
      <select class="prop-input prop-select"
        :value="element.direction"
        @change="(e) => update({ direction: (e.target as HTMLSelectElement).value } as any)">
        <option value="column">Dikey</option>
        <option value="row">Yatay</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="prop-label">Bosluk (mm)</label>
      <input class="prop-input" type="number" step="1" min="0"
        :value="element.gap"
        @input="(e) => update({ gap: parseFloat((e.target as HTMLInputElement).value) || 0 } as any)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">{{ element.direction === 'column' ? 'Yatay Hizalama' : 'Dikey Hizalama' }}</label>
      <select class="prop-input prop-select"
        :value="element.align"
        @change="(e) => update({ align: (e.target as HTMLSelectElement).value } as any)">
        <option value="start">{{ element.direction === 'column' ? 'Sol' : 'Ust' }}</option>
        <option value="center">Orta</option>
        <option value="end">{{ element.direction === 'column' ? 'Sag' : 'Alt' }}</option>
        <option value="stretch">Esnet</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="prop-label">{{ element.direction === 'column' ? 'Dikey Dagilim' : 'Yatay Dagilim' }}</label>
      <select class="prop-input prop-select"
        :value="element.justify"
        @change="(e) => update({ justify: (e.target as HTMLSelectElement).value } as any)">
        <option value="start">{{ element.direction === 'column' ? 'Ust' : 'Sol' }}</option>
        <option value="center">Orta</option>
        <option value="end">{{ element.direction === 'column' ? 'Alt' : 'Sag' }}</option>
        <option value="space-between">Esit Aralik</option>
      </select>
    </div>

    <div class="prop-section__subtitle">Padding (mm)</div>
    <PaddingBox
      :top="element.padding.top"
      :right="element.padding.right"
      :bottom="element.padding.bottom"
      :left="element.padding.left"
      @update="(side, value) => update({ padding: { ...element.padding, [side]: value } } as any)"
    />

    <div class="prop-section__subtitle">Stil</div>
    <div class="prop-row">
      <label class="prop-label">Arka plan</label>
      <div class="prop-row-inline">
        <input class="prop-input prop-color" type="color"
          :value="element.style.backgroundColor ?? '#ffffff'"
          @input="(e) => updateStyle('backgroundColor', (e.target as HTMLInputElement).value)" />
        <button v-if="element.style.backgroundColor" class="prop-clear" @click="updateStyle('backgroundColor', undefined)">x</button>
      </div>
    </div>
    <div class="prop-row">
      <label class="prop-label">Kenarlik (mm)</label>
      <input class="prop-input" type="number" step="0.1" min="0"
        :value="element.style.borderWidth ?? 0"
        @input="(e) => updateStyle('borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Kenarlik rengi</label>
      <div class="prop-row-inline">
        <input class="prop-input prop-color" type="color"
          :value="element.style.borderColor ?? '#000000'"
          @input="(e) => updateStyle('borderColor', (e.target as HTMLInputElement).value)" />
        <button v-if="element.style.borderColor" class="prop-clear" @click="updateStyle('borderColor', undefined)">x</button>
      </div>
    </div>
    <div class="prop-row">
      <label class="prop-label">Kenarlik stili</label>
      <select class="prop-input prop-select"
        :value="element.style.borderStyle ?? 'solid'"
        @change="(e) => updateStyle('borderStyle', (e.target as HTMLSelectElement).value)">
        <option value="solid">Duz</option>
        <option value="dashed">Kesikli</option>
        <option value="dotted">Noktali</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="prop-label">Radius (mm)</label>
      <input class="prop-input" type="number" step="0.5" min="0"
        :value="element.style.borderRadius ?? 0"
        @input="(e) => updateStyle('borderRadius', parseFloat((e.target as HTMLInputElement).value) || 0)" />
    </div>
  </div>
</template>
