import { describe, it, expect } from 'vitest'
import {
  parseSchema,
  findArrayFields,
  findScalarFields,
  schemaFormatToFormatType,
  defaultAlignForSchema,
  type JsonSchema,
  type SchemaNode,
} from '../schema-parser'

const testSchema: JsonSchema = {
  type: 'object',
  properties: {
    firma: {
      type: 'object',
      title: 'Firma',
      properties: {
        unvan: { type: 'string', title: 'Firma Unvani' },
        vergiNo: { type: 'string', title: 'Vergi No' },
      },
    },
    fatura: {
      type: 'object',
      title: 'Fatura',
      properties: {
        no: { type: 'string', title: 'Fatura No' },
        tutar: { type: 'number', title: 'Tutar', format: 'currency' },
        tarih: { type: 'string', title: 'Tarih', format: 'date' },
      },
    },
    kalemler: {
      type: 'array',
      title: 'Kalemler',
      items: {
        type: 'object',
        properties: {
          adi: { type: 'string', title: 'Adi' },
          miktar: { type: 'number', title: 'Miktar' },
        },
      },
    },
  },
}

describe('parseSchema', () => {
  it('parses nested object schema into correct tree structure', () => {
    const tree = parseSchema(testSchema)

    expect(tree.type).toBe('object')
    expect(tree.key).toBe('root')
    expect(tree.path).toBe('')
    expect(tree.children).toHaveLength(3)

    const firma = tree.children[0]
    expect(firma.key).toBe('firma')
    expect(firma.title).toBe('Firma')
    expect(firma.type).toBe('object')
    expect(firma.path).toBe('firma')
    expect(firma.children).toHaveLength(2)

    const unvan = firma.children[0]
    expect(unvan.key).toBe('unvan')
    expect(unvan.title).toBe('Firma Unvani')
    expect(unvan.type).toBe('string')
    expect(unvan.path).toBe('firma.unvan')
  })

  it('parses array schema with correct itemProperties', () => {
    const tree = parseSchema(testSchema)
    const kalemler = tree.children[2]

    expect(kalemler.key).toBe('kalemler')
    expect(kalemler.type).toBe('array')
    expect(kalemler.title).toBe('Kalemler')
    expect(kalemler.itemProperties).toBeDefined()
    expect(kalemler.itemProperties).toHaveLength(2)

    const adi = kalemler.itemProperties![0]
    expect(adi.key).toBe('adi')
    expect(adi.path).toBe('kalemler[].adi')
    expect(adi.type).toBe('string')

    const miktar = kalemler.itemProperties![1]
    expect(miktar.key).toBe('miktar')
    expect(miktar.path).toBe('kalemler[].miktar')
    expect(miktar.type).toBe('number')
  })

  it('preserves format field from schema', () => {
    const tree = parseSchema(testSchema)
    const fatura = tree.children[1]
    const tutar = fatura.children[1]
    const tarih = fatura.children[2]

    expect(tutar.format).toBe('currency')
    expect(tarih.format).toBe('date')
  })

  it('uses key as title when title is not provided', () => {
    const schema: JsonSchema = {
      type: 'object',
      properties: {
        foo: { type: 'string' },
      },
    }
    const tree = parseSchema(schema)
    expect(tree.children[0].title).toBe('foo')
  })

  it('handles empty schema with no properties', () => {
    const schema: JsonSchema = { type: 'object' }
    const tree = parseSchema(schema)

    expect(tree.type).toBe('object')
    expect(tree.children).toHaveLength(0)
    expect(tree.itemProperties).toBeUndefined()
  })
})

describe('findArrayFields', () => {
  it('returns only array nodes', () => {
    const tree = parseSchema(testSchema)
    const arrays = findArrayFields(tree)

    expect(arrays).toHaveLength(1)
    expect(arrays[0].key).toBe('kalemler')
    expect(arrays[0].type).toBe('array')
  })

  it('returns empty for schema with no arrays', () => {
    const schema: JsonSchema = {
      type: 'object',
      properties: {
        name: { type: 'string' },
      },
    }
    const tree = parseSchema(schema)
    expect(findArrayFields(tree)).toHaveLength(0)
  })
})

describe('findScalarFields', () => {
  it('returns only scalar nodes (string, number, integer, boolean)', () => {
    const tree = parseSchema(testSchema)
    const scalars = findScalarFields(tree)

    // firma.unvan, firma.vergiNo, fatura.no, fatura.tutar, fatura.tarih = 5
    expect(scalars).toHaveLength(5)

    const paths = scalars.map((s) => s.path)
    expect(paths).toContain('firma.unvan')
    expect(paths).toContain('firma.vergiNo')
    expect(paths).toContain('fatura.no')
    expect(paths).toContain('fatura.tutar')
    expect(paths).toContain('fatura.tarih')
  })

  it('does not include object or array nodes', () => {
    const tree = parseSchema(testSchema)
    const scalars = findScalarFields(tree)
    const types = scalars.map((s) => s.type)

    expect(types).not.toContain('object')
    expect(types).not.toContain('array')
  })
})

describe('schemaFormatToFormatType', () => {
  it('maps known formats correctly', () => {
    expect(schemaFormatToFormatType('currency')).toBe('currency')
    expect(schemaFormatToFormatType('date')).toBe('date')
    expect(schemaFormatToFormatType('percentage')).toBe('percentage')
  })

  it('returns undefined for unknown format', () => {
    expect(schemaFormatToFormatType('image')).toBeUndefined()
    expect(schemaFormatToFormatType('unknown')).toBeUndefined()
  })

  it('returns undefined for undefined input', () => {
    expect(schemaFormatToFormatType(undefined)).toBeUndefined()
  })
})

describe('defaultAlignForSchema', () => {
  it('returns right for number type', () => {
    const node: SchemaNode = { path: 'x', key: 'x', title: 'X', type: 'number', children: [] }
    expect(defaultAlignForSchema(node)).toBe('right')
  })

  it('returns right for integer type', () => {
    const node: SchemaNode = { path: 'x', key: 'x', title: 'X', type: 'integer', children: [] }
    expect(defaultAlignForSchema(node)).toBe('right')
  })

  it('returns right for currency format', () => {
    const node: SchemaNode = {
      path: 'x',
      key: 'x',
      title: 'X',
      type: 'string',
      format: 'currency',
      children: [],
    }
    expect(defaultAlignForSchema(node)).toBe('right')
  })

  it('returns right for percentage format', () => {
    const node: SchemaNode = {
      path: 'x',
      key: 'x',
      title: 'X',
      type: 'string',
      format: 'percentage',
      children: [],
    }
    expect(defaultAlignForSchema(node)).toBe('right')
  })

  it('returns center for date format', () => {
    const node: SchemaNode = {
      path: 'x',
      key: 'x',
      title: 'X',
      type: 'string',
      format: 'date',
      children: [],
    }
    expect(defaultAlignForSchema(node)).toBe('center')
  })

  it('returns left for plain string', () => {
    const node: SchemaNode = { path: 'x', key: 'x', title: 'X', type: 'string', children: [] }
    expect(defaultAlignForSchema(node)).toBe('left')
  })
})
