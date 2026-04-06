import type { Template, TemplateElement, RepeatingTableElement } from './types'
import { isContainer } from './types'

/**
 * Template'teki binding'lerden mock veri üretir.
 * Scalar binding → placeholder metin, Array binding → 3 satır örnek veri.
 */
export function generateMockData(template: Template): Record<string, unknown> {
  const data: Record<string, unknown> = {}
  collectBindings(template.root, data)
  return data
}

function collectBindings(el: TemplateElement, data: Record<string, unknown>) {
  if (el.type === 'text' && el.binding) {
    setNestedValue(data, el.binding.path, mockScalarValue(el.binding.path))
  }

  if (el.type === 'barcode' && el.binding) {
    setNestedValue(data, el.binding.path, mockScalarValue(el.binding.path))
  }

  if (el.type === 'repeating_table' && el.dataSource && el.dataSource.path) {
    const rows = generateMockRows(el)
    setNestedValue(data, el.dataSource.path, rows)
  }

  if (isContainer(el)) {
    for (const child of el.children) {
      collectBindings(child, data)
    }
  }
}

function generateMockRows(el: RepeatingTableElement): Record<string, unknown>[] {
  const rowCount = 3
  const rows: Record<string, unknown>[] = []

  for (let i = 0; i < rowCount; i++) {
    const row: Record<string, unknown> = {}
    for (const col of el.columns) {
      row[col.field] = mockColumnValue(col.field, col.format, i)
    }
    rows.push(row)
  }

  return rows
}

function mockColumnValue(field: string, format: string | undefined, index: number): unknown {
  if (format === 'currency') return [1500, 2750, 500][index % 3]
  if (format === 'date') return ['2026-01-15', '2026-02-20', '2026-03-10'][index % 3]
  if (format === 'percentage') return [18, 8, 20][index % 3]
  if (format === 'number') return [1, 2, 3][index % 3]

  // Alan adına göre tahmin
  const lower = field.toLowerCase()
  if (lower.includes('sira') || lower.includes('no') || lower === 'id') return index + 1
  if (lower.includes('miktar') || lower.includes('adet')) return [2, 1, 5][index % 3]
  if (lower.includes('fiyat') || lower.includes('tutar') || lower.includes('toplam'))
    return [1500, 2750, 500][index % 3]
  if (lower.includes('birim')) return ['Adet', 'Saat', 'Adet'][index % 3]
  if (lower.includes('tarih') || lower.includes('date'))
    return ['2026-01-15', '2026-02-20', '2026-03-10'][index % 3]
  if (lower.includes('ad') || lower.includes('isim') || lower.includes('name'))
    return ['Kalem A', 'Kalem B', 'Kalem C'][index % 3]

  return `Ornek ${index + 1}`
}

function mockScalarValue(path: string): string {
  const last = path.split('.').pop() ?? path
  const lower = last.toLowerCase()

  if (lower.includes('unvan') || lower.includes('firma')) return 'Ornek Firma A.S.'
  if (lower.includes('vergi')) return '1234567890'
  if (lower.includes('tarih') || lower.includes('date')) return '2026-03-29'
  if (lower.includes('no') || lower.includes('numara')) return 'FTR-2026-001'
  if (lower.includes('toplam') || lower.includes('tutar')) return '18.880,00'
  if (lower.includes('adres')) return 'Ornek Mah. Test Sk. No:1'
  if (lower.includes('tel') || lower.includes('phone')) return '0212 555 0000'
  if (lower.includes('mail') || lower.includes('email')) return 'info@ornek.com'

  return `[${path}]`
}

/** Noktalı yolu kullanarak nested objeye değer atar */
function setNestedValue(obj: Record<string, unknown>, path: string, value: unknown) {
  const parts = path.split('.')
  let current: Record<string, unknown> = obj

  for (let i = 0; i < parts.length - 1; i++) {
    const key = parts[i]
    if (!(key in current) || typeof current[key] !== 'object' || current[key] === null) {
      current[key] = {}
    }
    current = current[key] as Record<string, unknown>
  }

  const lastKey = parts[parts.length - 1]
  // Mevcut değeri override etme (ilk binding kazanır)
  if (!(lastKey in current)) {
    current[lastKey] = value
  }
}
