/**
 * JSON Schema'dan editör için kullanılabilir ağaç yapısı üretir.
 * Schema'daki array alanlarını ve alt özelliklerini tespit eder.
 */

export interface SchemaNode {
  path: string // Noktalı yol — ör: "firma.unvan"
  key: string // Son segment — ör: "unvan"
  title: string // Görüntüleme adı — schema'daki "title" veya key
  type: 'object' | 'array' | 'string' | 'number' | 'integer' | 'boolean'
  format?: string // "currency", "date", "percentage", "image" vs.
  children: SchemaNode[]
  /** Sadece array tipi için: array item'larının alt alanları */
  itemProperties?: SchemaNode[]
}

export interface JsonSchema {
  $id?: string
  type: string
  title?: string
  format?: string
  properties?: Record<string, JsonSchema>
  items?: JsonSchema
}

/** JSON Schema'yı SchemaNode ağacına dönüştürür */
export function parseSchema(schema: JsonSchema, path = '', key = 'root'): SchemaNode {
  const title = schema.title ?? key
  const type = schema.type as SchemaNode['type']

  const node: SchemaNode = {
    path,
    key,
    title,
    type,
    format: schema.format,
    children: [],
  }

  if (schema.type === 'object' && schema.properties) {
    for (const [propKey, propSchema] of Object.entries(schema.properties)) {
      const childPath = path ? `${path}.${propKey}` : propKey
      node.children.push(parseSchema(propSchema, childPath, propKey))
    }
  }

  if (schema.type === 'array' && schema.items) {
    // Array'in item schema'sını ayrıca parse et
    if (schema.items.type === 'object' && schema.items.properties) {
      const itemPath = path ? `${path}[]` : `${key}[]`
      node.itemProperties = []
      for (const [propKey, propSchema] of Object.entries(schema.items.properties)) {
        node.itemProperties.push(parseSchema(propSchema, `${itemPath}.${propKey}`, propKey))
      }
    }
  }

  return node
}

/** Schema ağacından tüm array alanlarını bulur (tablo binding için) */
export function findArrayFields(node: SchemaNode): SchemaNode[] {
  const result: SchemaNode[] = []
  if (node.type === 'array') {
    result.push(node)
  }
  for (const child of node.children) {
    result.push(...findArrayFields(child))
  }
  return result
}

/** Schema ağacından tüm scalar alanları bulur (metin binding için) */
export function findScalarFields(node: SchemaNode): SchemaNode[] {
  const result: SchemaNode[] = []
  if (
    node.type === 'string' ||
    node.type === 'number' ||
    node.type === 'integer' ||
    node.type === 'boolean'
  ) {
    result.push(node)
  }
  for (const child of node.children) {
    result.push(...findScalarFields(child))
  }
  return result
}

/** Format tipinden FormatType'a dönüşüm */
export function schemaFormatToFormatType(
  format?: string,
): 'currency' | 'date' | 'percentage' | 'number' | undefined {
  if (!format) return undefined
  switch (format) {
    case 'currency':
      return 'currency'
    case 'date':
      return 'date'
    case 'percentage':
      return 'percentage'
    default:
      return undefined
  }
}

/** Bir SchemaNode'un tipinden varsayılan tablo hizalamasını belirle */
export function defaultAlignForSchema(node: SchemaNode): 'left' | 'center' | 'right' {
  if (node.type === 'number' || node.type === 'integer') return 'right'
  if (node.format === 'currency') return 'right'
  if (node.format === 'percentage') return 'right'
  if (node.format === 'date') return 'center'
  return 'left'
}
