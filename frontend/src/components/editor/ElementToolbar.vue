<script setup lang="ts">
import { computed } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { isContainer } from '../../core/types'
import ContainerToolbar from './toolbars/ContainerToolbar.vue'
import TextToolbar from './toolbars/TextToolbar.vue'
import TableToolbar from './toolbars/TableToolbar.vue'
import ChartToolbar from './toolbars/ChartToolbar.vue'
import type {
  ContainerElement,
  RepeatingTableElement,
  TableStyle,
  ChartElement,
} from '../../core/types'
import type { LayoutMapEntry } from '../../core/layout-types'
import '../../styles/toolbar.css'

const PAGE_GAP_PX = 24

const props = defineProps<{
  scale: number
  layoutMap: Record<string, LayoutMapEntry>
  pageHeightPx?: number
}>()

const templateStore = useTemplateStore()
const editorStore = useEditorStore()

const selected = computed(() => {
  const id = editorStore.selectedElementId
  if (!id || id === 'root') return null
  return templateStore.getElementById(id) ?? null
})

const containerEl = computed(() => {
  const el = selected.value
  return el && isContainer(el) ? (el as ContainerElement) : null
})

const isText = computed(() => {
  const t = selected.value?.type
  return t === 'static_text' || t === 'text'
})

const isLine = computed(() => selected.value?.type === 'line')

const isTable = computed(() => selected.value?.type === 'repeating_table')
const tableStyle = computed(() =>
  isTable.value ? ((selected.value as RepeatingTableElement).style as TableStyle) : null,
)

const isChart = computed(() => selected.value?.type === 'chart')
const chartEl = computed(() => (isChart.value ? (selected.value as ChartElement) : null))

function pageYOffset(pageIndex: number): number {
  if (pageIndex <= 0) return 0
  const pageH = props.pageHeightPx ?? templateStore.template.page.height * props.scale
  return pageIndex * (pageH + PAGE_GAP_PX)
}

const toolbarStyle = computed(() => {
  const el = selected.value
  if (!el) return { display: 'none' }
  const l = props.layoutMap[el.id]
  if (!l) return { display: 'none' }

  const s = props.scale
  const pYOff = pageYOffset(l.pageIndex)
  return {
    position: 'absolute' as const,
    left: `${l.x_mm * s}px`,
    top: `${l.y_mm * s - 30 + pYOff}px`,
    zIndex: 1100,
  }
})

function update(updates: Record<string, unknown>) {
  if (!selected.value) return
  templateStore.updateElement(selected.value.id, updates as any)
}

function updateStyle(key: string, value: unknown) {
  if (!selected.value) return
  update({ style: { ...selected.value.style, [key]: value } })
}

// Z-order
function bringForward() {
  if (selected.value) templateStore.bringForward(selected.value.id)
}
function sendBackward() {
  if (selected.value) templateStore.sendBackward(selected.value.id)
}
function bringToFront() {
  if (selected.value) templateStore.bringToFront(selected.value.id)
}
function sendToBack() {
  if (selected.value) templateStore.sendToBack(selected.value.id)
}
</script>

<template>
  <div v-if="selected" class="et" :style="toolbarStyle" @pointerdown.stop>
    <!-- Container -->
    <ContainerToolbar v-if="containerEl" :container="containerEl" @update="update" />

    <!-- Text / Static Text -->
    <TextToolbar v-if="isText" :element="selected!" @update-style="updateStyle" />

    <!-- Repeating Table -->
    <TableToolbar v-if="isTable && tableStyle" :table-style="tableStyle" @update-style="updateStyle" />

    <!-- Chart -->
    <ChartToolbar v-if="isChart && chartEl" :chart="chartEl" @update="update" @update-style="updateStyle" />

    <!-- Line -->
    <template v-if="isLine">
      <div class="et__group et__group--gap">
        <svg class="et__gap-icon" width="12" height="12" viewBox="0 0 12 12" fill="none"><line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="2" stroke-linecap="round" /></svg>
        <input type="number" class="et__num" step="0.1" min="0.1" :value="(selected!.style as any).strokeWidth ?? 0.5" @input="(e) => updateStyle('strokeWidth', parseFloat((e.target as HTMLInputElement).value) || 0.5)" data-tip="Kalinlik (mm)" />
      </div>
      <div class="et__sep" />
      <div class="et__group">
        <label class="et__color-wrap" data-tip="Renk">
          <input type="color" class="et__color" :value="(selected!.style as any).strokeColor ?? '#000000'" @input="(e) => updateStyle('strokeColor', (e.target as HTMLInputElement).value)" />
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><line x1="2" y1="7" x2="12" y2="7" :stroke="(selected!.style as any).strokeColor ?? '#000000'" stroke-width="2.5" stroke-linecap="round" /></svg>
        </label>
      </div>
    </template>

    <!-- Z-Order (all elements) -->
    <template v-if="selected">
      <div class="et__sep" />
      <div class="et__group">
        <button class="et__btn" data-tip="Arkaya Gonder" @click="sendToBack">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="5" y="5" width="7" height="7" rx="1" fill="currentColor" opacity="0.3" /><rect x="2" y="2" width="7" height="7" rx="1" fill="currentColor" /></svg>
        </button>
        <button class="et__btn" data-tip="Bir Geri" @click="sendBackward">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="5" y="5" width="7" height="7" rx="1" fill="currentColor" opacity="0.3" /><rect x="2" y="2" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.2" fill="none" /></svg>
        </button>
        <button class="et__btn" data-tip="Bir Ileri" @click="bringForward">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="2" width="7" height="7" rx="1" fill="currentColor" opacity="0.3" /><rect x="5" y="5" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.2" fill="none" /></svg>
        </button>
        <button class="et__btn" data-tip="One Getir" @click="bringToFront">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="2" width="7" height="7" rx="1" fill="currentColor" opacity="0.3" /><rect x="5" y="5" width="7" height="7" rx="1" fill="currentColor" /></svg>
        </button>
      </div>
    </template>
  </div>
</template>

