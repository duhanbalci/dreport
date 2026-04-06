<script setup lang="ts">
import { computed } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { useSchemaStore } from '../../stores/schema'
import type { ImageElement, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ImageElement }>()
const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const schemaStore = useSchemaStore()

/** Statik mi dinamik mi? */
const isDynamic = computed(() => !!props.element.binding)

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
    update({ src: reader.result as string, binding: undefined } as Partial<TemplateElement>)
  }
  reader.readAsDataURL(file)
}

function setMode(mode: 'static' | 'dynamic') {
  if (mode === 'static') {
    update({ binding: undefined } as Partial<TemplateElement>)
  } else {
    // Dinamik moda geç — ilk uygun alanı seç veya boş bırak
    const imageFields = schemaStore.scalarFields.filter(
      (f) => f.format === 'image' || f.type === 'string',
    )
    const path = imageFields.length > 0 ? imageFields[0].path : ''
    update({ src: undefined, binding: { type: 'scalar', path } } as Partial<TemplateElement>)
  }
}

function setBindingPath(path: string) {
  update({ binding: { type: 'scalar', path } } as Partial<TemplateElement>)
}

/** Schema'dan görsel olabilecek alanlar (format: image veya string) */
const imageScalarFields = computed(() => {
  return schemaStore.scalarFields.filter((f) => f.format === 'image' || f.type === 'string')
})
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Gorsel</div>

    <!-- Statik / Dinamik toggle -->
    <div class="prop-row" data-tip="Gorsel kaynagi: dosya veya veri alanından">
      <label class="prop-label">Mod</label>
      <div class="prop-toggle-group">
        <button
          class="prop-toggle-btn"
          :class="{ 'prop-toggle-btn--active': !isDynamic }"
          @click="setMode('static')"
        >
          Statik
        </button>
        <button
          class="prop-toggle-btn"
          :class="{ 'prop-toggle-btn--active': isDynamic }"
          @click="setMode('dynamic')"
        >
          Dinamik
        </button>
      </div>
    </div>

    <!-- Statik: dosya seçimi -->
    <template v-if="!isDynamic">
      <div class="prop-row" data-tip="Gorsel dosyasi secin (PNG, JPG, SVG)">
        <label class="prop-label">Kaynak</label>
        <label class="prop-file-btn">
          Dosya Sec
          <input type="file" accept="image/*" style="display: none" @change="onImageFileSelect" />
        </label>
      </div>
      <div v-if="element.src" class="prop-row" data-tip="Yuklenen gorsel onizlemesi">
        <label class="prop-label">Onizleme</label>
        <img :src="element.src" class="prop-image-preview" />
      </div>
      <div v-if="element.src" class="prop-row" data-tip="Gorseli kaldirmak icin tiklayin">
        <label class="prop-label"></label>
        <button class="prop-clear" @click="update({ src: undefined } as any)">
          Gorseli kaldir
        </button>
      </div>
    </template>

    <!-- Dinamik: schema alan seçimi -->
    <template v-else>
      <div class="prop-row" data-tip="Gorsel URL'sinin gelecegi veri alani">
        <label class="prop-label">Veri Alani</label>
        <select
          class="prop-input prop-select"
          :value="element.binding?.path ?? ''"
          @change="(e) => setBindingPath((e.target as HTMLSelectElement).value)"
        >
          <option value="" disabled>Secin...</option>
          <option v-for="field in imageScalarFields" :key="field.path" :value="field.path">
            {{ field.title }} ({{ field.path }})
          </option>
        </select>
      </div>
      <div v-if="element.binding?.path" class="prop-row">
        <label class="prop-label">Path</label>
        <span class="prop-info">{{ element.binding.path }}</span>
      </div>
    </template>

    <!-- Sığdırma modu (ortak) -->
    <div class="prop-row" data-tip="Gorselin alana sigdirma modu">
      <label class="prop-label">Sigdirma</label>
      <select
        class="prop-input prop-select"
        :value="element.style.objectFit ?? 'contain'"
        @change="(e) => updateStyle('objectFit', (e.target as HTMLSelectElement).value)"
      >
        <option value="contain">Sigdir</option>
        <option value="cover">Kap</option>
        <option value="stretch">Esnet</option>
      </select>
    </div>
  </div>
</template>

<style scoped>
.prop-toggle-group {
  display: flex;
  gap: 0;
}

.prop-toggle-btn {
  flex: 1;
  padding: 3px 8px;
  border: 1px solid #e2e8f0;
  background: white;
  color: #64748b;
  font-size: 11px;
  cursor: pointer;
  transition:
    background 0.1s,
    color 0.1s;
}

.prop-toggle-btn:first-child {
  border-radius: 4px 0 0 4px;
}

.prop-toggle-btn:last-child {
  border-radius: 0 4px 4px 0;
  border-left: none;
}

.prop-toggle-btn--active {
  background: #3b82f6;
  color: white;
  border-color: #3b82f6;
}

.prop-info {
  font-size: 11px;
  color: #94a3b8;
  word-break: break-all;
}
</style>
