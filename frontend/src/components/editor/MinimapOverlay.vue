<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import type { LayoutResult } from '../../core/layout-types'

const props = defineProps<{
  layout: LayoutResult | null
  pageWidth: number // mm
  pageHeight: number // mm
  zoom: number
  panX: number
  panY: number
  containerWidth: number // px — editor canvas container genişliği
  containerHeight: number // px — editor canvas container yüksekliği
  scale: number // mm → px (zoom dahil)
  pageGap: number // px — sayfalar arası boşluk
}>()

const emit = defineEmits<{
  navigate: [x: number, y: number]
}>()

const MAX_MINIMAP_WIDTH = 140
const MAX_EXPANDED_HEIGHT = 300
const PADDING = 6

const canvasRef = ref<HTMLCanvasElement | null>(null)
const scrollRef = ref<HTMLElement | null>(null)
const isHovered = ref(false)
const isPointerDragging = ref(false)

// Offscreen canvas — sayfa içeriği cache'i (layout değiştiğinde yeniden çizilir)
let contentCanvas: OffscreenCanvas | null = null
let contentDirty = true

const pageCount = computed(() => Math.max(1, props.layout?.pages.length ?? 1))

// Minimap'te sayfalar arası sabit piksel boşluk
const MINIMAP_PAGE_GAP_PX = 4

// Editördeki toplam yükseklik (mm, viewport hesabı için)
const totalHeightMm = computed(() => {
  const gapMm = props.pageGap / props.scale
  return props.pageHeight * pageCount.value + gapMm * (pageCount.value - 1)
})

const minimapScale = computed(() => (MAX_MINIMAP_WIDTH - PADDING * 2) / props.pageWidth)

const pageHeightPx = computed(() => props.pageHeight * minimapScale.value)

const canvasWidth = computed(() => props.pageWidth * minimapScale.value + PADDING * 2)
const canvasHeight = computed(() => {
  const n = pageCount.value
  return pageHeightPx.value * n + MINIMAP_PAGE_GAP_PX * (n - 1) + PADDING * 2
})

const singlePageMinimapH = computed(() => pageHeightPx.value + PADDING * 2)

// Editördeki gap'in mm karşılığı (activePageIndex hesabı için)
const editorGapMm = computed(() => props.pageGap / props.scale)

const activePageIndex = computed(() => {
  const viewH = props.containerHeight - 60 - 40
  const viewCenterMm = (-props.panY + viewH / 2) / props.scale
  const stride = props.pageHeight + editorGapMm.value
  const idx = Math.floor(viewCenterMm / stride)
  return Math.max(0, Math.min(pageCount.value - 1, idx))
})

const visibleHeight = computed(() => {
  if (isHovered.value || isPointerDragging.value) {
    return Math.min(canvasHeight.value, MAX_EXPANDED_HEIGHT)
  }
  return Math.min(singlePageMinimapH.value, canvasHeight.value)
})

/** Sayfanın canvas üzerindeki Y pozisyonu (px) */
function pageTopOnCanvas(pageIdx: number): number {
  return PADDING + pageIdx * (pageHeightPx.value + MINIMAP_PAGE_GAP_PX)
}

const targetScrollTop = computed(() => {
  if (isHovered.value || isPointerDragging.value) {
    const vp = viewportRect.value
    const vpCenter = vp.y + vp.h / 2
    const half = visibleHeight.value / 2
    const maxScroll = canvasHeight.value - visibleHeight.value
    return Math.max(0, Math.min(maxScroll, vpCenter - half))
  }
  const top = pageTopOnCanvas(activePageIndex.value) - PADDING
  const maxScroll = canvasHeight.value - visibleHeight.value
  return Math.max(0, Math.min(maxScroll, top))
})

/** Editör mm koordinatını minimap canvas px'e çevir (Y ekseni, sayfa gap'leri hesaba katarak) */
function mmYToCanvasPx(mmY: number): number {
  const gapMm = editorGapMm.value
  const stride = props.pageHeight + gapMm
  const pageIdx = Math.min(pageCount.value - 1, Math.max(0, Math.floor(mmY / stride)))
  const withinPageMm = mmY - pageIdx * stride
  return pageTopOnCanvas(pageIdx) + withinPageMm * minimapScale.value
}

