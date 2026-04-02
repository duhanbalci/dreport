<script setup lang="ts">
import { computed } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { useSchemaStore } from '../../stores/schema'
import { sz } from '../../core/types'
import { schemaFormatToFormatType, defaultAlignForSchema } from '../../core/schema-parser'
import type { RepeatingTableElement, TableColumn, FormatType, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: RepeatingTableElement }>()
const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const schemaStore = useSchemaStore()

function update(updates: Partial<TemplateElement>) {
  const id = editorStore.selectedElementId
  if (!id) return
  templateStore.updateElement(id, updates)
}

let colIdCounter = Date.now()
function nextColId() {
  return `col_${(++colIdCounter).toString(36)}`
}

function updateTableDataSource(path: string) {
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
  const newStyle = { ...props.element.style, [key]: value }
  if (value === undefined || value === '') delete (newStyle as Record<string, unknown>)[key]
  update({ style: newStyle } as Partial<TemplateElement>)
}

function updateColumn(colId: string, updates: Partial<TableColumn>) {
  const columns = props.element.columns.map(c => c.id === colId ? { ...c, ...updates } : c)
  update({ columns } as Partial<TemplateElement>)
}

function addColumn() {
  const newCol: TableColumn = {
    id: nextColId(),
    field: 'alan',
    title: 'Yeni Sutun',
    width: sz.auto(),
    align: 'left',
  }
  update({ columns: [...props.element.columns, newCol] } as Partial<TemplateElement>)
}

function removeColumn(colId: string) {
  update({ columns: props.element.columns.filter(c => c.id !== colId) } as Partial<TemplateElement>)
}

function moveColumn(colId: string, direction: -1 | 1) {
  const cols = [...props.element.columns]
  const idx = cols.findIndex(c => c.id === colId)
  const newIdx = idx + direction
  if (newIdx < 0 || newIdx >= cols.length) return
  ;[cols[idx], cols[newIdx]] = [cols[newIdx], cols[idx]]
  update({ columns: cols } as Partial<TemplateElement>)
}

const tableItemFields = computed(() => {
  return schemaStore.getArrayItemFields(props.element.dataSource.path)
})
</script>

<template>
  <!-- Data source -->
  <div class="prop-section">
    <div class="prop-section__title">Veri Kaynagi</div>
    <div class="prop-row">
      <label class="prop-label">Kaynak</label>
      <select class="prop-input prop-select"
        :value="element.dataSource.path"
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

  <!-- Columns -->
  <div class="prop-section">
    <div class="prop-section__title">
      Sutunlar
      <button class="prop-add-btn" @click="addColumn">+</button>
    </div>
    <div
      v-for="col in element.columns"
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

  <!-- Sayfa bölme ayarları -->
  <div class="prop-section">
    <div class="prop-section__title">Sayfa Bolme</div>
    <div class="prop-row">
      <label class="prop-label">Header tekrarla</label>
      <input type="checkbox"
        :checked="element.repeatHeader !== false"
        @change="(e) => update({ repeatHeader: (e.target as HTMLInputElement).checked } as any)" />
    </div>
  </div>

  <!-- Table style -->
  <div class="prop-section">
    <div class="prop-section__title">Tablo Stili</div>
    <div class="prop-row">
      <label class="prop-label">Yazi boyutu</label>
      <input class="prop-input" type="number" step="1" min="6"
        :value="element.style.fontSize ?? 10"
        @input="(e) => updateTableStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 10)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Header bg</label>
      <input class="prop-input prop-color" type="color"
        :value="element.style.headerBg ?? '#f0f0f0'"
        @input="(e) => updateTableStyle('headerBg', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Header renk</label>
      <input class="prop-input prop-color" type="color"
        :value="element.style.headerColor ?? '#000000'"
        @input="(e) => updateTableStyle('headerColor', (e.target as HTMLInputElement).value)" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Zebra tek</label>
      <div class="prop-row-inline">
        <input class="prop-input prop-color" type="color"
          :value="element.style.zebraOdd ?? '#fafafa'"
          @input="(e) => updateTableStyle('zebraOdd', (e.target as HTMLInputElement).value)" />
        <button v-if="element.style.zebraOdd" class="prop-clear" @click="updateTableStyle('zebraOdd', undefined)">x</button>
      </div>
    </div>
    <div class="prop-row">
      <label class="prop-label">Kenarlik rengi</label>
      <div class="prop-row-inline">
        <input class="prop-input prop-color" type="color"
          :value="element.style.borderColor ?? '#cccccc'"
          @input="(e) => updateTableStyle('borderColor', (e.target as HTMLInputElement).value)" />
        <button v-if="element.style.borderColor" class="prop-clear" @click="updateTableStyle('borderColor', undefined)">x</button>
      </div>
    </div>
    <div class="prop-row">
      <label class="prop-label">Kenarlik (mm)</label>
      <input class="prop-input" type="number" step="0.1" min="0"
        :value="element.style.borderWidth ?? 0.5"
        @input="(e) => updateTableStyle('borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)" />
    </div>
  </div>
</template>
