<script setup lang="ts">
import { computed } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { useSchemaStore } from '../../stores/schema'
import type { ChartElement, ChartType, GroupMode, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: ChartElement }>()
const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const schemaStore = useSchemaStore()

function update(updates: Partial<ChartElement>) {
  const id = editorStore.selectedElementId
  if (!id) return
  templateStore.updateElement(id, updates as Partial<TemplateElement>)
}

function updateStyle(key: string, value: unknown) {
  const newStyle = { ...props.element.style, [key]: value }
  if (value === undefined || value === '') delete (newStyle as Record<string, unknown>)[key]
  update({ style: newStyle })
}

// Schema'daki array alanlari
const arrayFields = computed(() => schemaStore.arrayFields)

// Secili array'in item alanlari
const itemFields = computed(() => {
  const path = props.element.dataSource?.path
  if (!path) return []
  return schemaStore.getArrayItemFields(path)
})

const stringFields = computed(() => itemFields.value.filter((f) => f.type === 'string'))
const numberFields = computed(() =>
  itemFields.value.filter((f) => f.type === 'number' || f.type === 'integer'),
)

function updateDataSource(path: string) {
  const fields = schemaStore.getArrayItemFields(path)
  const strField = fields.find((f) => f.type === 'string')
  const numField = fields.find((f) => f.type === 'number' || f.type === 'integer')
  update({
    dataSource: { type: 'array', path },
    categoryField: strField?.key ?? fields[0]?.key ?? '',
    valueField: numField?.key ?? fields[1]?.key ?? '',
    groupField: undefined,
  })
}

function updateTitle(key: string, value: unknown) {
  const current = props.element.title ?? { text: '' }
  update({ title: { ...current, [key]: value } })
}

function updateLegend(key: string, value: unknown) {
  const current = props.element.legend ?? { show: false }
  update({ legend: { ...current, [key]: value } })
}

function updateLabels(key: string, value: unknown) {
  const current = props.element.labels ?? { show: false }
  update({ labels: { ...current, [key]: value } })
}

function updateAxis(key: string, value: unknown) {
  const current = props.element.axis ?? {}
  update({ axis: { ...current, [key]: value } })
}

const isPie = computed(() => props.element.chartType === 'pie')
const hasGroup = computed(() => !!props.element.groupField)

