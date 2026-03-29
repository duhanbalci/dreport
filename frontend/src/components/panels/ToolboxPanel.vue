<script setup lang="ts">
import { useEditorStore } from '../../stores/editor'
import type { TemplateElement } from '../../core/types'
import { sz } from '../../core/types'

const editorStore = useEditorStore()

let idCounter = Date.now()
function nextId(prefix: string) {
  return `${prefix}_${(++idCounter).toString(36)}`
}

interface ToolItem {
  label: string
  icon: string
  create: () => TemplateElement
}

const tools: ToolItem[] = [
  {
    label: 'Metin',
    icon: 'T',
    create: () => ({
      id: nextId('txt'),
      type: 'static_text',
      position: { type: 'flow' },
      size: { width: sz.auto(), height: sz.auto() },
      style: { fontSize: 11, color: '#000000' },
      content: 'Yeni metin',
    }),
  },
  {
    label: 'Container',
    icon: '▢',
    create: () => ({
      id: nextId('cnt'),
      type: 'container',
      position: { type: 'flow' },
      size: { width: sz.fr(1), height: sz.auto() },
      direction: 'column' as const,
      gap: 3,
      padding: { top: 5, right: 5, bottom: 5, left: 5 },
      align: 'stretch' as const,
      justify: 'start' as const,
      style: {},
      children: [],
    }),
  },
  {
    label: 'Cizgi',
    icon: '—',
    create: () => ({
      id: nextId('ln'),
      type: 'line',
      position: { type: 'flow' },
      size: { width: sz.fr(1), height: sz.auto() },
      style: { strokeColor: '#000000', strokeWidth: 0.5 },
    }),
  },
]

function onDragStart(e: DragEvent, tool: ToolItem) {
  const el = tool.create()
  editorStore.startDragNewElement(el)
  // Drag data (fallback)
  e.dataTransfer?.setData('text/plain', el.id)
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'copy'
  }
}

function onDragEnd() {
  editorStore.endDragNewElement()
}
</script>

<template>
  <div class="toolbox-panel">
    <div class="toolbox-panel__title">Arac Kutusu</div>
    <div class="toolbox-panel__grid">
      <div
        v-for="tool in tools"
        :key="tool.label"
        class="toolbox-item"
        draggable="true"
        @dragstart="(e) => onDragStart(e, tool)"
        @dragend="onDragEnd"
      >
        <span class="toolbox-item__icon">{{ tool.icon }}</span>
        <span class="toolbox-item__label">{{ tool.label }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.toolbox-panel {
  padding: 12px;
}

.toolbox-panel__title {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 10px;
}

.toolbox-panel__grid {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.toolbox-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: white;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  cursor: grab;
  font-size: 13px;
  color: #334155;
  transition: all 0.15s;
  user-select: none;
}

.toolbox-item:hover {
  background: #eff6ff;
  border-color: #bfdbfe;
}

.toolbox-item:active {
  cursor: grabbing;
}

.toolbox-item__icon {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f1f5f9;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 600;
  color: #475569;
}

.toolbox-item__label {
  font-size: 13px;
}
</style>
