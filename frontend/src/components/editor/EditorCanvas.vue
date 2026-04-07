<script setup lang="ts">
import { computed, ref, watch, provide, onMounted, onBeforeUnmount } from 'vue'
import { storeToRefs } from 'pinia'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { useLayoutEngine } from '../../composables/useLayoutEngine'
import LayoutRenderer from './LayoutRenderer.vue'
import InteractionOverlay from './InteractionOverlay.vue'
import RulerBar from './RulerBar.vue'
import MinimapOverlay from './MinimapOverlay.vue'

const props = withDefaults(
  defineProps<{
    handleErrors?: boolean
  }>(),
  {
    handleErrors: true,
  },
)

const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const { template, mockData, layoutVersion } = storeToRefs(templateStore)

const containerRef = ref<HTMLElement | null>(null)
const containerWidth = ref(800)
const containerHeight = ref(600)

const emit = defineEmits<{
  'compile-error': [error: string | null]
}>()

// Layout engine — template + data'yı worker'a gönderir, WASM ile layout hesaplar
const {
  layout,
  layoutMap,
  error,
  computing: compiling,
  generateBarcode,
  dispose,
} = useLayoutEngine(template, mockData, layoutVersion)

// LayoutRenderer'ın barcode üretmek için kullanabileceği fonksiyon
provide('generateBarcode', generateBarcode)

watch(error, (val) => emit('compile-error', val))

// ============================================================
// Zoom gesture: CSS transform ile anlık geri bildirim,
// debounce ile gerçek scale commit
// ============================================================

// committedZoom: son commit edilen zoom seviyesi (bu değer scale'i belirler)
const committedZoom = ref(editorStore.zoom)
// Gesture sırasında hedef zoom/pan (henüz commit edilmedi)
const gestureZoom = ref(editorStore.zoom)
const gesturePanX = ref(editorStore.panX)
const gesturePanY = ref(editorStore.panY)
const isZoomGesture = ref(false)
let zoomCommitTimer: ReturnType<typeof setTimeout> | null = null

// mm → px dönüşüm katsayısı (committed zoom'a bağlı)
const scale = computed(() => {
  return (containerWidth.value / templateStore.template.page.width) * committedZoom.value
})

// Layout sayfaları
const layoutPages = computed(() => layout.value?.pages ?? [])

// Sayfa yüksekliği px cinsinden
const pageHeightPx = computed(() => templateStore.template.page.height * scale.value)

// Görünür sayfa indeksleri — viewport dışındaki sayfaların DOM elemanları render edilmez
// Stabil: sadece gerçek indeksler değiştiğinde yeni Set oluştur
const _lastVisibleKey = ref('')
const _lastVisibleSet = ref(new Set<number>([0]))

const visiblePageIndices = computed(() => {
  // Gesture sırasında gesture değerlerini, yoksa store değerlerini kullan
  const currentPanY = isZoomGesture.value ? gesturePanY.value : editorStore.panY
  const currentZoom = isZoomGesture.value ? gestureZoom.value : editorStore.zoom
  const baseScale = containerWidth.value / templateStore.template.page.width
  const currentScale = baseScale * currentZoom
  const pageH = templateStore.template.page.height * currentScale
  const gap = 24
  const count = layoutPages.value.length
  if (count === 0) return _lastVisibleSet.value

  const pagesTop = 60 + currentPanY
  const viewH = containerHeight.value

  const indices: number[] = []
  for (let i = 0; i < count; i++) {
    const pageTop = pagesTop + i * (pageH + gap)
    const pageBottom = pageTop + pageH
    const buffer = pageH
    if (pageBottom > -buffer && pageTop < viewH + buffer) {
      indices.push(i)
    }
  }

  const key = indices.join(',')
  if (key !== _lastVisibleKey.value) {
    _lastVisibleKey.value = key
    _lastVisibleSet.value = new Set(indices)
  }
  return _lastVisibleSet.value
})

// CSS transform zoom oranı — gesture sırasında visual feedback
const zoomCssRatio = computed(() => {
  if (!isZoomGesture.value) return 1
  return gestureZoom.value / committedZoom.value
})

