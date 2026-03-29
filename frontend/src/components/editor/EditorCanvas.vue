<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { storeToRefs } from 'pinia'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { useTypstCompiler } from '../../composables/useTypstCompiler'
import TypstSvgLayer from './TypstSvgLayer.vue'
import InteractionOverlay from './InteractionOverlay.vue'

const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const { typstMarkup } = storeToRefs(templateStore)

const containerRef = ref<HTMLElement | null>(null)
const containerWidth = ref(800)

// Typst compiler
const { svg, error, compiling, layout, dispose } = useTypstCompiler(typstMarkup)

// mm → px dönüşüm katsayısı
const scale = computed(() => {
  return (containerWidth.value / templateStore.template.page.width) * editorStore.zoom
})

// Sayfa boyutu px cinsinden + margin CSS variables
const pageStyle = computed(() => {
  const w = templateStore.template.page.width * scale.value
  const h = templateStore.template.page.height * scale.value
  const m = templateStore.template.root.padding
  return {
    width: `${w}px`,
    height: `${h}px`,
    '--page-margin-top': `${m.top * scale.value}px`,
    '--page-margin-right': `${m.right * scale.value}px`,
    '--page-margin-bottom': `${m.bottom * scale.value}px`,
    '--page-margin-left': `${m.left * scale.value}px`,
  }
})

// Container boyutunu izle
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(entries => {
      const entry = entries[0]
      if (entry) containerWidth.value = entry.contentRect.width
    })
    resizeObserver.observe(containerRef.value)
  }
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  dispose()
})

// Zoom
function onWheel(e: WheelEvent) {
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault()
    const delta = e.deltaY > 0 ? -0.1 : 0.1
    editorStore.setZoom(editorStore.zoom + delta)
  }
}
</script>

<template>
  <div class="editor-canvas" ref="containerRef" @wheel="onWheel">
    <!-- Hata banner -->
    <div v-if="error" class="editor-canvas__error">
      {{ error }}
    </div>

    <!-- Derleme göstergesi -->
    <div v-if="compiling" class="editor-canvas__compiling">
      Derleniyor...
    </div>

    <!-- Sayfa -->
    <div class="editor-canvas__page" :style="pageStyle">
      <TypstSvgLayer :svg="svg" />
      <InteractionOverlay :scale="scale" :layout="layout" :page-width-pt="templateStore.template.page.width * 2.8346" />
    </div>

    <!-- Zoom göstergesi -->
    <div class="editor-canvas__zoom">
      %{{ editorStore.zoomPercent }}
    </div>
  </div>
</template>

<style scoped>
.editor-canvas {
  flex: 1;
  overflow: auto;
  background: #e5e7eb;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 40px;
  position: relative;
  min-height: 0;
}

.editor-canvas__page {
  background: white;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
  position: relative;
  flex-shrink: 0;
}

.editor-canvas__error {
  position: absolute;
  top: 8px;
  left: 50%;
  transform: translateX(-50%);
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
  border-radius: 6px;
  padding: 6px 16px;
  font-size: 13px;
  z-index: 100;
}

.editor-canvas__compiling {
  position: absolute;
  top: 8px;
  right: 16px;
  background: #eff6ff;
  color: #2563eb;
  border-radius: 6px;
  padding: 4px 12px;
  font-size: 12px;
  z-index: 100;
}

.editor-canvas__zoom {
  position: absolute;
  bottom: 12px;
  right: 16px;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 12px;
  z-index: 100;
}
</style>
