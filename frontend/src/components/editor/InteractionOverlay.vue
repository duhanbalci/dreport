<script setup lang="ts">
import { computed, ref } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import type { LayoutMapEntry } from '../../core/layout-types'
import type { TemplateElement, SizeValue, ContainerElement } from '../../core/types'
import { isContainer, sz } from '../../core/types'
import ElementToolbar from './ElementToolbar.vue'
import { useSnapGuides } from '../../composables/useSnapGuides'

const PAGE_GAP_PX = 24

const props = defineProps<{
  scale: number
  layoutMap: Record<string, LayoutMapEntry>
  pageCount?: number
  pageHeightPx?: number
}>()

const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const { activeGuides, collectEdges, calculateSnap, calculateResizeSnap, clearGuides } = useSnapGuides()

// Tüm elemanları flat olarak topla (root hariç)
const flatElements = computed(() => {
  const result: TemplateElement[] = []
  function walk(el: TemplateElement) {
    if (isContainer(el)) {
      for (const child of el.children) {
        result.push(child)
        walk(child)
      }
    }
  }
  // Header ve footer container'larını ve elemanlarını dahil et
  if (templateStore.template.header) {
    result.push(templateStore.template.header as unknown as TemplateElement)
    walk(templateStore.template.header as unknown as TemplateElement)
  }
  walk(templateStore.template.root)
  if (templateStore.template.footer) {
    result.push(templateStore.template.footer as unknown as TemplateElement)
    walk(templateStore.template.footer as unknown as TemplateElement)
  }
  return result
})

// Tüm container'lar (root dahil) — drop target tespiti için
const allContainers = computed(() => {
  const result: ContainerElement[] = [templateStore.template.root]
  function walk(el: TemplateElement) {
    if (isContainer(el)) {
      result.push(el)
      for (const child of el.children) walk(child)
    }
  }
  if (templateStore.template.header) {
    result.push(templateStore.template.header)
    for (const child of templateStore.template.header.children) walk(child)
  }
  for (const child of templateStore.template.root.children) walk(child)
  if (templateStore.template.footer) {
    result.push(templateStore.template.footer)
    for (const child of templateStore.template.footer.children) walk(child)
  }
  return result
})

/** Sayfa index'ine göre y offset hesapla (sayfalar arası gap dahil) */
function pageYOffset(pageIndex: number): number {
  if (pageIndex <= 0) return 0
  const pageH = props.pageHeightPx ?? (templateStore.template.page.height * props.scale)
  return pageIndex * (pageH + PAGE_GAP_PX)
}

function getElementStyle(el: TemplateElement) {
  const l = props.layoutMap[el.id]
  if (!l) return { display: 'none' }

  const s = props.scale
  const h = l.height_mm * s
  const minH = 8
  const actualH = Math.max(h, minH)
  const yOff = h < minH ? (minH - h) / 2 : 0
  const pYOff = pageYOffset(l.pageIndex)

  return {
    position: 'absolute' as const,
    left: `${l.x_mm * s}px`,
    top: `${l.y_mm * s - yOff + pYOff}px`,
    width: `${l.width_mm * s}px`,
    height: `${actualH}px`,
  }
}

// --- Seçim ---

function onElementClick(e: PointerEvent, id: string) {
  e.stopPropagation()
  if (didDrag.value) return
  editorStore.selectElement(id)
}

function onCanvasClick() {
  editorStore.selectElement('root')
}

// ============================================================
// Ortak drop target sistemi
// ============================================================

const dropTargetContainerId = ref<string | null>(null)
const dropVisualIndex = ref<number | null>(null)
const dropLogicalIndex = ref<number | null>(null)

/** Mouse pozisyonuna göre en derin container'ı bul */
function findDeepestContainer(mouseX: number, mouseY: number, excludeId?: string): ContainerElement {
  const s = props.scale
  let best: ContainerElement = templateStore.template.root

  for (const c of allContainers.value) {
    if (c.id === excludeId) continue
    const l = props.layoutMap[c.id]
    if (!l) continue

    const cx = l.x_mm * s
    const cy = l.y_mm * s + pageYOffset(l.pageIndex)
    const cw = l.width_mm * s
    const ch = l.height_mm * s

    if (mouseX >= cx && mouseX <= cx + cw && mouseY >= cy && mouseY <= cy + ch) {
      // Daha küçük (daha derin) container'ı tercih et
      const bestL = props.layoutMap[best.id]
      if (!bestL || (cw * ch < bestL.width_mm * s * bestL.height_mm * s)) {
        best = c
      }
    }
  }
  return best
}

