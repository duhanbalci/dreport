import { describe, it, expect } from 'vitest'
import { generateMockData } from '../mock-data-generator'
import type { Template, ContainerElement } from '../types'
import { sz } from '../types'

function makeTemplate(root: ContainerElement): Template {
  return {
    id: 'test',
    name: 'Test',
    page: { width: 210, height: 297 },
    fonts: ['Noto Sans'],
    root,
  }
}

function makeRoot(children: ContainerElement['children']): ContainerElement {
  return {
    id: 'root',
    type: 'container',
    position: { type: 'flow' },
    size: { width: sz.auto(), height: sz.auto() },
    direction: 'column',
    gap: 0,
    padding: { top: 0, right: 0, bottom: 0, left: 0 },
    align: 'stretch',
    justify: 'start',
    style: {},
    children,
  }
}

describe('generateMockData', () => {
  it('generates scalar data for text elements with bindings', () => {
    const template = makeTemplate(
      makeRoot([
        {
          id: 'el1',
          type: 'text',
          position: { type: 'flow' },
          size: { width: sz.auto(), height: sz.auto() },
          style: {},
          binding: { type: 'scalar', path: 'firma.unvan' },
        },
      ]),
    )

    const data = generateMockData(template)
    expect(data).toHaveProperty('firma')
    expect((data.firma as Record<string, unknown>).unvan).toBe('Ornek Firma A.S.')
  })

  it('generates array data for repeating_table elements', () => {
    const template = makeTemplate(
      makeRoot([
        {
          id: 'tbl1',
          type: 'repeating_table',
          position: { type: 'flow' },
          size: { width: sz.auto(), height: sz.auto() },
          dataSource: { type: 'array', path: 'kalemler' },
          columns: [
            { id: 'c1', field: 'adi', title: 'Adi', width: sz.fr(), align: 'left' },
            { id: 'c2', field: 'miktar', title: 'Miktar', width: sz.fr(), align: 'right' },
          ],
          style: {},
        },
      ]),
    )

    const data = generateMockData(template)
    const kalemler = data.kalemler as Record<string, unknown>[]
    expect(kalemler).toHaveLength(3)
    expect(kalemler[0]).toHaveProperty('adi')
    expect(kalemler[0]).toHaveProperty('miktar')
  })

  it('handles nested paths correctly', () => {
    const template = makeTemplate(
      makeRoot([
        {
          id: 'el1',
          type: 'text',
          position: { type: 'flow' },
          size: { width: sz.auto(), height: sz.auto() },
          style: {},
          binding: { type: 'scalar', path: 'a.b.c' },
        },
      ]),
    )

    const data = generateMockData(template)
    expect((data as any).a.b.c).toBe('[a.b.c]')
  })

  it('returns empty object for template with no bindings', () => {
    const template = makeTemplate(
      makeRoot([
        {
          id: 'el1',
          type: 'static_text',
          position: { type: 'flow' },
          size: { width: sz.auto(), height: sz.auto() },
          style: {},
          content: 'Hello',
        },
      ]),
    )

    const data = generateMockData(template)
    expect(Object.keys(data)).toHaveLength(0)
  })

  it('traverses nested containers to find bindings', () => {
    const template = makeTemplate(
      makeRoot([
        {
          id: 'inner',
          type: 'container',
          position: { type: 'flow' },
          size: { width: sz.auto(), height: sz.auto() },
          direction: 'row',
          gap: 0,
          padding: { top: 0, right: 0, bottom: 0, left: 0 },
          align: 'stretch',
          justify: 'start',
          style: {},
          children: [
            {
              id: 'el_deep',
              type: 'text',
              position: { type: 'flow' },
              size: { width: sz.auto(), height: sz.auto() },
              style: {},
              binding: { type: 'scalar', path: 'fatura.no' },
            },
          ],
        },
      ]),
    )

    const data = generateMockData(template)
    expect((data.fatura as Record<string, unknown>).no).toBe('FTR-2026-001')
  })
})
