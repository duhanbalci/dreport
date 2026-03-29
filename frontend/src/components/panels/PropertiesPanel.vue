<script setup lang="ts">
import { computed } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { isContainer } from '../../core/types'
import type {
  TemplateElement,
  ContainerElement,
  StaticTextElement,
  LineElement,
  TextStyle,
  SizeValue,
} from '../../core/types'

const templateStore = useTemplateStore()
const editorStore = useEditorStore()

const selectedElement = computed(() => {
  const id = editorStore.selectedElementId
  if (!id) return null
  return templateStore.getElementById(id) ?? null
})

const parentElement = computed(() => {
  const id = editorStore.selectedElementId
  if (!id) return null
  return templateStore.getParent(id) ?? null
})

// --- Generic updater ---

function update(updates: Partial<TemplateElement>) {
  const id = editorStore.selectedElementId
  if (!id) return
  templateStore.updateElement(id, updates)
}

function updateStyle(key: string, value: unknown) {
  const el = selectedElement.value
  if (!el) return
  update({ style: { ...el.style, [key]: value } } as Partial<TemplateElement>)
}

function updateSize(axis: 'width' | 'height', sv: SizeValue) {
  const id = editorStore.selectedElementId
  if (!id) return
  templateStore.updateElementSize(id, { [axis]: sv })
}

// --- Positioning ---

function togglePositioning() {
  const el = selectedElement.value
  if (!el) return
  if (el.position.type === 'flow') {
    templateStore.updateElementPosition(el.id, { type: 'absolute', x: 0, y: 0 })
  } else {
    templateStore.updateElementPosition(el.id, { type: 'flow' })
  }
}

