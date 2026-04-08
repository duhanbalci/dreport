<script setup lang="ts">
import { computed } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { isContainer } from '../../core/types'
import type {
  ContainerElement,
  LineElement,
  ImageElement,
  PageNumberElement,
  BarcodeElement,
  RepeatingTableElement,
  CurrentDateElement,
  ShapeElement,
  CheckboxElement,
  CalculatedTextElement,
  RichTextElement,
  ChartElement,
} from '../../core/types'
import PositioningProperties from '../properties/PositioningProperties.vue'
import SizeProperties from '../properties/SizeProperties.vue'
import TextProperties from '../properties/TextProperties.vue'
import LineProperties from '../properties/LineProperties.vue'
import ImageProperties from '../properties/ImageProperties.vue'
import PageNumberProperties from '../properties/PageNumberProperties.vue'
import BarcodeProperties from '../properties/BarcodeProperties.vue'
import CurrentDateProperties from '../properties/CurrentDateProperties.vue'
import ShapeProperties from '../properties/ShapeProperties.vue'
import CheckboxProperties from '../properties/CheckboxProperties.vue'
import CalculatedTextProperties from '../properties/CalculatedTextProperties.vue'
import RichTextProperties from '../properties/RichTextProperties.vue'
import ContainerProperties from '../properties/ContainerProperties.vue'
import RepeatingTableProperties from '../properties/RepeatingTableProperties.vue'
import ChartProperties from '../properties/ChartProperties.vue'
import PropCondition from '../properties/shared/PropCondition.vue'
import '../../styles/properties.css'

const templateStore = useTemplateStore()
const editorStore = useEditorStore()

const selectedElement = computed(() => {
  const ids = editorStore.selectedElementIds
  if (ids.size !== 1) return null
  const id = ids.values().next().value
  if (!id) return null
  return templateStore.getElementById(id) ?? null
})

const multipleSelected = computed(() => editorStore.selectedElementIds.size > 1)

const elementTypeLabel = computed(() => {
  const el = selectedElement.value
  if (!el) return ''
  switch (el.type) {
    case 'container':
      if (el.id === 'header') return 'Üst Bilgi'
      if (el.id === 'footer') return 'Alt Bilgi'
      return 'Container'
    case 'static_text':
      return 'Metin'
    case 'text':
      return 'Metin'
    case 'line':
      return 'Cizgi'
    case 'repeating_table':
      return 'Tablo'
    case 'image':
      return 'Gorsel'
    case 'page_number':
      return 'Sayfa No'
    case 'barcode':
      return 'Barkod'
    case 'checkbox':
      return 'Onay Kutusu'
    case 'shape':
      return 'Sekil'
    case 'current_date':
      return 'Tarih'
    case 'calculated_text':
      return 'Hesaplanan Metin'
    case 'rich_text':
      return 'Zengin Metin'
    case 'page_break':
      return 'Sayfa Sonu'
    case 'chart':
      return 'Grafik'
    default:
      return 'Eleman'
  }
})

function toggleHeader(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  if (checked) templateStore.enableHeader()
  else templateStore.disableHeader()
}

function toggleFooter(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  if (checked) templateStore.enableFooter()
  else templateStore.disableFooter()
}

function deleteElement() {
  const id = editorStore.selectedElementId
  if (!id || id === 'root') return
  editorStore.clearSelection()
  templateStore.removeElement(id)
}

function deleteSelected() {
  const ids = [...editorStore.selectedElementIds]
  editorStore.clearSelection()
  for (const id of ids) {
    if (id !== 'root') templateStore.removeElement(id)
  }
}
</script>

<template>
  <div class="properties-panel">
    <div v-if="multipleSelected" class="properties-panel__empty">
      {{ editorStore.selectedElementIds.size }} eleman secili
      <button class="prop-delete-btn" style="margin-top: 12px" @click="deleteSelected">
        Secilenleri Sil
      </button>
    </div>

    <div v-else-if="!selectedElement" class="properties-panel__empty">Bir eleman secin</div>

    <template v-else>
      <!-- Header -->
      <div class="prop-section">
        <div class="prop-section__title">
          {{ elementTypeLabel }}
          <span class="prop-id">{{ selectedElement.id }}</span>
        </div>
      </div>

      <!-- Page break: minimal info, just delete -->
      <template v-if="selectedElement.type === 'page_break'">
        <div class="prop-section">
          <button class="prop-delete-btn" @click="deleteElement">Sil</button>
        </div>
      </template>

      <template v-else>
        <PositioningProperties :element="selectedElement" />
        <SizeProperties :element="selectedElement" />

        <TextProperties
          v-if="selectedElement.type === 'static_text' || selectedElement.type === 'text'"
          :element="selectedElement"
        />

        <LineProperties
          v-if="selectedElement.type === 'line'"
          :element="selectedElement as LineElement"
        />

        <ImageProperties
          v-if="selectedElement.type === 'image'"
          :element="selectedElement as ImageElement"
        />

        <PageNumberProperties
          v-if="selectedElement.type === 'page_number'"
          :element="selectedElement as PageNumberElement"
        />

        <BarcodeProperties
          v-if="selectedElement.type === 'barcode'"
          :element="selectedElement as BarcodeElement"
        />

        <CurrentDateProperties
          v-if="selectedElement.type === 'current_date'"
          :element="selectedElement as CurrentDateElement"
        />

        <CheckboxProperties
          v-if="selectedElement.type === 'checkbox'"
          :element="selectedElement as CheckboxElement"
        />

        <CalculatedTextProperties
          v-if="selectedElement.type === 'calculated_text'"
          :element="selectedElement as CalculatedTextElement"
        />

        <RichTextProperties
          v-if="selectedElement.type === 'rich_text'"
          :element="selectedElement as RichTextElement"
        />

        <ShapeProperties
          v-if="selectedElement.type === 'shape'"
          :element="selectedElement as ShapeElement"
        />

        <ContainerProperties
          v-if="isContainer(selectedElement)"
          :element="selectedElement as ContainerElement"
        />

        <RepeatingTableProperties
          v-if="selectedElement.type === 'repeating_table'"
          :element="selectedElement as RepeatingTableElement"
        />

        <ChartProperties
          v-if="selectedElement.type === 'chart'"
          :element="selectedElement as ChartElement"
        />

        <!-- Header/Footer toggles for root element -->
        <div v-if="selectedElement.id === 'root'" class="prop-section">
          <div class="prop-section__title">Sayfa Ust/Alt Bilgi</div>
          <div class="prop-row">
            <label class="prop-label">Ust Bilgi (Header)</label>
            <input
              type="checkbox"
              :checked="!!templateStore.template.header"
              @change="toggleHeader"
            />
          </div>
          <div class="prop-row">
            <label class="prop-label">Alt Bilgi (Footer)</label>
            <input
              type="checkbox"
              :checked="!!templateStore.template.footer"
              @change="toggleFooter"
            />
          </div>
        </div>

        <!-- Condition -->
        <PropCondition
          v-if="selectedElement.id !== 'root'"
          :condition="selectedElement.condition"
          @update:condition="(v) => templateStore.updateElement(selectedElement!.id, { condition: v } as any)"
        />

        <!-- Delete -->
        <div v-if="selectedElement.id !== 'root'" class="prop-section">
          <button class="prop-delete-btn" @click="deleteElement">Sil</button>
        </div>
      </template>
    </template>
  </div>
</template>

<style scoped>
.properties-panel {
  padding: 12px;
}

.properties-panel__empty {
  color: #94a3b8;
  font-size: 13px;
  text-align: center;
  margin-top: 40px;
}
</style>
