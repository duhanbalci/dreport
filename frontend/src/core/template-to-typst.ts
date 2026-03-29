import type {
  Template,
  TemplateElement,
  ContainerElement,
  StaticTextElement,
  TextElement,
  LineElement,
  TextStyle,
  SizeValue,
  SizeConstraint,
} from './types'
import { isContainer } from './types'

/**
 * Template JSON → Typst markup dönüşümü.
 * Container-based layout + layout query (her element için pozisyon/boyut bilgisi).
 */
export function templateToTypst(template: Template, data?: Record<string, unknown>): string {
  const lines: string[] = []

  const { page, root } = template
  const p = root.padding
  lines.push(
    `#set page(width: ${page.width}mm, height: ${page.height}mm, margin: (top: ${p.top}mm, right: ${p.right}mm, bottom: ${p.bottom}mm, left: ${p.left}mm))`
  )
  lines.push('')

  if (data) {
    lines.push(`#let data = ${jsonToTypstDict(data)}`)
  } else {
    lines.push(`#let data = (:)`)
  }
  lines.push('')

  // Tüm elemanları topla — topological order: leaf'ler önce, container'lar sonra
  const allElements = collectTopological(root)

  // Her element'in content'ini #let ile tanımla + label ata
  for (const el of allElements) {
    const v = idToVar(el.id)
    // Root container: sayfa margin'leri zaten padding'i karşılıyor, inset ekleme
    const content = el === root
      ? renderContainerContent(el, true)
      : renderElementContent(el)
    lines.push(`#let ${v} = ${content}`)
  }
  lines.push('')

  // Kök container'ı renderla — her eleman label'lı olmalı
  lines.push(renderRootWithLabels(root))
  lines.push('')

  // Layout query — her eleman parent'ının available width'i ile ölçülür
  lines.push(generateLayoutQuery(allElements, root, page.width))

  return lines.join('\n')
}

// --- Topological sort: leaf'ler önce ---

function collectTopological(root: ContainerElement): TemplateElement[] {
  const result: TemplateElement[] = []
  function walk(el: TemplateElement) {
    if (isContainer(el)) {
      for (const child of el.children) walk(child)
    }
    result.push(el)
  }
  walk(root)
  return result
}

// --- Element content rendering ---

function renderElementContent(el: TemplateElement): string {
  switch (el.type) {
    case 'container':
      return renderContainerContent(el)
    case 'static_text':
      return renderStaticTextContent(el)
    case 'text':
      return renderTextContent(el)
    case 'line':
      return renderLineContent(el)
  }
}

function renderContainerContent(el: ContainerElement, skipPadding = false): string {
  const boxParams = buildBoxParams(el, skipPadding)

  const flowChildren = el.children.filter(c => c.position.type !== 'absolute')
  const absoluteChildren = el.children.filter(c => c.position.type === 'absolute')

  const innerParts: string[] = []

  if (flowChildren.length > 0) {
    const dir = el.direction === 'row' ? 'ltr' : 'ttb'
    const gap = el.gap > 0 ? `, spacing: ${el.gap}mm` : ''

    if (flowChildren.length === 1) {
      // Label'lı referans
      innerParts.push(`#[#${idToVar(flowChildren[0].id)} <${flowChildren[0].id}>]`)
    } else {
      const items = flowChildren.map(c =>
        `    [#${idToVar(c.id)} <${c.id}>]`
      ).join(',\n')
      innerParts.push(`#stack(dir: ${dir}${gap},\n${items}\n  )`)
    }
  }

  for (const child of absoluteChildren) {
    if (child.position.type === 'absolute') {
      innerParts.push(
        `#place(top + left, dx: ${child.position.x}mm, dy: ${child.position.y}mm)[#${idToVar(child.id)} <${child.id}>]`
      )
    }
  }

  // Boş container'a minimum yükseklik ver
  if (innerParts.length === 0) {
    innerParts.push('#v(5mm)')
  }

  const inner = innerParts.join('\n  ')
  return `box(${boxParams})[\n  ${inner}\n]`
}

function renderStaticTextContent(el: StaticTextElement): string {
  const sizeParams = buildBoxSizeParams(el.size, false)
  const textCmd = buildTextCommand(el.style, escapeTypstContent(el.content))

  if (sizeParams) {
    return `box(${sizeParams})[${textCmd}]`
  }
  return `[${textCmd}]`
}

function renderTextContent(el: TextElement): string {
  const sizeParams = buildBoxSizeParams(el.size, false)
  const dataAccess = `#data.${el.binding.path}`
  const content = el.content ? escapeTypstContent(el.content) + dataAccess : dataAccess
  const textCmd = buildTextCommand(el.style, content)

  if (sizeParams) {
    return `box(${sizeParams})[${textCmd}]`
  }
  return `[${textCmd}]`
}

