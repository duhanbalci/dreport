<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { RichTextElement, RichTextSpan, TextStyle } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: RichTextElement }>()
const templateStore = useTemplateStore()
const editorStore = useEditorStore()

function update(updates: Partial<RichTextElement>) {
  const id = editorStore.selectedElementId
  if (!id) return
  templateStore.updateElement(id, updates as any)
}

function updateStyle(key: string, value: unknown) {
  update({ style: { ...props.element.style, [key]: value } } as Partial<RichTextElement>)
}

function updateSpan(index: number, updates: Partial<RichTextSpan>) {
  const content = [...props.element.content]
  content[index] = { ...content[index], ...updates }
  update({ content })
}

function updateSpanStyle(index: number, key: string, value: unknown) {
  const span = props.element.content[index]
  updateSpan(index, { style: { ...span.style, [key]: value } })
}

function addSpan() {
  const content = [...props.element.content, { text: 'yeni', style: {} }]
  update({ content })
}

function removeSpan(index: number) {
  if (props.element.content.length <= 1) return
  const content = props.element.content.filter((_, i) => i !== index)
  update({ content })
}
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Varsayilan Stil</div>
    <div class="prop-row">
      <label class="prop-label">Boyut (pt)</label>
      <input class="prop-input" type="number" step="1" min="1"
        :value="element.style.fontSize ?? 11"
        @input="(e) => updateStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 11)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Renk</label>
      <input class="prop-input prop-color" type="color"
        :value="element.style.color ?? '#000000'"
        @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Hizalama</label>
      <select class="prop-input prop-select"
        :value="element.style.align ?? 'left'"
        @change="(e) => updateStyle('align', (e.target as HTMLSelectElement).value)">
        <option value="left">Sol</option>
        <option value="center">Orta</option>
        <option value="right">Sag</option>
      </select>
    </div>
  </div>

  <div class="prop-section">
    <div class="prop-section__title">
      Span'lar
      <button class="prop-add-btn" @click="addSpan" title="Span ekle">+</button>
    </div>

    <div v-for="(span, idx) in element.content" :key="idx" class="prop-span-card">
      <div class="prop-span-card__header">
        <span class="prop-span-card__label">Span {{ idx + 1 }}</span>
        <button
          v-if="element.content.length > 1"
          class="prop-span-card__remove"
          @click="removeSpan(idx)"
          title="Sil"
        >&times;</button>
      </div>

      <div class="prop-row">
        <label class="prop-label">Metin</label>
        <input class="prop-input" type="text"
          :value="span.text ?? ''"
          @input="(e) => updateSpan(idx, { text: (e.target as HTMLInputElement).value })" />
      </div>
      <div class="prop-row">
        <label class="prop-label">Boyut</label>
        <input class="prop-input" type="number" step="1" min="1"
          :value="(span.style as TextStyle).fontSize ?? ''"
          placeholder="varsayilan"
          @input="(e) => {
            const v = (e.target as HTMLInputElement).value
            updateSpanStyle(idx, 'fontSize', v ? parseFloat(v) : undefined)
          }" />
      </div>
      <div class="prop-row">
        <label class="prop-label">Kalinlik</label>
        <select class="prop-input prop-select"
          :value="(span.style as TextStyle).fontWeight ?? ''"
          @change="(e) => {
            const v = (e.target as HTMLSelectElement).value
            updateSpanStyle(idx, 'fontWeight', v || undefined)
          }">
          <option value="">Varsayilan</option>
          <option value="normal">Normal</option>
          <option value="bold">Kalin</option>
        </select>
      </div>
      <div class="prop-row">
        <label class="prop-label">Renk</label>
        <input class="prop-input prop-color" type="color"
          :value="(span.style as TextStyle).color ?? element.style.color ?? '#000000'"
          @input="(e) => updateSpanStyle(idx, 'color', (e.target as HTMLInputElement).value)" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.prop-add-btn {
  float: right;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 4px;
  width: 22px;
  height: 22px;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.prop-add-btn:hover {
  background: #2563eb;
}

.prop-span-card {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 8px;
  margin-bottom: 8px;
}

.prop-span-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.prop-span-card__label {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
}

.prop-span-card__remove {
  background: none;
  border: none;
  color: #ef4444;
  font-size: 16px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}

.prop-span-card__remove:hover {
  color: #dc2626;
}
</style>
