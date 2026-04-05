<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, computed } from 'vue'
import { EditorView, lineNumbers } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import { dexpr } from 'codemirror-lang-dexpr'
import type { DexprLanguageInfo } from 'codemirror-lang-dexpr'
import { useSchemaStore } from '../../stores/schema'
import type { SchemaNode } from '../../core/schema-parser'

const props = defineProps<{
  modelValue: string
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const editorEl = ref<HTMLDivElement>()
let view: EditorView | null = null
let debounceTimer: ReturnType<typeof setTimeout> | null = null

function emitDebounced(val: string) {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    emit('update:modelValue', val)
  }, 300)
}

const schemaStore = useSchemaStore()

/** Schema tree'den dexpr LanguageInfo formatina donustur */
function schemaToLanguageInfo(): DexprLanguageInfo {
  const info: DexprLanguageInfo = {
    functions: [
      { name: 'log', signature: '(...args) -> Null', doc: 'Deger yazdir' },
      { name: 'rand', signature: '(min: Number, max: Number) -> Number', doc: 'Rastgele sayi' },
    ],
    methods: {
      String: [
        { name: 'upper', signature: '() -> String' },
        { name: 'lower', signature: '() -> String' },
        { name: 'trim', signature: '() -> String' },
        { name: 'length', signature: '() -> Number' },
        { name: 'contains', signature: '(substr: String) -> Boolean' },
        { name: 'replace', signature: '(old: String, new: String) -> String' },
        { name: 'split', signature: '(delim: String) -> StringList' },
        { name: 'substring', signature: '(start: Number, end?: Number) -> String' },
        { name: 'startsWith', signature: '(prefix: String) -> Boolean' },
        { name: 'endsWith', signature: '(suffix: String) -> Boolean' },
        { name: 'charAt', signature: '(index: Number) -> String' },
        { name: 'trimStart', signature: '() -> String' },
        { name: 'trimEnd', signature: '() -> String' },
      ],
      Number: [],
      Boolean: [],
      NumberList: [
        { name: 'length', signature: '() -> Number' },
        { name: 'sum', signature: '() -> Number' },
        { name: 'avg', signature: '() -> Number' },
        { name: 'min', signature: '() -> Number' },
        { name: 'max', signature: '() -> Number' },
        { name: 'first', signature: '() -> Number' },
        { name: 'last', signature: '() -> Number' },
        { name: 'sort', signature: '() -> NumberList' },
        { name: 'reverse', signature: '() -> NumberList' },
        { name: 'contains', signature: '(value: Number) -> Boolean' },
      ],
      StringList: [
        { name: 'length', signature: '() -> Number' },
        { name: 'join', signature: '(delim?: String) -> String' },
        { name: 'first', signature: '() -> String' },
        { name: 'last', signature: '() -> String' },
        { name: 'sort', signature: '() -> StringList' },
        { name: 'reverse', signature: '() -> StringList' },
        { name: 'contains', signature: '(value: String) -> Boolean' },
      ],
      Object: [
        { name: 'keys', signature: '() -> StringList' },
        { name: 'values', signature: '() -> StringList | NumberList' },
        { name: 'length', signature: '() -> Number' },
        { name: 'contains', signature: '(key: String) -> Boolean' },
        { name: 'get', signature: '(key: String) -> any' },
      ],
    },
    variables: [],
  }

  // Schema tree'deki top-level object property'lerinden dexpr degiskenleri olustur
  const tree = schemaStore.schemaTree
  for (const child of tree.children) {
    if (child.type === 'object') {
      const fields = child.children.map(f => ({
        name: f.key,
        type: schemaToDexprType(f),
      }))
      info.variables!.push({
        name: child.key,
        type: 'Object',
        doc: child.title,
        fields,
      })
    } else {
      info.variables!.push({
        name: child.key,
        type: schemaToDexprType(child),
        doc: child.title,
      })
    }
  }

  return info
}

function schemaToDexprType(node: SchemaNode): 'String' | 'Number' | 'Boolean' | 'Object' | 'NumberList' | 'StringList' {
  switch (node.type) {
    case 'number':
    case 'integer':
      return 'Number'
    case 'boolean':
      return 'Boolean'
    case 'object':
      return 'Object'
    case 'array':
      return 'StringList'
    default:
      return 'String'
  }
}

const langInfo = computed(() => schemaToLanguageInfo())

function createState(doc: string): EditorState {
  return EditorState.create({
    doc,
    extensions: [
      EditorView.updateListener.of(update => {
        if (update.docChanged) {
          const val = update.state.doc.toString()
          if (val !== props.modelValue) {
            emitDebounced(val)
          }
        }
      }),
      lineNumbers(),
      dexpr(langInfo.value),
      EditorView.lineWrapping,
      EditorView.theme({
        '&': {
          fontSize: '11px',
          border: '1px solid #e2e8f0',
          borderRadius: '4px',
          backgroundColor: '#fff',
          maxHeight: '120px',
        },
        '&.cm-focused': {
          outline: '2px solid #93c5fd',
          outlineOffset: '-1px',
        },
        '.cm-scroller': {
          overflow: 'auto',
        },
        '.cm-content': {
          padding: '4px 6px',
          fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
          minHeight: '20px',
        },
        '.cm-line': {
          padding: '0',
        },
        '.cm-gutters': {
          backgroundColor: '#f8fafc',
          borderRight: '1px solid #e2e8f0',
          color: '#94a3b8',
          fontSize: '10px',
          minWidth: '20px',
          paddingLeft: '2px',
          paddingRight: '4px',
        },
        '.cm-activeLine': {
          backgroundColor: 'transparent',
        },
        '.cm-tooltip.cm-tooltip-autocomplete': {
          fontSize: '11px',
          zIndex: '9999',
        },
      }),
      EditorState.tabSize.of(2),
      EditorView.contentAttributes.of({
        'aria-label': 'dexpr expression editor',
      }),
    ],
  })
}

onMounted(() => {
  if (!editorEl.value) return
  view = new EditorView({
    state: createState(props.modelValue ?? ''),
    parent: editorEl.value,
  })
})

onBeforeUnmount(() => {
  view?.destroy()
  view = null
})

// Disaridan gelen deger degisikligi (undo/redo vs.)
watch(() => props.modelValue, (newVal) => {
  if (!view) return
  const current = view.state.doc.toString()
  if (current !== newVal) {
    view.dispatch({
      changes: { from: 0, to: current.length, insert: newVal ?? '' },
    })
  }
})

// Schema degisince editor'u yeniden olustur (autocomplete guncellenmeli)
watch(langInfo, () => {
  if (!view) return
  const doc = view.state.doc.toString()
  view.setState(createState(doc))
}, { deep: true })
</script>

<template>
  <div ref="editorEl" class="dexpr-editor" />
</template>

<style scoped>
.dexpr-editor {
  width: 100%;
  min-width: 0;
}
</style>