/** Container içinde drop index hesapla */
function computeDropIndex(container: ContainerElement, mouseX: number, mouseY: number, excludeId?: string) {
  const s = props.scale
  const flowChildren = container.children.filter(c => c.type !== 'page_break' && c.position.type !== 'absolute' && c.id !== excludeId)
  const isRow = container.direction === 'row'

  let visualIdx = flowChildren.length

  for (let i = 0; i < flowChildren.length; i++) {
    const l = props.layoutMap[flowChildren[i].id]
    if (!l) continue
    if (isRow) {
      const centerX = l.x_mm * s + (l.width_mm * s) / 2
      if (mouseX < centerX) { visualIdx = i; break }
    } else {
      const centerY = l.y_mm * s + pageYOffset(l.pageIndex) + (l.height_mm * s) / 2
      if (mouseY < centerY) { visualIdx = i; break }
    }
  }

  // Mantıksal index: excludeId aynı container'daysa offset hesapla
  let logicalIdx = visualIdx
  if (excludeId) {
    const allFlow = container.children.filter(c => c.type !== 'page_break' && c.position.type !== 'absolute')
    const currentIdx = allFlow.findIndex(c => c.id === excludeId)
    if (currentIdx >= 0) {
      // visualIdx, excludeId çıkarılmış listede. Gerçek listedeki pozisyona çevir.
      // flowChildren zaten excludeId hariç, dolayısıyla visualIdx doğrudan gerçek insert indexi
      // Ama reorderChild fromIndex/toIndex aynı liste üzerinde çalışır
      // Gerçek listedeki index'e çevir
      let realIdx = 0
      let count = 0
      for (let i = 0; i < allFlow.length; i++) {
        if (allFlow[i].id === excludeId) continue
        if (count === visualIdx) { realIdx = i; break }
        count++
        realIdx = i + 1
      }
      logicalIdx = realIdx
      if (realIdx > currentIdx) logicalIdx--
    }
  }

  return { visualIdx, logicalIdx }
}

function updateDropFromMouse(mouseX: number, mouseY: number, excludeId?: string) {
  const container = findDeepestContainer(mouseX, mouseY, excludeId)
  dropTargetContainerId.value = container.id

  const { visualIdx, logicalIdx } = computeDropIndex(container, mouseX, mouseY, excludeId)
  dropVisualIndex.value = visualIdx
  dropLogicalIndex.value = logicalIdx
}

function clearDropTarget() {
  dropTargetContainerId.value = null
  dropVisualIndex.value = null
  dropLogicalIndex.value = null
}

