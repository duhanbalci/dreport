<script setup lang="ts">
import { ref } from 'vue'
import type { SchemaNode } from '../../core/schema-parser'
import { schemaFormatToFormatType, defaultAlignForSchema } from '../../core/schema-parser'
import type { TemplateElement, RepeatingTableElement, TableColumn } from '../../core/types'
import { sz } from '../../core/types'
import { useEditorStore } from '../../stores/editor'
import { useSchemaStore } from '../../stores/schema'

const props = withDefaults(defineProps<{
  node: SchemaNode
  depth?: number
}>(), {
  depth: 0,
})

const editorStore = useEditorStore()
const schemaStore = useSchemaStore()

const expanded = ref(props.depth < 2)

let colIdCounter = 0

const isScalar = ['string', 'number', 'integer', 'boolean'].includes(props.node.type)
const isArray = props.node.type === 'array'
const isObject = props.node.type === 'object'
const isDraggable = isScalar || isArray
const hasChildren = isObject
  ? props.node.children.length > 0
  : isArray
    ? (props.node.itemProperties?.length ?? 0) > 0
    : false

const typeIcon: Record<string, string> = {
  string: 'Aa',
  number: '#',
  integer: '#',
  boolean: '\u2713',
  object: '{ }',
  array: '[ ]',
}

const borderColor: Record<string, string> = {
  string: '#3b82f6',
  number: '#22c55e',
  integer: '#22c55e',
  boolean: '#f59e0b',
  object: '#94a3b8',
  array: '#8b5cf6',
}

function toggle() {
  if (hasChildren) {
    expanded.value = !expanded.value
  }
}

function createBoundTextElement(node: SchemaNode): TemplateElement {
  return {
    id: `txt_${Date.now().toString(36)}`,
    type: 'text',
    position: { type: 'flow' },
    size: { width: sz.auto(), height: sz.auto() },
    style: { fontSize: 11, color: '#000000' },
    binding: { type: 'scalar', path: node.path },
  }
}

function createBoundTableElement(node: SchemaNode): RepeatingTableElement {
  const itemFields = schemaStore.getArrayItemFields(node.path)
  const columns: TableColumn[] = itemFields.map(field => ({
    id: `col_${(++colIdCounter).toString(36)}`,
    field: field.key,
    title: field.title,
    width: sz.auto(),
    align: defaultAlignForSchema(field),
    format: schemaFormatToFormatType(field.format),
  }))
  return {
    id: `tbl_${Date.now().toString(36)}`,
    type: 'repeating_table',
    position: { type: 'flow' },
    size: { width: sz.fr(1), height: sz.auto() },
    dataSource: { type: 'array', path: node.path },
    columns,
    style: { headerBg: '#f0f0f0', headerColor: '#000000', fontSize: 10, headerFontSize: 10 },
  }
}

function onDragStart(e: DragEvent) {
  if (!isDraggable) return

  let el: TemplateElement
  if (isScalar) {
    el = createBoundTextElement(props.node)
  } else {
    el = createBoundTableElement(props.node)
  }

  editorStore.startDragNewElement(el)
  e.dataTransfer?.setData('text/plain', el.id)
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'copy'
  }
}

function onDragEnd() {
  editorStore.endDragNewElement()
}

const displayChildren = isArray
  ? (props.node.itemProperties ?? [])
  : props.node.children
</script>

<template>
  <div class="schema-node">
    <div
      class="schema-node__row"
      :class="{
        'schema-node__row--draggable': isDraggable,
        'schema-node__row--object': isObject,
      }"
      :style="{
        paddingLeft: `${depth * 16 + 8}px`,
        borderLeftColor: borderColor[node.type] ?? '#94a3b8',
      }"
      :draggable="isDraggable"
      :title="node.path || node.key"
      @click="toggle"
      @dragstart="onDragStart"
      @dragend="onDragEnd"
    >
      <span v-if="hasChildren" class="schema-node__arrow" :class="{ 'schema-node__arrow--expanded': expanded }">
        &#9654;
      </span>
      <span v-else class="schema-node__arrow-placeholder" />

      <span class="schema-node__type-icon" :class="`schema-node__type-icon--${node.type}`">
        {{ typeIcon[node.type] ?? '?' }}
      </span>

      <span class="schema-node__title">{{ node.title }}</span>

      <span v-if="isScalar && node.path" class="schema-node__path">{{ node.path }}</span>
    </div>

    <div v-if="hasChildren && expanded" class="schema-node__children">
      <SchemaTreeNode
        v-for="child in displayChildren"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
      />
    </div>
  </div>
</template>

<style scoped>
.schema-node__row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-left: 3px solid transparent;
  cursor: default;
  user-select: none;
  font-size: 13px;
  color: #334155;
  border-radius: 0 4px 4px 0;
  transition: background 0.12s;
}

.schema-node__row--draggable {
  cursor: grab;
}

.schema-node__row--draggable:active {
  cursor: grabbing;
}

.schema-node__row:hover {
  background: #f1f5f9;
}

.schema-node__row--draggable:hover {
  background: #eff6ff;
}

.schema-node__arrow {
  width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  color: #94a3b8;
  transition: transform 0.15s;
  flex-shrink: 0;
}

.schema-node__arrow--expanded {
  transform: rotate(90deg);
}

.schema-node__arrow-placeholder {
  width: 14px;
  flex-shrink: 0;
}

.schema-node__type-icon {
  width: 22px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 700;
  flex-shrink: 0;
  font-family: monospace;
}

.schema-node__type-icon--string {
  background: #dbeafe;
  color: #2563eb;
}

.schema-node__type-icon--number,
.schema-node__type-icon--integer {
  background: #dcfce7;
  color: #16a34a;
}

.schema-node__type-icon--boolean {
  background: #fef3c7;
  color: #d97706;
}

.schema-node__type-icon--object {
  background: #f1f5f9;
  color: #64748b;
}

.schema-node__type-icon--array {
  background: #ede9fe;
  color: #7c3aed;
}

.schema-node__title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.schema-node__path {
  font-size: 10px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 80px;
}
</style>
