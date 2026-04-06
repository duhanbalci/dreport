/// Layout Engine Web Worker
/// Template JSON + Data JSON → Layout WASM → LayoutResult
/// Font loading is dynamic — fetches from backend API based on template needs.

import init, { loadFonts, addFonts, computeLayout, generateBarcode } from '../core/wasm-layout/dreport_layout.js'
import type { LayoutResult } from '../core/layout-types'

let initPromise: Promise<void> | null = null

/** Configurable font API base URL. Default: same origin /api/fonts */
let fontApiBase = '/api/fonts'

/** Font catalog from backend API */
interface FontVariantInfo {
  weight: number
  italic: boolean
}
interface FontFamilyInfo {
  family: string
  variants: FontVariantInfo[]
}
let fontCatalog: FontFamilyInfo[] = []

/** Track which font families are already loaded into WASM */
const loadedFamilies = new Set<string>()

async function doInit() {
  console.log('[layout-worker] WASM başlatılıyor...')
  await init({ module_or_path: '/wasm/dreport_layout_bg.wasm' })

  // Fetch font catalog from backend
  try {
    const res = await fetch(fontApiBase)
    if (res.ok) {
      fontCatalog = await res.json()
      console.log(`[layout-worker] Font kataloğu yüklendi (${fontCatalog.length} aile)`)
    } else {
      console.warn(`[layout-worker] Font kataloğu alınamadı (HTTP ${res.status}), static fallback deneniyor`)
      await loadStaticFallback()
      return
    }
  } catch {
    console.warn('[layout-worker] Font API erişilemedi, static fallback deneniyor')
    await loadStaticFallback()
    return
  }

  // Load default fonts (Noto Sans + Noto Sans Mono)
  await ensureFamiliesLoaded(['Noto Sans', 'Noto Sans Mono'])
  console.log('[layout-worker] Hazır')
}

/** Fallback: load fonts from static /fonts/ directory (backwards compat) */
async function loadStaticFallback() {
  const STATIC_FONTS = [
    '/fonts/NotoSans-Regular.ttf',
    '/fonts/NotoSans-Bold.ttf',
    '/fonts/NotoSans-Italic.ttf',
    '/fonts/NotoSans-BoldItalic.ttf',
    '/fonts/NotoSansMono-Regular.ttf',
  ]

  const buffers: Uint8Array[] = []
  await Promise.all(
    STATIC_FONTS.map(async (path) => {
      const url = new URL(path, self.location.origin).href
      const res = await fetch(url)
      if (res.ok) {
        buffers.push(new Uint8Array(await res.arrayBuffer()))
      }
    })
  )

  if (buffers.length > 0) {
    loadFonts(buffers)
    loadedFamilies.add('noto sans')
    loadedFamilies.add('noto sans mono')
    console.log(`[layout-worker] Static fallback: ${buffers.length} font yüklendi`)
  }
}

/** Load all variants of given families from the API into WASM */
async function ensureFamiliesLoaded(families: string[]): Promise<void> {
  const toLoad = families.filter(f => !loadedFamilies.has(f.toLowerCase()))
  if (toLoad.length === 0) return

  const buffers: Uint8Array[] = []

  for (const family of toLoad) {
    const info = fontCatalog.find(f => f.family.toLowerCase() === family.toLowerCase())
    if (!info) {
      console.warn(`[layout-worker] Font ailesi bulunamadı: ${family}`)
      continue
    }

    const fetches = info.variants.map(async (v) => {
      const url = `${fontApiBase}/${encodeURIComponent(info.family)}/${v.weight}/${v.italic}`
      const res = await fetch(url)
      if (res.ok) {
        return new Uint8Array(await res.arrayBuffer())
      }
      return null
    })

    const results = await Promise.all(fetches)
    for (const buf of results) {
      if (buf && buf.byteLength > 0) {
        buffers.push(buf)
      }
    }
    loadedFamilies.add(family.toLowerCase())
  }

  if (buffers.length > 0) {
    if (loadedFamilies.size <= toLoad.length) {
      // First load — use loadFonts
      loadFonts(buffers)
    } else {
      // Subsequent loads — use addFonts
      addFonts(buffers)
    }
    console.log(`[layout-worker] ${toLoad.join(', ')} yüklendi (${buffers.length} variant)`)
  }
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
  | { type: 'configure'; fontApiBase?: string }

self.onmessage = async (e: MessageEvent<WorkerMessage>) => {
  const msg = e.data

  if (msg.type === 'configure') {
    if (msg.fontApiBase) {
      fontApiBase = msg.fontApiBase
    }
    return
  }

  if (msg.type === 'compile') {
    try {
      await ensureInit()

      // Extract font families from template and ensure they're loaded
      try {
        const tpl = JSON.parse(msg.templateJson)
        if (Array.isArray(tpl.fonts) && tpl.fonts.length > 0) {
          await ensureFamiliesLoaded(tpl.fonts)
        }
      } catch {
        // Template parse failure will be caught by computeLayout below
      }

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
