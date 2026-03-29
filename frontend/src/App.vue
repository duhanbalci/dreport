<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import EditorCanvas from './components/editor/EditorCanvas.vue'
import ToolboxPanel from './components/panels/ToolboxPanel.vue'
import PropertiesPanel from './components/panels/PropertiesPanel.vue'
import { useTemplateStore } from './stores/template'
import { useEditorStore } from './stores/editor'

const templateStore = useTemplateStore()
const editorStore = useEditorStore()

const pdfLoading = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)

function exportTemplate() {
  const json = templateStore.exportTemplate()
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${templateStore.template.name || 'sablon'}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function triggerImport() {
  fileInputRef.value?.click()
}

function onImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    try {
      templateStore.importTemplate(reader.result as string)
    } catch {
      alert('Gecersiz sablon dosyasi')
    }
  }
  reader.readAsText(file)
  input.value = ''
}

async function downloadPdf() {
  pdfLoading.value = true
  try {
    const res = await fetch('http://localhost:3001/api/render', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        template: templateStore.template,
        data: templateStore.mockData,
      }),
    })
    if (!res.ok) {
      const text = await res.text()
      alert('PDF olusturulamadi: ' + text)
      return
    }
    const blob = await res.blob()
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${templateStore.template.name || 'belge'}.pdf`
    a.click()
    URL.revokeObjectURL(url)
  } catch (err) {
    alert('Backend baglantisi kurulamadi. Sunucu calisiyor mu?')
  } finally {
    pdfLoading.value = false
  }
}

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
      <div style="flex: 1"></div>
      <input ref="fileInputRef" type="file" accept=".json" style="display: none" @change="onImportFile" />
      <button class="header-btn header-btn--secondary" @click="triggerImport">Yukle</button>
      <button class="header-btn header-btn--secondary" @click="exportTemplate">Kaydet</button>
      <button class="header-btn" :disabled="pdfLoading" @click="downloadPdf">
        {{ pdfLoading ? 'Hazirlaniyor...' : 'PDF Indir' }}
      </button>
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

.header-btn {
  padding: 6px 16px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s;
}

.header-btn:hover:not(:disabled) {
  background: #2563eb;
}

.header-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.header-btn--secondary {
  background: transparent;
  border: 1px solid #475569;
  color: #cbd5e1;
}

.header-btn--secondary:hover {
  background: #334155;
  color: white;
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
