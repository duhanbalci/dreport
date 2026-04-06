import { describe, it, expect, beforeEach } from 'vitest'
import { useSnapGuides } from '../useSnapGuides'
import type { ElementLayout } from '../../core/layout-types'

function makeLayout(
  id: string,
  x: number,
  y: number,
  w: number,
  h: number,
): ElementLayout {
  return {
    id,
    x_mm: x,
    y_mm: y,
    width_mm: w,
    height_mm: h,
    element_type: 'static_text',
    style: {},
  } as ElementLayout
}

describe('useSnapGuides', () => {
  let guides: ReturnType<typeof useSnapGuides>

  beforeEach(() => {
    guides = useSnapGuides()
  })

  describe('collectEdges', () => {
    it('collects page edges and element edges', () => {
      const layoutMap: Record<string, ElementLayout> = {
        el1: makeLayout('el1', 10, 20, 50, 30),
      }

      guides.collectEdges(layoutMap, 'excluded', 210, 297)

      // After collecting, calculateSnap should work
      const result = guides.calculateSnap(0, 0, 10, 10)
      expect(result).toBeDefined()
    })

    it('excludes the dragged element', () => {
      const layoutMap: Record<string, ElementLayout> = {
        dragged: makeLayout('dragged', 50, 50, 20, 20),
        other: makeLayout('other', 100, 100, 30, 30),
      }

      guides.collectEdges(layoutMap, 'dragged', 210, 297)

      // Snap to "other" element's left edge (100mm)
      const result = guides.calculateSnap(99.5, 50, 20, 20)
      expect(result.snappedX_mm).toBe(100) // snaps to other's left edge
    })
  })

  describe('calculateSnap', () => {
    it('returns proposed position when no edges cached', () => {
      const result = guides.calculateSnap(42, 73, 10, 10)

      expect(result.snappedX_mm).toBe(42)
      expect(result.snappedY_mm).toBe(73)
      expect(result.guides).toHaveLength(0)
    })

    it('snaps left edge to page left (0)', () => {
      guides.collectEdges({}, 'none', 210, 297)

      // Proposed x=0.5 → should snap to 0 (within 1.5mm threshold)
      const result = guides.calculateSnap(0.5, 50, 20, 20)
      expect(result.snappedX_mm).toBe(0)
      expect(result.guides).toContainEqual({ type: 'vertical', position_mm: 0 })
    })

    it('snaps right edge to page right', () => {
      guides.collectEdges({}, 'none', 210, 297)

      // Element 20mm wide, proposed x=189 → right edge = 209, should snap to 210
      const result = guides.calculateSnap(189, 50, 20, 20)
      expect(result.snappedX_mm).toBe(190) // 210 - 20 = 190
      expect(result.guides).toContainEqual({ type: 'vertical', position_mm: 210 })
    })

    it('snaps center to page center', () => {
      guides.collectEdges({}, 'none', 210, 297)

      // Element 20mm wide, center at 105mm → x = 95
      // Proposed x=94.5 → center = 104.5, should snap to 105 → x = 95
      const result = guides.calculateSnap(94.5, 50, 20, 20)
      expect(result.snappedX_mm).toBe(95) // 105 - 10 = 95
    })

    it('snaps top edge to page top', () => {
      guides.collectEdges({}, 'none', 210, 297)

      const result = guides.calculateSnap(50, 1.0, 20, 20)
      expect(result.snappedY_mm).toBe(0)
      expect(result.guides).toContainEqual({ type: 'horizontal', position_mm: 0 })
    })

    it('does not snap when outside threshold', () => {
      guides.collectEdges({}, 'none', 210, 297)

      // Proposed x=50, far from any edge → no snap
      const result = guides.calculateSnap(50, 50, 20, 20)
      expect(result.snappedX_mm).toBe(50)
      expect(result.snappedY_mm).toBe(50)
    })

    it('snaps to other element edges', () => {
      const layoutMap: Record<string, ElementLayout> = {
        ref: makeLayout('ref', 30, 40, 50, 20),
      }
      guides.collectEdges(layoutMap, 'dragged', 210, 297)

      // Snap dragged element's left to ref's right (30+50=80)
      const result = guides.calculateSnap(79.5, 50, 20, 20)
      expect(result.snappedX_mm).toBe(80)
    })

    it('snaps both axes simultaneously', () => {
      guides.collectEdges({}, 'none', 210, 297)

      // Near page origin
      const result = guides.calculateSnap(0.5, 0.5, 20, 20)
      expect(result.snappedX_mm).toBe(0)
      expect(result.snappedY_mm).toBe(0)
      expect(result.guides).toHaveLength(2)
    })

    it('updates activeGuides ref', () => {
      guides.collectEdges({}, 'none', 210, 297)

      guides.calculateSnap(0.5, 0.5, 20, 20)
      expect(guides.activeGuides.value.length).toBeGreaterThan(0)
    })
  })

  describe('calculateResizeSnap', () => {
    it('returns proposed value when no edges', () => {
      const result = guides.calculateResizeSnap('right', 42)
      expect(result).toBe(42)
    })

    it('snaps right edge to nearest vertical', () => {
      const layoutMap: Record<string, ElementLayout> = {
        ref: makeLayout('ref', 100, 50, 40, 20),
      }
      guides.collectEdges(layoutMap, 'resizing', 210, 297)

      // Snap to ref's left edge (100mm)
      const result = guides.calculateResizeSnap('right', 99.5)
      expect(result).toBe(100)
    })

    it('snaps bottom edge to nearest horizontal', () => {
      const layoutMap: Record<string, ElementLayout> = {
        ref: makeLayout('ref', 50, 80, 40, 20),
      }
      guides.collectEdges(layoutMap, 'resizing', 210, 297)

      // Snap to ref's top edge (80mm)
      const result = guides.calculateResizeSnap('bottom', 79.5)
      expect(result).toBe(80)
    })

    it('does not snap when outside threshold', () => {
      guides.collectEdges({}, 'none', 210, 297)

      const result = guides.calculateResizeSnap('right', 50)
      expect(result).toBe(50) // no edge near 50mm
    })
  })

  describe('clearGuides', () => {
    it('clears active guides and cached edges', () => {
      guides.collectEdges({}, 'none', 210, 297)
      guides.calculateSnap(0.5, 0.5, 10, 10)
      expect(guides.activeGuides.value.length).toBeGreaterThan(0)

      guides.clearGuides()
      expect(guides.activeGuides.value).toHaveLength(0)

      // After clear, calculateSnap should return unsnapped
      const result = guides.calculateSnap(0.5, 0.5, 10, 10)
      expect(result.snappedX_mm).toBe(0.5)
    })
  })
})
