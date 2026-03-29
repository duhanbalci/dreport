import { ref, watch, type Ref } from 'vue'
import type { ElementLayout } from '../core/template-to-typst'
import type { Template } from '../core/types'

export function useTypstCompiler(
  template: Ref<Template>,
  data: Ref<Record<string, unknown>>,
) {
  const svg = ref<string | null>(null)
  const error = ref<string | null>(null)
  const compiling = ref(false)
  const layout = ref<Record<string, ElementLayout>>({})

  let worker: Worker | null = null
  let requestId = 0
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  function initWorker() {
    worker = new Worker(new URL('../workers/typst.worker.ts', import.meta.url), {
      type: 'module',
    })

    worker.onmessage = (e: MessageEvent<{
      type: string
      svg?: string
      layout?: Record<string, ElementLayout>
      error?: string
      id: number
    }>) => {
      const data = e.data
      if (data.id !== requestId) return

      compiling.value = false
      if (data.type === 'result') {
        svg.value = data.svg ?? null
        layout.value = data.layout ?? {}
        error.value = null
      } else if (data.type === 'error') {
        error.value = data.error ?? 'Bilinmeyen derleme hatası'
      }
    }

    worker.onerror = () => {
      compiling.value = false
      error.value = 'Worker hatası — yeniden başlatılıyor'
      worker?.terminate()
      worker = null
      setTimeout(initWorker, 500)
    }
  }

  function compile() {
    if (!worker) initWorker()
    requestId++
    compiling.value = true
    worker!.postMessage({
      type: 'compile',
      templateJson: JSON.stringify(template.value),
      dataJson: JSON.stringify(data.value),
      id: requestId,
    })
  }

  // template veya data değiştiğinde yeniden derle
  watch(
    [template, data],
    () => {
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(() => {
        compile()
      }, 200)
    },
    { immediate: true, deep: true }
  )

  function dispose() {
    worker?.terminate()
    worker = null
    if (debounceTimer) clearTimeout(debounceTimer)
  }

  return {
    svg,
    error,
    compiling,
    layout,
    compile,
    dispose,
  }
}
