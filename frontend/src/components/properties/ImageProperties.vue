<script setup lang="ts">
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { ImageElement, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ImageElement }>()
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

function onImageFileSelect(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    update({ src: reader.result as string } as Partial<TemplateElement>)
  }
  reader.readAsDataURL(file)
}
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Gorsel</div>
    <div class="prop-row">
      <label class="prop-label">Kaynak</label>
      <label class="prop-file-btn">
        Dosya Sec
        <input type="file" accept="image/*" style="display: none" @change="onImageFileSelect" />
      </label>
    </div>
    <div v-if="element.src" class="prop-row">
      <label class="prop-label">Onizleme</label>
      <img :src="element.src" class="prop-image-preview" />
    </div>
    <div v-if="element.src" class="prop-row">
      <label class="prop-label"></label>
      <button class="prop-clear" @click="update({ src: undefined } as any)">Gorseli kaldir</button>
    </div>
    <div class="prop-row">
      <label class="prop-label">Sigdirma</label>
      <select class="prop-input prop-select"
        :value="element.style.objectFit ?? 'contain'"
        @change="(e) => updateStyle('objectFit', (e.target as HTMLSelectElement).value)">
        <option value="contain">Sigdir</option>
        <option value="cover">Kap</option>
        <option value="stretch">Esnet</option>
      </select>
    </div>
  </div>
</template>