// Renk paleti (default 6 renk)
const colorList = computed(() => {
  return (
    props.element.style.colors ?? ['#4F46E5', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6', '#EC4899']
  )
})

function updateColor(index: number, value: string) {
  const colors = [...colorList.value]
  colors[index] = value
  updateStyle('colors', colors)
}

function addColor() {
  const colors = [...colorList.value, '#6B7280']
  updateStyle('colors', colors)
}

function removeColor(index: number) {
  const colors = colorList.value.filter((_, i) => i !== index)
  updateStyle('colors', colors.length > 0 ? colors : undefined)
}
</script>

<template>
  <div class="chart-properties">
    <!-- Grafik Tipi -->
    <div class="prop-section">
      <div class="prop-section__title">Grafik Tipi</div>
      <div class="prop-row">
        <select
          class="prop-input prop-select"
          :value="element.chartType"
          @change="update({ chartType: ($event.target as HTMLSelectElement).value as ChartType })"
        >
          <option value="bar">Bar</option>
          <option value="line">Line</option>
          <option value="pie">Pie</option>
        </select>
      </div>
    </div>

    <!-- Veri Kaynagi -->
    <div class="prop-section">
      <div class="prop-section__title">Veri Kaynagi</div>
      <div class="prop-row">
        <label class="prop-label">Array</label>
        <select
          class="prop-input prop-select"
          :value="element.dataSource?.path ?? ''"
          @change="updateDataSource(($event.target as HTMLSelectElement).value)"
        >
          <option value="" disabled>Sec...</option>
          <option v-for="arr in arrayFields" :key="arr.path" :value="arr.path">
            {{ arr.title || arr.path }}
          </option>
        </select>
      </div>
      <div class="prop-row">
        <label class="prop-label">Kategori</label>
        <select
          class="prop-input prop-select"
          :value="element.categoryField"
          @change="update({ categoryField: ($event.target as HTMLSelectElement).value })"
        >
          <option v-for="f in itemFields" :key="f.key" :value="f.key">
            {{ f.title || f.key }}
          </option>
        </select>
      </div>
      <div class="prop-row">
        <label class="prop-label">Deger</label>
        <select
          class="prop-input prop-select"
          :value="element.valueField"
          @change="update({ valueField: ($event.target as HTMLSelectElement).value })"
        >
          <option v-for="f in numberFields" :key="f.key" :value="f.key">
            {{ f.title || f.key }}
          </option>
        </select>
      </div>
      <div class="prop-row">
        <label class="prop-label">Gruplama</label>
        <select
          class="prop-input prop-select"
          :value="element.groupField ?? ''"
          @change="update({ groupField: ($event.target as HTMLSelectElement).value || undefined })"
        >
          <option value="">Yok</option>
          <option v-for="f in stringFields" :key="f.key" :value="f.key">
            {{ f.title || f.key }}
          </option>
        </select>
      </div>
      <div v-if="hasGroup && !isPie" class="prop-row">
        <label class="prop-label">Grup Modu</label>
        <select
          class="prop-input prop-select"
          :value="element.groupMode ?? 'grouped'"
          @change="update({ groupMode: ($event.target as HTMLSelectElement).value as GroupMode })"
        >
          <option value="grouped">Yan Yana</option>
          <option value="stacked">Yigin</option>
        </select>
      </div>
    </div>

    <!-- Baslik -->
    <div class="prop-section">
      <div class="prop-section__title">Baslik</div>
      <div class="prop-row">
        <label class="prop-label">Metin</label>
        <input
          class="prop-input"
          type="text"
          :value="element.title?.text ?? ''"
          @change="updateTitle('text', ($event.target as HTMLInputElement).value)"
          placeholder="Grafik basligi"
        />
      </div>
      <div class="prop-row" v-if="element.title?.text">
        <label class="prop-label">Boyut</label>
        <input
          class="prop-input prop-input--sm"
          type="number"
          :value="element.title?.fontSize ?? 4"
          step="0.5"
          @change="updateTitle('fontSize', parseFloat(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="prop-row" v-if="element.title?.text">
        <label class="prop-label">Renk</label>
        <input
          class="prop-color"
          type="color"
          :value="element.title?.color ?? '#333333'"
          @input="updateTitle('color', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="prop-row" v-if="element.title?.text">
        <label class="prop-label">Hiza</label>
        <select
          class="prop-input prop-select"
          :value="element.title?.align ?? 'center'"
          @change="updateTitle('align', ($event.target as HTMLSelectElement).value)"
        >
          <option value="left">Sol</option>
          <option value="center">Orta</option>
          <option value="right">Sag</option>
        </select>
      </div>
    </div>

    <!-- Gosterge (Legend) -->
    <div class="prop-section">
      <div class="prop-section__title">Gosterge</div>
      <div class="prop-row">
        <label class="prop-label">Goster</label>
        <input
          type="checkbox"
          :checked="element.legend?.show ?? false"
          @change="updateLegend('show', ($event.target as HTMLInputElement).checked)"
        />
      </div>
      <template v-if="element.legend?.show">
        <div class="prop-row">
          <label class="prop-label">Konum</label>
          <select
            class="prop-input prop-select"
            :value="element.legend?.position ?? 'bottom'"
            @change="updateLegend('position', ($event.target as HTMLSelectElement).value)"
          >
            <option value="top">Ust</option>
            <option value="bottom">Alt</option>
            <option value="right">Sag</option>
          </select>
        </div>
        <div class="prop-row">
          <label class="prop-label">Boyut</label>
          <input
            class="prop-input prop-input--sm"
            type="number"
            :value="element.legend?.fontSize ?? 2.8"
            step="0.2"
            @change="
              updateLegend('fontSize', parseFloat(($event.target as HTMLInputElement).value))
            "
          />
        </div>
      </template>
    </div>

    <!-- Etiketler -->
    <div class="prop-section">
      <div class="prop-section__title">Etiketler</div>
      <div class="prop-row">
        <label class="prop-label">Goster</label>
        <input
          type="checkbox"
          :checked="element.labels?.show ?? false"
          @change="updateLabels('show', ($event.target as HTMLInputElement).checked)"
        />
      </div>
      <template v-if="element.labels?.show">
        <div class="prop-row">
          <label class="prop-label">Boyut</label>
          <input
            class="prop-input prop-input--sm"
            type="number"
            :value="element.labels?.fontSize ?? 2.2"
            step="0.2"
            @change="
              updateLabels('fontSize', parseFloat(($event.target as HTMLInputElement).value))
            "
          />
        </div>
        <div class="prop-row">
          <label class="prop-label">Renk</label>
          <input
            class="prop-color"
            type="color"
            :value="element.labels?.color ?? '#333333'"
            @input="updateLabels('color', ($event.target as HTMLInputElement).value)"
          />
        </div>
      </template>
    </div>

    <!-- Eksenler (pie haric) -->
    <div class="prop-section" v-if="!isPie">
      <div class="prop-section__title">Eksenler</div>
      <div class="prop-row">
        <label class="prop-label">X Etiketi</label>
        <input
          class="prop-input"
          type="text"
          :value="element.axis?.xLabel ?? ''"
          @change="updateAxis('xLabel', ($event.target as HTMLInputElement).value || undefined)"
          placeholder="X ekseni"
        />
      </div>
      <div class="prop-row">
        <label class="prop-label">Y Etiketi</label>
        <input
          class="prop-input"
          type="text"
          :value="element.axis?.yLabel ?? ''"
          @change="updateAxis('yLabel', ($event.target as HTMLInputElement).value || undefined)"
          placeholder="Y ekseni"
        />
      </div>
      <div class="prop-row">
        <label class="prop-label">Izgara</label>
        <input
          type="checkbox"
          :checked="element.axis?.showGrid ?? true"
          @change="updateAxis('showGrid', ($event.target as HTMLInputElement).checked)"
        />
      </div>
      <div class="prop-row" v-if="element.axis?.showGrid !== false">
        <label class="prop-label">Izgara Renk</label>
        <input
          class="prop-color"
          type="color"
          :value="element.axis?.gridColor ?? '#E5E7EB'"
          @input="updateAxis('gridColor', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>

    <!-- Stil -->
    <div class="prop-section">
      <div class="prop-section__title">Stil</div>
      <div class="prop-row">
        <label class="prop-label">Arka Plan</label>
        <input
          class="prop-color"
          type="color"
          :value="element.style.backgroundColor ?? '#FFFFFF'"
          @input="updateStyle('backgroundColor', ($event.target as HTMLInputElement).value)"
        />
      </div>

      <!-- Renk Paleti -->
      <div class="prop-section__subtitle">Renk Paleti</div>
      <div v-for="(color, i) in colorList" :key="i" class="prop-row">
        <input
          class="prop-color"
          type="color"
          :value="color"
          @input="updateColor(i, ($event.target as HTMLInputElement).value)"
        />
        <button class="prop-btn-sm prop-btn-sm--danger" @click="removeColor(i)" title="Kaldir">
          ×
        </button>
      </div>
      <button class="prop-btn-sm" @click="addColor">+ Renk Ekle</button>
    </div>

    <!-- Tipe Ozel -->
    <div class="prop-section" v-if="element.chartType === 'bar'">
      <div class="prop-section__title">Bar Ayarlari</div>
      <div class="prop-row">
        <label class="prop-label">Bar Boslugu</label>
        <input
          class="prop-input prop-input--sm"
          type="number"
          :value="element.style.barGap ?? 0.2"
          step="0.05"
          min="0"
          max="0.8"
          @change="updateStyle('barGap', parseFloat(($event.target as HTMLInputElement).value))"
        />
      </div>
    </div>

    <div class="prop-section" v-if="element.chartType === 'line'">
      <div class="prop-section__title">Line Ayarlari</div>
      <div class="prop-row">
        <label class="prop-label">Cizgi Kalinligi</label>
        <input
          class="prop-input prop-input--sm"
          type="number"
          :value="element.style.lineWidth ?? 0.5"
          step="0.1"
          min="0.1"
          @change="updateStyle('lineWidth', parseFloat(($event.target as HTMLInputElement).value))"
        />
      </div>
      <div class="prop-row">
        <label class="prop-label">Noktalar</label>
        <input
          type="checkbox"
          :checked="element.style.showPoints ?? true"
          @change="updateStyle('showPoints', ($event.target as HTMLInputElement).checked)"
        />
      </div>
    </div>

    <div class="prop-section" v-if="element.chartType === 'pie'">
      <div class="prop-section__title">Pie Ayarlari</div>
      <div class="prop-row">
        <label class="prop-label">Ic Yaricap</label>
        <input
          class="prop-input prop-input--sm"
          type="number"
          :value="element.style.innerRadius ?? 0"
          step="0.05"
          min="0"
          max="0.9"
          @change="
            updateStyle('innerRadius', parseFloat(($event.target as HTMLInputElement).value))
          "
        />
      </div>
      <div class="prop-row" style="font-size: 11px; color: #94a3b8">0 = Pie, &gt;0 = Donut</div>
    </div>
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
