import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Template, TemplateElement, ContainerElement, SizeConstraint, PositionMode } from '../core/types'
import { findElementById, findParent, isContainer, sz } from '../core/types'
import { templateToTypst } from '../core/template-to-typst'
import { useUndoRedo } from '../composables/useUndoRedo'

function createDefaultTemplate(): Template {
  return {
    id: 'tpl_default',
    name: 'Yeni Şablon',
    page: { width: 210, height: 297 },
    fonts: ['Noto Sans'],
    root: {
      id: 'root',
      type: 'container',
      position: { type: 'flow' },
      size: { width: sz.auto(), height: sz.auto() },
      direction: 'column',
      gap: 5,
      padding: { top: 15, right: 15, bottom: 15, left: 15 },
      align: 'stretch',
      justify: 'start',
      style: {},
      children: [
        {
          id: 'el_001',
          type: 'static_text',
          position: { type: 'flow' },
          size: { width: sz.auto(), height: sz.auto() },
          style: { fontSize: 18, fontWeight: 'bold', color: '#1a1a1a' },
          content: 'dreport',
        },
        {
          id: 'el_002',
          type: 'static_text',
          position: { type: 'flow' },
          size: { width: sz.auto(), height: sz.auto() },
          style: { fontSize: 11, color: '#666666' },
          content: 'Belge tasarım aracı — sürükle ve bırak',
        },
      ],
    },
  }
}

export const useTemplateStore = defineStore('template', () => {
  const template = ref<Template>(createDefaultTemplate())

  const typstMarkup = computed(() => templateToTypst(template.value))

  // Undo / Redo
  const { undo, redo, canUndo, canRedo } = useUndoRedo(template)

  // --- Element CRUD ---

  function getElementById(id: string): TemplateElement | undefined {
    return findElementById(template.value.root, id)
  }

  function getParent(id: string): ContainerElement | undefined {
    return findParent(template.value.root, id)
  }

  /** Bir container'a çocuk ekle */
  function addChild(parentId: string, element: TemplateElement, index?: number) {
    const parent = getElementById(parentId)
    if (!parent || !isContainer(parent)) return
    if (index !== undefined) {
      parent.children.splice(index, 0, element)
    } else {
      parent.children.push(element)
    }
  }

  /** Element'i ağaçtan kaldır */
  function removeElement(elementId: string) {
    const parent = getParent(elementId)
    if (!parent) return
    const idx = parent.children.findIndex(c => c.id === elementId)
    if (idx !== -1) parent.children.splice(idx, 1)
  }

  /** Element'i başka bir container'a taşı */
  function moveElement(elementId: string, targetParentId: string, index?: number) {
    const el = getElementById(elementId)
    if (!el) return
    removeElement(elementId)
    addChild(targetParentId, el, index)
  }

  /** Absolute pozisyon güncelle */
  function updateElementPosition(elementId: string, position: PositionMode) {
    const el = getElementById(elementId)
    if (el) el.position = position
  }

  /** Boyut güncelle */
  function updateElementSize(elementId: string, size: Partial<SizeConstraint>) {
    const el = getElementById(elementId)
    if (el) {
      el.size = { ...el.size, ...size }
    }
  }

  /** Herhangi bir element özelliğini güncelle */
  function updateElement(elementId: string, updates: Partial<TemplateElement>) {
    const el = getElementById(elementId)
    if (el) Object.assign(el, updates)
  }

  /** Çocuk sırasını değiştir (aynı parent içinde) */
  function reorderChild(parentId: string, fromIndex: number, toIndex: number) {
    const parent = getElementById(parentId)
    if (!parent || !isContainer(parent)) return
    const [moved] = parent.children.splice(fromIndex, 1)
    parent.children.splice(toIndex, 0, moved)
  }

  return {
    template,
    typstMarkup,
    getElementById,
    getParent,
    addChild,
    removeElement,
    moveElement,
    updateElementPosition,
    updateElementSize,
    updateElement,
    reorderChild,
    undo,
    redo,
    canUndo,
    canRedo,
  }
})
