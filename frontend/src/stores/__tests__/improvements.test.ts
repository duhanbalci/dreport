/**
 * IMPROVEMENTS.md bölüm 1, 2, 3 implementasyonlarının testleri.
 *
 * Bölüm 1: Kritik Buglar (1.1–1.4)
 * Bölüm 2: Önemli Teknik Sorunlar (2.9, 2.11)
 * Bölüm 3: Eksik Özellikler (3.1, 3.2)
 *
 * Not: Rust tarafı testleri layout-engine/tests/improvements_test.rs dosyasındadır.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useTemplateStore } from '../template'
import { useEditorStore } from '../editor'
import type { Template, StaticTextElement, ContainerElement, ImageElement, TemplateElement } from '../../core/types'
import { sz } from '../../core/types'

function createTestTemplate(): Template {
  return {
    id: 'test',
    name: 'Test',
    page: { width: 210, height: 297 },
    fonts: ['Noto Sans'],
    root: {
      id: 'root',
      type: 'container' as const,
      position: { type: 'flow' as const },
      size: { width: sz.auto(), height: sz.auto() },
      direction: 'column' as const,
      gap: 5,
      padding: { top: 10, right: 10, bottom: 10, left: 10 },
      align: 'stretch' as const,
      justify: 'start' as const,
      style: {},
      children: [],
    },
  }
}

function createTextElement(id: string, content: string): StaticTextElement {
  return {
    id,
    type: 'static_text',
    position: { type: 'flow' },
    size: { width: sz.auto(), height: sz.auto() },
    style: { fontSize: 12 },
    content,
  }
}

// =============================================================================
// 1.1 Undo/Redo — Object.assign yerine reference replacement
// =============================================================================

describe('1.1 Undo/Redo reference replacement', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('undo properly removes keys that were added after snapshot', async () => {
    vi.useFakeTimers()
    const store = useTemplateStore()
    store.template = createTestTemplate()

    // Snapshot al (debounce beklenmeli)
    await vi.advanceTimersByTimeAsync(400)

    // Header ekle
    store.enableHeader()
    expect(store.template.header).toBeDefined()

    // Snapshot al
    await vi.advanceTimersByTimeAsync(400)

    // Undo: header eklenmeden önceki state'e dön
    store.undo()
    expect(store.template.header).toBeUndefined()

    vi.useRealTimers()
  })

  it('undo properly removes footer key', async () => {
    vi.useFakeTimers()
    const store = useTemplateStore()
    store.template = createTestTemplate()

    await vi.advanceTimersByTimeAsync(400)

    store.enableFooter()
    expect(store.template.footer).toBeDefined()
    await vi.advanceTimersByTimeAsync(400)

    store.undo()
    expect(store.template.footer).toBeUndefined()

    vi.useRealTimers()
  })

  it('redo restores the removed key after undo', async () => {
    vi.useFakeTimers()
    const store = useTemplateStore()
    store.template = createTestTemplate()

    await vi.advanceTimersByTimeAsync(400)

    store.enableHeader()
    await vi.advanceTimersByTimeAsync(400)

    store.undo()
    expect(store.template.header).toBeUndefined()

    store.redo()
    expect(store.template.header).toBeDefined()
    expect(store.template.header!.id).toBe('header')

    vi.useRealTimers()
  })
})

// =============================================================================
// 1.3 Image objectFit — LayoutRenderer'da style.objectFit okunmalı
// (Birim test olarak ImageElement tipi üzerinden doğrulanır)
// =============================================================================

describe('1.3 Image objectFit', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('ImageElement stores objectFit in style', () => {
    const store = useTemplateStore()
    store.template = createTestTemplate()

    const img: ImageElement = {
      id: 'img_1',
      type: 'image',
      position: { type: 'flow' },
      size: { width: sz.fixed(50), height: sz.fixed(30) },
      src: 'data:image/png;base64,abc',
      style: { objectFit: 'contain' },
    }

    store.addChild('root', img as unknown as TemplateElement)

    const el = store.getElementById('img_1') as ImageElement
    expect(el.style.objectFit).toBe('contain')
  })

  it('updateElement changes objectFit', () => {
    const store = useTemplateStore()
    store.template = createTestTemplate()

    const img: ImageElement = {
      id: 'img_2',
      type: 'image',
      position: { type: 'flow' },
      size: { width: sz.fixed(50), height: sz.fixed(30) },
      src: 'data:image/png;base64,abc',
      style: { objectFit: 'contain' },
    }

    store.addChild('root', img as unknown as TemplateElement)
    store.updateElement('img_2', { style: { objectFit: 'cover' } } as Partial<TemplateElement>)

    const el = store.getElementById('img_2') as ImageElement
    expect(el.style.objectFit).toBe('cover')
  })
})

// =============================================================================
// 2.9 importTemplate validasyon
// =============================================================================

describe('2.9 importTemplate validation', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('throws on invalid JSON', () => {
    const store = useTemplateStore()
    expect(() => store.importTemplate('not json')).toThrow('Geçersiz JSON')
  })

  it('throws on missing root', () => {
    const store = useTemplateStore()
    const bad = JSON.stringify({ page: { width: 210, height: 297 } })
    expect(() => store.importTemplate(bad)).toThrow('root')
  })

  it('throws on root that is not container', () => {
    const store = useTemplateStore()
    const bad = JSON.stringify({
      root: { type: 'text', id: 'r' },
      page: { width: 210, height: 297 },
    })
    expect(() => store.importTemplate(bad)).toThrow('container')
  })

  it('throws on missing page', () => {
    const store = useTemplateStore()
    const bad = JSON.stringify({
      root: { type: 'container', id: 'root', children: [] },
    })
    expect(() => store.importTemplate(bad)).toThrow('page')
  })

  it('throws on invalid page dimensions', () => {
    const store = useTemplateStore()
    const bad = JSON.stringify({
      root: { type: 'container', id: 'root', children: [] },
      page: { width: 'abc', height: 297 },
    })
    expect(() => store.importTemplate(bad)).toThrow('page')
  })

  it('preserves previous state on failed import', () => {
    const store = useTemplateStore()
    store.template = createTestTemplate()
    store.addChild('root', createTextElement('keep_me', 'Keep'))

    try {
      store.importTemplate('invalid json')
    } catch {
      // beklenen
    }

    // Önceki state korunmuş olmalı
    expect(store.getElementById('keep_me')).toBeDefined()
  })

  it('accepts valid template JSON', () => {
    const store = useTemplateStore()
    const tpl = createTestTemplate()
    tpl.name = 'Valid Import'
    const json = JSON.stringify(tpl)

    store.importTemplate(json)
    expect(store.template.name).toBe('Valid Import')
  })
})

// =============================================================================
// 2.11 moveElement — tek layoutVersion bump
// =============================================================================

describe('2.11 moveElement single layoutVersion bump', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('moveElement increments layoutVersion exactly once', () => {
    const store = useTemplateStore()
    store.template = createTestTemplate()

    // İç içe container yapısı oluştur
    const child: ContainerElement = {
      id: 'child_container',
      type: 'container',
      position: { type: 'flow' },
      size: { width: sz.auto(), height: sz.auto() },
      direction: 'column',
      gap: 0,
      padding: { top: 0, right: 0, bottom: 0, left: 0 },
      align: 'stretch',
      justify: 'start',
      style: {},
      children: [],
    }
    store.addChild('root', child as unknown as TemplateElement)
    store.addChild('root', createTextElement('el_move', 'Move me'))

    const versionBefore = store.layoutVersion

    store.moveElement('el_move', 'child_container')

    // Tek bump: tam olarak 1 artmalı
    expect(store.layoutVersion).toBe(versionBefore + 1)

    // Eleman taşınmış olmalı
    const moved = store.getElementById('el_move')
    expect(moved).toBeDefined()
    const parent = store.getParent('el_move')
    expect(parent?.id).toBe('child_container')
  })
})

// =============================================================================
// 3.1 Çoklu Seçim (Multi-Selection)
// =============================================================================

describe('3.1 Multi-Selection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('selectedElementIds starts empty', () => {
    const store = useEditorStore()
    expect(store.selectedElementIds.size).toBe(0)
    expect(store.selectedElementId).toBeNull()
  })

  it('selectElement sets single selection', () => {
    const store = useEditorStore()
    store.selectElement('el_1')
    expect(store.selectedElementIds.size).toBe(1)
    expect(store.selectedElementId).toBe('el_1')
  })

  it('selectElement clears previous selection', () => {
    const store = useEditorStore()
    store.selectElement('el_1')
    store.selectElement('el_2')
    expect(store.selectedElementIds.size).toBe(1)
    expect(store.selectedElementId).toBe('el_2')
    expect(store.isSelected('el_1')).toBe(false)
  })

  it('toggleSelection adds to selection', () => {
    const store = useEditorStore()
    store.selectElement('el_1')
    store.toggleSelection('el_2')
    expect(store.selectedElementIds.size).toBe(2)
    expect(store.isSelected('el_1')).toBe(true)
    expect(store.isSelected('el_2')).toBe(true)
  })

  it('toggleSelection removes from selection', () => {
    const store = useEditorStore()
    store.selectElement('el_1')
    store.toggleSelection('el_2')
    store.toggleSelection('el_1')
    expect(store.selectedElementIds.size).toBe(1)
    expect(store.isSelected('el_1')).toBe(false)
    expect(store.isSelected('el_2')).toBe(true)
  })

  it('clearSelection clears all', () => {
    const store = useEditorStore()
    store.selectElement('el_1')
    store.toggleSelection('el_2')
    store.toggleSelection('el_3')
    expect(store.selectedElementIds.size).toBe(3)

    store.clearSelection()
    expect(store.selectedElementIds.size).toBe(0)
    expect(store.selectedElementId).toBeNull()
  })

  it('isSelected returns correct state', () => {
    const store = useEditorStore()
    expect(store.isSelected('el_1')).toBe(false)
    store.selectElement('el_1')
    expect(store.isSelected('el_1')).toBe(true)
    expect(store.isSelected('el_2')).toBe(false)
  })

  it('selectedElementId returns first selected (backward compat)', () => {
    const store = useEditorStore()
    store.selectElement('el_1')
    store.toggleSelection('el_2')
    // İlk eklenen eleman
    expect(store.selectedElementId).toBe('el_1')
  })

  it('selectElement(null) clears selection', () => {
    const store = useEditorStore()
    store.selectElement('el_1')
    store.selectElement(null)
    expect(store.selectedElementIds.size).toBe(0)
  })
})

// =============================================================================
// 3.2 Z-Order Kontrolleri
// =============================================================================

describe('3.2 Z-Order controls', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  function setupThreeElements() {
    const store = useTemplateStore()
    store.template = createTestTemplate()
    store.addChild('root', createTextElement('a', 'A'))
    store.addChild('root', createTextElement('b', 'B'))
    store.addChild('root', createTextElement('c', 'C'))
    return store
  }

  it('bringForward moves element one step up', () => {
    const store = setupThreeElements()
    // Sıra: [a, b, c] → bringForward(a) → [b, a, c]
    store.bringForward('a')
    expect(store.template.root.children.map(c => c.id)).toEqual(['b', 'a', 'c'])
  })

  it('sendBackward moves element one step down', () => {
    const store = setupThreeElements()
    // Sıra: [a, b, c] → sendBackward(c) → [a, c, b]
    store.sendBackward('c')
    expect(store.template.root.children.map(c => c.id)).toEqual(['a', 'c', 'b'])
  })

  it('bringToFront moves element to end', () => {
    const store = setupThreeElements()
    // Sıra: [a, b, c] → bringToFront(a) → [b, c, a]
    store.bringToFront('a')
    expect(store.template.root.children.map(c => c.id)).toEqual(['b', 'c', 'a'])
  })

  it('sendToBack moves element to beginning', () => {
    const store = setupThreeElements()
    // Sıra: [a, b, c] → sendToBack(c) → [c, a, b]
    store.sendToBack('c')
    expect(store.template.root.children.map(c => c.id)).toEqual(['c', 'a', 'b'])
  })

  it('bringForward on last element is no-op', () => {
    const store = setupThreeElements()
    store.bringForward('c')
    expect(store.template.root.children.map(c => c.id)).toEqual(['a', 'b', 'c'])
  })

  it('sendBackward on first element is no-op', () => {
    const store = setupThreeElements()
    store.sendBackward('a')
    expect(store.template.root.children.map(c => c.id)).toEqual(['a', 'b', 'c'])
  })

  it('bringToFront on last element is no-op', () => {
    const store = setupThreeElements()
    store.bringToFront('c')
    expect(store.template.root.children.map(c => c.id)).toEqual(['a', 'b', 'c'])
  })

  it('sendToBack on first element is no-op', () => {
    const store = setupThreeElements()
    store.sendToBack('a')
    expect(store.template.root.children.map(c => c.id)).toEqual(['a', 'b', 'c'])
  })
})

// =============================================================================
// 3.3 Dinamik Image Binding
// =============================================================================

describe('3.3 Dynamic Image Binding', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('ImageElement supports binding field', () => {
    const store = useTemplateStore()
    store.template = createTestTemplate()

    const img: ImageElement = {
      id: 'img_dyn',
      type: 'image',
      position: { type: 'flow' },
      size: { width: sz.fixed(40), height: sz.fixed(40) },
      binding: { type: 'scalar', path: 'firma.logo' },
      style: { objectFit: 'contain' },
    }

    store.addChild('root', img as unknown as TemplateElement)

    const el = store.getElementById('img_dyn') as ImageElement
    expect(el.binding).toBeDefined()
    expect(el.binding!.path).toBe('firma.logo')
    expect(el.src).toBeUndefined()
  })

  it('can switch from static to dynamic mode', () => {
    const store = useTemplateStore()
    store.template = createTestTemplate()

    const img: ImageElement = {
      id: 'img_switch',
      type: 'image',
      position: { type: 'flow' },
      size: { width: sz.fixed(40), height: sz.fixed(40) },
      src: 'data:image/png;base64,abc',
      style: {},
    }

    store.addChild('root', img as unknown as TemplateElement)

    // Dinamik moda geç
    store.updateElement('img_switch', {
      src: undefined,
      binding: { type: 'scalar', path: 'firma.logo' },
    } as Partial<TemplateElement>)

    const el = store.getElementById('img_switch') as ImageElement
    expect(el.binding).toBeDefined()
    expect(el.binding!.path).toBe('firma.logo')
  })

  it('can switch from dynamic to static mode', () => {
    const store = useTemplateStore()
    store.template = createTestTemplate()

    const img: ImageElement = {
      id: 'img_back',
      type: 'image',
      position: { type: 'flow' },
      size: { width: sz.fixed(40), height: sz.fixed(40) },
      binding: { type: 'scalar', path: 'firma.logo' },
      style: {},
    }

    store.addChild('root', img as unknown as TemplateElement)

    store.updateElement('img_back', {
      binding: undefined,
      src: 'data:image/png;base64,xyz',
    } as Partial<TemplateElement>)

    const el = store.getElementById('img_back') as ImageElement
    expect(el.src).toBe('data:image/png;base64,xyz')
  })
})
