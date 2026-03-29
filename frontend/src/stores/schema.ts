import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { JsonSchema, SchemaNode } from '../core/schema-parser'
import { parseSchema, findArrayFields, findScalarFields } from '../core/schema-parser'

/** Minimal boş schema — gerçek schema dışarıdan (DreportEditor prop) gelir */
const emptySchema: JsonSchema = {
  type: 'object',
  properties: {},
}

export const useSchemaStore = defineStore('schema', () => {
  const rawSchema = ref<JsonSchema>(emptySchema)

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