// --- Delete ---

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
      Bir eleman seçin
    </div>

    <template v-else>
      <!-- Header -->
      <div class="prop-section">
        <div class="prop-section__title">
          {{ selectedElement.type === 'container' ? 'Container' : selectedElement.type === 'static_text' ? 'Metin' : selectedElement.type === 'line' ? 'Çizgi' : 'Eleman' }}
          <span class="prop-id">{{ selectedElement.id }}</span>
        </div>
      </div>

      <!-- Positioning -->
      <div class="prop-section">
        <div class="prop-section__title">Pozisyon</div>
        <div class="prop-row">
          <label class="prop-label">Mod</label>
          <select class="prop-input prop-select" :value="selectedElement.position.type" @change="togglePositioning">
            <option value="flow">Flow</option>
            <option value="absolute">Absolute</option>
          </select>
        </div>
        <template v-if="selectedElement.position.type === 'absolute'">
          <div class="prop-row">
            <label class="prop-label">X (mm)</label>
            <input class="prop-input" type="number" step="0.5"
              :value="selectedElement.position.x"
              @input="(e) => templateStore.updateElementPosition(selectedElement!.id, { type: 'absolute', x: parseFloat((e.target as HTMLInputElement).value) || 0, y: (selectedElement!.position as any).y ?? 0 })" />
          </div>
          <div class="prop-row">
            <label class="prop-label">Y (mm)</label>
            <input class="prop-input" type="number" step="0.5"
              :value="selectedElement.position.y"
              @input="(e) => templateStore.updateElementPosition(selectedElement!.id, { type: 'absolute', x: (selectedElement!.position as any).x ?? 0, y: parseFloat((e.target as HTMLInputElement).value) || 0 })" />
          </div>
        </template>
      </div>

      <!-- Size -->
      <div class="prop-section">
        <div class="prop-section__title">Boyut</div>
        <div class="prop-row">
          <label class="prop-label">Genişlik</label>
          <select class="prop-input prop-select"
            :value="selectedElement.size.width.type"
            @change="(e) => {
              const t = (e.target as HTMLSelectElement).value
              if (t === 'auto') updateSize('width', { type: 'auto' })
              else if (t === 'fr') updateSize('width', { type: 'fr', value: 1 })
              else updateSize('width', { type: 'fixed', value: 50 })
            }">
            <option value="auto">Otomatik</option>
            <option value="fixed">Sabit (mm)</option>
            <option value="fr">Oran (fr)</option>
          </select>
        </div>
        <div v-if="selectedElement.size.width.type === 'fixed'" class="prop-row">
          <label class="prop-label">mm</label>
          <input class="prop-input" type="number" step="1" min="1"
            :value="(selectedElement.size.width as any).value"
            @input="(e) => updateSize('width', { type: 'fixed', value: parseFloat((e.target as HTMLInputElement).value) || 10 })" />
        </div>
        <div v-if="selectedElement.size.width.type === 'fr'" class="prop-row">
          <label class="prop-label">fr</label>
          <input class="prop-input" type="number" step="1" min="1"
            :value="(selectedElement.size.width as any).value"
            @input="(e) => updateSize('width', { type: 'fr', value: parseFloat((e.target as HTMLInputElement).value) || 1 })" />
        </div>

        <div class="prop-row">
          <label class="prop-label">Yükseklik</label>
          <select class="prop-input prop-select"
            :value="selectedElement.size.height.type"
            @change="(e) => {
              const t = (e.target as HTMLSelectElement).value
              if (t === 'auto') updateSize('height', { type: 'auto' })
              else if (t === 'fr') updateSize('height', { type: 'fr', value: 1 })
              else updateSize('height', { type: 'fixed', value: 20 })
            }">
            <option value="auto">Otomatik</option>
            <option value="fixed">Sabit (mm)</option>
            <option value="fr">Oran (fr)</option>
          </select>
        </div>
        <div v-if="selectedElement.size.height.type === 'fixed'" class="prop-row">
          <label class="prop-label">mm</label>
          <input class="prop-input" type="number" step="1" min="1"
            :value="(selectedElement.size.height as any).value"
            @input="(e) => updateSize('height', { type: 'fixed', value: parseFloat((e.target as HTMLInputElement).value) || 10 })" />
        </div>
      </div>

      <!-- Text style (static_text, text) -->
      <div v-if="selectedElement.type === 'static_text' || selectedElement.type === 'text'" class="prop-section">
        <div class="prop-section__title">Metin Stili</div>

        <div v-if="selectedElement.type === 'static_text'" class="prop-row">
          <label class="prop-label">Metin</label>
          <input class="prop-input" type="text"
            :value="(selectedElement as StaticTextElement).content"
            @input="(e) => update({ content: (e.target as HTMLInputElement).value } as any)" />
        </div>

        <div class="prop-row">
          <label class="prop-label">Boyut (pt)</label>
          <input class="prop-input" type="number" step="1" min="1"
            :value="(selectedElement.style as TextStyle).fontSize ?? 11"
            @input="(e) => updateStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 11)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Kalınlık</label>
          <select class="prop-input prop-select"
            :value="(selectedElement.style as TextStyle).fontWeight ?? 'normal'"
            @change="(e) => updateStyle('fontWeight', (e.target as HTMLSelectElement).value)">
            <option value="normal">Normal</option>
            <option value="bold">Kalın</option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">Renk</label>
          <input class="prop-input prop-color" type="color"
            :value="(selectedElement.style as TextStyle).color ?? '#000000'"
            @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Hizalama</label>
          <select class="prop-input prop-select"
            :value="(selectedElement.style as TextStyle).align ?? 'left'"
            @change="(e) => updateStyle('align', (e.target as HTMLSelectElement).value)">
            <option value="left">Sol</option>
            <option value="center">Orta</option>
            <option value="right">Sag</option>
          </select>
        </div>
      </div>

      <!-- Line style -->
      <div v-if="selectedElement.type === 'line'" class="prop-section">
        <div class="prop-section__title">Çizgi Stili</div>
        <div class="prop-row">
          <label class="prop-label">Kalınlık (pt)</label>
          <input class="prop-input" type="number" step="0.25" min="0.25"
            :value="(selectedElement as LineElement).style.strokeWidth ?? 0.5"
            @input="(e) => updateStyle('strokeWidth', parseFloat((e.target as HTMLInputElement).value) || 0.5)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Renk</label>
          <input class="prop-input prop-color" type="color"
            :value="(selectedElement as LineElement).style.strokeColor ?? '#000000'"
            @input="(e) => updateStyle('strokeColor', (e.target as HTMLInputElement).value)" />
        </div>
      </div>

      <!-- Container properties -->
      <div v-if="isContainer(selectedElement)" class="prop-section">
        <div class="prop-section__title">Container Ayarları</div>
        <div class="prop-row">
          <label class="prop-label">Yön</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as ContainerElement).direction"
            @change="(e) => update({ direction: (e.target as HTMLSelectElement).value } as any)">
            <option value="column">Dikey</option>
            <option value="row">Yatay</option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">Boşluk (mm)</label>
          <input class="prop-input" type="number" step="1" min="0"
            :value="(selectedElement as ContainerElement).gap"
            @input="(e) => update({ gap: parseFloat((e.target as HTMLInputElement).value) || 0 } as any)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Hizalama</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as ContainerElement).align"
            @change="(e) => update({ align: (e.target as HTMLSelectElement).value } as any)">
            <option value="start">Baş</option>
            <option value="center">Orta</option>
            <option value="end">Son</option>
            <option value="stretch">Esnet</option>
          </select>
        </div>

        <!-- Padding -->
        <div class="prop-section__subtitle">Padding (mm)</div>
        <div class="prop-row-grid">
          <div class="prop-row">
            <label class="prop-label">Üst</label>
            <input class="prop-input" type="number" step="1" min="0"
              :value="(selectedElement as ContainerElement).padding.top"
              @input="(e) => update({ padding: { ...(selectedElement as ContainerElement).padding, top: parseFloat((e.target as HTMLInputElement).value) || 0 } } as any)" />
          </div>
          <div class="prop-row">
            <label class="prop-label">Sag</label>
            <input class="prop-input" type="number" step="1" min="0"
              :value="(selectedElement as ContainerElement).padding.right"
              @input="(e) => update({ padding: { ...(selectedElement as ContainerElement).padding, right: parseFloat((e.target as HTMLInputElement).value) || 0 } } as any)" />
          </div>
          <div class="prop-row">
            <label class="prop-label">Alt</label>
            <input class="prop-input" type="number" step="1" min="0"
              :value="(selectedElement as ContainerElement).padding.bottom"
              @input="(e) => update({ padding: { ...(selectedElement as ContainerElement).padding, bottom: parseFloat((e.target as HTMLInputElement).value) || 0 } } as any)" />
          </div>
          <div class="prop-row">
            <label class="prop-label">Sol</label>
            <input class="prop-input" type="number" step="1" min="0"
              :value="(selectedElement as ContainerElement).padding.left"
              @input="(e) => update({ padding: { ...(selectedElement as ContainerElement).padding, left: parseFloat((e.target as HTMLInputElement).value) || 0 } } as any)" />
          </div>
        </div>

        <!-- Container Style -->
        <div class="prop-section__subtitle">Stil</div>
        <div class="prop-row">
          <label class="prop-label">Arka plan</label>
          <div class="prop-row-inline">
            <input class="prop-input prop-color" type="color"
              :value="(selectedElement as ContainerElement).style.backgroundColor ?? '#ffffff'"
              @input="(e) => updateStyle('backgroundColor', (e.target as HTMLInputElement).value)" />
            <button v-if="(selectedElement as ContainerElement).style.backgroundColor" class="prop-clear" @click="updateStyle('backgroundColor', undefined)">x</button>
          </div>
        </div>
        <div class="prop-row">
          <label class="prop-label">Kenarlık rengi</label>
          <div class="prop-row-inline">
            <input class="prop-input prop-color" type="color"
              :value="(selectedElement as ContainerElement).style.borderColor ?? '#000000'"
              @input="(e) => updateStyle('borderColor', (e.target as HTMLInputElement).value)" />
          </div>
        </div>
        <div class="prop-row">
          <label class="prop-label">Kenarlık (pt)</label>
          <input class="prop-input" type="number" step="0.5" min="0"
            :value="(selectedElement as ContainerElement).style.borderWidth ?? 0"
            @input="(e) => updateStyle('borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Radius (pt)</label>
          <input class="prop-input" type="number" step="1" min="0"
            :value="(selectedElement as ContainerElement).style.borderRadius ?? 0"
            @input="(e) => updateStyle('borderRadius', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </div>
      </div>

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

.prop-section {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #f1f5f9;
}

.prop-section__title {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.prop-section__subtitle {
  font-size: 11px;
  font-weight: 500;
  color: #94a3b8;
  margin: 8px 0 4px;
}

.prop-id {
  font-weight: 400;
  color: #94a3b8;
  font-size: 10px;
  margin-left: 6px;
}

.prop-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.prop-row-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px;
}

.prop-row-inline {
  display: flex;
  align-items: center;
  gap: 4px;
}

.prop-label {
  font-size: 12px;
  color: #475569;
  flex-shrink: 0;
  min-width: 70px;
}

.prop-input {
  width: 100px;
  padding: 4px 6px;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  font-size: 12px;
  background: white;
  color: #334155;
}

.prop-input:focus {
  outline: none;
  border-color: #93c5fd;
}

.prop-select {
  cursor: pointer;
}

.prop-color {
  width: 32px;
  height: 24px;
  padding: 1px;
  cursor: pointer;
}

.prop-clear {
  background: none;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
  cursor: pointer;
  font-size: 11px;
  color: #94a3b8;
  padding: 2px 5px;
}

.prop-delete-btn {
  width: 100%;
  padding: 6px;
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
}

.prop-delete-btn:hover {
  background: #fee2e2;
}
</style>