// Sayfalar container stili — committed scale'e göre
const pagesContainerStyle = computed(() => {
  const w = templateStore.template.page.width * scale.value
  const m = templateStore.template.root.padding
  const pageCount = Math.max(1, layoutPages.value.length)
  const pageGap = 24
  const totalH = pageHeightPx.value * pageCount + pageGap * (pageCount - 1)
  return {
    width: `${w}px`,
    height: `${totalH}px`,
    position: 'relative' as const,
    flexShrink: 0,
    willChange: 'transform' as const,
    '--page-margin-top': `${m.top * scale.value}px`,
    '--page-margin-right': `${m.right * scale.value}px`,
    '--page-margin-bottom': `${m.bottom * scale.value}px`,
    '--page-margin-left': `${m.left * scale.value}px`,
  }
})

// Pan sınırları
function clampPan(x: number, y: number, zoomOverride?: number): [number, number] {
  const z = zoomOverride ?? committedZoom.value
  const baseScale = containerWidth.value / templateStore.template.page.width
  const s = baseScale * z
  const pageW = templateStore.template.page.width * s
  const pageCount = Math.max(1, layoutPages.value.length)
  const pageGap = 24
  const phPx = templateStore.template.page.height * s
  const totalH = phPx * pageCount + pageGap * (pageCount - 1)

  const viewH = (containerRef.value?.clientHeight ?? 600) - 60 - 40

  const clampX = pageW / 2
  const maxY = viewH * 0.5
  const minY = viewH * 0.5 - totalH

  return [
    Math.max(-clampX, Math.min(clampX, x)),
    Math.max(minY, Math.min(maxY, y)),
  ]
}

// Pages container transform — pan + gesture zoom CSS scale
const pagesTransform = computed(() => {
  const ratio = zoomCssRatio.value
  const panX = isZoomGesture.value ? gesturePanX.value : editorStore.panX
  const panY = isZoomGesture.value ? gesturePanY.value : editorStore.panY

  if (ratio === 1) {
    if (panX === 0 && panY === 0) return undefined
    return `translate(${panX}px, ${panY}px)`
  }

  // Scale from top-left (0 0). Centering düzeltmesi:
  // Flex container ortalar → naturalLeft = (containerW - w) / 2
  // Scale sonrası visual width = w * ratio, visual center kayar
  // Düzeltme: tx += w * (1 - ratio) / 2
  const w = templateStore.template.page.width * scale.value
  const centerCorrection = w * (1 - ratio) / 2
  const tx = panX + centerCorrection
  const ty = panY

  return `translate(${tx}px, ${ty}px) scale(${ratio})`
})

const pagesTransformOrigin = computed(() => {
  if (zoomCssRatio.value === 1) return undefined
  return '0 0'
})

// Zoom commit: gesture sonunda gerçek scale'i güncelle
function commitZoom() {
  const z = gestureZoom.value
  const px = gesturePanX.value
  let py = gesturePanY.value

  const ratio = z / committedZoom.value
  const pageCount = layoutPages.value.length

  // Gap düzeltmesi: CSS scale sırasında 24px gap'ler de ratio ile ölçekleniyor.
  // Commit sonrası gap'ler tekrar 24px'e dönüyor → dikey kayma.
  // Viewport merkezindeki sayfanın üstündeki gap sayısı × 24 × (ratio - 1) kadar düzelt.
  if (ratio !== 1 && pageCount > 1) {
    const pageH_dom = templateStore.template.page.height * scale.value // committed scale'de
    const strideVisual = (pageH_dom + 24) * ratio

    // Viewport merkezinin container visual koordinatındaki Y pozisyonu
    const viewCenterY = containerHeight.value / 2 - 60 - py
    if (viewCenterY > 0 && strideVisual > 0) {
      const gapsAbove = Math.min(pageCount - 1, Math.max(0, Math.floor(viewCenterY / strideVisual)))
      py += gapsAbove * 24 * (ratio - 1)
    }
  }

  committedZoom.value = z
  editorStore.setZoom(z)
  const [cx, cy] = clampPan(px, py, z)
  editorStore.setPan(cx, cy)
  isZoomGesture.value = false
  zoomCommitTimer = null
}

function scheduleZoomCommit() {
  if (zoomCommitTimer) clearTimeout(zoomCommitTimer)
  zoomCommitTimer = setTimeout(commitZoom, 120)
}

// Pan: Space+drag veya orta fare tuşu
const isPanning = ref(false)
const panStart = ref({ x: 0, y: 0 })
const spaceHeld = ref(false)

// Pan cursor style
const canvasCursor = computed(() => {
  if (isPanning.value) return 'grabbing'
  if (spaceHeld.value) return 'grab'
  return 'default'
})

