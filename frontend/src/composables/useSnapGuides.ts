import { ref } from 'vue'
import type { ElementLayout } from '../core/layout-types'

export interface SnapGuide {
  type: 'vertical' | 'horizontal'
  position_mm: number
}

export interface SnapResult {
  snappedX_mm: number
  snappedY_mm: number
  guides: SnapGuide[]
}

interface EdgeSet {
  verticals: number[]  // x positions in mm (left, right, center of elements + page)
  horizontals: number[] // y positions in mm (top, bottom, center of elements + page)
}

export function useSnapGuides() {
  const SNAP_THRESHOLD_MM = 1.5
  const activeGuides = ref<SnapGuide[]>([])
  let cachedEdges: EdgeSet | null = null

  /** Collect edges from all elements except the one being dragged. Call once on drag start. */
  function collectEdges(
    layoutMap: Record<string, ElementLayout>,
    excludeId: string,
    pageWidth: number,
    pageHeight: number
  ) {
    const verticals: number[] = [0, pageWidth / 2, pageWidth]  // page edges + center
    const horizontals: number[] = [0, pageHeight / 2, pageHeight]

    for (const [id, el] of Object.entries(layoutMap)) {
      if (id === excludeId) continue
      // Left, center, right
      verticals.push(el.x_mm, el.x_mm + el.width_mm / 2, el.x_mm + el.width_mm)
      // Top, center, bottom
      horizontals.push(el.y_mm, el.y_mm + el.height_mm / 2, el.y_mm + el.height_mm)
    }

    cachedEdges = { verticals, horizontals }
  }

  /** Calculate snap for a dragged element. Returns adjusted position + active guides. */
  function calculateSnap(
    proposedX_mm: number,
    proposedY_mm: number,
    width_mm: number,
    height_mm: number
  ): SnapResult {
    if (!cachedEdges) {
      return { snappedX_mm: proposedX_mm, snappedY_mm: proposedY_mm, guides: [] }
    }

    const guides: SnapGuide[] = []
    let snappedX = proposedX_mm
    let snappedY = proposedY_mm

    // Element edges to check
    const myLeft = proposedX_mm
    const myCenter = proposedX_mm + width_mm / 2
    const myRight = proposedX_mm + width_mm

    // Find closest vertical snap
    let bestVDist = SNAP_THRESHOLD_MM
    let bestVSnap: { edge: number; offset: number } | null = null

    for (const v of cachedEdges.verticals) {
      // Check left edge
      const dLeft = Math.abs(myLeft - v)
      if (dLeft < bestVDist) {
        bestVDist = dLeft
        bestVSnap = { edge: v, offset: 0 }
      }
      // Check center
      const dCenter = Math.abs(myCenter - v)
      if (dCenter < bestVDist) {
        bestVDist = dCenter
        bestVSnap = { edge: v, offset: width_mm / 2 }
      }
      // Check right edge
      const dRight = Math.abs(myRight - v)
      if (dRight < bestVDist) {
        bestVDist = dRight
        bestVSnap = { edge: v, offset: width_mm }
      }
    }

    if (bestVSnap) {
      snappedX = bestVSnap.edge - bestVSnap.offset
      guides.push({ type: 'vertical', position_mm: bestVSnap.edge })
    }

    // Element edges to check (Y axis)
    const myTop = proposedY_mm
    const myMiddle = proposedY_mm + height_mm / 2
    const myBottom = proposedY_mm + height_mm

    // Find closest horizontal snap
    let bestHDist = SNAP_THRESHOLD_MM
    let bestHSnap: { edge: number; offset: number } | null = null

    for (const h of cachedEdges.horizontals) {
      const dTop = Math.abs(myTop - h)
      if (dTop < bestHDist) {
        bestHDist = dTop
        bestHSnap = { edge: h, offset: 0 }
      }
      const dMiddle = Math.abs(myMiddle - h)
      if (dMiddle < bestHDist) {
        bestHDist = dMiddle
        bestHSnap = { edge: h, offset: height_mm / 2 }
      }
      const dBottom = Math.abs(myBottom - h)
      if (dBottom < bestHDist) {
        bestHDist = dBottom
        bestHSnap = { edge: h, offset: height_mm }
      }
    }

    if (bestHSnap) {
      snappedY = bestHSnap.edge - bestHSnap.offset
      guides.push({ type: 'horizontal', position_mm: bestHSnap.edge })
    }

    activeGuides.value = guides
    return { snappedX_mm: snappedX, snappedY_mm: snappedY, guides }
  }

  /** Calculate snap for resize edge */
  function calculateResizeSnap(
    edge: 'left' | 'right' | 'top' | 'bottom',
    proposedValue_mm: number
  ): number {
    if (!cachedEdges) return proposedValue_mm

    const targets = (edge === 'left' || edge === 'right')
      ? cachedEdges.verticals
      : cachedEdges.horizontals

    const guides: SnapGuide[] = []
    let snapped = proposedValue_mm

    let bestDist = SNAP_THRESHOLD_MM
    for (const t of targets) {
      const d = Math.abs(proposedValue_mm - t)
      if (d < bestDist) {
        bestDist = d
        snapped = t
      }
    }

    if (snapped !== proposedValue_mm) {
      guides.push({
        type: (edge === 'left' || edge === 'right') ? 'vertical' : 'horizontal',
        position_mm: snapped,
      })
    }

    activeGuides.value = guides
    return snapped
  }

  function clearGuides() {
    activeGuides.value = []
    cachedEdges = null
  }

  return {
    activeGuides,
    collectEdges,
    calculateSnap,
    calculateResizeSnap,
    clearGuides,
  }
}
