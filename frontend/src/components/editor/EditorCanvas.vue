<script setup lang="ts">
import { computed, ref, watch, provide, onMounted, onBeforeUnmount } from 'vue'
import { storeToRefs } from 'pinia'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { useLayoutEngine } from '../../composables/useLayoutEngine'
import LayoutRenderer from './LayoutRenderer.vue'
import InteractionOverlay from './InteractionOverlay.vue'

const props = withDefaults(defineProps<{
  handleErrors?: boolean
}>(), {
  handleErrors: true,
})

const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const { template, mockData, layoutVersion } = storeToRefs(templateStore)

const containerRef = ref<HTMLElement | null>(null)
const containerWidth = ref(800)

const emit = defineEmits<{
  'compile-error': [error: string | null]
}>()

// Layout engine — template + data'yı worker'a gönderir, WASM ile layout hesaplar
const { layout, layoutMap, error, computing: compiling, generateBarcode, dispose } = useLayoutEngine(template, mockData, layoutVersion)

// LayoutRenderer'ın barcode üretmek için kullanabileceği fonksiyon
provide('generateBarcode', generateBarcode)

watch(error, (val) => emit('compile-error', val))

// mm → px dönüşüm katsayısı
const scale = computed(() => {
  return (containerWidth.value / templateStore.template.page.width) * editorStore.zoom
})

// Sayfa boyutu px cinsinden + margin CSS variables
const pageStyle = computed(() => {
  const w = templateStore.template.page.width * scale.value
  const h = templateStore.template.page.height * scale.value
  const m = templateStore.template.root.padding
  return {
    width: `${w}px`,
    height: `${h}px`,
    '--page-margin-top': `${m.top * scale.value}px`,
    '--page-margin-right': `${m.right * scale.value}px`,
    '--page-margin-bottom': `${m.bottom * scale.value}px`,
    '--page-margin-left': `${m.left * scale.value}px`,
  }
})

// Pan transform — sayfa container'ına uygulanacak
const panTransform = computed(() => {
  if (editorStore.panX === 0 && editorStore.panY === 0) return undefined
  return `translate(${editorStore.panX}px, ${editorStore.panY}px)`
})

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
    resizeObserver = new ResizeObserver(entries => {
      const entry = entries[0]
      if (entry) containerWidth.value = entry.contentRect.width
    })
    resizeObserver.observe(containerRef.value)
  }
  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('keyup', onKeyUp)
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  dispose()
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('keyup', onKeyUp)
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
    editorStore.setPan(
      editorStore.panX - e.deltaX,
      editorStore.panY - e.deltaY,
    )
  }
}

function applyZoom(delta: number, clientX: number, clientY: number) {
  const pageEl = pageRef.value
  if (!pageEl) return

  const oldZoom = editorStore.zoom
  const zoomFactor = Math.pow(0.99, delta)
  const newZoom = Math.max(0.25, Math.min(4, oldZoom * zoomFactor))
  if (newZoom === oldZoom) return

  // Sayfa elemanının şu anki ekran pozisyonunu al (centering + pan dahil)
  const pageRect = pageEl.getBoundingClientRect()

  // Mouse'un sayfa üzerindeki pozisyonu (mm cinsinden)
  const baseScale = containerWidth.value / templateStore.template.page.width
  const oldScale = baseScale * oldZoom
  const newScale = baseScale * newZoom
  const mousePageMmX = (clientX - pageRect.left) / oldScale
  const mousePageMmY = (clientY - pageRect.top) / oldScale

  const pageW = templateStore.template.page.width

  // Yeni pan: mouse'un gösterdiği mm noktası aynı ekran pozisyonunda kalmalı
  const newPanX = editorStore.panX + (mousePageMmX - pageW / 2) * (oldScale - newScale)
  const newPanY = editorStore.panY + mousePageMmY * (oldScale - newScale)

  editorStore.setZoom(newZoom)
  editorStore.setPan(newPanX, newPanY)
}

function onKeyDown(e: KeyboardEvent) {
  if (e.code === 'Space' && !e.repeat && !(e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement || e.target instanceof HTMLTextAreaElement)) {
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
  editorStore.setPan(e.clientX - panStart.value.x, e.clientY - panStart.value.y)
}

function onPointerUp(e: PointerEvent) {
  if (isPanning.value) {
    isPanning.value = false
    ;(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId)
  }
}
</script>

<template>
  <div class="editor-canvas-wrapper">
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
      <!-- Sayfa -->
      <div ref="pageRef" class="editor-canvas__page" :style="[pageStyle, panTransform ? { transform: panTransform } : {}]">
        <LayoutRenderer :layout="layout" :scale="scale" />
        <InteractionOverlay :scale="scale" :layout-map="layoutMap" />
      </div>
    </div>

    <!-- Sabit overlay'ler — scroll dışında -->
    <div v-if="props.handleErrors && error" class="editor-canvas__error">
      {{ error }}
    </div>
    <div v-if="compiling" class="editor-canvas__compiling">
      Derleniyor...
    </div>
    <div class="editor-canvas__zoom">
      %{{ editorStore.zoomPercent }}
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
}

.editor-canvas__page {
  background: white;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
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

.editor-canvas__zoom {
  position: absolute;
  bottom: 12px;
  right: 16px;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 12px;
  z-index: 100;
}
</style>