const viewportRect = computed(() => {
  const s = minimapScale.value
  const pageWidthPx = props.pageWidth * props.scale
  const pageLeftPx = (props.containerWidth - pageWidthPx) / 2 + props.panX
  const pageTopPx = props.panY

  const viewW = props.containerWidth
  const viewH = props.containerHeight - 60 - 40

  const visLeftMm = -pageLeftPx / props.scale
  const visTopMm = -pageTopPx / props.scale
  const visWidthMm = viewW / props.scale
  const visHeightMm = viewH / props.scale

  // Clamp to page boundaries
  const clampedLeft = Math.max(0, visLeftMm)
  const clampedTop = Math.max(0, visTopMm)
  const clampedRight = Math.min(props.pageWidth, visLeftMm + visWidthMm)
  const clampedBottom = Math.min(totalHeightMm.value, visTopMm + visHeightMm)

  const y1 = mmYToCanvasPx(clampedTop)
  const y2 = mmYToCanvasPx(clampedBottom)

  return {
    x: PADDING + clampedLeft * s,
    y: y1,
    w: Math.max(0, (clampedRight - clampedLeft) * s),
    h: Math.max(0, y2 - y1),
  }
})

function elementColor(type: string): string {
  switch (type) {
    case 'text':
    case 'static_text':
    case 'rich_text':
      return '#93c5fd'
    case 'container':
      return '#c4b5fd'
    case 'repeating_table':
      return '#86efac'
    case 'image':
      return '#fdba74'
    case 'line':
      return '#9ca3af'
    case 'barcode':
      return '#fca5a5'
    case 'chart':
      return '#67e8f9'
    default:
      return '#d1d5db'
  }
}

// --- İki aşamalı çizim: content (pahalı, cache'li) + viewport overlay (ucuz) ---

/** Sayfa içeriğini offscreen canvas'a çizer — sadece layout değiştiğinde çağrılır */
function drawContent() {
  const dpr = window.devicePixelRatio || 1
  const w = canvasWidth.value
  const h = canvasHeight.value

  if (!contentCanvas || contentCanvas.width !== Math.ceil(w * dpr) || contentCanvas.height !== Math.ceil(h * dpr)) {
    contentCanvas = new OffscreenCanvas(Math.ceil(w * dpr), Math.ceil(h * dpr))
  }

  const ctx = contentCanvas.getContext('2d')!
  ctx.resetTransform()
  ctx.scale(dpr, dpr)
  ctx.clearRect(0, 0, w, h)

  const s = minimapScale.value
  const pages = props.layout?.pages ?? []

  for (let i = 0; i < Math.max(1, pages.length); i++) {
    const px = PADDING
    const py = pageTopOnCanvas(i)
    const pw = props.pageWidth * s
    const ph = props.pageHeight * s

    ctx.fillStyle = '#ffffff'
    ctx.fillRect(px, py, pw, ph)
    ctx.strokeStyle = '#d1d5db'
    ctx.lineWidth = 0.5
    ctx.strokeRect(px, py, pw, ph)

    const page = pages[i]
    if (page) {
      for (const el of page.elements) {
        if (el.element_type === 'container') continue
        const ex = px + el.x_mm * s
        const ey = py + el.y_mm * s
        const ew = Math.max(1, el.width_mm * s)
        const eh = Math.max(1, el.height_mm * s)

        ctx.fillStyle = elementColor(el.element_type)
        ctx.globalAlpha = 0.7
        ctx.fillRect(ex, ey, ew, eh)
        ctx.globalAlpha = 1
      }
    }
  }

  contentDirty = false
}

/** Ana canvas'a composite: cached content + viewport dikdörtgeni */
function compose() {
  const canvas = canvasRef.value
  if (!canvas) return

  if (contentDirty || !contentCanvas) {
    drawContent()
  }

  const dpr = window.devicePixelRatio || 1
  const w = canvasWidth.value
  const h = canvasHeight.value

  canvas.width = Math.ceil(w * dpr)
  canvas.height = Math.ceil(h * dpr)
  canvas.style.width = `${w}px`
  canvas.style.height = `${h}px`

  const ctx = canvas.getContext('2d')!
  ctx.resetTransform()

  // Offscreen content'i kopyala (1:1 pixel, zaten dpr ölçekli)
  ctx.drawImage(contentCanvas!, 0, 0)

  // Viewport dikdörtgenini çiz (dpr ölçekli)
  ctx.scale(dpr, dpr)
  const v = viewportRect.value
  ctx.strokeStyle = '#2563eb'
  ctx.lineWidth = 1.5
  ctx.strokeRect(v.x, v.y, v.w, v.h)
  ctx.fillStyle = 'rgba(37, 99, 235, 0.08)'
  ctx.fillRect(v.x, v.y, v.w, v.h)
}

// rAF throttle — aynı frame'de birden fazla compose çağrısını engelle
let composeRAF: number | null = null
function scheduleCompose() {
  if (composeRAF !== null) return
  composeRAF = requestAnimationFrame(() => {
    composeRAF = null
    compose()
  })
}

// --- Scroll yönetimi ---

function smoothScrollTo(target: number) {
  scrollRef.value?.scrollTo({ top: target, behavior: 'smooth' })
}

function jumpScrollTo(target: number) {
  if (scrollRef.value) scrollRef.value.scrollTop = target
}

// --- Pointer etkileşimi ---

