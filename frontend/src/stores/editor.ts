import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TemplateElement } from '../core/types'

export const useEditorStore = defineStore('editor', () => {
  const selectedElementId = ref<string | null>(null)
  const zoom = ref(1)
  const panX = ref(0)
  const panY = ref(0)
  const isDragging = ref(false)

  // Toolbox'tan sürüklenen eleman (henüz eklenmedi)
  const draggedNewElement = ref<TemplateElement | null>(null)
  const dropTargetContainerId = ref<string | null>(null)

  const zoomPercent = computed(() => Math.round(zoom.value * 100))

  function selectElement(id: string | null) {
    selectedElementId.value = id
  }

  function clearSelection() {
    selectedElementId.value = null
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
    selectedElementId,
    zoom,
    panX,
    panY,
    isDragging,
    draggedNewElement,
    dropTargetContainerId,
    zoomPercent,
    selectElement,
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
