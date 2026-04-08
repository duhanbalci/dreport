<script setup lang="ts">
import { computed } from 'vue'
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import { useSchemaStore } from '../../stores/schema'
import { sz } from '../../core/types'
import { schemaFormatToFormatType, defaultAlignForSchema } from '../../core/schema-parser'
import PropSection from './shared/PropSection.vue'
import PropFieldSelect from './shared/PropFieldSelect.vue'
import TableColumnEditor from './table/TableColumnEditor.vue'
import TableStyleEditor from './table/TableStyleEditor.vue'
import type { RepeatingTableElement, TableColumn, TableStyle } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: RepeatingTableElement }>()
const { update } = usePropertyUpdate(() => props.element)
const schemaStore = useSchemaStore()

let colIdCounter = Date.now()
function nextColId() {
  return `col_${(++colIdCounter).toString(36)}`
}

function updateTableDataSource(path: string) {
  const itemFields = schemaStore.getArrayItemFields(path)
  if (itemFields.length > 0) {
    const columns: TableColumn[] = itemFields.map((field) => ({
      id: nextColId(),
      field: field.key,
      title: field.title,
      width: sz.auto(),
      align: defaultAlignForSchema(field),
      format: schemaFormatToFormatType(field.format),
    }))
    update({ dataSource: { type: 'array', path }, columns } as any)
  } else {
    update({ dataSource: { type: 'array', path } } as any)
  }
}

function updateTableStyle(key: string, value: unknown) {
  const newStyle = { ...props.element.style, [key]: value }
  if (value === undefined || value === '') delete (newStyle as Record<string, unknown>)[key]
  update({ style: newStyle } as any)
}

function updateColumn(colId: string, updates: Partial<TableColumn>) {
  const columns = props.element.columns.map((c) => (c.id === colId ? { ...c, ...updates } : c))
  update({ columns } as any)
}

function addColumn() {
  const newCol: TableColumn = {
    id: nextColId(),
    field: 'alan',
    title: 'Yeni Sutun',
    width: sz.auto(),
    align: 'left',
  }
  update({ columns: [...props.element.columns, newCol] } as any)
}

function removeColumn(colId: string) {
  update({ columns: props.element.columns.filter((c) => c.id !== colId) } as any)
}

function moveColumn(colId: string, direction: -1 | 1) {
  const cols = [...props.element.columns]
  const idx = cols.findIndex((c) => c.id === colId)
  const newIdx = idx + direction
  if (newIdx < 0 || newIdx >= cols.length) return
  ;[cols[idx], cols[newIdx]] = [cols[newIdx], cols[idx]]
  update({ columns: cols } as any)
}

const tableItemFields = computed(() => {
  return schemaStore.getArrayItemFields(props.element.dataSource.path)
})
</script>

<template>
  <!-- Data source -->
  <PropSection title="Veri Kaynagi">
    <PropFieldSelect
      label="Kaynak"
      :model-value="element.dataSource.path"
      :fields="schemaStore.arrayFields"
      data-tip="Tablonun baglanacagi array veri kaynagi"
      @update:model-value="updateTableDataSource"
    />
  </PropSection>

  <!-- Columns -->
  <PropSection title="Sutunlar">
    <template #actions>
      <button class="prop-add-btn" @click="addColumn">+</button>
    </template>
    <TableColumnEditor
      v-for="col in element.columns"
      :key="col.id"
      :column="col"
      :item-fields="tableItemFields"
      @update="updateColumn"
      @remove="removeColumn"
      @move="moveColumn"
    />
  </PropSection>

  <!-- Table style -->
  <PropSection title="Tablo Stili">
    <TableStyleEditor
      :style="element.style as TableStyle"
      :repeat-header="element.repeatHeader !== false"
      @update:style="updateTableStyle"
      @update:repeat-header="(v) => update({ repeatHeader: v } as any)"
    />
  </PropSection>
</template>