/** Canvas px → editör mm (sayfa gap dönüşümü dahil) */
function canvasToMm(clientX: number, clientY: number): { mmX: number; mmY: number } {
  const canvas = canvasRef.value!
  const rect = canvas.getBoundingClientRect()
  const mx = clientX - rect.left - PADDING
  const my = clientY - rect.top - PADDING
  const s = minimapScale.value

  // Y: canvas px'ten hangi sayfadayız bul, editör mm'e çevir
  const pageStridePx = pageHeightPx.value + MINIMAP_PAGE_GAP_PX
  const pageIdx = Math.min(pageCount.value - 1, Math.max(0, Math.floor(my / pageStridePx)))
  const withinPagePx = my - pageIdx * pageStridePx
  const withinPageMm = withinPagePx / s
  const editorStride = props.pageHeight + editorGapMm.value
  const mmY = pageIdx * editorStride + withinPageMm

  return { mmX: mx / s, mmY }
}

function navigateTo(clientX: number, clientY: number) {
  const { mmX, mmY } = canvasToMm(clientX, clientY)
  const viewW = props.containerWidth
  const viewH = props.containerHeight - 60 - 40
  const pageWidthPx = props.pageWidth * props.scale

  const newPanX = -(mmX * props.scale) + viewW / 2 - (viewW - pageWidthPx) / 2
  const newPanY = -(mmY * props.scale) + viewH / 2

  emit('navigate', newPanX, newPanY)
}

function onPointerDown(e: PointerEvent) {
  e.preventDefault()
  e.stopPropagation()
  isPointerDragging.value = true
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  navigateTo(e.clientX, e.clientY)
}

function onPointerMove(e: PointerEvent) {
  if (!isPointerDragging.value) return
  navigateTo(e.clientX, e.clientY)
}

function onPointerUp(e: PointerEvent) {
  if (isPointerDragging.value) {
    isPointerDragging.value = false
    ;(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId)
  }
}

function onMouseEnter() {
  isHovered.value = true
}

function onMouseLeave() {
  if (!isPointerDragging.value) {
    isHovered.value = false
  }
}

watch(isPointerDragging, (dragging) => {
  if (!dragging) {
    nextTick(() => {
      const el = scrollRef.value
      if (el && !el.matches(':hover')) {
        isHovered.value = false
      }
    })
  }
})

// Layout değiştiğinde content'i dirty işaretle + tam redraw
watch(() => props.layout, () => {
  contentDirty = true
  scheduleCompose()
}, { deep: true })

// Scale değiştiğinde (zoom) content'i de yeniden çizmek gerekir (gapMm değişir)
watch(() => props.scale, () => {
  contentDirty = true
  scheduleCompose()
})

// Pan değiştiğinde sadece viewport overlay'i yeniden çiz (ucuz)
// Minimap drag sırasında scroll yapma — kullanıcı zaten sürükleyerek kontrol ediyor
watch([() => props.panX, () => props.panY], () => {
  scheduleCompose()
  if (!isPointerDragging.value) {
    smoothScrollTo(targetScrollTop.value)
  }
})

// Zoom değiştiğinde scroll da güncelle
watch(() => props.zoom, () => {
  smoothScrollTo(targetScrollTop.value)
})

// Container boyutu değiştiğinde
watch([() => props.containerWidth, () => props.containerHeight], () => {
  scheduleCompose()
})

// Hover/collapse durumu değiştiğinde
watch([isHovered, isPointerDragging], () => {
  nextTick(() => {
    if (isHovered.value || isPointerDragging.value) {
      smoothScrollTo(targetScrollTop.value)
    } else {
      jumpScrollTo(targetScrollTop.value)
    }
  })
})

onMounted(() => {
  drawContent()
  compose()
  jumpScrollTo(targetScrollTop.value)
})
</script>

<template>
  <div
    class="minimap"
    :class="{
      'minimap--expanded': isHovered || isPointerDragging,
      'minimap--dragging': isPointerDragging,
    }"
    :style="{ width: `${canvasWidth}px`, height: `${visibleHeight}px` }"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <div
      ref="scrollRef"
      class="minimap__scroll"
      :style="{ height: `${visibleHeight}px` }"
    >
      <canvas
        ref="canvasRef"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
      />
    </div>
  </div>
</template>

<style scoped>
.minimap {
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid #d1d5db;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  cursor: crosshair;
  user-select: none;
  backdrop-filter: blur(4px);
  overflow: hidden;
  transition: height 0.25s ease;
}

.minimap--expanded {
  border-color: #93c5fd;
  box-shadow: 0 2px 12px rgba(37, 99, 235, 0.15);
}

.minimap--dragging {
  cursor: grabbing;
}

.minimap__scroll {
  overflow: hidden;
  scroll-behavior: auto;
}

.minimap__scroll canvas {
  display: block;
}
</style>
