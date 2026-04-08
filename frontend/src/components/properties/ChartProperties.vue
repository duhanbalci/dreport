<script setup lang="ts">
import { computed } from 'vue'
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import { useSchemaStore } from '../../stores/schema'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropNumberInput from './shared/PropNumberInput.vue'
import PropColorInput from './shared/PropColorInput.vue'
import PropCheckbox from './shared/PropCheckbox.vue'
import PropFieldSelect from './shared/PropFieldSelect.vue'
import type { ChartElement, ChartType, GroupMode } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ChartElement }>()
const { update, updateStyle, updateNested } = usePropertyUpdate(() => props.element)
const schemaStore = useSchemaStore()

const chartTypeOptions = [
  { value: 'bar', label: 'Bar' },
  { value: 'line', label: 'Line' },
  { value: 'pie', label: 'Pie' },
]

const groupModeOptions = [
  { value: 'grouped', label: 'Yan Yana' },
  { value: 'stacked', label: 'Yigin' },
]

const alignOptions = [
  { value: 'left', label: 'Sol' },
  { value: 'center', label: 'Orta' },
  { value: 'right', label: 'Sag' },
]

const legendPositionOptions = [
  { value: 'top', label: 'Ust' },
  { value: 'bottom', label: 'Alt' },
  { value: 'right', label: 'Sag' },
]

const itemFields = computed(() => {
  const path = props.element.dataSource?.path
  if (!path) return []
  return schemaStore.getArrayItemFields(path)
})

const stringFields = computed(() => itemFields.value.filter((f) => f.type === 'string'))
const numberFields = computed(() =>
  itemFields.value.filter((f) => f.type === 'number' || f.type === 'integer'),
)

const isPie = computed(() => props.element.chartType === 'pie')
const hasGroup = computed(() => !!props.element.groupField)