function renderLineContent(el: LineElement): string {
  const stroke = el.style.strokeWidth ?? 0.5
  const color = el.style.strokeColor ?? '#000000'
  // line() fr kabul etmez; measure() göreceli birimleri çözemez
  // Bu yüzden line'ı box(width: 100%) ile sarıyoruz
  if (el.size.width.type === 'fr' || el.size.width.type === 'auto') {
    return `box(width: 100%)[#line(length: 100%, stroke: ${stroke}pt + rgb("${color}"))]`
  }
  const widthStr = sizeValueToTypst(el.size.width)
  return `line(length: ${widthStr}, stroke: ${stroke}pt + rgb("${color}"))`
}

// --- Root rendering with labels ---

function renderRootWithLabels(root: ContainerElement): string {
  return `#[#${idToVar(root.id)} <${root.id}>]`
}

// --- Layout query ---

function generateLayoutQuery(
  elements: TemplateElement[],
  root: ContainerElement,
  pageWidth: number,
): string {
  // Her eleman için parent'ın available width'ini hesapla
  const parentMap = buildParentMap(root)
  const widthMap = computeAvailableWidths(root, pageWidth, parentMap)

  const varLines = elements.map(el => {
    const v = idToVar(el.id)
    const availW = widthMap.get(el.id) ?? pageWidth
    return `  let ${v}p = locate(<${el.id}>).position()
  let ${v}s = measure(${v}, width: ${Math.round(availW * 100) / 100}mm)
  result += "${el.id}:" + repr(${v}p.x) + "," + repr(${v}p.y) + "," + repr(${v}s.width) + "," + repr(${v}s.height) + "|"`
  }).join('\n')

  return `#context {
  let result = ""
${varLines}
  place(bottom + right, text(size: 0.1pt, fill: white)[#result])
}`
}

/** Her elemanın parent'ını tutan map */
function buildParentMap(root: ContainerElement): Map<string, ContainerElement> {
  const map = new Map<string, ContainerElement>()
  function walk(parent: ContainerElement) {
    for (const child of parent.children) {
      map.set(child.id, parent)
      if (isContainer(child)) walk(child)
    }
  }
  walk(root)
  return map
}

/** Her eleman için measure'a verilecek available width (mm) hesapla */
function computeAvailableWidths(
  root: ContainerElement,
  pageWidth: number,
  parentMap: Map<string, ContainerElement>,
): Map<string, number> {
  const map = new Map<string, number>()

  // Root: sayfa margin'leri root.padding'den geliyor, root box'ta inset yok
  // Root'un content area genişliği = sayfa - margin sol - margin sağ
  const rootContentWidth = pageWidth - root.padding.left - root.padding.right
  map.set(root.id, rootContentWidth)

  function getContainerInnerWidth(c: ContainerElement): number {
    const ownWidth = map.get(c.id) ?? rootContentWidth
    // Root'un padding'i zaten sayfa margin olarak uygulandı, tekrar çıkarma
    if (c.id === root.id) return ownWidth
    return ownWidth - c.padding.left - c.padding.right
  }

  function walk(container: ContainerElement) {
    const innerW = getContainerInnerWidth(container)

    // row container ise çocuklar genişliği paylaşır
    // column container ise her çocuk full genişlik alır
    if (container.direction === 'column') {
      for (const child of container.children) {
        // Fixed genişlikli çocuk kendi genişliğini alır, diğerleri parent inner width
        const childW = child.size.width.type === 'fixed' ? child.size.width.value : innerW
        map.set(child.id, childW)
        if (isContainer(child)) walk(child)
      }
    } else {
      // row: fixed genişlikli çocukları çıkar, kalanı fr'lara dağıt
      let usedWidth = 0
      let totalFr = 0
      const gap = container.gap * Math.max(0, container.children.length - 1)

      for (const child of container.children) {
        if (child.size.width.type === 'fixed') {
          usedWidth += child.size.width.value
        } else if (child.size.width.type === 'fr') {
          totalFr += child.size.width.value
        }
      }

      const remainingW = Math.max(0, innerW - usedWidth - gap)

      for (const child of container.children) {
        let childW: number
        if (child.size.width.type === 'fixed') {
          childW = child.size.width.value
        } else if (child.size.width.type === 'fr') {
          childW = totalFr > 0 ? (child.size.width.value / totalFr) * remainingW : remainingW
        } else {
          childW = innerW // auto
        }
        map.set(child.id, childW)
        if (isContainer(child)) walk(child)
      }
    }
  }

  walk(root)
  return map
}

