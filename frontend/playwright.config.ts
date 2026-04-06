import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './tests/visual',
  outputDir: './tests/visual/test-results',
  use: {
    baseURL: 'http://localhost:5173',
    viewport: { width: 1400, height: 900 },
    // Disable HiDPI scaling for pixel-exact comparison
    deviceScaleFactor: 1,
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
  projects: [
    {
      name: 'editor',
      testMatch: 'editor.spec.ts',
    },
    {
      name: 'cross-renderer',
      testMatch: 'cross-renderer.spec.ts',
      use: {
        // Render test page needs larger viewport for A4 at 150 DPI
        // A4 = 210mm x 297mm → 1240 x 1754 px at 150 DPI
        viewport: { width: 1300, height: 1800 },
      },
    },
  ],
})