// Drop indicator pozisyonu (ortak)
const dropIndicatorStyle = computed(() => {
  if (dropTargetContainerId.value === null || dropVisualIndex.value === null) {
    return { display: 'none' }
  }

  const container = templateStore.getElementById(dropTargetContainerId.value)
  if (!container || !isContainer(container)) return { display: 'none' }

  const s = props.scale
  const idx = dropVisualIndex.value
  const isRow = container.direction === 'row'

  // Sürüklenen elemanı çıkar
  const dragId = dragElementId.value
  const flowChildren = container.children.filter(c => c.type !== 'page_break' && c.position.type !== 'absolute' && c.id !== dragId)

  const cl = props.layoutMap[container.id]
  if (!cl) return { display: 'none' }

  if (isRow) {
    // Row container: dikey gösterge çizgisi
    let x = 0
    if (idx === 0 && flowChildren.length > 0) {
      const l = props.layoutMap[flowChildren[0].id]
      if (l) x = (cl.x_mm * s + l.x_mm * s) / 2
      else x = cl.x_mm * s
    } else if (idx < flowChildren.length && idx > 0) {
      const left = props.layoutMap[flowChildren[idx - 1].id]
      const right = props.layoutMap[flowChildren[idx].id]
      if (left && right) {
        const leftEnd = (left.x_mm + left.width_mm) * s
        const rightStart = right.x_mm * s
        x = (leftEnd + rightStart) / 2
      }
    } else if (idx === 0 && flowChildren.length === 0) {
      x = cl.x_mm * s + 8
    } else if (flowChildren.length > 0) {
      const last = flowChildren[flowChildren.length - 1]
      const l = props.layoutMap[last.id]
      if (l) {
        const gapPx = container.gap * props.scale
        x = (l.x_mm + l.width_mm) * s + gapPx / 2
      }
    }

    const clPageOff = pageYOffset(cl.pageIndex)
    const top = cl.y_mm * s + clPageOff
    const height = cl.height_mm * s

    return {
      position: 'absolute' as const,
      left: `${x}px`,
      top: `${top}px`,
      width: '2px',
      height: `${height}px`,
      background: 'rgb(59, 130, 246)',
      borderRadius: '1px',
      zIndex: 1000,
      pointerEvents: 'none' as const,
    }
  }

  // Column container: yatay gösterge çizgisi
  const colPageOff = pageYOffset(cl.pageIndex)
  let y = 0
  if (idx === 0 && flowChildren.length > 0) {
    const l = props.layoutMap[flowChildren[0].id]
    if (l) {
      y = (cl.y_mm * s + colPageOff + l.y_mm * s + pageYOffset(l.pageIndex)) / 2
    } else {
      y = cl.y_mm * s + colPageOff - 4
    }
  } else if (idx < flowChildren.length && idx > 0) {
    const above = props.layoutMap[flowChildren[idx - 1].id]
    const below = props.layoutMap[flowChildren[idx].id]
    if (above && below) {
      const aboveBottom = (above.y_mm + above.height_mm) * s + pageYOffset(above.pageIndex)
      const belowTop = below.y_mm * s + pageYOffset(below.pageIndex)
      y = (aboveBottom + belowTop) / 2
    }
  } else if (idx === 0 && flowChildren.length === 0) {
    y = cl.y_mm * s + colPageOff + 8
  } else if (flowChildren.length > 0) {
    const last = flowChildren[flowChildren.length - 1]
    const l = props.layoutMap[last.id]
    if (l) {
      const gapPx = container.gap * props.scale
      y = (l.y_mm + l.height_mm) * s + pageYOffset(l.pageIndex) + gapPx / 2
    }
  }

  const x = cl.x_mm * s
  const width = cl.width_mm * s

  return {
    position: 'absolute' as const,
    left: `${x}px`,
    top: `${y}px`,
    width: `${width}px`,
    height: '2px',
    background: 'rgb(59, 130, 246)',
    borderRadius: '1px',
    zIndex: 1000,
    pointerEvents: 'none' as const,
  }
})

// ============================================================
// Mevcut eleman sürükleme (reorder + cross-container move)
// ============================================================

const isDragging = ref(false)
const didDrag = ref(false)
const dragElementId = ref<string | null>(null)
const dragOffset = ref({ x: 0, y: 0 })
const dragGhost = ref({ x: 0, y: 0, width: 0, height: 0 })

function onDragStart(e: PointerEvent, el: TemplateElement) {
  if (el.type === 'page_break') return
  if (el.position.type === 'absolute') {
    onAbsoluteDragStart(e, el)
    return
  }

  const l = props.layoutMap[el.id]
  if (!l) return

  const s = props.scale
  dragElementId.value = el.id
  didDrag.value = false

  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  dragOffset.value = { x: e.clientX - rect.left, y: e.clientY - rect.top }
  dragGhost.value = {
    x: l.x_mm * s,
    y: l.y_mm * s,
    width: l.width_mm * s,
    height: l.height_mm * s,
  }

  window.addEventListener('pointermove', onDragMove)
  window.addEventListener('pointerup', onDragEnd)
}

function onDragMove(e: PointerEvent) {
  if (!dragElementId.value) return

  const overlayEl = document.querySelector('.interaction-overlay')
  if (!overlayEl) return
  const overlayRect = overlayEl.getBoundingClientRect()

  const x = e.clientX - overlayRect.left - dragOffset.value.x
  const y = e.clientY - overlayRect.top - dragOffset.value.y
  const mouseX = e.clientX - overlayRect.left
  const mouseY = e.clientY - overlayRect.top

  if (!isDragging.value) {
    const dx = Math.abs(x - dragGhost.value.x)
    const dy = Math.abs(y - dragGhost.value.y)
    if (dx < 4 && dy < 4) return
    isDragging.value = true
    didDrag.value = true
    editorStore.setDragging(true)
  }

  dragGhost.value.x = x
  dragGhost.value.y = y

  updateDropFromMouse(mouseX, mouseY, dragElementId.value)
}

