<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import type { Template } from '../core/types'
import type { JsonSchema } from '../core/schema-parser'
import { useTemplateStore } from '../stores/template'
import { useSchemaStore } from '../stores/schema'
import { useEditorStore } from '../stores/editor'
import EditorCanvas from '../components/editor/EditorCanvas.vue'
import ToolboxPanel from '../components/panels/ToolboxPanel.vue'
import SchemaTreePanel from '../components/panels/SchemaTreePanel.vue'
import { setupTooltips } from '../directives/tip'
import PropertiesPanel from '../components/panels/PropertiesPanel.vue'

export interface DreportEditorConfig {
  apiBaseUrl?: string
}

const props = withDefaults(defineProps<{
  schema: JsonSchema
  modelValue: Template
  data?: Record<string, unknown>
  config?: DreportEditorConfig
  handleErrors?: boolean
}>(), {
  handleErrors: true,
})

const emit = defineEmits<{
  'update:modelValue': [value: Template]
  'compile-error': [error: string | null]
}>()

const leftTab = ref<'tools' | 'schema'>('tools')

const templateStore = useTemplateStore()
const schemaStore = useSchemaStore()
const editorStore = useEditorStore()

// --- Prop ↔ Store sync ---

let syncing = false

// Schema sync
onMounted(() => {
  schemaStore.setSchema(props.schema)
  syncing = true
  templateStore.template = JSON.parse(JSON.stringify(props.modelValue))
  nextTick(() => { syncing = false })
  templateStore.setOverrideData(props.data ?? null)
  setupTooltips()
})

watch(() => props.schema, (val) => {
  schemaStore.setSchema(val)
}, { deep: true })

watch(() => props.data, (val) => {
  templateStore.setOverrideData(val ?? null)
}, { deep: true })

// Template: prop → store (only on reference change from parent)
watch(() => props.modelValue, (val) => {
  if (syncing) return
  syncing = true
  templateStore.template = JSON.parse(JSON.stringify(val))
  nextTick(() => { syncing = false })
})

// Template: store → emit
watch(() => templateStore.template, (val) => {
  if (syncing) return
  syncing = true
  emit('update:modelValue', JSON.parse(JSON.stringify(val)))
  nextTick(() => { syncing = false })
}, { deep: true })

// --- Error forwarding ---

function onCompileError(error: string | null) {
  emit('compile-error', error)
}

// --- Keyboard shortcuts ---

function onKeyDown(e: KeyboardEvent) {
  const target = e.target as HTMLElement
  const tag = target?.tagName
  const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || target?.isContentEditable

  // Delete / Backspace
  if ((e.key === 'Delete' || e.key === 'Backspace') && editorStore.selectedElementId) {
    if (isInput) return
    e.preventDefault()
    const id = editorStore.selectedElementId
    if (id && id !== 'root') {
      editorStore.clearSelection()
      templateStore.removeElement(id)
    }
  }

  // Escape
  if (e.key === 'Escape') {
    editorStore.clearSelection()
  }

  // Ctrl+Z — undo
  if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
    e.preventDefault()
    templateStore.undo()
  }

  // Ctrl+Shift+Z — redo
  if ((e.ctrlKey || e.metaKey) && e.key === 'z' && e.shiftKey) {
    e.preventDefault()
    templateStore.redo()
  }
}

// Browser'ın native pinch-zoom'unu editör alanında engelle
function onGlobalWheel(e: WheelEvent) {
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown)
  // passive: false olmadan preventDefault çalışmaz
  document.addEventListener('wheel', onGlobalWheel, { passive: false })
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown)
  document.removeEventListener('wheel', onGlobalWheel)
})

// --- Exposed API ---

function getTemplate(): Template {
  return JSON.parse(JSON.stringify(templateStore.template))
}

function setTemplate(tpl: Template) {
  templateStore.template = JSON.parse(JSON.stringify(tpl))
}

function exportTemplate(): string {
  return templateStore.exportTemplate()
}

function importTemplate(json: string) {
  templateStore.importTemplate(json)
}

async function exportPdf(): Promise<Blob> {
  const baseUrl = props.config?.apiBaseUrl ?? '/api'
  const res = await fetch(`${baseUrl}/render`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      template: templateStore.template,
      data: templateStore.mockData,
    }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(`PDF olusturulamadi: ${text}`)
  }
  return res.blob()
}

defineExpose({
  getTemplate,
  setTemplate,
  exportTemplate,
  importTemplate,
  exportPdf,
  undo: () => templateStore.undo(),
  redo: () => templateStore.redo(),
  canUndo: templateStore.canUndo,
  canRedo: templateStore.canRedo,
})
</script>

<template>
  <div class="dreport-editor">
    <aside class="dreport-editor__sidebar dreport-editor__sidebar--left">
      <div class="sidebar-tabs">
        <button class="sidebar-tab" :class="{ 'sidebar-tab--active': leftTab === 'tools' }" @click="leftTab = 'tools'">Araclar</button>
        <button class="sidebar-tab" :class="{ 'sidebar-tab--active': leftTab === 'schema' }" @click="leftTab = 'schema'">Schema</button>
      </div>
      <ToolboxPanel v-if="leftTab === 'tools'" />
      <SchemaTreePanel v-else />
    </aside>
    <EditorCanvas :handle-errors="handleErrors" @compile-error="onCompileError" />
    <aside class="dreport-editor__sidebar dreport-editor__sidebar--right">
      <PropertiesPanel />
    </aside>
  </div>
</template>

<style scoped>
.dreport-editor {
  display: flex;
  flex: 1;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}

.dreport-editor__sidebar {
  width: 260px;
  background: #f8fafc;
  border-right: 1px solid #e2e8f0;
  flex-shrink: 0;
  overflow-y: auto;
}

.dreport-editor__sidebar--left {
  display: flex;
  flex-direction: column;
}

.dreport-editor__sidebar--right {
  border-right: none;
  border-left: 1px solid #e2e8f0;
}

.sidebar-tabs {
  display: flex;
  border-bottom: 1px solid #e2e8f0;
  flex-shrink: 0;
}

.sidebar-tab {
  flex: 1;
  padding: 8px 0;
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.sidebar-tab--active {
  color: #3b82f6;
  border-bottom-color: #3b82f6;
}

.sidebar-tab:hover:not(.sidebar-tab--active) {
  color: #64748b;
}
</style>
