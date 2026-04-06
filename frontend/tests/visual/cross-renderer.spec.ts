/**
 * Cross-renderer visual test: compares HTML render (browser) vs PDF render (Rust).
 *
 * Both renderers consume the same LayoutResult from the same layout engine.
 * This test verifies that the visual output is consistent between:
 *   - LayoutRenderer.vue (HTML divs in browser)
 *   - pdf_render.rs → pdftoppm (PDF rasterized to PNG)
 *
 * Prerequisites:
 *   cargo test -p dreport-layout --test visual_test -- generate_cross_renderer --ignored
 *   (or: just visual-refs)
 */

import { test, expect } from '@playwright/test'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { PNG } from 'pngjs'
import pixelmatch from 'pixelmatch'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const FIXTURES_DIR = path.resolve(__dirname, '../../../layout-engine/tests/fixtures')
const REFS_DIR = path.resolve(__dirname, 'cross-renderer-refs')
const DIFFS_DIR = path.resolve(__dirname, 'cross-renderer-diffs')

interface TestFixture {
  name: string
  templateFile: string
  dataFile: string
  /** Max allowed pixel diff ratio (0.0 – 1.0) */
  maxDiffRatio: number
}

const FIXTURES: TestFixture[] = [
  {
    name: 'visual_test',
    templateFile: 'visual_test_template.json',
    dataFile: 'visual_test_data.json',
    // Text rendering differences between browser and PDF are expected.
    // Font hinting, anti-aliasing, and line-height handling differ.
    maxDiffRatio: 0.05,
  },
  {
    name: 'chart_test',
    templateFile: 'chart_test_template.json',
    dataFile: 'chart_test_data.json',
    // Charts have more visual complexity (SVG vs PDF primitives)
    maxDiffRatio: 0.08,
  },
  {
    name: 'comprehensive_test',
    templateFile: 'comprehensive_test_template.json',
    dataFile: 'comprehensive_test_data.json',
    // All element types: text, rich_text, table, charts, barcodes, shapes,
    // checkboxes, calculated_text, current_date, page_number, lines.
    // Barcodes render differently (canvas vs PDF) so higher tolerance.
    maxDiffRatio: 0.08,
  },
]

test.describe('Cross-renderer: HTML vs PDF', () => {
  test.beforeAll(() => {
    fs.mkdirSync(DIFFS_DIR, { recursive: true })
  })

  for (const fixture of FIXTURES) {
    test(`${fixture.name}: HTML render matches PDF render`, async ({ page }) => {
      const refPath = path.join(REFS_DIR, `${fixture.name}.png`)
      if (!fs.existsSync(refPath)) {
        test.skip(true, `PDF reference not found: ${refPath}. Run: just visual-refs`)
        return
      }

      // Read fixture files
      const templateJson = fs.readFileSync(path.join(FIXTURES_DIR, fixture.templateFile), 'utf-8')
      const dataJson = fs.readFileSync(path.join(FIXTURES_DIR, fixture.dataFile), 'utf-8')

      // Inject fixture data before page loads
      await page.addInitScript((fixtureData: { template: string; data: string }) => {
        ;(window as any).__DREPORT_FIXTURE__ = fixtureData
      }, { template: templateJson, data: dataJson })

      // Navigate to render test page
      await page.goto('/render-test.html')

      // Wait for layout to compute and render
      await page.waitForSelector('[data-render-ready]', { timeout: 20000 })

      // Small delay for font rendering and any async canvas operations
      await page.waitForTimeout(500)

      // Screenshot the rendered page (first page only)
      const pageEl = page.locator('.layout-page').first()
      const htmlScreenshot = await pageEl.screenshot({ type: 'png' })

      // Load PDF reference PNG
      const refBuffer = fs.readFileSync(refPath)
      const refPng = PNG.sync.read(refBuffer)
      const htmlPng = PNG.sync.read(htmlScreenshot)

      // Handle dimension mismatch by resizing to the smaller of the two
      const width = Math.min(htmlPng.width, refPng.width)
      const height = Math.min(htmlPng.height, refPng.height)

      // Crop both images to common size
      const croppedHtml = cropPng(htmlPng, width, height)
      const croppedRef = cropPng(refPng, width, height)

      // Pixel comparison
      const diffPng = new PNG({ width, height })
      const numDiffPixels = pixelmatch(
        croppedHtml.data,
        croppedRef.data,
        diffPng.data,
        width,
        height,
        {
          threshold: 0.15, // Per-pixel color distance threshold
          alpha: 0.3,
          includeAA: false, // Ignore anti-aliasing differences
        },
      )

      const totalPixels = width * height
      const diffRatio = numDiffPixels / totalPixels

      // Save diff image for debugging
      const diffPath = path.join(DIFFS_DIR, `${fixture.name}_diff.png`)
      const htmlPath = path.join(DIFFS_DIR, `${fixture.name}_html.png`)
      fs.writeFileSync(diffPath, PNG.sync.write(diffPng))
      fs.writeFileSync(htmlPath, htmlScreenshot)

      // Dimension info
      const dimInfo = htmlPng.width !== refPng.width || htmlPng.height !== refPng.height
        ? ` (HTML: ${htmlPng.width}x${htmlPng.height}, PDF: ${refPng.width}x${refPng.height}, compared: ${width}x${height})`
        : ` (${width}x${height})`

      console.log(
        `[${fixture.name}] diff: ${(diffRatio * 100).toFixed(2)}% pixels${dimInfo}`,
      )

      expect(
        diffRatio,
        `Visual diff too large for ${fixture.name}: ${(diffRatio * 100).toFixed(2)}% pixels differ (max: ${(fixture.maxDiffRatio * 100).toFixed(0)}%). Check diff at: ${diffPath}`,
      ).toBeLessThanOrEqual(fixture.maxDiffRatio)
    })
  }
})

/** Crop a PNG to the given width and height (top-left origin) */
function cropPng(src: PNG, width: number, height: number): PNG {
  if (src.width === width && src.height === height) return src

  const cropped = new PNG({ width, height })
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const srcIdx = (y * src.width + x) * 4
      const dstIdx = (y * width + x) * 4
      cropped.data[dstIdx] = src.data[srcIdx]
      cropped.data[dstIdx + 1] = src.data[srcIdx + 1]
      cropped.data[dstIdx + 2] = src.data[srcIdx + 2]
      cropped.data[dstIdx + 3] = src.data[srcIdx + 3]
    }
  }
  return cropped
}
