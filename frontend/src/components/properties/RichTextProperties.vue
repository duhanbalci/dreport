<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import { useSchemaStore } from '../../stores/schema'
import PropSection from './shared/PropSection.vue'
import PropColorInput from './shared/PropColorInput.vue'
import PropSelect from './shared/PropSelect.vue'
import PropFieldSelect from './shared/PropFieldSelect.vue'
import PropTextStyleGroup from './shared/PropTextStyleGroup.vue'
import type { RichTextElement, RichTextSpan, TextStyle } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: RichTextElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const schemaStore = useSchemaStore()

function updateSpan(index: number, updates: Partial<RichTextSpan>) {
  const content = [...props.element.content]
  content[index] = { ...content[index], ...updates }
  update({ content } as any)
}

function updateSpanStyle(index: number, key: string, value: unknown) {
  const span = props.element.content[index]
  updateSpan(index, { style: { ...span.style, [key]: value } })
}

function addSpan() {
  const content = [...props.element.content, { text: 'yeni', style: {} }]
  update({ content } as any)
}

function removeSpan(index: number) {
  if (props.element.content.length <= 1) return
  const content = props.element.content.filter((_, i) => i !== index)
  update({ content } as any)
}

const weightOptions = [
  { value: '', label: 'Varsayilan' },
  { value: 'normal', label: 'Normal' },
  { value: 'bold', label: 'Kalin' },
]
</script>

<template>
  <PropSection title="Varsayilan Stil">
    <PropTextStyleGroup
      :font-size="element.style.fontSize ?? 11"
      :font-weight="element.style.fontWeight ?? 'normal'"
      :font-family="element.style.fontFamily"
      :color="element.style.color ?? '#000000'"
      :align="element.style.align ?? 'left'"
      @update:font-size="(v) => updateStyle('fontSize', v)"
      @update:font-weight="(v) => updateStyle('fontWeight', v)"
      @update:font-family="(v) => updateStyle('fontFamily', v)"
      @update:color="(v) => updateStyle('color', v)"
      @update:align="(v) => updateStyle('align', v)"
    />
  </PropSection>

  <PropSection title="Span'lar">
    <template #actions>
      <button class="prop-add-btn" @click="addSpan" title="Span ekle">+</button>
    </template>

    <div v-for="(span, idx) in element.content" :key="idx" class="prop-span-card">
      <div class="prop-span-card__header">
        <span class="prop-span-card__label">Span {{ idx + 1 }}</span>
        <button
          v-if="element.content.length > 1"
          class="prop-span-card__remove"
          @click="removeSpan(idx)"
          title="Sil"
        >
          &times;
        </button>
      </div>

      <div class="prop-row" data-tip="Span metin icerigi">
        <label class="prop-label">Metin</label>
        <input
          class="prop-input"
          type="text"
          :value="span.text ?? ''"
          @input="(e) => updateSpan(idx, { text: (e.target as HTMLInputElement).value })"
        />
      </div>
      <PropFieldSelect
        label="Binding"
        :model-value="span.binding?.path ?? ''"
        :fields="schemaStore.scalarFields"
        :allow-empty="true"
        empty-label="Yok (statik)"
        data-tip="Span'in baglanacagi veri alani"
        @update:model-value="(v) => updateSpan(idx, { binding: v ? { type: 'scalar', path: v } : undefined })"
      />
      <div class="prop-row" data-tip="Span yazi boyutu — bos birakilirsa varsayilan kullanilir">
        <label class="prop-label">Boyut</label>
        <input
          class="prop-input"
          type="number"
          step="1"
          min="1"
          :value="(span.style as TextStyle).fontSize ?? ''"
          placeholder="varsayilan"
          @input="
            (e) => {
              const v = (e.target as HTMLInputElement).value
              updateSpanStyle(idx, 'fontSize', v ? parseFloat(v) : undefined)
            }
          "
        />
      </div>
      <PropSelect
        label="Kalinlik"
        :model-value="(span.style as TextStyle).fontWeight ?? ''"
        :options="weightOptions"
        data-tip="Span yazi kalinligi"
        @update:model-value="(v) => updateSpanStyle(idx, 'fontWeight', v || undefined)"
      />
      <PropColorInput
        label="Renk"
        :model-value="(span.style as TextStyle).color ?? element.style.color ?? '#000000'"
        data-tip="Span metin rengi"
        @update:model-value="(v) => updateSpanStyle(idx, 'color', v)"
      />
      <PropSelect
        label="Hizalama"
        :model-value="(span.style as TextStyle).align ?? ''"
        :options="[{ value: '', label: 'Varsayilan' }, { value: 'left', label: 'Sol' }, { value: 'center', label: 'Orta' }, { value: 'right', label: 'Sag' }]"
        data-tip="Span hizalamasi"
        @update:model-value="(v) => updateSpanStyle(idx, 'align', v || undefined)"
      />
    </div>
  </PropSection>
</template>

<style scoped>
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