// Container boyutunu izle
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (containerRef.value) {
    resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (entry) {
        containerWidth.value = entry.contentRect.width
        containerHeight.value = entry.contentRect.height
      }
    })
    resizeObserver.observe(containerRef.value)
  }
  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('keyup', onKeyUp)
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  dispose()
  if (zoomCommitTimer) clearTimeout(zoomCommitTimer)
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('keyup', onKeyUp)
})

// Store'daki zoom değiştiğinde (dışarıdan, ör. zoom butonları) committed'ı da güncelle
watch(() => editorStore.zoom, (z) => {
  if (!isZoomGesture.value) {
    committedZoom.value = z
    gestureZoom.value = z
  }
})

// Zoom & Pan via wheel/trackpad
const pageRef = ref<HTMLElement | null>(null)

let zoomRAF: number | null = null
let zoomDeltaAccum = 0
let zoomClientX = 0
let zoomClientY = 0

function onWheel(e: WheelEvent) {
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault()

    zoomDeltaAccum += e.deltaY
    zoomClientX = e.clientX
    zoomClientY = e.clientY

    if (zoomRAF === null) {
      zoomRAF = requestAnimationFrame(() => {
        const delta = Math.max(-4, Math.min(4, zoomDeltaAccum))
        if (Math.abs(delta) > 0.01) {
          applyZoom(delta, zoomClientX, zoomClientY)
        }
        zoomDeltaAccum = 0
        zoomRAF = null
      })
    }
  } else {
    // İki parmak pan (touchpad) veya normal scroll
    e.preventDefault()
    const curPanX = isZoomGesture.value ? gesturePanX.value : editorStore.panX
    const curPanY = isZoomGesture.value ? gesturePanY.value : editorStore.panY
    const curZoom = isZoomGesture.value ? gestureZoom.value : editorStore.zoom
    const [cx, cy] = clampPan(curPanX - e.deltaX, curPanY - e.deltaY, curZoom)

    if (isZoomGesture.value) {
      gesturePanX.value = cx
      gesturePanY.value = cy
    } else {
      editorStore.setPan(cx, cy)
    }
  }
}

function applyZoom(delta: number, clientX: number, clientY: number) {
  const pageEl = pageRef.value
  if (!pageEl) return

  // Gesture başlat veya devam et
  if (!isZoomGesture.value) {
    isZoomGesture.value = true
    gestureZoom.value = editorStore.zoom
    gesturePanX.value = editorStore.panX
    gesturePanY.value = editorStore.panY
  }

  const oldZoom = gestureZoom.value
  const zoomFactor = Math.pow(0.99, delta)
  const newZoom = Math.max(0.25, Math.min(4, oldZoom * zoomFactor))
  if (newZoom === oldZoom) return

  // Mouse'un sayfa üzerindeki pozisyonu (mm cinsinden)
  // pageRef'in ekran pozisyonunu al (CSS transform dahil)
  const pageRect = pageEl.getBoundingClientRect()
  const baseScale = containerWidth.value / templateStore.template.page.width
  const oldGestureScale = baseScale * oldZoom
  const newGestureScale = baseScale * newZoom
  const mousePageMmX = (clientX - pageRect.left) / oldGestureScale
  const mousePageMmY = (clientY - pageRect.top) / oldGestureScale

  const pageW = templateStore.template.page.width

  // Yeni pan: mouse'un gösterdiği mm noktası aynı ekran pozisyonunda kalmalı
  const newPanX = gesturePanX.value + (mousePageMmX - pageW / 2) * (oldGestureScale - newGestureScale)
  const newPanY = gesturePanY.value + mousePageMmY * (oldGestureScale - newGestureScale)

  gestureZoom.value = newZoom
  const [cx, cy] = clampPan(newPanX, newPanY, newZoom)
  gesturePanX.value = cx
  gesturePanY.value = cy

  scheduleZoomCommit()
}

function onKeyDown(e: KeyboardEvent) {
  if (
    e.code === 'Space' &&
    !e.repeat &&
    !(
      e.target instanceof HTMLInputElement ||
      e.target instanceof HTMLSelectElement ||
      e.target instanceof HTMLTextAreaElement ||
      (e.target as HTMLElement)?.isContentEditable
    )
  ) {
    e.preventDefault()
    spaceHeld.value = true
  }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.code === 'Space') {
    spaceHeld.value = false
  }
}

function onPointerDown(e: PointerEvent) {
  if (e.button === 1 || (e.button === 0 && spaceHeld.value)) {
    e.preventDefault()
    isPanning.value = true
    panStart.value = { x: e.clientX - editorStore.panX, y: e.clientY - editorStore.panY }
    ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  }
}

