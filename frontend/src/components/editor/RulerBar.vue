<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'

const props = defineProps<{
  /** Sayfa genişliği mm */
  pageWidth: number
  /** Sayfa yüksekliği mm */
  pageHeight: number
  /** mm → px dönüşüm katsayısı (scale * zoom) */
  scale: number
  /** Pan offset X (px) */
  panX: number
  /** Pan offset Y (px) */
  panY: number
  /** Cetvel kalınlığı px */
  rulerSize?: number
}>()

const RULER_SIZE = computed(() => props.rulerSize ?? 20)

const hCanvas = ref<HTMLCanvasElement | null>(null)
const vCanvas = ref<HTMLCanvasElement | null>(null)

function drawRuler(
  canvas: HTMLCanvasElement | null,
  direction: 'horizontal' | 'vertical',
) {
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const size = RULER_SIZE.value

  if (direction === 'horizontal') {
    const w = canvas.clientWidth
    canvas.width = w * dpr
    canvas.height = size * dpr
    ctx.scale(dpr, dpr)
    ctx.clearRect(0, 0, w, size)
    ctx.fillStyle = '#f1f5f9'
    ctx.fillRect(0, 0, w, size)
    ctx.strokeStyle = '#e2e8f0'
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(0, size - 0.5)
    ctx.lineTo(w, size - 0.5)
    ctx.stroke()
    drawTicks(ctx, direction, w, size)
  } else {
    const h = canvas.clientHeight
    canvas.width = size * dpr
    canvas.height = h * dpr
    ctx.scale(dpr, dpr)
    ctx.clearRect(0, 0, size, h)
    ctx.fillStyle = '#f1f5f9'
    ctx.fillRect(0, 0, size, h)
    ctx.strokeStyle = '#e2e8f0'
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(size - 0.5, 0)
    ctx.lineTo(size - 0.5, h)
    ctx.stroke()
    drawTicks(ctx, direction, h, size)
  }
}

function drawTicks(
  ctx: CanvasRenderingContext2D,
  direction: 'horizontal' | 'vertical',
  length: number,
  size: number,
) {
  const s = props.scale
  const pageMm = direction === 'horizontal' ? props.pageWidth : props.pageHeight
  const pan = direction === 'horizontal' ? props.panX : props.panY

  // Sayfa başlangıcı: ortaya hizalı + pan
  // EditorCanvas sayfayı ortalar, ruler da buna uymalı
  // Yatay: canvas ortası - sayfa genişliği/2
  // Sayfanın canvas üzerindeki orijin px'i
  const canvasCenter = direction === 'horizontal'
    ? (length / 2)  // flex centering approximation
    : 40  // EditorCanvas padding-top: 40px

  const pageStartPx = canvasCenter - (pageMm * s) / 2 + pan

  // Tick aralığı belirleme (zoom'a göre)
  const mmPerPx = 1 / s
  let tickMm: number
  if (mmPerPx > 2) tickMm = 50
  else if (mmPerPx > 1) tickMm = 20
  else if (mmPerPx > 0.5) tickMm = 10
  else if (mmPerPx > 0.2) tickMm = 5
  else tickMm = 1

  ctx.fillStyle = '#94a3b8'
  ctx.strokeStyle = '#94a3b8'
  ctx.lineWidth = 0.5
  ctx.font = '9px system-ui, sans-serif'
  ctx.textBaseline = 'top'

  // Sayfanın mm aralığını çiz
  const startMm = 0
  const endMm = pageMm

  for (let mm = startMm; mm <= endMm; mm += tickMm) {
    const px = pageStartPx + mm * s

    if (px < -10 || px > length + 10) continue

    const isMajor = mm % 10 === 0
    const isMedium = mm % 5 === 0

    let tickLen = 4
    if (isMajor) tickLen = size * 0.6
    else if (isMedium) tickLen = size * 0.35

    ctx.beginPath()
    if (direction === 'horizontal') {
      ctx.moveTo(px, size)
      ctx.lineTo(px, size - tickLen)
    } else {
      ctx.moveTo(size, px)
      ctx.lineTo(size - tickLen, px)
    }
    ctx.stroke()

    // Sayı etiketi (her 10mm'de bir)
    if (isMajor && mm > 0) {
      const label = String(mm)
      if (direction === 'horizontal') {
        ctx.textAlign = 'center'
        ctx.fillText(label, px, 2)
      } else {
        ctx.save()
        ctx.translate(2, px)
        ctx.rotate(-Math.PI / 2)
        ctx.textAlign = 'center'
        ctx.fillText(label, 0, 0)
        ctx.restore()
      }
    }
  }

  // Sayfa kenar çizgileri (margin göstergesi)
  ctx.strokeStyle = 'rgba(59, 130, 246, 0.3)'
  ctx.lineWidth = 1
  const startPx = pageStartPx
  const endPx = pageStartPx + pageMm * s
  ctx.beginPath()
  if (direction === 'horizontal') {
    ctx.moveTo(startPx, 0)
    ctx.lineTo(startPx, size)
    ctx.moveTo(endPx, 0)
    ctx.lineTo(endPx, size)
  } else {
    ctx.moveTo(0, startPx)
    ctx.lineTo(size, startPx)
    ctx.moveTo(0, endPx)
    ctx.lineTo(size, endPx)
  }
  ctx.stroke()
}

function redraw() {
  drawRuler(hCanvas.value, 'horizontal')
  drawRuler(vCanvas.value, 'vertical')
}

watch(() => [props.scale, props.panX, props.panY, props.pageWidth, props.pageHeight], redraw)

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  redraw()
  const parent = hCanvas.value?.parentElement?.parentElement
  if (parent) {
    resizeObserver = new ResizeObserver(() => redraw())
    resizeObserver.observe(parent)
  }
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
})
</script>

<template>
  <div class="ruler-corner" :style="{ width: `${RULER_SIZE}px`, height: `${RULER_SIZE}px` }" />
  <canvas
    ref="hCanvas"
    class="ruler-h"
    :style="{ height: `${RULER_SIZE}px` }"
  />
  <canvas
    ref="vCanvas"
    class="ruler-v"
    :style="{ width: `${RULER_SIZE}px` }"
  />
</template>

<style scoped>
.ruler-corner {
  position: absolute;
  top: 0;
  left: 0;
  background: #f1f5f9;
  border-right: 1px solid #e2e8f0;
  border-bottom: 1px solid #e2e8f0;
  z-index: 50;
}

.ruler-h {
  position: absolute;
  top: 0;
  left: 20px;
  right: 0;
  z-index: 50;
  pointer-events: none;
}

.ruler-v {
  position: absolute;
  top: 20px;
  left: 0;
  bottom: 0;
  z-index: 50;
  pointer-events: none;
}
</style>
