/// Typst WASM Web Worker
/// Ana thread'i bloklamadan Typst markup → SVG derleme yapar.

import { $typst, TypstSnippet } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs'

let initialized = false

const FONT_FILES = [
  '/fonts/NotoSans-Regular.ttf',
  '/fonts/NotoSans-Bold.ttf',
  '/fonts/NotoSans-Italic.ttf',
  '/fonts/NotoSans-BoldItalic.ttf',
  '/fonts/NotoSansMono-Regular.ttf',
]

async function ensureInit() {
  if (initialized) return

  console.log('[typst-worker] Başlatılıyor...')

  try {
    // Fontları URL olarak preload et (init öncesinde)
    const fontUrls = FONT_FILES.map(f => new URL(f, self.location.origin).href)
    $typst.use(TypstSnippet.preloadFonts(fontUrls))

    await $typst.setCompilerInitOptions({
      getModule: () =>
        fetch('/wasm/typst_ts_web_compiler_bg.wasm').then(r => {
          console.log('[typst-worker] Compiler WASM yüklendi:', r.status)
          return r.arrayBuffer()
        }),
    })
    await $typst.setRendererInitOptions({
      getModule: () =>
        fetch('/wasm/typst_ts_renderer_bg.wasm').then(r => {
          console.log('[typst-worker] Renderer WASM yüklendi:', r.status)
          return r.arrayBuffer()
        }),
    })

    initialized = true
    console.log('[typst-worker] Başlatma tamamlandı')
  } catch (initErr) {
    console.error('[typst-worker] Başlatma hatası:', initErr)
    throw initErr
  }
}

self.onmessage = async (e: MessageEvent<{ type: string; markup: string; id: number }>) => {
  const { type, markup, id } = e.data

  if (type === 'compile') {
    console.log(`[typst-worker] Derleme başladı (id: ${id})`)
    try {
      await ensureInit()
      const svg = await $typst.svg({ mainContent: markup })

      // SVG'den layout bilgisini parse et
      const layout: Record<string, { x: number; y: number; width: number; height: number }> = {}
      const matches = svg.matchAll(/([a-zA-Z0-9_-]+):([\d.]+)pt,([\d.]+)pt,([\d.]+)pt,([\d.]+)pt\|/g)
      for (const m of matches) {
        layout[m[1]] = {
          x: parseFloat(m[2]),
          y: parseFloat(m[3]),
          width: parseFloat(m[4]),
          height: parseFloat(m[5]),
        }
      }

      console.log(`[typst-worker] Derleme başarılı (id: ${id}, elements: ${Object.keys(layout).length})`)
      self.postMessage({ type: 'result', svg, layout, id })
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      console.error(`[typst-worker] Derleme hatası (id: ${id}):`, err)
      self.postMessage({
        type: 'error',
        error: errorMsg,
        id,
      })
    }
  }
}
