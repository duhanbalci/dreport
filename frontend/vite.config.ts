import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  worker: {
    format: 'es',
  },
  server: {
    proxy: {
      '/api': 'http://localhost:3001',
    },
  },
})
