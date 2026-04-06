<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { CalculatedTextElement, TextStyle, TemplateElement } from '../../core/types'
import DexprEditor from '../common/DexprEditor.vue'
import '../../styles/properties.css'

const props = defineProps<{ element: CalculatedTextElement }>()
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

function onExpressionChange(value: string) {
  update({ expression: value } as any)
}
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Hesaplanan Metin</div>
    <div
      class="prop-row-stack"
      data-tip="Hesaplama ifadesi (orn: toplamlar.kdv + toplamlar.araToplam)"
    >
      <label class="prop-label">Ifade</label>
      <DexprEditor
        :model-value="element.expression"
        @update:model-value="onExpressionChange"
        placeholder="toplamlar.kdv + toplamlar.araToplam"
      />
    </div>
    <div class="prop-row" data-tip="Sonucun gosterim formati">
      <label class="prop-label">Format</label>
      <select
        class="prop-input prop-select"
        :value="element.format ?? ''"
        @change="
          (e) => update({ format: (e.target as HTMLSelectElement).value || undefined } as any)
        "
      >
        <option value="">Yok</option>
        <option value="currency">Para Birimi</option>
        <option value="number">Sayi</option>
        <option value="percentage">Yuzde</option>
      </select>
    </div>
    <div class="prop-row" data-tip="Yazi tipi boyutu (point)">
      <label class="prop-label">Boyut (pt)</label>
      <input
        class="prop-input"
        type="number"
        step="1"
        min="1"
        :value="(element.style as TextStyle).fontSize ?? 11"
        @input="
          (e) => updateStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 11)
        "
      />
    </div>
    <div class="prop-row" data-tip="Metin rengi">
      <label class="prop-label">Renk</label>
      <input
        class="prop-input prop-color"
        type="color"
        :value="(element.style as TextStyle).color ?? '#000000'"
        @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row" data-tip="Yazi tipi kalinligi">
      <label class="prop-label">Kalinlik</label>
      <select
        class="prop-input prop-select"
        :value="(element.style as TextStyle).fontWeight ?? 'normal'"
        @change="(e) => updateStyle('fontWeight', (e.target as HTMLSelectElement).value)"
      >
        <option value="normal">Normal</option>
        <option value="bold">Kalin</option>
      </select>
    </div>
    <div class="prop-row" data-tip="Metnin yatay hizalamasi">
      <label class="prop-label">Hizalama</label>
      <select
        class="prop-input prop-select"
        :value="(element.style as TextStyle).align ?? 'left'"
        @change="(e) => updateStyle('align', (e.target as HTMLSelectElement).value)"
      >
        <option value="left">Sol</option>
        <option value="center">Orta</option>
        <option value="right">Sag</option>
      </select>
    </div>
  </div>
</template>