function onDragEnd() {
  window.removeEventListener('pointermove', onDragMove)
  window.removeEventListener('pointerup', onDragEnd)

  if (isDragging.value && dragElementId.value && dropTargetContainerId.value !== null && dropLogicalIndex.value !== null) {
    const currentParent = templateStore.getParent(dragElementId.value)
    const targetContainerId = dropTargetContainerId.value

    if (currentParent && currentParent.id === targetContainerId) {
      // Aynı container içinde reorder
      const currentIdx = currentParent.children.findIndex(c => c.id === dragElementId.value)
      if (currentIdx !== -1 && currentIdx !== dropLogicalIndex.value) {
        templateStore.reorderChild(currentParent.id, currentIdx, dropLogicalIndex.value)
      }
    } else {
      // Farklı container'a taşı
      templateStore.moveElement(dragElementId.value, targetContainerId, dropLogicalIndex.value)
    }
  }

  isDragging.value = false
  dragElementId.value = null
  editorStore.setDragging(false)
  clearDropTarget()
  setTimeout(() => { didDrag.value = false }, 50)
}

// --- Absolute eleman drag ---

const absoluteDragId = ref<string | null>(null)
const absoluteDragStart = ref({ mouseX: 0, mouseY: 0, elX: 0, elY: 0 })

function onAbsoluteDragStart(e: PointerEvent, el: TemplateElement) {
  if (el.position.type !== 'absolute') return

  absoluteDragId.value = el.id
  didDrag.value = false
  absoluteDragStart.value = {
    mouseX: e.clientX,
    mouseY: e.clientY,
    elX: el.position.x,
    elY: el.position.y,
  }

  collectEdges(props.layoutMap, el.id, templateStore.template.page.width, templateStore.template.page.height)

  window.addEventListener('pointermove', onAbsoluteDragMove)
  window.addEventListener('pointerup', onAbsoluteDragEnd)
}

function onAbsoluteDragMove(e: PointerEvent) {
  if (!absoluteDragId.value) return

  const dx = e.clientX - absoluteDragStart.value.mouseX
  const dy = e.clientY - absoluteDragStart.value.mouseY

  if (!isDragging.value) {
    if (Math.abs(dx) < 4 && Math.abs(dy) < 4) return
    isDragging.value = true
    didDrag.value = true
    editorStore.setDragging(true)
  }

  const pxToMm = 1 / props.scale
  const proposedX = Math.max(0, absoluteDragStart.value.elX + dx * pxToMm)
  const proposedY = Math.max(0, absoluteDragStart.value.elY + dy * pxToMm)

  const layout = props.layoutMap[absoluteDragId.value]
  const elW = layout ? layout.width_mm : 0
  const elH = layout ? layout.height_mm : 0

  const snap = calculateSnap(proposedX, proposedY, elW, elH)
  const newX = snap.snappedX_mm
  const newY = snap.snappedY_mm

  templateStore.updateElementPosition(absoluteDragId.value, {
    type: 'absolute',
    x: Math.round(newX * 10) / 10,
    y: Math.round(newY * 10) / 10,
  })
}

function onAbsoluteDragEnd() {
  window.removeEventListener('pointermove', onAbsoluteDragMove)
  window.removeEventListener('pointerup', onAbsoluteDragEnd)

  isDragging.value = false
  absoluteDragId.value = null
  editorStore.setDragging(false)
  clearGuides()
  setTimeout(() => { didDrag.value = false }, 50)
}

// --- Resize ---

const isResizing = ref(false)
const resizeElementId = ref<string | null>(null)
const resizeHandle = ref('')
const resizeStart = ref({ mouseX: 0, mouseY: 0, x: 0, y: 0, width: 0, height: 0 })
const resizeGhost = ref({ x: 0, y: 0, width: 0, height: 0 })
const resizeFinalMm = ref({ width: 0, height: 0 })
const resizeAspectRatio = ref(0) // > 0 ise aspect ratio korunur (width / height)

