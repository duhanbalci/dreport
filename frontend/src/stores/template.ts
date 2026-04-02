import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Template, TemplateElement, ContainerElement, SizeConstraint, PositionMode } from '../core/types'
import { findElementById, findParent, isContainer, sz } from '../core/types'
import { generateMockData } from '../core/mock-data-generator'
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

  /** Dışarıdan verilen önizleme verisi (null ise mock data üretilir) */
  const overrideData = ref<Record<string, unknown> | null>(null)

  const mockData = computed(() => overrideData.value ?? generateMockData(template.value))

  /**
   * Layout version counter — her template/data mutasyonunda artar.
   * useLayoutEngine bu counter'ı izler (deep watch yerine).
   * Vue'nun tüm template ağacını recursive karşılaştırması yerine
   * tek bir sayı karşılaştırması yapılır.
   */
  const layoutVersion = ref(0)

  /** Layout yeniden hesaplamasını tetikle */
  function bumpLayoutVersion() {
    layoutVersion.value++
  }

  function setOverrideData(data: Record<string, unknown> | null) {
    overrideData.value = data
    bumpLayoutVersion()
  }

  // Undo / Redo
  const { undo: _undo, redo: _redo, canUndo, canRedo } = useUndoRedo(template)

  function undo() {
    _undo()
    bumpLayoutVersion()
  }

  function redo() {
    _redo()
    bumpLayoutVersion()
  }

  // --- Element CRUD ---

  function getElementById(id: string): TemplateElement | undefined {
    const inRoot = findElementById(template.value.root, id)
    if (inRoot) return inRoot
    if (template.value.header) {
      const inHeader = findElementById(template.value.header, id)
      if (inHeader) return inHeader
    }
    if (template.value.footer) {
      const inFooter = findElementById(template.value.footer, id)
      if (inFooter) return inFooter
    }
    return undefined
  }

  function getParent(id: string): ContainerElement | undefined {
    const inRoot = findParent(template.value.root, id)
    if (inRoot) return inRoot
    if (template.value.header) {
      // Check if the header itself is the target element's parent
      if (template.value.header.id === id) return undefined
      const inHeader = findParent(template.value.header, id)
      if (inHeader) return inHeader
    }
    if (template.value.footer) {
      if (template.value.footer.id === id) return undefined
      const inFooter = findParent(template.value.footer, id)
      if (inFooter) return inFooter
    }
    return undefined
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
    bumpLayoutVersion()
  }

  /** Element'i ağaçtan kaldır */
  function removeElement(elementId: string) {
    const parent = getParent(elementId)
    if (!parent) return
    const idx = parent.children.findIndex(c => c.id === elementId)
    if (idx !== -1) {
      parent.children.splice(idx, 1)
      bumpLayoutVersion()
    }
  }

  /** Element'i başka bir container'a taşı */
  function moveElement(elementId: string, targetParentId: string, index?: number) {
    const el = getElementById(elementId)
    if (!el) return
    // removeElement bump'lar, addChild de bump'lar — ama tek mantıksal operasyon.
    // Fazladan 1 bump sorun değil (debounce var), ama istersek optimize edebiliriz.
    removeElement(elementId)
    addChild(targetParentId, el, index)
  }

  /** Absolute pozisyon güncelle */
  function updateElementPosition(elementId: string, position: PositionMode) {
    const el = getElementById(elementId)
    if (el) {
      el.position = position
      bumpLayoutVersion()
    }
  }

  /** Boyut güncelle */
  function updateElementSize(elementId: string, size: Partial<SizeConstraint>) {
    const el = getElementById(elementId)
    if (el) {
      el.size = { ...el.size, ...size }
      bumpLayoutVersion()
    }
  }

  /** Herhangi bir element özelliğini güncelle */
  function updateElement(elementId: string, updates: Partial<TemplateElement>) {
    const el = getElementById(elementId)
    if (el) {
      Object.assign(el, updates)
      bumpLayoutVersion()
    }
  }

  /** Çocuk sırasını değiştir (aynı parent içinde) */
  function reorderChild(parentId: string, fromIndex: number, toIndex: number) {
    const parent = getElementById(parentId)
    if (!parent || !isContainer(parent)) return
    const [moved] = parent.children.splice(fromIndex, 1)
    parent.children.splice(toIndex, 0, moved)
    bumpLayoutVersion()
  }

  /** Şablonu JSON olarak dışa aktar */
  function exportTemplate(): string {
    return JSON.stringify(template.value, null, 2)
  }

  /** JSON'dan şablon yükle */
  function importTemplate(json: string) {
    const parsed = JSON.parse(json) as Template
    template.value = parsed
    bumpLayoutVersion()
  }

  /** Yeni boş şablon oluştur */
  function resetTemplate() {
    template.value = createDefaultTemplate()
    bumpLayoutVersion()
  }

  /** Header container'ı etkinleştir */
  function enableHeader() {
    if (template.value.header) return
    template.value.header = {
      id: 'header',
      type: 'container',
      position: { type: 'flow' },
      size: { width: sz.auto(), height: sz.fixed(10), minHeight: 10 },
      direction: 'row',
      gap: 0,
      padding: { top: 2, right: 5, bottom: 2, left: 5 },
      align: 'stretch',
      justify: 'start',
      style: {},
      children: [],
    }
    bumpLayoutVersion()
  }

  /** Header container'ı kaldır */
  function disableHeader() {
    if (!template.value.header) return
    template.value.header = undefined
    bumpLayoutVersion()
  }

  /** Footer container'ı etkinleştir */
  function enableFooter() {
    if (template.value.footer) return
    template.value.footer = {
      id: 'footer',
      type: 'container',
      position: { type: 'flow' },
      size: { width: sz.auto(), height: sz.fixed(10), minHeight: 10 },
      direction: 'row',
      gap: 0,
      padding: { top: 2, right: 5, bottom: 2, left: 5 },
      align: 'stretch',
      justify: 'start',
      style: {},
      children: [],
    }
    bumpLayoutVersion()
  }

  /** Footer container'ı kaldır */
  function disableFooter() {
    if (!template.value.footer) return
    template.value.footer = undefined
    bumpLayoutVersion()
  }

  return {
    template,
    mockData,
    layoutVersion,
    bumpLayoutVersion,
    getElementById,
    getParent,
    addChild,
    removeElement,
    moveElement,
    updateElementPosition,
    updateElementSize,
    updateElement,
    reorderChild,
    exportTemplate,
    importTemplate,
    resetTemplate,
    setOverrideData,
    undo,
    redo,
    canUndo,
    canRedo,
    enableHeader,
    disableHeader,
    enableFooter,
    disableFooter,
  }
})
