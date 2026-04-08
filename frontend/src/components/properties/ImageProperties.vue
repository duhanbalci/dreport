<script setup lang="ts">
import { computed } from 'vue'
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import { useSchemaStore } from '../../stores/schema'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropFieldSelect from './shared/PropFieldSelect.vue'
import type { ImageElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ImageElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const schemaStore = useSchemaStore()

const isDynamic = computed(() => !!props.element.binding)

const imageScalarFields = computed(() =>
  schemaStore.scalarFields.filter((f) => f.format === 'image' || f.type === 'string'),
)

const fitOptions = [
  { value: 'contain', label: 'Sigdir' },
  { value: 'cover', label: 'Kap' },
  { value: 'stretch', label: 'Esnet' },
]

function onImageFileSelect(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    update({ src: reader.result as string, binding: undefined } as any)
  }
  reader.readAsDataURL(file)
}

function setMode(mode: 'static' | 'dynamic') {
  if (mode === 'static') {
    update({ binding: undefined } as any)
  } else {
    const path = imageScalarFields.value.length > 0 ? imageScalarFields.value[0].path : ''
    update({ src: undefined, binding: { type: 'scalar', path } } as any)
  }
}
</script>

<template>
  <PropSection title="Gorsel">
    <div class="prop-row" data-tip="Gorsel kaynagi: dosya veya veri alanindan">
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

    <template v-else>
      <PropFieldSelect
        label="Veri Alani"
        :model-value="element.binding?.path ?? ''"
        :fields="imageScalarFields"
        data-tip="Gorsel URL'sinin gelecegi veri alani"
        @update:model-value="(v) => update({ binding: { type: 'scalar', path: v } } as any)"
      />
      <div v-if="element.binding?.path" class="prop-row">
        <label class="prop-label">Path</label>
        <span class="prop-info">{{ element.binding.path }}</span>
      </div>
    </template>

    <PropSelect
      label="Sigdirma"
      :model-value="element.style.objectFit ?? 'contain'"
      :options="fitOptions"
      data-tip="Gorselin alana sigdirma modu"
      @update:model-value="(v) => updateStyle('objectFit', v)"
    />
  </PropSection>
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
