import { ref, watch, type Ref } from 'vue'
import type { Template } from '../core/types'
import type { LayoutResult, ElementLayout } from '../core/layout-types'

export type { ElementLayout }

export function useLayoutEngine(
  template: Ref<Template>,
  data: Ref<Record<string, unknown>>,
  layoutVersion?: Ref<number>,
) {
  const layout = ref<LayoutResult | null>(null)
  const error = ref<string | null>(null)
  const computing = ref(false)

  // Uyumluluk: InteractionOverlay'ın beklediği flat layout map (id → ElementLayout)
  const layoutMap = ref<Record<string, ElementLayout>>({})

  let worker: Worker | null = null
  let requestId = 0

  function initWorker() {
    worker = new Worker(new URL('../workers/layout.worker.ts', import.meta.url), {
      type: 'module',
    })

    worker.onmessage = (e: MessageEvent<any>) => {
      const msg = e.data

      // Barcode yanıtları
      if (msg.type === 'barcode-result' || msg.type === 'barcode-error') {
        handleBarcodeResponse(msg)
        return
      }

      if (msg.id !== requestId) return

      computing.value = false
      if (msg.type === 'result' && msg.layout) {
        layout.value = msg.layout
        error.value = null

        // Flat map oluştur: id → ElementLayout
        const map: Record<string, ElementLayout> = {}
        for (const page of msg.layout.pages) {
          for (const el of page.elements) {
            map[el.id] = el
          }
        }
        layoutMap.value = map
      } else if (msg.type === 'error') {
        error.value = msg.error ?? 'Bilinmeyen layout hatası'
      }
    }

    worker.onerror = () => {
      computing.value = false
      error.value = 'Worker hatası — yeniden başlatılıyor'
      worker?.terminate()
      worker = null
      setTimeout(initWorker, 500)
    }
  }

  function compute() {
    if (!worker) initWorker()
    requestId++
    computing.value = true
    worker!.postMessage({
      type: 'compile',
      templateJson: JSON.stringify(template.value),
      dataJson: JSON.stringify(data.value),
      id: requestId,
    })
  }

  // template veya data değiştiğinde yeniden hesapla.
  // layoutVersion verilmişse sadece onu izle (cheap integer comparison).
  // Verilmemişse eski davranış: deep watch (geriye uyumluluk).
  if (layoutVersion) {
    watch(
      layoutVersion,
      () => {
        compute()
      },
      { immediate: true },
    )
  } else {
    watch(
      [template, data],
      () => {
        compute()
      },
      { immediate: true, deep: true },
    )
  }

  // --- Barcode üretimi (WASM üzerinden) ---
  let barcodeReqId = 0
  const barcodeCallbacks = new Map<number, (result: { width: number; height: number; rgba: ArrayBuffer } | null) => void>()

  function generateBarcode(format: string, value: string, width: number, height: number, includeText: boolean = false): Promise<{ width: number; height: number; rgba: ArrayBuffer } | null> {
    if (!worker) initWorker()
    return new Promise(resolve => {
      barcodeReqId++
      const id = barcodeReqId + 100000 // compile id'leriyle çakışmasın
      barcodeCallbacks.set(id, resolve)
      worker!.postMessage({ type: 'barcode', format, value, width, height, includeText, id })
    })
  }

  function handleBarcodeResponse(msg: any) {
    if (msg.type === 'barcode-result' || msg.type === 'barcode-error') {
      const cb = barcodeCallbacks.get(msg.id)
      if (cb) {
        barcodeCallbacks.delete(msg.id)
        cb(msg.type === 'barcode-result' ? { width: msg.width, height: msg.height, rgba: msg.rgba } : null)
      }
    }
  }

  function dispose() {
    worker?.terminate()
    worker = null
    barcodeCallbacks.clear()
  }

  return {
    layout,
    layoutMap,
    error,
    computing,
    compute,
    generateBarcode,
    dispose,
  }
}
