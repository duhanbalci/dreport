<script setup lang="ts">
import { ref, computed } from 'vue'
import type { TemplateElement, ContainerElement } from '../../core/types'
import { isContainer } from '../../core/types'
import { useEditorStore } from '../../stores/editor'
import { useTemplateStore } from '../../stores/template'

const props = defineProps<{
  element: TemplateElement
  scale: number
}>()

const editorStore = useEditorStore()
const templateStore = useTemplateStore()

const isSelected = computed(() => editorStore.isSelected(props.element.id))
const isContainerEl = computed(() => isContainer(props.element))
const isAbsolute = computed(() => props.element.position.type === 'absolute')

// --- CSS style: layout engine sonuçlarına göre ---
const layoutStyle = computed(() => {
  const el = props.element
  const s = props.scale
  const style: Record<string, string> = {}

  // Absolute positioning
  if (el.position.type === 'absolute') {
    style.position = 'absolute'
    style.left = `${el.position.x * s}px`
    style.top = `${el.position.y * s}px`
  }

  // Boyut
  const w = el.size.width
  const h = el.size.height
  if (w.type === 'fixed') style.width = `${w.value * s}px`
  else if (w.type === 'fr') style.flex = `${w.value} 1 0%`
  // auto → doğal boyut, CSS default

  if (h.type === 'fixed') style.height = `${h.value * s}px`
  // auto/fr height → CSS default

  // Container ise flexbox
  if (isContainer(el)) {
    const c = el as ContainerElement
    style.display = 'flex'
    style.flexDirection = c.direction === 'row' ? 'row' : 'column'
    if (c.gap > 0) style.gap = `${c.gap * s}px`
    if (c.padding.top || c.padding.right || c.padding.bottom || c.padding.left) {
      style.padding = `${c.padding.top * s}px ${c.padding.right * s}px ${c.padding.bottom * s}px ${c.padding.left * s}px`
    }

    // align (cross-axis)
    const alignMap = { start: 'flex-start', center: 'center', end: 'flex-end', stretch: 'stretch' }
    if (c.direction === 'column') {
      style.alignItems = alignMap[c.align] || 'stretch'
    } else {
      style.alignItems = alignMap[c.align] || 'flex-start'
    }

    // justify (main-axis)
    const justifyMap = {
      start: 'flex-start',
      center: 'center',
      end: 'flex-end',
      'space-between': 'space-between',
    }
    style.justifyContent = justifyMap[c.justify] || 'flex-start'
  }

  return style
})

// --- Drag state (sadece absolute elemanlar) ---
const pointerStart = ref({ x: 0, y: 0 })
const isDragging = ref(false)
const dragTransform = ref({ x: 0, y: 0 })

const isInteracting = computed(() => isDragging.value)

function onPointerDown(e: PointerEvent) {
  e.stopPropagation()
  editorStore.selectElement(props.element.id)

  if (!isAbsolute.value) return

  const target = e.currentTarget as HTMLElement
  target.setPointerCapture(e.pointerId)

  pointerStart.value = { x: e.clientX, y: e.clientY }
  dragTransform.value = { x: 0, y: 0 }
  isDragging.value = true
  editorStore.setDragging(true)
}

function onPointerMove(e: PointerEvent) {
  if (!isDragging.value) return
  dragTransform.value = {
    x: e.clientX - pointerStart.value.x,
    y: e.clientY - pointerStart.value.y,
  }
}

function onPointerUp() {
  if (!isDragging.value) return
  isDragging.value = false
  editorStore.setDragging(false)

  if (props.element.position.type !== 'absolute') return

  const dxMm = dragTransform.value.x / props.scale
  const dyMm = dragTransform.value.y / props.scale

  if (Math.abs(dxMm) > 0.1 || Math.abs(dyMm) > 0.1) {
    templateStore.updateElementPosition(props.element.id, {
      type: 'absolute',
      x: Math.round((props.element.position.x + dxMm) * 10) / 10,
      y: Math.round((props.element.position.y + dyMm) * 10) / 10,
    })
  }
  dragTransform.value = { x: 0, y: 0 }
}

const dragStyle = computed(() => {
  if (!isDragging.value) return {}
  return { transform: `translate(${dragTransform.value.x}px, ${dragTransform.value.y}px)` }
})
</script>

<template>
  <div
    class="element-handle"
    :class="{
      'element-handle--selected': isSelected,
      'element-handle--interacting': isInteracting,
      'element-handle--container': isContainerEl,
      'element-handle--absolute': isAbsolute,
      'element-handle--leaf': !isContainerEl,
    }"
    :style="{ ...layoutStyle, ...dragStyle }"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
  >
    <!-- Container çocuklarını recursive render et -->
    <template v-if="isContainerEl && 'children' in element">
      <ElementHandle
        v-for="child in (element as ContainerElement).children"
        :key="child.id"
        :element="child"
        :scale="scale"
      />
    </template>

    <!-- Seçim göstergesi -->
    <div v-if="isSelected" class="selection-border" />
  </div>
</template>

<style scoped>
.element-handle {
  position: relative;
  box-sizing: border-box;
  min-height: 2px;
}

.element-handle--absolute {
  position: absolute;
  cursor: move;
}

.element-handle--leaf {
  /* Leaf elemanlar tıklanabilir alan */
  min-height: 4px;
}

/* Hover efekti */
.element-handle:hover > .selection-border,
.element-handle--selected > .selection-border {
  display: block;
}

.selection-border {
  display: none;
  position: absolute;
  inset: -1px;
  border: 1.5px solid rgb(59, 130, 246);
  pointer-events: none;
  z-index: 10;
}

.element-handle--container > .selection-border {
  border-color: rgb(139, 92, 246);
  border-style: dashed;
}

.element-handle:hover > .selection-border {
  border-color: rgba(59, 130, 246, 0.4);
}

.element-handle--container:hover > .selection-border {
  border-color: rgba(139, 92, 246, 0.3);
}

.element-handle--selected > .selection-border {
  border-color: rgb(59, 130, 246);
}

.element-handle--selected.element-handle--container > .selection-border {
  border-color: rgb(139, 92, 246);
}

.element-handle--interacting {
  opacity: 0.7;
}
</style>
