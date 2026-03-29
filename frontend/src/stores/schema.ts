import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { JsonSchema, SchemaNode } from '../core/schema-parser'
import { parseSchema, findArrayFields, findScalarFields } from '../core/schema-parser'

/** Varsayılan fatura schema'sı */
const defaultSchema: JsonSchema = {
  $id: 'fatura-schema',
  type: 'object',
  properties: {
    firma: {
      type: 'object',
      title: 'Firma',
      properties: {
        unvan: { type: 'string', title: 'Firma Unvani' },
        vergiNo: { type: 'string', title: 'Vergi No' },
        logo: { type: 'string', title: 'Logo', format: 'image' },
        adres: { type: 'string', title: 'Adres' },
        telefon: { type: 'string', title: 'Telefon' },
      },
    },
    fatura: {
      type: 'object',
      title: 'Fatura',
      properties: {
        no: { type: 'string', title: 'Fatura No' },
        tarih: { type: 'string', title: 'Tarih', format: 'date' },
      },
    },
    musteri: {
      type: 'object',
      title: 'Musteri',
      properties: {
        unvan: { type: 'string', title: 'Musteri Unvani' },
        vergiNo: { type: 'string', title: 'Vergi No' },
        adres: { type: 'string', title: 'Adres' },
      },
    },
    kalemler: {
      type: 'array',
      title: 'Fatura Kalemleri',
      items: {
        type: 'object',
        properties: {
          siraNo: { type: 'integer', title: 'Sira No' },
          adi: { type: 'string', title: 'Urun / Hizmet Adi' },
          miktar: { type: 'number', title: 'Miktar' },
          birim: { type: 'string', title: 'Birim' },
          birimFiyat: { type: 'number', title: 'Birim Fiyat', format: 'currency' },
          tutar: { type: 'number', title: 'Tutar', format: 'currency' },
        },
      },
    },
    toplamlar: {
      type: 'object',
      title: 'Toplamlar',
      properties: {
        araToplam: { type: 'number', title: 'Ara Toplam', format: 'currency' },
        kdv: { type: 'number', title: 'KDV', format: 'currency' },
        genelToplam: { type: 'number', title: 'Genel Toplam', format: 'currency' },
      },
    },
  },
}

export const useSchemaStore = defineStore('schema', () => {
  const rawSchema = ref<JsonSchema>(defaultSchema)

  const schemaTree = computed<SchemaNode>(() => parseSchema(rawSchema.value))

  /** Tüm array alanları (tablo binding için) */
  const arrayFields = computed(() => findArrayFields(schemaTree.value))

  /** Tüm scalar alanlar (metin binding için) */
  const scalarFields = computed(() => findScalarFields(schemaTree.value))

  /** Schema'yı güncelle (ör: JSON import) */
  function setSchema(schema: JsonSchema) {
    rawSchema.value = schema
  }

  /** Belirli bir path'teki SchemaNode'u bul */
  function getNodeByPath(path: string): SchemaNode | undefined {
    return findNode(schemaTree.value, path)
  }

  /** Bir array alanının item property'lerini getir */
  function getArrayItemFields(arrayPath: string): SchemaNode[] {
    const node = findNode(schemaTree.value, arrayPath)
    return node?.itemProperties ?? []
  }

  return {
    rawSchema,
    schemaTree,
    arrayFields,
    scalarFields,
    setSchema,
    getNodeByPath,
    getArrayItemFields,
  }
})

function findNode(node: SchemaNode, path: string): SchemaNode | undefined {
  if (node.path === path) return node
  for (const child of node.children) {
    const found = findNode(child, path)
    if (found) return found
  }
  return undefined
}
