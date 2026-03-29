<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { useSchemaStore } from '../../stores/schema'
import { isContainer, sz } from '../../core/types'
import PaddingBox from '../properties/PaddingBox.vue'
import { schemaFormatToFormatType, defaultAlignForSchema } from '../../core/schema-parser'
import type {
  TemplateElement,
  ContainerElement,
  StaticTextElement,
  LineElement,
  RepeatingTableElement,
  ImageElement,
  PageNumberElement,
  BarcodeElement,
  TableColumn,
  TextStyle,
  SizeValue,
  FormatType,
} from '../../core/types'

const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const schemaStore = useSchemaStore()

const selectedElement = computed(() => {
  const id = editorStore.selectedElementId
  if (!id) return null
  return templateStore.getElementById(id) ?? null
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

// --- Table helpers ---

let colIdCounter = Date.now()
function nextColId() {
  return `col_${(++colIdCounter).toString(36)}`
}

function updateTableDataSource(path: string) {
  // Veri kaynağı değişince schema'dan sütunları otomatik doldur
  const itemFields = schemaStore.getArrayItemFields(path)
  if (itemFields.length > 0) {
    const columns: TableColumn[] = itemFields.map(field => ({
      id: nextColId(),
      field: field.key,
      title: field.title,
      width: sz.auto(),
      align: defaultAlignForSchema(field),
      format: schemaFormatToFormatType(field.format),
    }))
    update({
      dataSource: { type: 'array', path },
      columns,
    } as Partial<TemplateElement>)
  } else {
    update({ dataSource: { type: 'array', path } } as Partial<TemplateElement>)
  }
}

function updateTableStyle(key: string, value: unknown) {
  const el = selectedElement.value as RepeatingTableElement
  if (!el || el.type !== 'repeating_table') return
  const newStyle = { ...el.style, [key]: value }
  if (value === undefined || value === '') delete (newStyle as Record<string, unknown>)[key]
  update({ style: newStyle } as Partial<TemplateElement>)
}

function updateColumn(colId: string, updates: Partial<TableColumn>) {
  const el = selectedElement.value as RepeatingTableElement
  if (!el || el.type !== 'repeating_table') return
  const columns = el.columns.map(c => c.id === colId ? { ...c, ...updates } : c)
  update({ columns } as Partial<TemplateElement>)
}

function addColumn() {
  const el = selectedElement.value as RepeatingTableElement
  if (!el || el.type !== 'repeating_table') return
  const newCol: TableColumn = {
    id: nextColId(),
    field: 'alan',
    title: 'Yeni Sutun',
    width: sz.auto(),
    align: 'left',
  }
  update({ columns: [...el.columns, newCol] } as Partial<TemplateElement>)
}

function removeColumn(colId: string) {
  const el = selectedElement.value as RepeatingTableElement
  if (!el || el.type !== 'repeating_table') return
  update({ columns: el.columns.filter(c => c.id !== colId) } as Partial<TemplateElement>)
}

function moveColumn(colId: string, direction: -1 | 1) {
  const el = selectedElement.value as RepeatingTableElement
  if (!el || el.type !== 'repeating_table') return
  const cols = [...el.columns]
  const idx = cols.findIndex(c => c.id === colId)
  const newIdx = idx + direction
  if (newIdx < 0 || newIdx >= cols.length) return
  ;[cols[idx], cols[newIdx]] = [cols[newIdx], cols[idx]]
  update({ columns: cols } as Partial<TemplateElement>)
}

/** Seçili tablonun veri kaynağının item alanları (sütun field seçimi için) */
const tableItemFields = computed(() => {
  const el = selectedElement.value
  if (!el || el.type !== 'repeating_table') return []
  return schemaStore.getArrayItemFields(el.dataSource.path)
})

// --- Image ---

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

// --- Barcode ---

import type { BarcodeFormat } from '../../core/types'

const barcodeDefaults: Record<BarcodeFormat, string> = {
  qr: 'https://example.com',
  ean13: '5901234123457',
  ean8: '96385074',
  code128: 'DREPORT-001',
  code39: 'DREPORT',
}

/** EAN kontrol basamağı hesapla (12 veya 7 haneli data için) */
function eanCheckDigit(data: string): number {
  let sum = 0
  for (let i = 0; i < data.length; i++) {
    const d = parseInt(data[i])
    // EAN ağırlıkları: 1, 3, 1, 3, ... (soldan sağa)
    sum += d * (i % 2 === 0 ? 1 : 3)
  }
  return (10 - (sum % 10)) % 10
}

function validateBarcode(format: BarcodeFormat, value: string): boolean {
  if (!value) return false
  switch (format) {
    case 'ean13':
      // Tam 13 haneli + geçerli kontrol basamağı
      if (!/^\d{13}$/.test(value)) return false
      return eanCheckDigit(value.slice(0, 12)) === parseInt(value[12])
    case 'ean8':
      // Tam 8 haneli + geçerli kontrol basamağı
      if (!/^\d{8}$/.test(value)) return false
      return eanCheckDigit(value.slice(0, 7)) === parseInt(value[7])
    case 'code39':
      return /^[A-Z0-9\-. $/+%]+$/i.test(value)
    case 'code128':
      return value.length > 0 && [...value].every(c => c.charCodeAt(0) < 128)
    case 'qr':
      return value.length > 0
    default:
      return value.length > 0
  }
}

const barcodeInputValue = ref('')
const barcodeInputInvalid = ref(false)

// Seçili eleman değişince input'u senkronla
watch(() => {
  const el = selectedElement.value
  if (el?.type === 'barcode') return (el as BarcodeElement).value ?? ''
  return ''
}, (val) => {
  barcodeInputValue.value = val
  barcodeInputInvalid.value = false
}, { immediate: true })

function onBarcodeValueInput(e: Event) {
  const val = (e.target as HTMLInputElement).value
  barcodeInputValue.value = val
  const el = selectedElement.value as BarcodeElement
  if (!el || el.type !== 'barcode') return

  if (validateBarcode(el.format, val)) {
    barcodeInputInvalid.value = false
    update({ value: val } as any)
  } else {
    barcodeInputInvalid.value = true
    // Template'i güncelleme — eski değer ile render devam eder
  }
}

function onBarcodeFormatChange(newFormat: BarcodeFormat) {
  const el = selectedElement.value as BarcodeElement
  if (!el || el.type !== 'barcode') return

  const currentValue = el.value ?? ''
  if (validateBarcode(newFormat, currentValue)) {
    update({ format: newFormat } as any)
  } else {
    // Değer yeni formata uymuyor → default değer ata
    const defaultVal = barcodeDefaults[newFormat]
    barcodeInputValue.value = defaultVal
    barcodeInputInvalid.value = false
    update({ format: newFormat, value: defaultVal } as any)
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
          {{ selectedElement.type === 'container' ? 'Container' : selectedElement.type === 'static_text' ? 'Metin' : selectedElement.type === 'line' ? 'Çizgi' : selectedElement.type === 'repeating_table' ? 'Tablo' : selectedElement.type === 'image' ? 'Gorsel' : selectedElement.type === 'page_number' ? 'Sayfa No' : 'Eleman' }}
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
          <label class="prop-label">Kalınlık (mm)</label>
          <input class="prop-input" type="number" step="0.1" min="0.1"
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

      <!-- Image properties -->
      <div v-if="selectedElement.type === 'image'" class="prop-section">
        <div class="prop-section__title">Gorsel</div>
        <div class="prop-row">
          <label class="prop-label">Kaynak</label>
          <label class="prop-file-btn">
            Dosya Sec
            <input type="file" accept="image/*" style="display: none" @change="onImageFileSelect" />
          </label>
        </div>
        <div v-if="(selectedElement as ImageElement).src" class="prop-row">
          <label class="prop-label">Onizleme</label>
          <img :src="(selectedElement as ImageElement).src" class="prop-image-preview" />
        </div>
        <div v-if="(selectedElement as ImageElement).src" class="prop-row">
          <label class="prop-label"></label>
          <button class="prop-clear" @click="update({ src: undefined } as any)">Gorseli kaldir</button>
        </div>
        <div class="prop-row">
          <label class="prop-label">Sigdirma</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as ImageElement).style.objectFit ?? 'contain'"
            @change="(e) => updateStyle('objectFit', (e.target as HTMLSelectElement).value)">
            <option value="contain">Sigdir</option>
            <option value="cover">Kap</option>
            <option value="stretch">Esnet</option>
          </select>
        </div>
      </div>

      <!-- Page number properties -->
      <div v-if="selectedElement.type === 'page_number'" class="prop-section">
        <div class="prop-section__title">Sayfa Numarasi</div>
        <div class="prop-row">
          <label class="prop-label">Format</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as PageNumberElement).format ?? '{current} / {total}'"
            @change="(e) => update({ format: (e.target as HTMLSelectElement).value } as any)">
            <option value="{current} / {total}">1 / 5</option>
            <option value="{current}">1</option>
            <option value="Sayfa {current}">Sayfa 1</option>
            <option value="Sayfa {current} / {total}">Sayfa 1 / 5</option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">Boyut (pt)</label>
          <input class="prop-input" type="number" step="1" min="1"
            :value="(selectedElement.style as TextStyle).fontSize ?? 10"
            @input="(e) => updateStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 10)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Renk</label>
          <input class="prop-input prop-color" type="color"
            :value="(selectedElement.style as TextStyle).color ?? '#666666'"
            @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Hizalama</label>
          <select class="prop-input prop-select"
            :value="(selectedElement.style as TextStyle).align ?? 'center'"
            @change="(e) => updateStyle('align', (e.target as HTMLSelectElement).value)">
            <option value="left">Sol</option>
            <option value="center">Orta</option>
            <option value="right">Sag</option>
          </select>
        </div>
      </div>

      <!-- Barcode properties -->
      <div v-if="selectedElement.type === 'barcode'" class="prop-section">
        <div class="prop-section__title">Barkod Ayarları</div>
        <div class="prop-row">
          <label class="prop-label">Format</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as BarcodeElement).format"
            @change="(e) => onBarcodeFormatChange((e.target as HTMLSelectElement).value as BarcodeFormat)">
            <option value="qr">QR Kod</option>
            <option value="ean13">EAN-13</option>
            <option value="ean8">EAN-8</option>
            <option value="code128">Code 128</option>
            <option value="code39">Code 39</option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">Deger</label>
          <input class="prop-input" type="text"
            :class="{ 'prop-input--invalid': barcodeInputInvalid }"
            :value="barcodeInputValue"
            @input="onBarcodeValueInput" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Renk</label>
          <div class="prop-row-inline">
            <input class="prop-input prop-color" type="color"
              :value="(selectedElement as BarcodeElement).style.color ?? '#000000'"
              @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
            <button v-if="(selectedElement as BarcodeElement).style.color" class="prop-clear" @click="updateStyle('color', undefined)">x</button>
          </div>
        </div>
        <div v-if="(selectedElement as BarcodeElement).format !== 'qr'" class="prop-row">
          <label class="prop-label">Metin Goster</label>
          <input type="checkbox"
            :checked="(selectedElement as BarcodeElement).style.includeText ?? ((selectedElement as BarcodeElement).format === 'ean13' || (selectedElement as BarcodeElement).format === 'ean8')"
            @change="(e) => updateStyle('includeText', (e.target as HTMLInputElement).checked)" />
        </div>
        <div v-if="schemaStore.scalarFields.length > 0" class="prop-row">
          <label class="prop-label">Veri Baglama</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as BarcodeElement).binding?.path ?? ''"
            @change="(e) => {
              const val = (e.target as HTMLSelectElement).value
              if (val) {
                update({ binding: { type: 'scalar', path: val } } as any)
              } else {
                update({ binding: undefined } as any)
              }
            }">
            <option value="">Yok (statik deger)</option>
            <option
              v-for="field in schemaStore.scalarFields"
              :key="field.path"
              :value="field.path"
            >{{ field.title }} ({{ field.path }})</option>
          </select>
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
          <label class="prop-label">{{ (selectedElement as ContainerElement).direction === 'column' ? 'Yatay Hizalama' : 'Dikey Hizalama' }}</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as ContainerElement).align"
            @change="(e) => update({ align: (e.target as HTMLSelectElement).value } as any)">
            <option value="start">{{ (selectedElement as ContainerElement).direction === 'column' ? 'Sol' : 'Üst' }}</option>
            <option value="center">Orta</option>
            <option value="end">{{ (selectedElement as ContainerElement).direction === 'column' ? 'Sag' : 'Alt' }}</option>
            <option value="stretch">Esnet</option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">{{ (selectedElement as ContainerElement).direction === 'column' ? 'Dikey Dagılım' : 'Yatay Dagılım' }}</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as ContainerElement).justify"
            @change="(e) => update({ justify: (e.target as HTMLSelectElement).value } as any)">
            <option value="start">{{ (selectedElement as ContainerElement).direction === 'column' ? 'Üst' : 'Sol' }}</option>
            <option value="center">Orta</option>
            <option value="end">{{ (selectedElement as ContainerElement).direction === 'column' ? 'Alt' : 'Sag' }}</option>
            <option value="space-between">Esit Aralık</option>
          </select>
        </div>

        <!-- Padding -->
        <div class="prop-section__subtitle">Padding (mm)</div>
        <PaddingBox
          :top="(selectedElement as ContainerElement).padding.top"
          :right="(selectedElement as ContainerElement).padding.right"
          :bottom="(selectedElement as ContainerElement).padding.bottom"
          :left="(selectedElement as ContainerElement).padding.left"
          @update="(side, value) => update({ padding: { ...(selectedElement as ContainerElement).padding, [side]: value } } as any)"
        />

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
          <label class="prop-label">Kenarlık (mm)</label>
          <input class="prop-input" type="number" step="0.1" min="0"
            :value="(selectedElement as ContainerElement).style.borderWidth ?? 0"
            @input="(e) => updateStyle('borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Kenarlık rengi</label>
          <div class="prop-row-inline">
            <input class="prop-input prop-color" type="color"
              :value="(selectedElement as ContainerElement).style.borderColor ?? '#000000'"
              @input="(e) => updateStyle('borderColor', (e.target as HTMLInputElement).value)" />
            <button v-if="(selectedElement as ContainerElement).style.borderColor" class="prop-clear" @click="updateStyle('borderColor', undefined)">x</button>
          </div>
        </div>
        <div class="prop-row">
          <label class="prop-label">Kenarlık stili</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as ContainerElement).style.borderStyle ?? 'solid'"
            @change="(e) => updateStyle('borderStyle', (e.target as HTMLSelectElement).value)">
            <option value="solid">Düz</option>
            <option value="dashed">Kesikli</option>
            <option value="dotted">Noktalı</option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">Radius (mm)</label>
          <input class="prop-input" type="number" step="0.5" min="0"
            :value="(selectedElement as ContainerElement).style.borderRadius ?? 0"
            @input="(e) => updateStyle('borderRadius', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </div>
      </div>

      <!-- Repeating Table properties -->
      <div v-if="selectedElement.type === 'repeating_table'" class="prop-section">
        <div class="prop-section__title">Veri Kaynagi</div>
        <div class="prop-row">
          <label class="prop-label">Kaynak</label>
          <select class="prop-input prop-select"
            :value="(selectedElement as RepeatingTableElement).dataSource.path"
            @change="(e) => updateTableDataSource((e.target as HTMLSelectElement).value)">
            <option value="" disabled>Secin...</option>
            <option
              v-for="arr in schemaStore.arrayFields"
              :key="arr.path"
              :value="arr.path"
            >{{ arr.title }} ({{ arr.path }})</option>
          </select>
        </div>
      </div>

      <div v-if="selectedElement.type === 'repeating_table'" class="prop-section">
        <div class="prop-section__title">
          Sutunlar
          <button class="prop-add-btn" @click="addColumn">+</button>
        </div>
        <div
          v-for="col in (selectedElement as RepeatingTableElement).columns"
          :key="col.id"
          class="prop-column-card"
        >
          <div class="prop-column-header">
            <span class="prop-column-title">{{ col.title || col.field }}</span>
            <div class="prop-column-actions">
              <button class="prop-icon-btn" @click="moveColumn(col.id, -1)" title="Yukari">&#8593;</button>
              <button class="prop-icon-btn" @click="moveColumn(col.id, 1)" title="Asagi">&#8595;</button>
              <button class="prop-icon-btn prop-icon-btn--danger" @click="removeColumn(col.id)" title="Sil">x</button>
            </div>
          </div>
          <div class="prop-row">
            <label class="prop-label">Baslik</label>
            <input class="prop-input" type="text" :value="col.title"
              @change="(e) => updateColumn(col.id, { title: (e.target as HTMLInputElement).value })" />
          </div>
          <div class="prop-row">
            <label class="prop-label">Alan</label>
            <select v-if="tableItemFields.length > 0" class="prop-input prop-select" :value="col.field"
              @change="(e) => {
                const field = (e.target as HTMLSelectElement).value
                const node = tableItemFields.find(f => f.key === field)
                if (node) {
                  updateColumn(col.id, {
                    field,
                    title: node.title,
                    align: defaultAlignForSchema(node),
                    format: schemaFormatToFormatType(node.format),
                  })
                } else {
                  updateColumn(col.id, { field })
                }
              }">
              <option v-for="f in tableItemFields" :key="f.key" :value="f.key">{{ f.title }} ({{ f.key }})</option>
            </select>
            <input v-else class="prop-input" type="text" :value="col.field"
              @change="(e) => updateColumn(col.id, { field: (e.target as HTMLInputElement).value })" />
          </div>
          <div class="prop-row">
            <label class="prop-label">Hizalama</label>
            <select class="prop-input prop-select" :value="col.align"
              @change="(e) => updateColumn(col.id, { align: (e.target as HTMLSelectElement).value as 'left'|'center'|'right' })">
              <option value="left">Sol</option>
              <option value="center">Orta</option>
              <option value="right">Sag</option>
            </select>
          </div>
          <div class="prop-row">
            <label class="prop-label">Format</label>
            <select class="prop-input prop-select" :value="col.format ?? ''"
              @change="(e) => updateColumn(col.id, { format: ((e.target as HTMLSelectElement).value || undefined) as FormatType | undefined })">
              <option value="">Yok</option>
              <option value="currency">Para birimi</option>
              <option value="number">Sayi</option>
              <option value="date">Tarih</option>
              <option value="percentage">Yuzde</option>
            </select>
          </div>
          <div class="prop-row">
            <label class="prop-label">Genislik</label>
            <select class="prop-input prop-select"
              :value="col.width.type"
              @change="(e) => {
                const t = (e.target as HTMLSelectElement).value
                if (t === 'auto') updateColumn(col.id, { width: { type: 'auto' } })
                else if (t === 'fr') updateColumn(col.id, { width: { type: 'fr', value: 1 } })
                else updateColumn(col.id, { width: { type: 'fixed', value: 30 } })
              }">
              <option value="auto">Otomatik</option>
              <option value="fixed">Sabit (mm)</option>
              <option value="fr">Oran (fr)</option>
            </select>
          </div>
          <div v-if="col.width.type === 'fixed'" class="prop-row">
            <label class="prop-label">mm</label>
            <input class="prop-input" type="number" step="1" min="5"
              :value="(col.width as any).value"
              @change="(e) => updateColumn(col.id, { width: { type: 'fixed', value: parseFloat((e.target as HTMLInputElement).value) || 30 } })" />
          </div>
        </div>
      </div>

      <div v-if="selectedElement.type === 'repeating_table'" class="prop-section">
        <div class="prop-section__title">Tablo Stili</div>
        <div class="prop-row">
          <label class="prop-label">Yazi boyutu</label>
          <input class="prop-input" type="number" step="1" min="6"
            :value="(selectedElement as RepeatingTableElement).style.fontSize ?? 10"
            @input="(e) => updateTableStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 10)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Header bg</label>
          <input class="prop-input prop-color" type="color"
            :value="(selectedElement as RepeatingTableElement).style.headerBg ?? '#f0f0f0'"
            @input="(e) => updateTableStyle('headerBg', (e.target as HTMLInputElement).value)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Header renk</label>
          <input class="prop-input prop-color" type="color"
            :value="(selectedElement as RepeatingTableElement).style.headerColor ?? '#000000'"
            @input="(e) => updateTableStyle('headerColor', (e.target as HTMLInputElement).value)" />
        </div>
        <div class="prop-row">
          <label class="prop-label">Zebra tek</label>
          <div class="prop-row-inline">
            <input class="prop-input prop-color" type="color"
              :value="(selectedElement as RepeatingTableElement).style.zebraOdd ?? '#fafafa'"
              @input="(e) => updateTableStyle('zebraOdd', (e.target as HTMLInputElement).value)" />
            <button v-if="(selectedElement as RepeatingTableElement).style.zebraOdd" class="prop-clear" @click="updateTableStyle('zebraOdd', undefined)">x</button>
          </div>
        </div>
        <div class="prop-row">
          <label class="prop-label">Kenarlık rengi</label>
          <div class="prop-row-inline">
            <input class="prop-input prop-color" type="color"
              :value="(selectedElement as RepeatingTableElement).style.borderColor ?? '#cccccc'"
              @input="(e) => updateTableStyle('borderColor', (e.target as HTMLInputElement).value)" />
            <button v-if="(selectedElement as RepeatingTableElement).style.borderColor" class="prop-clear" @click="updateTableStyle('borderColor', undefined)">x</button>
          </div>
        </div>
        <div class="prop-row">
          <label class="prop-label">Kenarlık (mm)</label>
          <input class="prop-input" type="number" step="0.1" min="0"
            :value="(selectedElement as RepeatingTableElement).style.borderWidth ?? 0.5"
            @input="(e) => updateTableStyle('borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)" />
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

