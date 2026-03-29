<script setup lang="ts">
import { computed, inject, watch, nextTick } from 'vue'
import type { ElementLayout, LayoutResult } from '../../core/layout-types'

const props = defineProps<{
  layout: LayoutResult | null
  scale: number
}>()

// WASM barcode üretme fonksiyonu (EditorCanvas'tan provide edilir)
const generateBarcode = inject<(format: string, value: string, width: number, height: number, includeText: boolean) => Promise<{ width: number; height: number; rgba: ArrayBuffer } | null>>('generateBarcode')

const pageElements = computed(() => {
  if (!props.layout || props.layout.pages.length === 0) return []
  return props.layout.pages[0].elements
})

function elStyle(el: ElementLayout): Record<string, string> {
  const s = props.scale
  return {
    position: 'absolute',
    left: `${el.x_mm * s}px`,
    top: `${el.y_mm * s}px`,
    width: `${el.width_mm * s}px`,
    height: `${el.height_mm * s}px`,
  }
}

function textStyle(el: ElementLayout): Record<string, string> {
  const s = props.scale
  const st = el.style
  const result: Record<string, string> = {}

  // fontSize pt cinsinden → mm'ye çevir (1pt = 0.3528mm), sonra scale ile px'e
  if (st.fontSize) result.fontSize = `${st.fontSize * 0.3528 * s}px`
  if (st.fontWeight) result.fontWeight = st.fontWeight
  if (st.fontFamily) result.fontFamily = st.fontFamily
  if (st.color) result.color = st.color
  if (st.textAlign) result.textAlign = st.textAlign

  result.lineHeight = '1.2'
  result.overflow = 'hidden'
  result.wordBreak = 'break-word'

  return result
}

function containerStyle(el: ElementLayout): Record<string, string> {
  const st = el.style
  const result: Record<string, string> = {}

  if (st.backgroundColor) result.backgroundColor = st.backgroundColor
  if (st.borderColor && st.borderWidth) {
    result.border = `${st.borderWidth * props.scale}px ${st.borderStyle ?? 'solid'} ${st.borderColor}`
  }
  if (st.borderRadius) result.borderRadius = `${st.borderRadius * props.scale}px`

  return result
}

function lineStyle(el: ElementLayout): Record<string, string> {
  const st = el.style
  return {
    borderTop: `${(st.strokeWidth ?? 0.5) * props.scale}px solid ${st.strokeColor ?? '#000'}`,
    width: '100%',
    height: '0',
  }
}

// --- Barcode rendering (WASM ile) ---

async function renderBarcodeToCanvas(canvas: HTMLCanvasElement, format: string, value: string, includeText: boolean = false) {
  if (!value || !generateBarcode) return

  try {
    // WASM'dan yüksek çözünürlüklü pixel verisi al
    // QR her zaman kare
    const isQr = format === 'qr'
    const size = isQr ? 300 : 400
    const height = isQr ? 300 : 150
    const result = await generateBarcode(format, value, size, height, isQr ? false : includeText)
    if (!result) return

    // Canvas boyutlarını WASM çıktısına ayarla (crisp rendering)
    canvas.width = result.width
    canvas.height = result.height

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const imageData = new ImageData(
      new Uint8ClampedArray(result.rgba),
      result.width,
      result.height,
    )
    ctx.putImageData(imageData, 0, 0)
  } catch (e) {
    console.warn(`[dreport] WASM barcode render hatası (${format}):`, e)
    renderBarcodeFallback(canvas, format)
  }
}

function renderBarcodeFallback(canvas: HTMLCanvasElement, format: string) {
  canvas.width = 200
  canvas.height = 80
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  ctx.fillStyle = '#f3f4f6'
  ctx.fillRect(0, 0, 200, 80)
  ctx.fillStyle = '#ef4444'
  ctx.font = '11px sans-serif'
  ctx.textAlign = 'center'
  ctx.fillText(`[${format}] hata`, 100, 44)
}

/** Canvas mount olduğunda render et */
function onBarcodeCanvasMounted(el: HTMLCanvasElement | null) {
  if (!el) return
  const format = el.dataset.format
  const value = el.dataset.value
  const includeText = el.dataset.includeText === 'true'
  if (format && value) {
    renderBarcodeToCanvas(el, format, value, includeText)
  }
}

// Layout değiştiğinde tüm barcode canvas'ları yeniden render et
watch(
  () => props.layout,
  async () => {
    await nextTick()
    await nextTick()
    const canvases = document.querySelectorAll<HTMLCanvasElement>('canvas[data-barcode]')
    canvases.forEach(canvas => {
      const format = canvas.dataset.format
      const value = canvas.dataset.value
      const includeText = canvas.dataset.includeText === 'true'
      if (format && value) {
        renderBarcodeToCanvas(canvas, format, value, includeText)
      }
    })
  },
  { deep: true }
)
</script>

<template>
  <div class="layout-renderer" v-if="layout">
    <template v-for="el in pageElements" :key="el.id">
      <!-- Container -->
      <div
        v-if="el.element_type === 'container'"
        class="layout-el layout-el--container"
        :style="{ ...elStyle(el), ...containerStyle(el) }"
      />

      <!-- Static text / Text / Page number -->
      <div
        v-else-if="el.element_type === 'static_text' || el.element_type === 'text' || el.element_type === 'page_number'"
        class="layout-el layout-el--text"
        :style="{ ...elStyle(el), ...textStyle(el) }"
      >
        {{ el.content?.type === 'text' ? el.content.value : '' }}
      </div>

      <!-- Line -->
      <div
        v-else-if="el.element_type === 'line'"
        class="layout-el layout-el--line"
        :style="elStyle(el)"
      >
        <div :style="lineStyle(el)" />
      </div>

      <!-- Image -->
      <div
        v-else-if="el.element_type === 'image'"
        class="layout-el layout-el--image"
        :style="elStyle(el)"
      >
        <img
          v-if="el.content?.type === 'image' && el.content.src"
          :src="el.content.src"
          :style="{
            width: '100%',
            height: '100%',
            objectFit: 'fill',
          }"
        />
        <div v-else class="layout-el__placeholder">Görsel</div>
      </div>

      <!-- Barcode -->
      <div
        v-else-if="el.element_type === 'barcode'"
        class="layout-el layout-el--barcode"
        :style="elStyle(el)"
      >
        <canvas
          v-if="el.content?.type === 'barcode' && el.content.value"
          :ref="(ref) => onBarcodeCanvasMounted(ref as HTMLCanvasElement)"
          data-barcode
          :data-format="el.content.format"
          :data-value="el.content.value"
          :data-include-text="el.style.barcodeIncludeText ?? (el.content.format === 'ean13' || el.content.format === 'ean8')"
          :style="{ width: '100%', height: '100%', display: 'block' }"
        />
        <div v-else class="layout-el__placeholder">
          {{ el.content?.type === 'barcode' ? `[${el.content.format}]` : '[barcode]' }}
        </div>
      </div>
    </template>
  </div>

  <div class="layout-renderer layout-renderer--empty" v-else>
    <span>Hesaplanıyor...</span>
  </div>
</template>

<style scoped>
.layout-renderer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  user-select: none;
}

.layout-renderer--empty {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
  font-size: 14px;
}

.layout-el {
  box-sizing: border-box;
}

.layout-el--text {
  white-space: pre-wrap;
  font-family: 'Noto Sans', sans-serif;
}

.layout-el--line {
  display: flex;
  align-items: center;
}

.layout-el__placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f3f4f6;
  color: #9ca3af;
  font-size: 11px;
  border: 1px dashed #d1d5db;
  border-radius: 2px;
}
</style>
