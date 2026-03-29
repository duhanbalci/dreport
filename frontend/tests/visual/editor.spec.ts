import { test, expect } from '@playwright/test'

test.describe('Editor Visual Tests', () => {
  test('full editor renders correctly', async ({ page }) => {
    await page.goto('/')

    // Wait for the editor to fully load (WASM + layout)
    await page.waitForSelector('.dreport-editor', { timeout: 15000 })

    // Wait for layout to render (layout renderer should have elements)
    await page.waitForSelector('.layout-renderer div[style]', { timeout: 10000 })

    // Small delay for any CSS transitions
    await page.waitForTimeout(500)

    // Screenshot the full editor area
    await expect(page).toHaveScreenshot('editor-full.png', {
      maxDiffPixelRatio: 0.02,
    })
  })

  test('editor canvas renders template', async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.dreport-editor', { timeout: 15000 })
    await page.waitForSelector('.layout-renderer div[style]', { timeout: 10000 })
    await page.waitForTimeout(500)

    // Screenshot just the canvas area
    const canvas = page.locator('.editor-canvas-wrapper')
    await expect(canvas).toHaveScreenshot('editor-canvas.png', {
      maxDiffPixelRatio: 0.02,
    })
  })

  test('toolbox panel renders correctly', async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.toolbox-panel', { timeout: 15000 })

    const toolbox = page.locator('.toolbox-panel')
    await expect(toolbox).toHaveScreenshot('toolbox-panel.png')
  })

  test('properties panel shows on element selection', async ({ page }) => {
    await page.goto('/')
    await page.waitForSelector('.dreport-editor', { timeout: 15000 })
    await page.waitForSelector('.layout-renderer div[style]', { timeout: 10000 })
    await page.waitForTimeout(500)

    // Click on an element in the editor to select it
    // The interaction overlay has clickable elements positioned absolutely
    const overlay = page.locator('.interaction-overlay')
    // Click approximately in the center-top area where the header text should be
    await overlay.click({ position: { x: 300, y: 50 } })

    await page.waitForTimeout(300)

    // Screenshot the properties panel (right sidebar)
    const sidebar = page.locator('.dreport-editor__sidebar--right')
    await expect(sidebar).toHaveScreenshot('properties-panel-selected.png', {
      maxDiffPixelRatio: 0.02,
    })
  })
})
