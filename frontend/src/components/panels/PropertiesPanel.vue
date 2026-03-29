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
} from '../../core/types'
import PositioningProperties from '../properties/PositioningProperties.vue'
import SizeProperties from '../properties/SizeProperties.vue'
import TextProperties from '../properties/TextProperties.vue'
import LineProperties from '../properties/LineProperties.vue'
import ImageProperties from '../properties/ImageProperties.vue'
import PageNumberProperties from '../properties/PageNumberProperties.vue'
import BarcodeProperties from '../properties/BarcodeProperties.vue'
import ContainerProperties from '../properties/ContainerProperties.vue'
import RepeatingTableProperties from '../properties/RepeatingTableProperties.vue'
import '../../styles/properties.css'

const templateStore = useTemplateStore()
const editorStore = useEditorStore()

const selectedElement = computed(() => {
  const id = editorStore.selectedElementId
  if (!id) return null
  return templateStore.getElementById(id) ?? null
})

const elementTypeLabel = computed(() => {
  const el = selectedElement.value
  if (!el) return ''
  switch (el.type) {
    case 'container': return 'Container'
    case 'static_text': return 'Metin'
    case 'text': return 'Metin'
    case 'line': return 'Cizgi'
    case 'repeating_table': return 'Tablo'
    case 'image': return 'Gorsel'
    case 'page_number': return 'Sayfa No'
    case 'barcode': return 'Barkod'
    default: return 'Eleman'
  }
})

function deleteElement() {
  const id = editorStore.selectedElementId
  if (!id || id === 'root') return
  editorStore.clearSelection()
  templateStore.removeElement(id)
}
</script>

<template>
  <div class="properties-panel">
    <div v-if="!selectedElement" class="properties-panel__empty">
      Bir eleman secin
    </div>

    <template v-else>
      <!-- Header -->
      <div class="prop-section">
        <div class="prop-section__title">
          {{ elementTypeLabel }}
          <span class="prop-id">{{ selectedElement.id }}</span>
        </div>
      </div>

      <PositioningProperties :element="selectedElement" />
      <SizeProperties :element="selectedElement" />

      <TextProperties
        v-if="selectedElement.type === 'static_text' || selectedElement.type === 'text'"
        :element="selectedElement" />

      <LineProperties
        v-if="selectedElement.type === 'line'"
        :element="(selectedElement as LineElement)" />

      <ImageProperties
        v-if="selectedElement.type === 'image'"
        :element="(selectedElement as ImageElement)" />

      <PageNumberProperties
        v-if="selectedElement.type === 'page_number'"
        :element="(selectedElement as PageNumberElement)" />

      <BarcodeProperties
        v-if="selectedElement.type === 'barcode'"
        :element="(selectedElement as BarcodeElement)" />

      <ContainerProperties
        v-if="isContainer(selectedElement)"
        :element="(selectedElement as ContainerElement)" />

      <RepeatingTableProperties
        v-if="selectedElement.type === 'repeating_table'"
        :element="(selectedElement as RepeatingTableElement)" />

      <!-- Delete -->
      <div v-if="selectedElement.id !== 'root'" class="prop-section">
        <button class="prop-delete-btn" @click="deleteElement">Sil</button>
      </div>
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
