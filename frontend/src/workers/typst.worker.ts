/// Typst WASM Web Worker
/// Template JSON + Data JSON → (dreport-core WASM ile) Typst markup → (typst.ts WASM ile) SVG

import { $typst, TypstSnippet } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs'
import initCore, { templateToTypstEditor } from '../core/wasm/dreport_core.js'

let typstInitialized = false
let coreInitialized = false

const FONT_FILES = [
  '/fonts/NotoSans-Regular.ttf',
  '/fonts/NotoSans-Bold.ttf',
  '/fonts/NotoSans-Italic.ttf',
  '/fonts/NotoSans-BoldItalic.ttf',
  '/fonts/NotoSansMono-Regular.ttf',
]

async function ensureInit() {
  if (!coreInitialized) {
    console.log('[typst-worker] dreport-core WASM başlatılıyor...')
    await initCore({ module_or_path: '/wasm/dreport_core_bg.wasm' })
    coreInitialized = true
    console.log('[typst-worker] dreport-core WASM hazır')
  }

  if (!typstInitialized) {
    console.log('[typst-worker] Typst WASM başlatılıyor...')

    const fontUrls = FONT_FILES.map(f => new URL(f, self.location.origin).href)
    $typst.use(TypstSnippet.preloadFonts(fontUrls))
    $typst.use(TypstSnippet.fetchPackageRegistry())

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

    typstInitialized = true
    console.log('[typst-worker] Typst WASM hazır')
  }
}

interface CompileMessage {
  type: 'compile'
  templateJson: string
  dataJson: string
  id: number
}

// Geriye uyumluluk için eski markup tabanlı mesaj desteği
interface LegacyCompileMessage {
  type: 'compile'
  markup: string
  id: number
}

type WorkerMessage = CompileMessage | LegacyCompileMessage

self.onmessage = async (e: MessageEvent<WorkerMessage>) => {
  const { type, id } = e.data

  if (type === 'compile') {
    console.log(`[typst-worker] Derleme başladı (id: ${id})`)
    try {
      await ensureInit()

      let markup: string

      if ('templateJson' in e.data) {
        // Yeni yol: Template JSON → Typst markup (dreport-core WASM)
        markup = templateToTypstEditor(e.data.templateJson, e.data.dataJson)
        console.log('[typst-worker] Generated Typst markup:\n', markup)
      } else {
        // Eski yol: doğrudan markup (geriye uyumluluk)
        markup = (e.data as LegacyCompileMessage).markup
      }

      // Typst markup → SVG
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
