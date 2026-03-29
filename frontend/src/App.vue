<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'
import EditorCanvas from './components/editor/EditorCanvas.vue'
import ToolboxPanel from './components/panels/ToolboxPanel.vue'
import PropertiesPanel from './components/panels/PropertiesPanel.vue'
import { useTemplateStore } from './stores/template'
import { useEditorStore } from './stores/editor'

const templateStore = useTemplateStore()
const editorStore = useEditorStore()

function onKeyDown(e: KeyboardEvent) {
  // Delete / Backspace — seçili elemanı sil
  if ((e.key === 'Delete' || e.key === 'Backspace') && editorStore.selectedElementId) {
    // Input/textarea içindeyse yoksay
    const tag = (e.target as HTMLElement)?.tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return

    e.preventDefault()
    const id = editorStore.selectedElementId
    if (id && id !== 'root') {
      editorStore.clearSelection()
      templateStore.removeElement(id)
    }
  }

  // Escape — seçimi temizle
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

onMounted(() => window.addEventListener('keydown', onKeyDown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeyDown))
</script>

<template>
  <div class="app-layout">
    <header class="app-header">
      <h1>dreport</h1>
      <span class="app-header__subtitle">Belge Tasarim Araci</span>
    </header>
    <main class="app-main">
      <aside class="app-sidebar app-sidebar--left">
        <ToolboxPanel />
      </aside>
      <EditorCanvas />
      <aside class="app-sidebar app-sidebar--right">
        <PropertiesPanel />
      </aside>
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.app-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  padding: 8px 16px;
  background: #1e293b;
  color: white;
  flex-shrink: 0;
}

.app-header h1 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.app-header__subtitle {
  font-size: 13px;
  color: #94a3b8;
}

.app-main {
  display: flex;
  flex: 1;
  min-height: 0;
}

.app-sidebar {
  width: 260px;
  background: #f8fafc;
  border-right: 1px solid #e2e8f0;
  flex-shrink: 0;
  overflow-y: auto;
}

.app-sidebar--right {
  border-right: none;
  border-left: 1px solid #e2e8f0;
}
</style>
