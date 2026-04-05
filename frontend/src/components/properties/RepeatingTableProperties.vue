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
    <div class="prop-row" data-tip="Tablonun baglanacagi array veri kaynagi">
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
      class="tbl-col"
    >
      <!-- Row 1: title + actions -->
      <div class="tbl-col__head">
        <input class="tbl-col__title" type="text" :value="col.title"
          @change="(e) => updateColumn(col.id, { title: (e.target as HTMLInputElement).value })"
          :placeholder="col.field"
          data-tip="Sutun basligi" />
        <div class="tbl-col__actions">
          <button class="tbl-col__act" @click="moveColumn(col.id, -1)" data-tip="Yukari tasi">
            <svg width="10" height="10" viewBox="0 0 10 10"><path d="M5 2L2 6h6L5 2z" fill="currentColor"/></svg>
          </button>
          <button class="tbl-col__act" @click="moveColumn(col.id, 1)" data-tip="Asagi tasi">
            <svg width="10" height="10" viewBox="0 0 10 10"><path d="M5 8L2 4h6L5 8z" fill="currentColor"/></svg>
          </button>
          <button class="tbl-col__act tbl-col__act--del" @click="removeColumn(col.id)" data-tip="Sutunu sil">
            <svg width="10" height="10" viewBox="0 0 10 10"><path d="M2 2l6 6M8 2l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
          </button>
        </div>
      </div>

      <!-- Row 2: field + align + format + width compact -->
      <div class="tbl-col__controls">
        <!-- Field -->
        <select v-if="tableItemFields.length > 0" class="tbl-col__field" :value="col.field" data-tip="Veri alani"
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
          <option v-for="f in tableItemFields" :key="f.key" :value="f.key">{{ f.key }}</option>
        </select>
        <input v-else class="tbl-col__field" type="text" :value="col.field"
          @change="(e) => updateColumn(col.id, { field: (e.target as HTMLInputElement).value })"
          data-tip="Veri alani" />

        <!-- Alignment icons -->
        <div class="tbl-col__align">
          <button class="tbl-col__align-btn" :class="{ 'tbl-col__align-btn--on': col.align === 'left' }" @click="updateColumn(col.id, { align: 'left' })" data-tip="Sola hizala">
            <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1" y1="3" x2="11" y2="3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><line x1="1" y1="6" x2="8" y2="6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><line x1="1" y1="9" x2="10" y2="9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>
          </button>
          <button class="tbl-col__align-btn" :class="{ 'tbl-col__align-btn--on': col.align === 'center' }" @click="updateColumn(col.id, { align: 'center' })" data-tip="Ortala">
            <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1" y1="3" x2="11" y2="3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><line x1="2.5" y1="6" x2="9.5" y2="6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><line x1="1.5" y1="9" x2="10.5" y2="9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>
          </button>
          <button class="tbl-col__align-btn" :class="{ 'tbl-col__align-btn--on': col.align === 'right' }" @click="updateColumn(col.id, { align: 'right' })" data-tip="Saga hizala">
            <svg width="12" height="12" viewBox="0 0 12 12"><line x1="1" y1="3" x2="11" y2="3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><line x1="4" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/><line x1="2" y1="9" x2="11" y2="9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>
          </button>
        </div>
      </div>

      <!-- Row 3: format + width -->
      <div class="tbl-col__extra" data-tip="Veri gosterim formati">
        <label class="tbl-col__elabel">Format</label>
        <select class="tbl-col__fmt" :value="col.format ?? ''"
          @change="(e) => updateColumn(col.id, { format: ((e.target as HTMLSelectElement).value || undefined) as FormatType | undefined })">
          <option value="">Yok</option>
          <option value="currency">Para birimi</option>
          <option value="number">Sayi</option>
          <option value="date">Tarih</option>
          <option value="percentage">Yuzde</option>
        </select>
      </div>
      <div class="tbl-col__extra" data-tip="Sutun genislik modu">
        <label class="tbl-col__elabel">Genislik</label>
        <select class="tbl-col__wtype" :value="col.width.type"
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
        <span v-if="col.width.type === 'fixed' || col.width.type === 'fr'" class="ts-tip-wrap" :data-tip="col.width.type === 'fixed' ? 'Sabit genislik (mm)' : 'Oran degeri (fr)'">
          <input class="tbl-col__wval" type="number" step="1"
            :min="col.width.type === 'fixed' ? 5 : 1"
            :value="(col.width as any).value"
            @change="(e) => updateColumn(col.id, { width: { type: col.width.type, value: parseFloat((e.target as HTMLInputElement).value) || (col.width.type === 'fixed' ? 30 : 1) } as any })" />
        </span>
      </div>
    </div>
  </div>

  <!-- Table style -->
  <div class="prop-section">
    <div class="prop-section__title">Tablo Stili</div>

    <div class="ts-form">
      <!-- Font sizes -->
      <label class="ts-lbl" data-tip="Icerik ve header yazi boyutu (pt)">Yazi boyutu</label>
      <div class="ts-val ts-val--pair">
        <span class="ts-sep">Icerik</span>
        <span class="ts-tip-wrap" data-tip="Icerik yazi boyutu (pt)">
          <input class="ts-num" type="number" step="1" min="6" max="99"
            :value="element.style.fontSize ?? 10"
            @input="(e) => updateTableStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 10)" />
        </span>
        <span class="ts-sep">Header</span>
        <span class="ts-tip-wrap" data-tip="Header yazi boyutu (pt)">
          <input class="ts-num" type="number" step="1" min="6" max="99"
            :value="element.style.headerFontSize ?? element.style.fontSize ?? 10"
            @input="(e) => updateTableStyle('headerFontSize', parseFloat((e.target as HTMLInputElement).value) || 10)" />
        </span>
      </div>

      <!-- Colors -->
      <label class="ts-lbl" data-tip="Header, metin ve zebra satirlari renkleri">Renkler</label>
      <div class="ts-val ts-val--colors">
        <div class="ts-color-item" data-tip="Header arkaplan rengi">
          <input class="ts-swatch" type="color"
            :value="element.style.headerBg ?? '#f0f0f0'"
            @input="(e) => updateTableStyle('headerBg', (e.target as HTMLInputElement).value)" />
          <span class="ts-clbl">Arkaplan</span>
        </div>
        <div class="ts-color-item" data-tip="Header metin rengi">
          <input class="ts-swatch" type="color"
            :value="element.style.headerColor ?? '#000000'"
            @input="(e) => updateTableStyle('headerColor', (e.target as HTMLInputElement).value)" />
          <span class="ts-clbl">Metin</span>
        </div>
        <div class="ts-color-item" data-tip="Zebra satir rengi — tek satirlar">
          <div class="ts-swatch-wrap">
            <input class="ts-swatch" type="color"
              :value="element.style.zebraOdd ?? '#fafafa'"
              @input="(e) => updateTableStyle('zebraOdd', (e.target as HTMLInputElement).value)" />
            <button v-if="element.style.zebraOdd" class="ts-swatch-clr" @click="updateTableStyle('zebraOdd', undefined)">&times;</button>
          </div>
          <span class="ts-clbl">Zebra</span>
        </div>
      </div>

      <!-- Border -->
      <label class="ts-lbl" data-tip="Tablo kenarlik rengi ve kalinligi">Kenarlik</label>
      <div class="ts-val ts-val--pair">
        <div class="ts-swatch-wrap" data-tip="Kenarlik rengi">
          <input class="ts-swatch" type="color"
            :value="element.style.borderColor ?? '#cccccc'"
            @input="(e) => updateTableStyle('borderColor', (e.target as HTMLInputElement).value)" />
          <button v-if="element.style.borderColor" class="ts-swatch-clr" @click="updateTableStyle('borderColor', undefined)">&times;</button>
        </div>
        <span class="ts-tip-wrap" data-tip="Kenarlik kalinligi (mm)">
          <input class="ts-num" type="number" step="0.1" min="0" max="99"
            :value="element.style.borderWidth ?? 0.5"
            @input="(e) => updateTableStyle('borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </span>
        <span class="ts-unit">mm</span>
      </div>

      <!-- Cell padding -->
      <label class="ts-lbl" data-tip="Hucre ic bosluklari — yatay ve dikey (mm)">Ic bosluk</label>
      <div class="ts-val ts-val--pair">
        <span class="ts-pad-icon" data-tip="Yatay bosluk (mm)">&#8596;</span>
        <span class="ts-tip-wrap" data-tip="Yatay ic bosluk (mm)">
          <input class="ts-num" type="number" step="0.5" min="0" max="99"
            :value="element.style.cellPaddingH ?? 2"
            @input="(e) => updateTableStyle('cellPaddingH', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </span>
        <span class="ts-pad-icon" data-tip="Dikey bosluk (mm)">&#8597;</span>
        <span class="ts-tip-wrap" data-tip="Dikey ic bosluk (mm)">
          <input class="ts-num" type="number" step="0.5" min="0" max="99"
            :value="element.style.cellPaddingV ?? 1"
            @input="(e) => updateTableStyle('cellPaddingV', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </span>
      </div>

      <!-- Header padding -->
      <label class="ts-lbl" data-tip="Header hucre bosluklari — yatay ve dikey (mm)">Header bosluk</label>
      <div class="ts-val ts-val--pair">
        <span class="ts-pad-icon" data-tip="Yatay bosluk (mm)">&#8596;</span>
        <span class="ts-tip-wrap" data-tip="Header yatay bosluk (mm)">
          <input class="ts-num" type="number" step="0.5" min="0" max="99"
            :value="element.style.headerPaddingH ?? element.style.cellPaddingH ?? 2"
            @input="(e) => updateTableStyle('headerPaddingH', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </span>
        <span class="ts-pad-icon" data-tip="Dikey bosluk (mm)">&#8597;</span>
        <span class="ts-tip-wrap" data-tip="Header dikey bosluk (mm)">
          <input class="ts-num" type="number" step="0.5" min="0" max="99"
            :value="element.style.headerPaddingV ?? element.style.cellPaddingV ?? 1"
            @input="(e) => updateTableStyle('headerPaddingV', parseFloat((e.target as HTMLInputElement).value) || 0)" />
        </span>
      </div>

      <!-- Repeat header -->
      <label class="ts-lbl" data-tip="Cok sayfali tablolarda header'i her sayfada tekrarla">Header tekrarla</label>
      <div class="ts-val">
        <label class="ts-toggle">
          <input type="checkbox"
            :checked="element.repeatHeader !== false"
            @change="(e) => update({ repeatHeader: (e.target as HTMLInputElement).checked } as any)" />
          <span class="ts-toggle__track"></span>
        </label>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Column card - compact */
