<script setup lang="ts">
import { ref, provide, onMounted } from 'vue'
import LayoutRenderer from '../components/editor/LayoutRenderer.vue'
import { useLayoutEngine } from '../composables/useLayoutEngine'
import type { Template } from '../core/types'

// Fixture data is injected by Playwright via window.__DREPORT_FIXTURE__
declare global {
  interface Window {
    __DREPORT_FIXTURE__?: { template: string; data: string }
  }
}

// DPI matching: 150 DPI = 150/25.4 px/mm (must match pdftoppm -r 150)
const DPI = 150
const SCALE = DPI / 25.4

const ready = ref(false)
const errorMsg = ref<string | null>(null)

const template = ref<Template>({
  id: 'empty',
  name: 'empty',
  page: { width: 210, height: 297 },
  fonts: ['Noto Sans'],
  root: {
    id: 'root',
    type: 'container',
    position: { type: 'flow' },
    size: { width: { type: 'auto' }, height: { type: 'auto' } },
    direction: 'column',
    gap: 0,
    padding: { top: 0, right: 0, bottom: 0, left: 0 },
    align: 'stretch',
    justify: 'start',
    style: {},
    children: [],
  },
})
const data = ref<Record<string, unknown>>({})

const { layout, generateBarcode, error } = useLayoutEngine(template, data)

// Provide barcode generator for LayoutRenderer
provide('generateBarcode', generateBarcode)

// Watch for layout computation to complete
const checkReady = setInterval(() => {
  if (layout.value && layout.value.pages.length > 0) {
    ready.value = true
    clearInterval(checkReady)
  }
  if (error.value) {
    errorMsg.value = error.value
    clearInterval(checkReady)
  }
}, 50)

// Timeout after 20 seconds
setTimeout(() => {
  clearInterval(checkReady)
  if (!ready.value) {
    errorMsg.value = 'Layout computation timed out'
  }
}, 20000)

onMounted(() => {
  const fixture = window.__DREPORT_FIXTURE__
  if (!fixture) {
    errorMsg.value = 'No fixture data found. Set window.__DREPORT_FIXTURE__ = { template, data }'
    return
  }
  try {
    template.value = JSON.parse(fixture.template)
    data.value = JSON.parse(fixture.data)
  } catch (e) {
    errorMsg.value = `Failed to parse fixture data: ${e}`
  }
})
</script>

<template>
  <div
    class="render-test-root"
    :data-render-ready="ready || undefined"
    :data-render-error="errorMsg || undefined"
  >
    <LayoutRenderer v-if="layout" :layout="layout" :scale="SCALE" />
    <div v-else-if="errorMsg" class="error">{{ errorMsg }}</div>
    <div v-else class="loading">Computing layout...</div>
  </div>
</template>

<style>
.render-test-root {
  /* No padding/margin — pixel-exact rendering */
  background: white;
}

.error {
  color: red;
  padding: 20px;
  font-family: monospace;
}

.loading {
  color: #999;
  padding: 20px;
  font-family: monospace;
}

/* Override layout-page styles for test — no shadow, no margin */
.layout-page {
  box-shadow: none !important;
  margin: 0 !important;
}

.layout-page + .layout-page {
  margin-top: 0 !important;
}
</style>
