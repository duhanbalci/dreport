// Template JSON veri modeli tip tanımları

// --- Boyut sistemi ---

/** Sabit mm, içeriğe göre (auto), veya kalan alanı doldur (fr) */
export type SizeValue =
  | { type: 'fixed'; value: number } // mm
  | { type: 'auto' }
  | { type: 'fr'; value: number } // ör: 1fr, 2fr

export interface SizeConstraint {
  width: SizeValue
  height: SizeValue
  minWidth?: number // mm
  minHeight?: number // mm
  maxWidth?: number // mm
  maxHeight?: number // mm
}

// Kısayol oluşturucular
export const sz = {
  fixed: (value: number): SizeValue => ({ type: 'fixed', value }),
  auto: (): SizeValue => ({ type: 'auto' }),
  fr: (value = 1): SizeValue => ({ type: 'fr', value }),
}

export interface PageSettings {
  width: number // mm
  height: number // mm
}

export interface Padding {
  top: number
  right: number
  bottom: number
  left: number
}

// --- Positioning ---

export type PositionMode =
  | { type: 'flow' } // Container flow'una katıl (varsayılan)
  | { type: 'absolute'; x: number; y: number } // Container içinde absolute (mm)

// --- Stil ---

export interface TextStyle {
  fontSize?: number // pt
  fontWeight?: 'normal' | 'bold'
  fontFamily?: string
  color?: string // hex
  align?: 'left' | 'center' | 'right'
}

export interface LineStyle {
  strokeColor?: string
  strokeWidth?: number // pt
}

export interface ContainerStyle {
  backgroundColor?: string
  borderColor?: string
  borderWidth?: number // pt
  borderRadius?: number // pt
}

// --- Binding ---

export interface ScalarBinding {
  type: 'scalar'
  path: string // ör: "firma.unvan"
}

export type ElementBinding = ScalarBinding

// --- Element tipleri ---

interface BaseElement {
  id: string
  position: PositionMode
  size: SizeConstraint
}

export interface StaticTextElement extends BaseElement {
  type: 'static_text'
  content: string
  style: TextStyle
}

export interface TextElement extends BaseElement {
  type: 'text'
  content?: string // opsiyonel prefix
  binding: ScalarBinding
  style: TextStyle
}

export interface LineElement extends BaseElement {
  type: 'line'
  style: LineStyle
}

export interface ContainerElement extends BaseElement {
  type: 'container'
  direction: 'row' | 'column'
  gap: number // mm — çocuklar arası boşluk
  padding: Padding
  align: 'start' | 'center' | 'end' | 'stretch'
  justify: 'start' | 'center' | 'end' | 'space-between'
  style: ContainerStyle
  children: TemplateElement[]
}

export type LeafElement = StaticTextElement | TextElement | LineElement
export type TemplateElement = LeafElement | ContainerElement

// --- Template ---

/** Sayfa kök container gibi davranır */
export interface Template {
  id: string
  name: string
  page: PageSettings
  fonts: string[]
  root: ContainerElement // kök container = sayfa
}

// --- Editor state ---

export interface EditorState {
  selectedElementId: string | null
  zoom: number // 0.25 - 4.0
  panX: number
  panY: number
  isDragging: boolean
}

// --- Yardımcılar ---

export function isContainer(el: TemplateElement): el is ContainerElement {
  return el.type === 'container'
}

export function isLeaf(el: TemplateElement): el is LeafElement {
  return el.type !== 'container'
}

/** Ağaçta bir element'i ID ile bulur */
export function findElementById(
  root: ContainerElement,
  id: string
): TemplateElement | undefined {
  if (root.id === id) return root
  for (const child of root.children) {
    if (child.id === id) return child
    if (isContainer(child)) {
      const found = findElementById(child, id)
      if (found) return found
    }
  }
  return undefined
}

/** Bir element'in parent container'ını bulur */
export function findParent(
  root: ContainerElement,
  id: string
): ContainerElement | undefined {
  for (const child of root.children) {
    if (child.id === id) return root
    if (isContainer(child)) {
      const found = findParent(child, id)
      if (found) return found
    }
  }
  return undefined
}
