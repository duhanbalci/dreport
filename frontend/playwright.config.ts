import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './tests/visual',
  outputDir: './tests/visual/test-results',
  use: {
    baseURL: 'http://localhost:5173',
    viewport: { width: 1400, height: 900 },
  },
  webServer: {
    command: 'bun run dev',
    port: 5173,
    reuseExistingServer: true,
    timeout: 30000,
  },
  expect: {
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.01,
    },
  },
})