.tbl-col {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 5px;
  padding: 5px 6px;
  margin-bottom: 5px;
}

.tbl-col__head {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 4px;
}

.tbl-col__title {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  font-size: 12px;
  font-weight: 500;
  color: #334155;
  padding: 1px 0;
  outline: none;
}

.tbl-col__title:focus {
  border-bottom: 1px solid #93c5fd;
}

.tbl-col__actions {
  display: flex;
  gap: 1px;
  flex-shrink: 0;
}

.tbl-col__act {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: #94a3b8;
  cursor: pointer;
  padding: 0;
}

.tbl-col__act:hover {
  background: #e2e8f0;
  color: #475569;
}

.tbl-col__act--del:hover {
  background: #fef2f2;
  color: #dc2626;
}

.tbl-col__controls {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 3px;
}

.tbl-col__field {
  flex: 1;
  min-width: 0;
  padding: 2px 4px;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
  font-size: 11px;
  background: white;
  color: #334155;
}

.tbl-col__field:focus {
  outline: none;
  border-color: #93c5fd;
}

.tbl-col__align {
  display: flex;
  gap: 0;
  flex-shrink: 0;
}

.tbl-col__align-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: 1px solid #e2e8f0;
  background: white;
  color: #94a3b8;
  cursor: pointer;
  padding: 0;
}