function onPointerMove(e: PointerEvent) {
  if (!isPanning.value) return
  const [cx2, cy2] = clampPan(e.clientX - panStart.value.x, e.clientY - panStart.value.y)
  editorStore.setPan(cx2, cy2)
}

function onPointerUp(e: PointerEvent) {
  if (isPanning.value) {
    isPanning.value = false
    ;(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId)
  }
}

function onMinimapNavigate(x: number, y: number) {
  const [cx, cy] = clampPan(x, y)
  editorStore.setPan(cx, cy)
}

// Minimap'e gerçek scale'i geçir (gesture dahil)
const minimapScale = computed(() => {
  const z = isZoomGesture.value ? gestureZoom.value : editorStore.zoom
  return (containerWidth.value / templateStore.template.page.width) * z
})
</script>

<template>
  <div class="editor-canvas-wrapper">
    <!-- Cetvel -->
    <RulerBar
      :page-width="templateStore.template.page.width"
      :page-height="templateStore.template.page.height"
      :scale="minimapScale"
      :pan-x="isZoomGesture ? gesturePanX : editorStore.panX"
      :pan-y="isZoomGesture ? gesturePanY : editorStore.panY"
      :container-width="containerWidth"
      :page-count="layoutPages.length"
      :page-gap="24"
    />

    <!-- Scroll alanı -->
    <div
      class="editor-canvas"
      ref="containerRef"
      :style="{ cursor: canvasCursor }"
      @wheel="onWheel"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
    >
      <!-- Sayfalar -->
      <div
        ref="pageRef"
        class="editor-canvas__pages"
        :style="[
          pagesContainerStyle,
          pagesTransform ? { transform: pagesTransform } : {},
          pagesTransformOrigin ? { transformOrigin: pagesTransformOrigin } : {},
        ]"
      >
        <LayoutRenderer :layout="layout" :scale="scale" :visible-page-indices="visiblePageIndices" />
        <InteractionOverlay
          :scale="scale"
          :layout-map="layoutMap"
          :page-count="layoutPages.length"
          :page-height-px="pageHeightPx"
        />
      </div>
    </div>

    <!-- Sabit overlay'ler — scroll dışında -->
    <div v-if="props.handleErrors && error" class="editor-canvas__error">
      {{ error }}
    </div>
    <div v-if="compiling" class="editor-canvas__compiling">Derleniyor...</div>

    <!-- Minimap + zoom göstergesi -->
    <div class="editor-canvas__minimap-area">
      <MinimapOverlay
        :layout="layout"
        :page-width="templateStore.template.page.width"
        :page-height="templateStore.template.page.height"
        :zoom="isZoomGesture ? gestureZoom : editorStore.zoom"
        :pan-x="isZoomGesture ? gesturePanX : editorStore.panX"
        :pan-y="isZoomGesture ? gesturePanY : editorStore.panY"
        :container-width="containerWidth"
        :container-height="containerHeight"
        :scale="minimapScale"
        :page-gap="24"
        @navigate="onMinimapNavigate"
      />
      <div class="editor-canvas__zoom">%{{ Math.round((isZoomGesture ? gestureZoom : editorStore.zoom) * 100) }}</div>
    </div>
  </div>
</template>

<style scoped>
.editor-canvas-wrapper {
  flex: 1;
  position: relative;
  min-height: 0;
  min-width: 0;
  overflow: hidden;
}

.editor-canvas {
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: #e5e7eb;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 40px;
  padding-top: 60px; /* cetvel için üstten ek boşluk */
  padding-left: 60px; /* cetvel için soldan ek boşluk */
}

.editor-canvas__pages {
  position: relative;
  flex-shrink: 0;
}

.editor-canvas__error {
  position: absolute;
  top: 8px;
  left: 50%;
  transform: translateX(-50%);
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
  border-radius: 6px;
  padding: 6px 16px;
  font-size: 13px;
  z-index: 100;
}

.editor-canvas__compiling {
  position: absolute;
  top: 8px;
  right: 16px;
  background: #eff6ff;
  color: #2563eb;
  border-radius: 6px;
  padding: 4px 12px;
  font-size: 12px;
  z-index: 100;
}

.editor-canvas__minimap-area {
  position: absolute;
  bottom: 12px;
  right: 16px;
  z-index: 100;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
}

.editor-canvas__zoom {
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 12px;
}
</style>
