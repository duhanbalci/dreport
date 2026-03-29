import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useEditorStore } from '../editor'
import type { StaticTextElement } from '../../core/types'
import { sz } from '../../core/types'

describe('useEditorStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('selectElement sets selectedElementId', () => {
    const store = useEditorStore()

    store.selectElement('el_123')
    expect(store.selectedElementId).toBe('el_123')
  })

  it('clearSelection resets to null', () => {
    const store = useEditorStore()
    store.selectElement('el_123')

    store.clearSelection()
    expect(store.selectedElementId).toBeNull()
  })

  it('setZoom clamps between 0.25 and 4', () => {
    const store = useEditorStore()

    store.setZoom(2)
    expect(store.zoom).toBe(2)

    store.setZoom(0.1)
    expect(store.zoom).toBe(0.25)

    store.setZoom(10)
    expect(store.zoom).toBe(4)

    store.setZoom(0.25)
    expect(store.zoom).toBe(0.25)

    store.setZoom(4)
    expect(store.zoom).toBe(4)
  })

  it('zoomPercent reflects zoom value', () => {
    const store = useEditorStore()

    store.setZoom(1.5)
    expect(store.zoomPercent).toBe(150)

    store.setZoom(0.5)
    expect(store.zoomPercent).toBe(50)
  })

  it('startDragNewElement / endDragNewElement manage drag state', () => {
    const store = useEditorStore()
    const el: StaticTextElement = {
      id: 'new_el',
      type: 'static_text',
      position: { type: 'flow' },
      size: { width: sz.auto(), height: sz.auto() },
      style: {},
      content: 'Drag me',
    }

    expect(store.draggedNewElement).toBeNull()

    store.startDragNewElement(el)
    expect(store.draggedNewElement).toBeDefined()
    expect(store.draggedNewElement!.id).toBe('new_el')

    store.endDragNewElement()
    expect(store.draggedNewElement).toBeNull()
    expect(store.dropTargetContainerId).toBeNull()
  })

  it('setDropTargetContainer sets drop target ID', () => {
    const store = useEditorStore()

    store.setDropTargetContainer('container_1')
    expect(store.dropTargetContainerId).toBe('container_1')

    store.setDropTargetContainer(null)
    expect(store.dropTargetContainerId).toBeNull()
  })

  it('setPan / resetPan manage pan values', () => {
    const store = useEditorStore()

    store.setPan(100, 200)
    expect(store.panX).toBe(100)
    expect(store.panY).toBe(200)

    store.resetPan()
    expect(store.panX).toBe(0)
    expect(store.panY).toBe(0)
  })

  it('setDragging manages isDragging flag', () => {
    const store = useEditorStore()

    expect(store.isDragging).toBe(false)
    store.setDragging(true)
    expect(store.isDragging).toBe(true)
    store.setDragging(false)
    expect(store.isDragging).toBe(false)
  })
})
