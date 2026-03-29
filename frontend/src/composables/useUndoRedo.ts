import { ref, watch, type Ref } from 'vue'

export function useUndoRedo<T>(source: Ref<T>, maxHistory = 50) {
  const undoStack = ref<string[]>([]) as Ref<string[]>
  const redoStack = ref<string[]>([]) as Ref<string[]>

  let skipWatch = false
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  // Başlangıç snapshot'ı
  undoStack.value.push(JSON.stringify(source.value))

  watch(
    source,
    () => {
      if (skipWatch) return

      // Debounce: hızlı ardışık değişiklikleri birleştir
      if (debounceTimer) clearTimeout(debounceTimer)
      debounceTimer = setTimeout(() => {
        const snap = JSON.stringify(source.value)
        const last = undoStack.value[undoStack.value.length - 1]
        if (snap === last) return

        undoStack.value.push(snap)
        if (undoStack.value.length > maxHistory) {
          undoStack.value.shift()
        }
        redoStack.value = []
      }, 300)
    },
    { deep: true }
  )

  function undo() {
    if (undoStack.value.length <= 1) return
    const current = undoStack.value.pop()!
    redoStack.value.push(current)
    const prev = undoStack.value[undoStack.value.length - 1]
    applySnapshot(prev)
  }

  function redo() {
    if (redoStack.value.length === 0) return
    const next = redoStack.value.pop()!
    undoStack.value.push(next)
    applySnapshot(next)
  }

  function applySnapshot(snap: string) {
    skipWatch = true
    Object.assign(source.value as object, JSON.parse(snap))
    skipWatch = false
  }

  const canUndo = () => undoStack.value.length > 1
  const canRedo = () => redoStack.value.length > 0

  return { undo, redo, canUndo, canRedo }
}
