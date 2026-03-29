/**
 * Layout data parsing — SVG'den element pozisyon bilgilerini çıkarır.
 * Template → Typst dönüşümü artık dreport-core WASM tarafından yapılır.
 */

export interface ElementLayout {
  x: number // pt
  y: number // pt
  width: number // pt
  height: number // pt
}

export function parseLayoutFromSvg(svgString: string): Record<string, ElementLayout> {
  const result: Record<string, ElementLayout> = {}
  const matches = svgString.matchAll(/([a-zA-Z0-9_-]+):([\d.]+)pt,([\d.]+)pt,([\d.]+)pt,([\d.]+)pt\|/g)
  for (const m of matches) {
    result[m[1]] = {
      x: parseFloat(m[2]),
      y: parseFloat(m[3]),
      width: parseFloat(m[4]),
      height: parseFloat(m[5]),
    }
  }
  return result
}