const colorList = computed(() => {
  return (
    props.element.style.colors ?? ['#4F46E5', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6', '#EC4899']
  )
})

function updateDataSource(path: string) {
  const fields = schemaStore.getArrayItemFields(path)
  const strField = fields.find((f) => f.type === 'string')
  const numField = fields.find((f) => f.type === 'number' || f.type === 'integer')
  update({
    dataSource: { type: 'array', path },
    categoryField: strField?.key ?? fields[0]?.key ?? '',
    valueField: numField?.key ?? fields[1]?.key ?? '',
    groupField: undefined,
  } as any)
}

function updateColor(index: number, value: string) {
  const colors = [...colorList.value]
  colors[index] = value
  updateStyle('colors', colors)
}

function addColor() {
  updateStyle('colors', [...colorList.value, '#6B7280'])
}

function removeColor(index: number) {
  const colors = colorList.value.filter((_, i) => i !== index)
  updateStyle('colors', colors.length > 0 ? colors : undefined)
}
</script>

<template>
  <div class="chart-properties">
    <!-- Grafik Tipi -->
    <PropSection title="Grafik Tipi">
      <PropSelect
        label=""
        :model-value="element.chartType"
        :options="chartTypeOptions"
        @update:model-value="(v) => update({ chartType: v as ChartType } as any)"
      />
    </PropSection>

    <!-- Veri Kaynagi -->
    <PropSection title="Veri Kaynagi">
      <PropFieldSelect
        label="Array"
        :model-value="element.dataSource?.path ?? ''"
        :fields="schemaStore.arrayFields"
        placeholder="Sec..."
        @update:model-value="updateDataSource"
      />
      <PropFieldSelect
        label="Kategori"
        :model-value="element.categoryField"
        :fields="itemFields"
        @update:model-value="(v) => update({ categoryField: v } as any)"
      />
      <PropFieldSelect
        label="Deger"
        :model-value="element.valueField"
        :fields="numberFields"
        @update:model-value="(v) => update({ valueField: v } as any)"
      />
      <PropFieldSelect
        label="Gruplama"
        :model-value="element.groupField ?? ''"
        :fields="stringFields"
        :allow-empty="true"
        empty-label="Yok"
        @update:model-value="(v) => update({ groupField: v || undefined } as any)"
      />
      <PropSelect
        v-if="hasGroup && !isPie"
        label="Grup Modu"
        :model-value="element.groupMode ?? 'grouped'"
        :options="groupModeOptions"
        @update:model-value="(v) => update({ groupMode: v as GroupMode } as any)"
      />
    </PropSection>

    <!-- Baslik -->
    <PropSection title="Baslik">
      <div class="prop-row">
        <label class="prop-label">Metin</label>
        <input
          class="prop-input"
          type="text"
          :value="element.title?.text ?? ''"
          @change="(e) => updateNested('title', 'text', (e.target as HTMLInputElement).value, { text: '' })"
          placeholder="Grafik basligi"
        />
      </div>
      <template v-if="element.title?.text">
        <PropNumberInput
          label="Boyut"
          :model-value="element.title?.fontSize ?? 4"
          :step="0.5"
          @update:model-value="(v) => updateNested('title', 'fontSize', v, { text: '' })"
        />
        <PropColorInput
          label="Renk"
          :model-value="element.title?.color ?? '#333333'"
          @update:model-value="(v) => updateNested('title', 'color', v, { text: '' })"
        />
        <PropSelect
          label="Hiza"
          :model-value="element.title?.align ?? 'center'"
          :options="alignOptions"
          @update:model-value="(v) => updateNested('title', 'align', v, { text: '' })"
        />
      </template>
    </PropSection>

    <!-- Gosterge (Legend) -->
    <PropSection title="Gosterge">
      <PropCheckbox
        label="Goster"
        :model-value="element.legend?.show ?? false"
        @update:model-value="(v) => updateNested('legend', 'show', v, { show: false })"
      />
      <template v-if="element.legend?.show">
        <PropSelect
          label="Konum"
          :model-value="element.legend?.position ?? 'bottom'"
          :options="legendPositionOptions"
          @update:model-value="(v) => updateNested('legend', 'position', v)"
        />
        <PropNumberInput
          label="Boyut"
          :model-value="element.legend?.fontSize ?? 2.8"
          :step="0.2"
          @update:model-value="(v) => updateNested('legend', 'fontSize', v)"
        />
      </template>
    </PropSection>

    <!-- Etiketler -->
    <PropSection title="Etiketler">
      <PropCheckbox
        label="Goster"
        :model-value="element.labels?.show ?? false"
        @update:model-value="(v) => updateNested('labels', 'show', v, { show: false })"
      />
      <template v-if="element.labels?.show">
        <PropNumberInput
          label="Boyut"
          :model-value="element.labels?.fontSize ?? 2.2"
          :step="0.2"
          @update:model-value="(v) => updateNested('labels', 'fontSize', v)"
        />
        <PropColorInput
          label="Renk"
          :model-value="element.labels?.color ?? '#333333'"
          @update:model-value="(v) => updateNested('labels', 'color', v)"
        />
      </template>
    </PropSection>

    <!-- Eksenler (pie haric) -->
    <PropSection v-if="!isPie" title="Eksenler">
      <div class="prop-row">
        <label class="prop-label">X Etiketi</label>
        <input
          class="prop-input"
          type="text"
          :value="element.axis?.xLabel ?? ''"
          @change="(e) => updateNested('axis', 'xLabel', (e.target as HTMLInputElement).value || undefined, {})"
          placeholder="X ekseni"
        />
      </div>
      <div class="prop-row">
        <label class="prop-label">Y Etiketi</label>
        <input
          class="prop-input"
          type="text"
          :value="element.axis?.yLabel ?? ''"
          @change="(e) => updateNested('axis', 'yLabel', (e.target as HTMLInputElement).value || undefined, {})"
          placeholder="Y ekseni"
        />
      </div>
      <PropCheckbox
        label="Izgara"
        :model-value="element.axis?.showGrid ?? true"
        @update:model-value="(v) => updateNested('axis', 'showGrid', v, {})"
      />
      <PropColorInput
        v-if="element.axis?.showGrid !== false"
        label="Izgara Renk"
        :model-value="element.axis?.gridColor ?? '#E5E7EB'"
        @update:model-value="(v) => updateNested('axis', 'gridColor', v, {})"
      />
      <template v-if="element.chartType === 'line'">
        <PropCheckbox
          label="Dikey Izgara"
          :model-value="element.axis?.showVerticalGrid ?? true"
          @update:model-value="(v) => updateNested('axis', 'showVerticalGrid', v, {})"
        />
        <PropColorInput
          v-if="element.axis?.showVerticalGrid !== false"
          label="Dikey Izgara Renk"
          :model-value="element.axis?.verticalGridColor ?? '#E5E7EB'"
          @update:model-value="(v) => updateNested('axis', 'verticalGridColor', v, {})"
        />
      </template>
    </PropSection>

    <!-- Stil -->
    <PropSection title="Stil">
      <PropColorInput
        label="Arka Plan"
        :model-value="element.style.backgroundColor ?? '#FFFFFF'"
        @update:model-value="(v) => updateStyle('backgroundColor', v)"
      />

      <div class="prop-section__subtitle">Renk Paleti</div>
      <div v-for="(color, i) in colorList" :key="i" class="prop-row">
        <input
          class="prop-color"
          type="color"
          :value="color"
          @input="(e) => updateColor(i, (e.target as HTMLInputElement).value)"
        />
        <button class="prop-btn-sm prop-btn-sm--danger" @click="removeColor(i)" title="Kaldir">
          ×
        </button>
      </div>
      <button class="prop-btn-sm" @click="addColor">+ Renk Ekle</button>
    </PropSection>

    <!-- Tipe Ozel -->
    <PropSection v-if="element.chartType === 'bar'" title="Bar Ayarlari">
      <PropNumberInput
        label="Bar Boslugu"
        :model-value="element.style.barGap ?? 0.2"
        :step="0.05"
        :min="0"
        :max="0.8"
        @update:model-value="(v) => updateStyle('barGap', v)"
      />
    </PropSection>

    <PropSection v-if="element.chartType === 'line'" title="Line Ayarlari">
      <PropNumberInput
        label="Cizgi Kalinligi"
        :model-value="element.style.lineWidth ?? 0.5"
        :step="0.1"
        :min="0.1"
        @update:model-value="(v) => updateStyle('lineWidth', v)"
      />
      <PropSelect
        label="Egri Tipi"
        :model-value="element.style.curveType ?? 'linear'"
        :options="[{ value: 'linear', label: 'Duz' }, { value: 'smooth', label: 'Yumusak' }]"
        @update:model-value="(v) => updateStyle('curveType', v)"
      />
      <PropCheckbox
        label="Noktalar"
        :model-value="element.style.showPoints ?? true"
        @update:model-value="(v) => updateStyle('showPoints', v)"
      />
    </PropSection>

    <PropSection v-if="element.chartType === 'pie'" title="Pie Ayarlari">
      <PropNumberInput
        label="Ic Yaricap"
        :model-value="element.style.innerRadius ?? 0"
        :step="0.05"
        :min="0"
        :max="0.9"
        @update:model-value="(v) => updateStyle('innerRadius', v)"
      />
      <div class="prop-row" style="font-size: 11px; color: #94a3b8">0 = Pie, &gt;0 = Donut</div>
    </PropSection>
  </div>
</template>

<style scoped>
.chart-properties {
  padding: 0;
}

.prop-btn-sm {
  padding: 2px 8px;
  font-size: 11px;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  background: white;
  color: #475569;
  cursor: pointer;
}

.prop-btn-sm:hover {
  background: #f8fafc;
}

.prop-btn-sm--danger {
  color: #ef4444;
  border-color: #fecaca;
}

.prop-btn-sm--danger:hover {
  background: #fef2f2;
}
</style>
