import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TemplateElement } from '../core/types'

export const useEditorStore = defineStore('editor', () => {
  /** Seçili eleman ID'leri — çoklu seçim desteği */
  const selectedElementIds = ref<Set<string>>(new Set())
  const zoom = ref(1)
  const panX = ref(0)
  const panY = ref(0)
  const isDragging = ref(false)

  // Toolbox'tan sürüklenen eleman (henüz eklenmedi)
  const draggedNewElement = ref<TemplateElement | null>(null)
  const dropTargetContainerId = ref<string | null>(null)

  const zoomPercent = computed(() => Math.round(zoom.value * 100))

  /** Geriye uyumluluk: tek seçili eleman ID'si (ilk seçili veya null) */
  const selectedElementId = computed<string | null>(() => {
    const ids = selectedElementIds.value
    if (ids.size === 0) return null
    return ids.values().next().value ?? null
  })

  /** Tek eleman seç (önceki seçimi temizler) */
  function selectElement(id: string | null) {
    selectedElementIds.value = id ? new Set([id]) : new Set()
  }

  /** Shift+click: seçime ekle/çıkar (toggle) */
  function toggleSelection(id: string) {
    const next = new Set(selectedElementIds.value)
    if (next.has(id)) {
      next.delete(id)
    } else {
      next.add(id)
    }
    selectedElementIds.value = next
  }

  /** Eleman seçili mi? */
  function isSelected(id: string): boolean {
    return selectedElementIds.value.has(id)
  }

  function clearSelection() {
    selectedElementIds.value = new Set()
  }

  function setZoom(value: number) {
    zoom.value = Math.max(0.25, Math.min(4, value))
  }

  function setPan(x: number, y: number) {
    panX.value = x
    panY.value = y
  }

  function resetPan() {
    panX.value = 0
    panY.value = 0
  }

  function setDragging(value: boolean) {
    isDragging.value = value
  }

  // Toolbox drag
  function startDragNewElement(el: TemplateElement) {
    draggedNewElement.value = el
  }

  function setDropTargetContainer(id: string | null) {
    dropTargetContainerId.value = id
  }

  function endDragNewElement() {
    draggedNewElement.value = null
    dropTargetContainerId.value = null
  }

  return {
    selectedElementIds,
    selectedElementId,
    zoom,
    panX,
    panY,
    isDragging,
    draggedNewElement,
    dropTargetContainerId,
    zoomPercent,
    selectElement,
    toggleSelection,
    isSelected,
    clearSelection,
    setZoom,
    setPan,
    resetPan,
    setDragging,
    startDragNewElement,
    setDropTargetContainer,
    endDragNewElement,
  }
})