.prop-input--invalid {
  border-color: #ef4444;
  background: #fef2f2;
  color: #991b1b;
}

.prop-input--invalid:focus {
  border-color: #ef4444;
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

.prop-file-btn {
  padding: 4px 10px;
  background: #eff6ff;
  color: #3b82f6;
  border: 1px solid #bfdbfe;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
}

.prop-file-btn:hover {
  background: #dbeafe;
}

.prop-image-preview {
  max-width: 80px;
  max-height: 60px;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  object-fit: contain;
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

.prop-add-btn {
  float: right;
  background: #eff6ff;
  color: #3b82f6;
  border: 1px solid #bfdbfe;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  width: 22px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.prop-add-btn:hover {
  background: #dbeafe;
}

.prop-column-card {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 8px;
  margin-bottom: 8px;
}

.prop-column-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.prop-column-title {
  font-size: 12px;
  font-weight: 500;
  color: #334155;
}

.prop-column-actions {
  display: flex;
  gap: 2px;
}

.prop-icon-btn {
  background: none;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
  cursor: pointer;
  font-size: 11px;
  color: #64748b;
  padding: 1px 4px;
  line-height: 1;
}

.prop-icon-btn:hover {
  background: #f1f5f9;
}

.prop-icon-btn--danger:hover {
  background: #fef2f2;
  color: #dc2626;
  border-color: #fecaca;
}
</style>
