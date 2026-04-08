<script setup lang="ts">
import { defaultAlignForSchema, schemaFormatToFormatType } from '../../../core/schema-parser'
import type { TableColumn, FormatType } from '../../../core/types'

type ItemField = { key: string; title: string; type?: string; format?: string }
import '../../../styles/properties.css'

defineProps<{
  column: TableColumn
  itemFields: ItemField[]
}>()

const emit = defineEmits<{
  update: [colId: string, updates: Partial<TableColumn>]
  remove: [colId: string]
  move: [colId: string, direction: -1 | 1]
}>()
</script>

<template>
  <div class="tbl-col">
    <!-- Row 1: title + actions -->
    <div class="tbl-col__head">
      <input
        class="tbl-col__title"
        type="text"
        :value="column.title"
        @change="(e) => emit('update', column.id, { title: (e.target as HTMLInputElement).value })"
        :placeholder="column.field"
        data-tip="Sutun basligi"
      />
      <div class="tbl-col__actions">
        <button class="tbl-col__act" @click="emit('move', column.id, -1)" data-tip="Yukari tasi">
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path d="M5 2L2 6h6L5 2z" fill="currentColor" />
          </svg>
        </button>
        <button class="tbl-col__act" @click="emit('move', column.id, 1)" data-tip="Asagi tasi">
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path d="M5 8L2 4h6L5 8z" fill="currentColor" />
          </svg>
        </button>
        <button
          class="tbl-col__act tbl-col__act--del"
          @click="emit('remove', column.id)"
          data-tip="Sutunu sil"
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path
              d="M2 2l6 6M8 2l-6 6"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </div>

    <!-- Row 2: field + align -->
    <div class="tbl-col__controls">
      <select
        v-if="itemFields.length > 0"
        class="tbl-col__field"
        :value="column.field"
        data-tip="Veri alani"
        @change="
          (e) => {
            const field = (e.target as HTMLSelectElement).value
            const node = itemFields.find((f) => f.key === field)
            if (node) {
              emit('update', column.id, {
                field,
                title: node.title,
                align: defaultAlignForSchema(node as any),
                format: schemaFormatToFormatType(node.format),
              })
            } else {
              emit('update', column.id, { field })
            }
          }
        "
      >
        <option v-for="f in itemFields" :key="f.key" :value="f.key">{{ f.key }}</option>
      </select>
      <input
        v-else
        class="tbl-col__field"
        type="text"
        :value="column.field"
        @change="(e) => emit('update', column.id, { field: (e.target as HTMLInputElement).value })"
        data-tip="Veri alani"
      />

      <!-- Alignment icons -->
      <div class="tbl-col__align">
        <button
          class="tbl-col__align-btn"
          :class="{ 'tbl-col__align-btn--on': column.align === 'left' }"
          @click="emit('update', column.id, { align: 'left' })"
          data-tip="Sola hizala"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1" y1="3" x2="11" y2="3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            <line x1="1" y1="6" x2="8" y2="6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            <line x1="1" y1="9" x2="10" y2="9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </button>
        <button
          class="tbl-col__align-btn"
          :class="{ 'tbl-col__align-btn--on': column.align === 'center' }"
          @click="emit('update', column.id, { align: 'center' })"
          data-tip="Ortala"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1" y1="3" x2="11" y2="3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            <line x1="2.5" y1="6" x2="9.5" y2="6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            <line x1="1.5" y1="9" x2="10.5" y2="9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </button>
        <button
          class="tbl-col__align-btn"
          :class="{ 'tbl-col__align-btn--on': column.align === 'right' }"
          @click="emit('update', column.id, { align: 'right' })"
          data-tip="Saga hizala"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <line x1="1" y1="3" x2="11" y2="3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            <line x1="4" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
            <line x1="2" y1="9" x2="11" y2="9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Row 3: format + width -->
    <div class="tbl-col__extra" data-tip="Veri gosterim formati">
      <label class="tbl-col__elabel">Format</label>
      <select
        class="tbl-col__fmt"
        :value="column.format ?? ''"
        @change="
          (e) =>
            emit('update', column.id, {
              format: ((e.target as HTMLSelectElement).value || undefined) as FormatType | undefined,
            })
        "
      >
        <option value="">Yok</option>
        <option value="currency">Para birimi</option>
        <option value="number">Sayi</option>
        <option value="date">Tarih</option>
        <option value="percentage">Yuzde</option>
      </select>
    </div>
    <div class="tbl-col__extra" data-tip="Sutun genislik modu">
      <label class="tbl-col__elabel">Genislik</label>
      <select
        class="tbl-col__wtype"
        :value="column.width.type"
        @change="
          (e) => {
            const t = (e.target as HTMLSelectElement).value
            if (t === 'auto') emit('update', column.id, { width: { type: 'auto' } })
            else if (t === 'fr') emit('update', column.id, { width: { type: 'fr', value: 1 } })
            else emit('update', column.id, { width: { type: 'fixed', value: 30 } })
          }
        "
      >
        <option value="auto">Otomatik</option>
        <option value="fixed">Sabit (mm)</option>
        <option value="fr">Oran (fr)</option>
      </select>
      <span
        v-if="column.width.type === 'fixed' || column.width.type === 'fr'"
        class="ts-tip-wrap"
        :data-tip="column.width.type === 'fixed' ? 'Sabit genislik (mm)' : 'Oran degeri (fr)'"
      >
        <input
          class="tbl-col__wval"
          type="number"
          step="1"
          :min="column.width.type === 'fixed' ? 5 : 1"
          :value="(column.width as any).value"
          @change="
            (e) =>
              emit('update', column.id, {
                width: {
                  type: column.width.type,
                  value:
                    parseFloat((e.target as HTMLInputElement).value) ||
                    (column.width.type === 'fixed' ? 30 : 1),
                } as any,
              })
          "
        />
      </span>
    </div>
  </div>
</template>

<style scoped>
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

.ts-tip-wrap {
  position: relative;
  display: inline-flex;
}
</style>