.tbl-col__align-btn:first-child {
  border-radius: 3px 0 0 3px;
}

.tbl-col__align-btn:last-child {
  border-radius: 0 3px 3px 0;
}

.tbl-col__align-btn:not(:first-child) {
  border-left: none;
}

.tbl-col__align-btn--on {
  background: #3b82f6;
  color: white;
  border-color: #3b82f6;
}

.tbl-col__extra {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 3px;
}

.tbl-col__elabel {
  font-size: 11px;
  color: #64748b;
  flex-shrink: 0;
}

.tbl-col__fmt {
  flex: 1;
  min-width: 0;
  padding: 2px 4px;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
  font-size: 11px;
  background: white;
  color: #334155;
  cursor: pointer;
}

.tbl-col__wtype {
  width: 80px;
  padding: 2px 4px;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
  font-size: 11px;
  background: white;
  color: #334155;
  cursor: pointer;
}

.tbl-col__wval {
  width: 36px;
  padding: 2px 3px;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
  font-size: 11px;
  background: white;
  color: #334155;
  text-align: center;
  -moz-appearance: textfield;
}

.tbl-col__wval::-webkit-inner-spin-button,
.tbl-col__wval::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.tbl-col__wval:focus {
  outline: none;
  border-color: #93c5fd;
}

