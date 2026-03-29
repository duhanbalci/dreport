/// Layout Engine Web Worker
/// Template JSON + Data JSON → Layout WASM → LayoutResult

import init, { loadFonts, computeLayout, generateBarcode } from '../core/wasm-layout/dreport_layout.js'
import type { LayoutResult } from '../core/layout-types'

let initPromise: Promise<void> | null = null

const FONT_FILES = [
  { path: '/fonts/NotoSans-Regular.ttf', family: 'Noto Sans' },
  { path: '/fonts/NotoSans-Bold.ttf', family: 'Noto Sans' },
  { path: '/fonts/NotoSans-Italic.ttf', family: 'Noto Sans' },
  { path: '/fonts/NotoSans-BoldItalic.ttf', family: 'Noto Sans' },
  { path: '/fonts/NotoSansMono-Regular.ttf', family: 'Noto Sans Mono' },
]

async function doInit() {
  console.log('[layout-worker] WASM başlatılıyor...')
  await init({ module_or_path: '/wasm/dreport_layout_bg.wasm' })

  console.log('[layout-worker] Fontlar yükleniyor...')
  const families: string[] = []
  const buffers: Uint8Array[] = []

  await Promise.all(
    FONT_FILES.map(async (f) => {
      const res = await fetch(new URL(f.path, self.location.origin).href)
      const buf = await res.arrayBuffer()
      families.push(f.family)
      buffers.push(new Uint8Array(buf))
    })
  )

  loadFonts(JSON.stringify(families), buffers)
  console.log('[layout-worker] Hazır')
}

function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = doInit()
  }
  return initPromise
}

type WorkerMessage =
  | { type: 'compile'; templateJson: string; dataJson: string; id: number }
  | { type: 'barcode'; format: string; value: string; width: number; height: number; includeText: boolean; id: number }

self.onmessage = async (e: MessageEvent<WorkerMessage>) => {
  const msg = e.data

  if (msg.type === 'compile') {
    try {
      await ensureInit()

      const t0 = performance.now()
      const resultJson = computeLayout(msg.templateJson, msg.dataJson)
      const layout: LayoutResult = JSON.parse(resultJson)
      console.log(`[layout-worker] render ${(performance.now() - t0).toFixed(1)}ms`)

      self.postMessage({ type: 'result', layout, id: msg.id })
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      console.error(`[layout-worker] Hata (id: ${msg.id}):`, err)
      self.postMessage({ type: 'error', error: errorMsg, id: msg.id })
    }
  } else if (msg.type === 'barcode') {
    try {
      await ensureInit()

      const raw = generateBarcode(msg.format, msg.value, msg.width, msg.height, msg.includeText)
      // İlk 8 byte header: width (4 byte LE) + height (4 byte LE)
      const dv = new DataView(raw.buffer, raw.byteOffset, 8)
      const w = dv.getUint32(0, true)
      const h = dv.getUint32(4, true)
      const rgba = raw.slice(8)

      self.postMessage(
        { type: 'barcode-result', width: w, height: h, rgba: rgba.buffer, id: msg.id },
        [rgba.buffer] as any,
      )
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      console.error(`[layout-worker] Barcode hatası (id: ${msg.id}):`, err)
      self.postMessage({ type: 'barcode-error', error: errorMsg, id: msg.id })
    }
  }
}
