import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    dedupe: [
      '@codemirror/state',
      '@codemirror/view',
      '@codemirror/language',
      '@codemirror/autocomplete',
      '@lezer/common',
      '@lezer/lr',
      '@lezer/highlight',
    ],
  },
  worker: {
    format: 'es',
  },
  server: {
    host: '0.0.0.0',
    proxy: {
      '/api': 'http://localhost:3001',
    },
  },
})