/* Table style — aligned 2-column form */
.ts-form {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 5px 8px;
  align-items: center;
}

.ts-lbl {
  font-size: 11px;
  color: #64748b;
  white-space: nowrap;
}

.ts-val {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.ts-val--pair {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
}

.ts-val--colors {
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  gap: 6px;
}

.ts-sep {
  font-size: 10px;
  color: #94a3b8;
}

.ts-num {
  width: 32px;
  padding: 2px 3px;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
  font-size: 11px;
  background: white;
  color: #334155;
  text-align: center;
  -moz-appearance: textfield;
}

.ts-num::-webkit-inner-spin-button,
.ts-num::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.ts-num:focus {
  outline: none;
  border-color: #93c5fd;
}

.ts-unit {
  font-size: 10px;
  color: #94a3b8;
}

/* Color swatches */
.ts-color-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.ts-clbl {
  font-size: 9px;
  color: #94a3b8;
  white-space: nowrap;
}

.ts-swatch {
  width: 22px;
  height: 22px;
  padding: 0;
  cursor: pointer;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
}

.ts-swatch-wrap {
  position: relative;
  display: inline-flex;
}

.ts-swatch-clr {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #f1f5f9;
  border: 1px solid #e2e8f0;
  font-size: 9px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #94a3b8;
  padding: 0;
}

.ts-swatch-clr:hover {
  background: #fef2f2;
  color: #dc2626;
  border-color: #fecaca;
}

.ts-pad-icon {
  font-size: 11px;
  color: #94a3b8;
  line-height: 1;
}

.ts-tip-wrap {
  position: relative;
  display: inline-flex;
}

/* Toggle switch */
.ts-toggle {
  position: relative;
  display: inline-block;
  cursor: pointer;
}

.ts-toggle input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.ts-toggle__track {
  display: block;
  width: 28px;
  height: 16px;
  background: #e2e8f0;
  border-radius: 8px;
  transition: background 0.15s;
  position: relative;
}

.ts-toggle__track::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 12px;
  height: 12px;
  background: white;
  border-radius: 50%;
  transition: transform 0.15s;
  box-shadow: 0 1px 2px rgba(0,0,0,0.1);
}

.ts-toggle input:checked + .ts-toggle__track {
  background: #3b82f6;
}

.ts-toggle input:checked + .ts-toggle__track::after {
  transform: translateX(12px);
}
</style>
