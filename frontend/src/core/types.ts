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
  borderStyle?: 'solid' | 'dashed' | 'dotted'
}

// --- Binding ---

export interface ScalarBinding {
  type: 'scalar'
  path: string // ör: "firma.unvan"
}

export interface ArrayBinding {
  type: 'array'
  path: string // ör: "kalemler"
}

export type ElementBinding = ScalarBinding | ArrayBinding

// --- Tablo ---

export type FormatType = 'currency' | 'date' | 'percentage' | 'number'

export interface TableColumn {
  id: string
  field: string // array item içindeki alan — ör: "adi", "tutar"
  title: string // Sütun başlığı
  width: SizeValue
  align: 'left' | 'center' | 'right'
  format?: FormatType
}

export interface TableStyle {
  headerBg?: string // hex — header arka plan rengi
  headerColor?: string // hex — header metin rengi
  zebraOdd?: string // hex — tek satır arka plan
  zebraEven?: string // hex — çift satır arka plan
  borderColor?: string // hex
  borderWidth?: number // pt
  fontSize?: number // pt
  headerFontSize?: number // pt
  cellPaddingH?: number // mm — hücre yatay iç boşluk (sol+sağ). Default: 2
  cellPaddingV?: number // mm — hücre dikey iç boşluk (üst+alt). Default: 1
  headerPaddingH?: number // mm — header yatay iç boşluk. Default: cellPaddingH
  headerPaddingV?: number // mm — header dikey iç boşluk. Default: cellPaddingV
}

// --- Barcode ---

export type BarcodeFormat = 'qr' | 'ean13' | 'ean8' | 'code128' | 'code39'

export interface BarcodeStyle {
  color?: string // ön plan rengi (varsayılan: siyah)
  includeText?: boolean // barkod altına değer yazılsın mı (QR hariç)
}

// --- Condition (koşullu gösterim) ---

export interface Condition {
  path: string
  operator: string
  value?: unknown
}

// --- Element tipleri ---

interface BaseElement {
  id: string
  condition?: Condition
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

export interface ImageStyle {
  objectFit?: 'contain' | 'cover' | 'stretch'
}

export interface ImageElement extends BaseElement {
  type: 'image'
  src?: string // statik görsel: data URI veya URL
  binding?: ScalarBinding // dinamik görsel: schema'dan path
  style: ImageStyle
}

export interface PageNumberElement extends BaseElement {
  type: 'page_number'
  style: TextStyle
  format?: string // ör: "{current} / {total}"
}

export interface BarcodeElement extends BaseElement {
  type: 'barcode'
  format: BarcodeFormat
  value?: string // statik değer
  binding?: ScalarBinding // dinamik değer (schema'dan)
  style: BarcodeStyle
}

export interface CurrentDateElement extends BaseElement {
  type: 'current_date'
  style: TextStyle
  format?: string // ör: "DD.MM.YYYY", "DD MMMM YYYY", "DD.MM.YYYY HH:mm"
}

export interface ShapeElement extends BaseElement {
  type: 'shape'
  shapeType: 'rectangle' | 'ellipse' | 'rounded_rectangle'
  style: ContainerStyle
}

export interface CheckboxStyle {
  size?: number // mm — kare boyutu
  checkColor?: string // checkmark rengi
  borderColor?: string
  borderWidth?: number
}

export interface CheckboxElement extends BaseElement {
  type: 'checkbox'
  checked?: boolean
  binding?: ScalarBinding
  style: CheckboxStyle
}

export interface CalculatedTextElement extends BaseElement {
  type: 'calculated_text'
  expression: string
  format?: FormatType
  style: TextStyle
}

export interface RichTextSpan {
  text?: string
  binding?: ScalarBinding
  style: TextStyle
}

export interface RichTextElement extends BaseElement {
  type: 'rich_text'
  content: RichTextSpan[]
  style: TextStyle // varsayılan stil
}

export interface PageBreakElement extends BaseElement {
  type: 'page_break'
  style: Record<string, never>
}

// --- Chart ---

export type ChartType = 'bar' | 'line' | 'pie'
export type GroupMode = 'grouped' | 'stacked'

export interface ChartTitle {
  text: string
  fontSize?: number
  color?: string
  align?: 'left' | 'center' | 'right'
}

export interface ChartLegend {
  show: boolean
  position?: 'top' | 'bottom' | 'right'
  fontSize?: number
}

export interface ChartLabels {
  show: boolean
  fontSize?: number
  color?: string
}

export interface ChartAxis {
  xLabel?: string
  yLabel?: string
  showGrid?: boolean
  gridColor?: string
}

export interface ChartStyle {
  colors?: string[]
  backgroundColor?: string
  barGap?: number // 0.0-1.0
  lineWidth?: number // mm
  showPoints?: boolean
  curveType?: 'linear' | 'smooth'
  innerRadius?: number // 0=pie, >0=donut (0-0.9)
}

export interface ChartElement extends BaseElement {
  type: 'chart'
  chartType: ChartType
  dataSource: ArrayBinding
  categoryField: string
  valueField: string
  groupField?: string
  groupMode?: GroupMode
  title?: ChartTitle
  legend?: ChartLegend
  labels?: ChartLabels
  axis?: ChartAxis
  style: ChartStyle
}

export interface ContainerElement extends BaseElement {
  type: 'container'
  direction: 'row' | 'column'
  gap: number // mm — çocuklar arası boşluk
  padding: Padding
  align: 'start' | 'center' | 'end' | 'stretch'
  justify: 'start' | 'center' | 'end' | 'space-between'
  breakInside?: 'auto' | 'avoid'
  style: ContainerStyle
  children: TemplateElement[]
}

export interface RepeatingTableElement extends BaseElement {
  type: 'repeating_table'
  dataSource: ArrayBinding
  columns: TableColumn[]
  style: TableStyle
  repeatHeader?: boolean
}

export type LeafElement =
  | StaticTextElement
  | TextElement
  | LineElement
  | RepeatingTableElement
  | ImageElement
  | PageNumberElement
  | BarcodeElement
  | PageBreakElement
  | CurrentDateElement
  | ShapeElement
  | CheckboxElement
  | CalculatedTextElement
  | RichTextElement
  | ChartElement
export type TemplateElement = LeafElement | ContainerElement

// --- Template ---

/** Sayfa kök container gibi davranır */
export interface Template {
  id: string
  name: string
  page: PageSettings
  fonts: string[]
  root: ContainerElement // kök container = sayfa
  header?: ContainerElement
  footer?: ContainerElement
}

// --- Editor state ---

export interface EditorState {
  selectedElementIds: Set<string>
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
export function findElementById(root: ContainerElement, id: string): TemplateElement | undefined {
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
export function findParent(root: ContainerElement, id: string): ContainerElement | undefined {
  for (const child of root.children) {
    if (child.id === id) return root
    if (isContainer(child)) {
      const found = findParent(child, id)
      if (found) return found
    }
  }
  return undefined
}
