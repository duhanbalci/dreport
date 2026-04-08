import { useTemplateStore } from '../stores/template'
import { useEditorStore } from '../stores/editor'
import type { TemplateElement } from '../core/types'

export function usePropertyUpdate(elementRef: () => TemplateElement) {
  const templateStore = useTemplateStore()
  const editorStore = useEditorStore()

  function update(updates: Partial<TemplateElement>) {
    const id = editorStore.selectedElementId
    if (!id) return
    templateStore.updateElement(id, updates)
  }

  function updateStyle(key: string, value: unknown) {
    update({ style: { ...elementRef().style, [key]: value } } as Partial<TemplateElement>)
  }

  function updateNested(
    field: string,
    key: string,
    value: unknown,
    defaults: Record<string, unknown> = {},
  ) {
    const current = (elementRef() as any)[field] ?? defaults
    update({ [field]: { ...current, [key]: value } } as any)
  }

  return { update, updateStyle, updateNested }
}
