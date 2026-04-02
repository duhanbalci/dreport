// Layout engine çıktı tipleri — Rust LayoutResult ile birebir eşleşir

export interface LayoutResult {
  pages: PageLayout[]
}

export interface PageLayout {
  page_index: number
  width_mm: number
  height_mm: number
  elements: ElementLayout[]
}

export interface ElementLayout {
  id: string
  x_mm: number
  y_mm: number
  width_mm: number
  height_mm: number
  element_type: string
  content: ResolvedContent | null
  style: ResolvedStyle
  children: string[]
}

export interface LayoutMapEntry extends ElementLayout {
  pageIndex: number
}

export interface ResolvedRichSpan {
  text: string
  fontSize?: number
  fontWeight?: string
  fontFamily?: string
  color?: string
}

export type ResolvedContent =
  | { type: 'text'; value: string }
  | { type: 'image'; src: string }
  | { type: 'line' }
  | { type: 'barcode'; format: string; value: string }
  | { type: 'page_number'; current: number; total: number }
  | { type: 'shape'; shapeType: string }
  | { type: 'checkbox'; checked: boolean }
  | { type: 'rich_text'; spans: ResolvedRichSpan[] }
  | { type: 'table'; headers: TableHeaderCell[]; rows: TableCell[][]; column_widths_mm: number[] }

export interface TableHeaderCell {
  text: string
  align: string
}

export interface TableCell {
  text: string
  align: string
}

export interface ResolvedStyle {
  fontSize?: number
  fontWeight?: string
  fontFamily?: string
  color?: string
  textAlign?: string
  strokeColor?: string
  strokeWidth?: number
  backgroundColor?: string
  borderColor?: string
  borderWidth?: number
  borderRadius?: number
  borderStyle?: string
  headerBg?: string
  headerColor?: string
  zebraOdd?: string
  zebraEven?: string
  headerFontSize?: number
  objectFit?: string
  barcodeColor?: string
  barcodeIncludeText?: boolean
}