function onResizeStart(e: PointerEvent, elId: string, handle: string) {
  e.stopPropagation()
  e.preventDefault()

  const l = props.layoutMap[elId]
  if (!l) return

  resizeElementId.value = elId
  resizeHandle.value = handle
  isResizing.value = true

  const s = props.scale

  // Barkod ve görsel elemanları için aspect ratio'yu kaydet
  const el = flatElements.value.find(e => e.id === elId)
  resizeAspectRatio.value = ((el?.type === 'barcode' || el?.type === 'image') && l.height_mm > 0) ? l.width_mm / l.height_mm : 0

  resizeStart.value = {
    mouseX: e.clientX, mouseY: e.clientY,
    x: l.x_mm * s, y: l.y_mm * s,
    width: l.width_mm * s, height: l.height_mm * s,
  }
  resizeGhost.value = { x: l.x_mm * s, y: l.y_mm * s, width: l.width_mm * s, height: l.height_mm * s }
  resizeFinalMm.value = { width: l.width_mm, height: l.height_mm }

  collectEdges(props.layoutMap, elId, templateStore.template.page.width, templateStore.template.page.height)

  window.addEventListener('pointermove', onResizeMove)
  window.addEventListener('pointerup', onResizeEnd)
}

function onResizeMove(e: PointerEvent) {
  if (!resizeElementId.value) return

  const dx = e.clientX - resizeStart.value.mouseX
  const dy = e.clientY - resizeStart.value.mouseY
  const handle = resizeHandle.value
  const pxToMm = 1 / props.scale
  const ar = resizeAspectRatio.value

  let gx = resizeStart.value.x, gy = resizeStart.value.y
  let gw = resizeStart.value.width, gh = resizeStart.value.height

  if (handle.includes('e')) gw = Math.max(20, resizeStart.value.width + dx)
  if (handle.includes('w')) { gw = Math.max(20, resizeStart.value.width - dx); gx = resizeStart.value.x + dx }
  if (handle.includes('s')) gh = Math.max(10, resizeStart.value.height + dy)
  if (handle.includes('n')) { gh = Math.max(10, resizeStart.value.height - dy); gy = resizeStart.value.y + dy }

  // Aspect ratio koruma (barkod)
  if (ar > 0) {
    gh = gw / ar
  }

  resizeGhost.value = { x: gx, y: gy, width: gw, height: gh }

  const startWMm = resizeStart.value.width * pxToMm
  const startHMm = resizeStart.value.height * pxToMm
  const startXMm = resizeStart.value.x * pxToMm
  const startYMm = resizeStart.value.y * pxToMm
  let wMm = startWMm, hMm = startHMm
  if (handle.includes('e')) {
    const rightEdge = calculateResizeSnap('right', startXMm + startWMm + dx * pxToMm)
    wMm = Math.max(5, rightEdge - startXMm)
  }
  if (handle.includes('w')) {
    const leftEdge = calculateResizeSnap('left', startXMm + dx * pxToMm)
    wMm = Math.max(5, startXMm + startWMm - leftEdge)
  }
  if (handle.includes('s')) {
    const bottomEdge = calculateResizeSnap('bottom', startYMm + startHMm + dy * pxToMm)
    hMm = Math.max(3, bottomEdge - startYMm)
  }
  if (handle.includes('n')) {
    const topEdge = calculateResizeSnap('top', startYMm + dy * pxToMm)
    hMm = Math.max(3, startYMm + startHMm - topEdge)
  }

  if (ar > 0) {
    hMm = wMm / ar
  }

  resizeFinalMm.value = { width: Math.round(wMm * 10) / 10, height: Math.round(hMm * 10) / 10 }
}

function onResizeEnd() {
  window.removeEventListener('pointermove', onResizeMove)
  window.removeEventListener('pointerup', onResizeEnd)

  if (resizeElementId.value) {
    const handle = resizeHandle.value
    const ar = resizeAspectRatio.value
    const sizeUpdate: { width?: SizeValue; height?: SizeValue } = {}
    if (handle.includes('e') || handle.includes('w')) sizeUpdate.width = sz.fixed(resizeFinalMm.value.width)
    if (handle.includes('s') || handle.includes('n')) sizeUpdate.height = sz.fixed(resizeFinalMm.value.height)
    // Aspect ratio aktifken her zaman hem width hem height güncelle
    if (ar > 0) {
      sizeUpdate.width = sz.fixed(resizeFinalMm.value.width)
      sizeUpdate.height = sz.fixed(resizeFinalMm.value.height)
    }
    templateStore.updateElementSize(resizeElementId.value, sizeUpdate)
  }

  isResizing.value = false
  resizeElementId.value = null
  resizeHandle.value = ''
  clearGuides()
}