// --- Yardımcılar ---

function idToVar(id: string): string {
  return 'v_' + id.replace(/[^a-zA-Z0-9]/g, '_')
}

function buildBoxParams(el: ContainerElement, skipPadding = false): string {
  const parts: string[] = []

  // box() fr kabul etmez, fr → 100% olarak çevir
  const sizeParams = buildBoxSizeParams(el.size, false)
  if (sizeParams) parts.push(sizeParams)

  if (!skipPadding) {
    const hasPadding = el.padding.top > 0 || el.padding.right > 0 || el.padding.bottom > 0 || el.padding.left > 0
    if (hasPadding) {
      parts.push(`inset: (top: ${el.padding.top}mm, right: ${el.padding.right}mm, bottom: ${el.padding.bottom}mm, left: ${el.padding.left}mm)`)
    }
  }

  const styleParams = buildContainerStyleParams(el)
  if (styleParams) parts.push(styleParams)

  return parts.join(', ')
}

function buildBoxSizeParams(size: SizeConstraint, allowFr = true): string {
  const parts: string[] = []
  const w = sizeValueToTypst(size.width, allowFr)
  if (w !== 'auto') parts.push(`width: ${w}`)
  const h = sizeValueToTypst(size.height, allowFr)
  if (h !== 'auto') parts.push(`height: ${h}`)
  return parts.join(', ')
}

function sizeValueToTypst(sv: SizeValue, allowFr = true): string {
  switch (sv.type) {
    case 'fixed': return `${sv.value}mm`
    case 'auto': return 'auto'
    case 'fr': return allowFr ? `${sv.value}fr` : '100%'
  }
}

function buildContainerStyleParams(el: ContainerElement): string {
  const parts: string[] = []
  if (el.style.backgroundColor) parts.push(`fill: rgb("${el.style.backgroundColor}")`)
  if (el.style.borderColor && (el.style.borderWidth ?? 0) > 0) {
    parts.push(`stroke: ${el.style.borderWidth ?? 1}pt + rgb("${el.style.borderColor}")`)
  }
  if (el.style.borderRadius && el.style.borderRadius > 0) {
    parts.push(`radius: ${el.style.borderRadius}pt`)
  }
  return parts.join(', ')
}

function buildTextCommand(style: TextStyle, content: string): string {
  const parts: string[] = []
  if (style.fontSize) parts.push(`size: ${style.fontSize}pt`)
  if (style.fontWeight === 'bold') parts.push(`weight: "bold"`)
  if (style.fontFamily) parts.push(`font: "${style.fontFamily}"`)
  if (style.color) parts.push(`fill: rgb("${style.color}")`)

  const params = parts.join(', ')
  let result = `#text(${params})[${content}]`

  if (style.align && style.align !== 'left') {
    result = `#align(${style.align})[${result}]`
  }
  return result
}

function escapeTypstContent(text: string): string {
  return text
    .replace(/\\/g, '\\\\')
    .replace(/\[/g, '\\[')
    .replace(/\]/g, '\\]')
    .replace(/#/g, '\\#')
    .replace(/\$/g, '\\$')
    .replace(/@/g, '\\@')
    .replace(/</g, '\\<')
    .replace(/>/g, '\\>')
}

function jsonToTypstDict(obj: unknown): string {
  if (obj === null || obj === undefined) return 'none'
  if (typeof obj === 'string') return `"${obj.replace(/"/g, '\\"')}"`
  if (typeof obj === 'number') return String(obj)
  if (typeof obj === 'boolean') return obj ? 'true' : 'false'
  if (Array.isArray(obj)) {
    const items = obj.map(item => jsonToTypstDict(item)).join(', ')
    return `(${items},)`
  }
  if (typeof obj === 'object') {
    const entries = Object.entries(obj as Record<string, unknown>)
      .map(([key, val]) => `${key}: ${jsonToTypstDict(val)}`)
      .join(', ')
    return `(${entries})`
  }
  return 'none'
}

// --- Layout data parsing ---

export interface ElementLayout {
  x: number // pt
  y: number // pt
  width: number // pt
  height: number // pt
}

export function parseLayoutFromSvg(svgString: string): Record<string, ElementLayout> {
  const result: Record<string, ElementLayout> = {}
  const matches = svgString.matchAll(/([a-zA-Z0-9_-]+):([\d.]+)pt,([\d.]+)pt,([\d.]+)pt,([\d.]+)pt\|/g)
  for (const m of matches) {
    result[m[1]] = {
      x: parseFloat(m[2]),
      y: parseFloat(m[3]),
      width: parseFloat(m[4]),
      height: parseFloat(m[5]),
    }
  }
  return result
}