// ============================================================
// Toolbox sürükle-bırak (HTML5 drag API)
// ============================================================

function onToolboxDragOver(e: DragEvent) {
  if (!editorStore.draggedNewElement) return
  e.preventDefault()

  const overlayEl = e.currentTarget as HTMLElement
  const rect = overlayEl.getBoundingClientRect()
  const mouseX = e.clientX - rect.left
  const mouseY = e.clientY - rect.top

  updateDropFromMouse(mouseX, mouseY)
}

function onToolboxDragLeave() {
  clearDropTarget()
}

function onToolboxDrop(_e: DragEvent) {
  const newEl = editorStore.draggedNewElement
  if (!newEl) return

  const targetId = dropTargetContainerId.value ?? 'root'
  const idx = dropLogicalIndex.value ?? undefined

  templateStore.addChild(targetId, newEl, idx)
  editorStore.selectElement(newEl.id)
  editorStore.endDragNewElement()
  clearDropTarget()
}

// Aktif sürükleme var mı (eleman veya toolbox)
const isAnyDragActive = computed(() =>
  (isDragging.value && dragElementId.value !== null) || !!editorStore.draggedNewElement
)
</script>

<template>
  <div
    class="interaction-overlay"
    :class="{ 'interaction-overlay--drop-active': isAnyDragActive }"
    @pointerdown.self="onCanvasClick"
    @dragover.prevent="onToolboxDragOver"
    @dragleave="onToolboxDragLeave"
    @drop.prevent="onToolboxDrop"
  >
    <!-- Element handles -->
    <div
      v-for="el in flatElements"
      :key="el.id"
      class="element-handle"
      :class="{
        'element-handle--selected': editorStore.selectedElementId === el.id,
        'element-handle--container': isContainer(el),
        'element-handle--dragging': isDragging && dragElementId === el.id,
        'element-handle--drop-target': isContainer(el) && dropTargetContainerId === el.id && isAnyDragActive,
      }"
      :style="getElementStyle(el)"
      @pointerdown="(e: PointerEvent) => { onElementClick(e, el.id); onDragStart(e, el) }"
    >
      <!-- Selection border -->
      <div v-if="editorStore.selectedElementId === el.id" class="selection-border" />

      <!-- Resize handles -->
      <template v-if="editorStore.selectedElementId === el.id && !isResizing && el.type !== 'page_break'">
        <template v-if="el.type === 'barcode' || el.type === 'image'">
          <!-- Barkod/Görsel: sadece yatay resize (aspect ratio korunur) -->
          <div class="resize-handle resize-handle--e" @pointerdown="(e: PointerEvent) => onResizeStart(e, el.id, 'e')" />
          <div class="resize-handle resize-handle--w" @pointerdown="(e: PointerEvent) => onResizeStart(e, el.id, 'w')" />
        </template>
        <template v-else>
          <div class="resize-handle resize-handle--se" @pointerdown="(e: PointerEvent) => onResizeStart(e, el.id, 'se')" />
          <div class="resize-handle resize-handle--sw" @pointerdown="(e: PointerEvent) => onResizeStart(e, el.id, 'sw')" />
          <div class="resize-handle resize-handle--ne" @pointerdown="(e: PointerEvent) => onResizeStart(e, el.id, 'ne')" />
          <div class="resize-handle resize-handle--nw" @pointerdown="(e: PointerEvent) => onResizeStart(e, el.id, 'nw')" />
        </template>
      </template>
    </div>

    <!-- Drag ghost (mevcut eleman sürükleme) -->
    <div
      v-if="isDragging && dragElementId"
      class="drag-ghost"
      :style="{
        left: `${dragGhost.x}px`,
        top: `${dragGhost.y}px`,
        width: `${dragGhost.width}px`,
        height: `${dragGhost.height}px`,
      }"
    />

    <!-- Resize ghost -->
    <div
      v-if="isResizing && resizeElementId"
      class="resize-ghost"
      :style="{
        left: `${resizeGhost.x}px`,
        top: `${resizeGhost.y}px`,
        width: `${resizeGhost.width}px`,
        height: `${resizeGhost.height}px`,
      }"
    />

    <!-- Drop indicator (ortak — hem eleman hem toolbox sürükleme) -->
    <div v-if="isAnyDragActive" :style="dropIndicatorStyle" />

    <!-- Snap guides -->
    <div
      v-for="(guide, gi) in activeGuides"
      :key="'guide-' + gi"
      class="snap-guide"
      :style="{
        position: 'absolute',
        ...(guide.type === 'vertical'
          ? { left: `${guide.position_mm * scale}px`, top: '0', bottom: '0', width: '1px' }
          : { top: `${guide.position_mm * scale}px`, left: '0', right: '0', height: '1px' }),
        background: '#3b82f6',
        opacity: 0.7,
        pointerEvents: 'none',
        zIndex: 9999,
      }"
    />

    <!-- Element toolbar — seçili elemanın üstünde -->
    <ElementToolbar
      v-if="!isDragging && !isResizing"
      :scale="scale"
      :layout-map="layoutMap"
      :page-height-px="pageHeightPx"
    />
  </div>
</template>

<style scoped>
.interaction-overlay {
  position: absolute;
  inset: 0;
}

.element-handle {
  box-sizing: border-box;
  cursor: pointer;
}

.element-handle--dragging {
  opacity: 0.3;
}

/* Selection border */
.selection-border {
  position: absolute;
  inset: -1px;
  border: 1.5px solid rgb(59, 130, 246);
  pointer-events: none;
  display: block;
}

.element-handle--container > .selection-border {
  border-color: rgb(139, 92, 246);
  border-style: dashed;
}

/* Container'ları hafif kenarlıkla göster (root hariç — root overlay'de flatElements'te yok) */
.element-handle--container {
  outline: 1px dashed rgba(139, 92, 246, 0.25);
  outline-offset: -1px;
}

/* Hover efekti */
.element-handle:not(.element-handle--selected):hover::after {
  content: '';
  position: absolute;
  inset: -1px;
  border: 1.5px solid rgba(59, 130, 246, 0.4);
  pointer-events: none;
}

/* Resize handles */
.resize-handle {
  position: absolute;
  width: 6px;
  height: 6px;
  background: white;
  border: 1.5px solid rgb(59, 130, 246);
  border-radius: 1px;
  z-index: 10;
}

.resize-handle--se { right: -3px; bottom: -3px; cursor: se-resize; }
.resize-handle--sw { left: -3px; bottom: -3px; cursor: sw-resize; }
.resize-handle--ne { right: -3px; top: -3px; cursor: ne-resize; }
.resize-handle--nw { left: -3px; top: -3px; cursor: nw-resize; }
.resize-handle--e { right: -3px; top: calc(50% - 3px); cursor: e-resize; }
.resize-handle--w { left: -3px; top: calc(50% - 3px); cursor: w-resize; }

/* Drag ghost */
.drag-ghost {
  position: absolute;
  background: rgba(59, 130, 246, 0.1);
  border: 1.5px dashed rgb(59, 130, 246);
  border-radius: 2px;
  pointer-events: none;
  z-index: 999;
}

/* Resize ghost */
.resize-ghost {
  position: absolute;
  border: 1.5px solid rgb(59, 130, 246);
  background: rgba(59, 130, 246, 0.05);
  pointer-events: none;
  z-index: 999;
}

/* Sürükleme aktifken container'ları göster */
.interaction-overlay--drop-active .element-handle--container::after {
  content: '';
  position: absolute;
  inset: 0;
  border: 1.5px dashed rgba(139, 92, 246, 0.5);
  border-radius: 2px;
  pointer-events: none;
}

/* Drop hedef container highlight */
.element-handle--drop-target::after {
  content: '';
  position: absolute;
  inset: -2px;
  border: 2px solid rgb(139, 92, 246) !important;
  background: rgba(139, 92, 246, 0.08);
  border-radius: 3px;
  pointer-events: none;
}
</style>
