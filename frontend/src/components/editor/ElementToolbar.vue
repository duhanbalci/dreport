<script setup lang="ts">
import { computed } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { isContainer } from '../../core/types'
import type { ContainerElement, TextStyle } from '../../core/types'
import type { ElementLayout } from '../../core/layout-types'

const props = defineProps<{
  scale: number
  layoutMap: Record<string, ElementLayout>
}>()

const templateStore = useTemplateStore()
const editorStore = useEditorStore()

const selected = computed(() => {
  const id = editorStore.selectedElementId
  if (!id || id === 'root') return null
  return templateStore.getElementById(id) ?? null
})

const container = computed(() => {
  const el = selected.value
  return el && isContainer(el) ? el as ContainerElement : null
})

const isText = computed(() => {
  const t = selected.value?.type
  return t === 'static_text' || t === 'text'
})

const isLine = computed(() => selected.value?.type === 'line')

const toolbarStyle = computed(() => {
  const el = selected.value
  if (!el) return { display: 'none' }
  const l = props.layoutMap[el.id]
  if (!l) return { display: 'none' }

  const s = props.scale
  return {
    position: 'absolute' as const,
    left: `${l.x_mm * s}px`,
    top: `${l.y_mm * s - 30}px`,
    zIndex: 1100,
  }
})

function update(updates: Record<string, unknown>) {
  if (!selected.value) return
  templateStore.updateElement(selected.value.id, updates as any)
}

function updateStyle(key: string, value: unknown) {
  if (!selected.value) return
  update({ style: { ...selected.value.style, [key]: value } })
}

// Container
function setDirection(dir: 'row' | 'column') { update({ direction: dir }) }
function setAlign(align: string) { update({ align }) }
function setJustify(justify: string) { update({ justify }) }
function setGap(e: Event) { update({ gap: parseFloat((e.target as HTMLInputElement).value) || 0 }) }

// Text
function setFontWeight(w: string) { updateStyle('fontWeight', w) }
function setTextAlign(a: string) { updateStyle('align', a) }
</script>

<template>
  <div v-if="selected" class="et" :style="toolbarStyle" @pointerdown.stop>
    <!-- ===== Container ===== -->
    <template v-if="container">
      <!-- Yön -->
      <div class="et__group">
        <button class="et__btn" :class="{ 'et__btn--active': container.direction === 'column' }" data-tip="Dikey" @click="setDirection('column')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="2" y="1" width="10" height="3" rx="0.5" fill="currentColor"/><rect x="2" y="5.5" width="10" height="3" rx="0.5" fill="currentColor"/><rect x="2" y="10" width="10" height="3" rx="0.5" fill="currentColor"/>
          </svg>
        </button>
        <button class="et__btn" :class="{ 'et__btn--active': container.direction === 'row' }" data-tip="Yatay" @click="setDirection('row')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="1" y="2" width="3" height="10" rx="0.5" fill="currentColor"/><rect x="5.5" y="2" width="3" height="10" rx="0.5" fill="currentColor"/><rect x="10" y="2" width="3" height="10" rx="0.5" fill="currentColor"/>
          </svg>
        </button>
      </div>

      <div class="et__sep" />

      <!-- Align -->
      <div class="et__group">
        <template v-if="container.direction === 'column'">
          <button class="et__btn" :class="{ 'et__btn--active': container.align === 'start' }" data-tip="Sol" @click="setAlign('start')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3.5" y="3" width="8" height="2.5" rx="0.5" fill="currentColor"/><rect x="3.5" y="8" width="5" height="2.5" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.align === 'center' }" data-tip="Orta" @click="setAlign('center')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="6.25" y="1" width="1.5" height="12" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="3" width="8" height="2.5" rx="0.5" fill="currentColor"/><rect x="4.5" y="8" width="5" height="2.5" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.align === 'end' }" data-tip="Sag" @click="setAlign('end')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="11.5" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="2.5" y="3" width="8" height="2.5" rx="0.5" fill="currentColor"/><rect x="5.5" y="8" width="5" height="2.5" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.align === 'stretch' }" data-tip="Esnet" @click="setAlign('stretch')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="11.5" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3.5" y="3" width="7" height="2.5" rx="0.5" fill="currentColor"/><rect x="3.5" y="8" width="7" height="2.5" rx="0.5" fill="currentColor"/></svg>
          </button>
        </template>
        <template v-else>
          <button class="et__btn" :class="{ 'et__btn--active': container.align === 'start' }" data-tip="Ust" @click="setAlign('start')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="3.5" width="2.5" height="8" rx="0.5" fill="currentColor"/><rect x="8" y="3.5" width="2.5" height="5" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.align === 'center' }" data-tip="Orta" @click="setAlign('center')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="6.25" width="12" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="2" width="2.5" height="10" rx="0.5" fill="currentColor"/><rect x="8" y="3.5" width="2.5" height="7" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.align === 'end' }" data-tip="Alt" @click="setAlign('end')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="11.5" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="2.5" width="2.5" height="8" rx="0.5" fill="currentColor"/><rect x="8" y="5.5" width="2.5" height="5" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.align === 'stretch' }" data-tip="Esnet" @click="setAlign('stretch')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="2" y="11.5" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="3.5" width="2.5" height="7" rx="0.5" fill="currentColor"/><rect x="8" y="3.5" width="2.5" height="7" rx="0.5" fill="currentColor"/></svg>
          </button>
        </template>
      </div>

      <div class="et__sep" />

      <!-- Justify -->
      <div class="et__group">
        <template v-if="container.direction === 'column'">
          <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'start' }" data-tip="Ust" @click="setJustify('start')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="3.5" width="8" height="2" rx="0.5" fill="currentColor"/><rect x="3" y="6.5" width="8" height="2" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'center' }" data-tip="Orta" @click="setJustify('center')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="6.25" width="12" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="3" width="8" height="2" rx="0.5" fill="currentColor"/><rect x="3" y="9" width="8" height="2" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'end' }" data-tip="Alt" @click="setJustify('end')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="11.5" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="5.5" width="8" height="2" rx="0.5" fill="currentColor"/><rect x="3" y="8.5" width="8" height="2" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'space-between' }" data-tip="Esit Aralik" @click="setJustify('space-between')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="2" y="1" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="2" y="11.5" width="10" height="1.5" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3" y="3.5" width="8" height="2" rx="0.5" fill="currentColor"/><rect x="3" y="8.5" width="8" height="2" rx="0.5" fill="currentColor"/></svg>
          </button>
        </template>
        <template v-else>
          <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'start' }" data-tip="Sol" @click="setJustify('start')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3.5" y="3" width="3" height="8" rx="0.5" fill="currentColor"/><rect x="7.5" y="3" width="3" height="8" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'center' }" data-tip="Orta" @click="setJustify('center')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="6.25" y="1" width="1.5" height="12" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="2" y="3" width="3" height="8" rx="0.5" fill="currentColor"/><rect x="9" y="3" width="3" height="8" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'end' }" data-tip="Sag" @click="setJustify('end')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="11.5" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3.5" y="3" width="3" height="8" rx="0.5" fill="currentColor"/><rect x="7.5" y="3" width="3" height="8" rx="0.5" fill="currentColor"/></svg>
          </button>
          <button class="et__btn" :class="{ 'et__btn--active': container.justify === 'space-between' }" data-tip="Esit Aralik" @click="setJustify('space-between')">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="11.5" y="2" width="1.5" height="10" rx="0.5" fill="currentColor" opacity="0.4"/><rect x="3.5" y="3" width="3" height="8" rx="0.5" fill="currentColor"/><rect x="7.5" y="3" width="3" height="8" rx="0.5" fill="currentColor"/></svg>
          </button>
        </template>
      </div>

      <div class="et__sep" />

      <!-- Gap -->
      <div class="et__group et__group--gap" data-tip="Bosluk (mm)">
        <svg class="et__gap-icon" width="12" height="12" viewBox="0 0 12 12" fill="none">
          <rect x="1" y="1" width="3.5" height="10" rx="0.5" stroke="currentColor" stroke-width="1" fill="none"/><rect x="7.5" y="1" width="3.5" height="10" rx="0.5" stroke="currentColor" stroke-width="1" fill="none"/><line x1="6" y1="3" x2="6" y2="9" stroke="currentColor" stroke-width="1" stroke-dasharray="1.5 1"/>
        </svg>
        <input type="number" class="et__num" step="1" min="0" :value="container.gap" @input="setGap" />
      </div>
    </template>

    <!-- ===== Text / Static Text ===== -->
    <template v-if="isText">
      <!-- Bold -->
      <div class="et__group">
        <button class="et__btn" :class="{ 'et__btn--active': (selected!.style as TextStyle).fontWeight === 'bold' }" data-tip="Kalin" @click="setFontWeight((selected!.style as TextStyle).fontWeight === 'bold' ? 'normal' : 'bold')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M4 2.5h3.5a2.5 2.5 0 0 1 0 5H4V2.5z" stroke="currentColor" stroke-width="1.5" fill="none"/>
            <path d="M4 7.5h4a2.5 2.5 0 0 1 0 5H4V7.5z" stroke="currentColor" stroke-width="1.5" fill="none"/>
          </svg>
        </button>
      </div>

      <div class="et__sep" />

      <!-- Align -->
      <div class="et__group">
        <button class="et__btn" :class="{ 'et__btn--active': ((selected!.style as TextStyle).align ?? 'left') === 'left' }" data-tip="Sola Hizala" @click="setTextAlign('left')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <line x1="2" y1="3" x2="12" y2="3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="2" y1="7" x2="9" y2="7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="2" y1="11" x2="11" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="et__btn" :class="{ 'et__btn--active': (selected!.style as TextStyle).align === 'center' }" data-tip="Ortala" @click="setTextAlign('center')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <line x1="2" y1="3" x2="12" y2="3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="3.5" y1="7" x2="10.5" y2="7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="2.5" y1="11" x2="11.5" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="et__btn" :class="{ 'et__btn--active': (selected!.style as TextStyle).align === 'right' }" data-tip="Saga Hizala" @click="setTextAlign('right')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <line x1="2" y1="3" x2="12" y2="3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="5" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="3" y1="11" x2="12" y2="11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <div class="et__sep" />

      <!-- Font size -->
      <div class="et__group et__group--gap">
        <svg class="et__gap-icon" width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M2 10L6 2l4 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
          <line x1="3.5" y1="7" x2="8.5" y2="7" stroke="currentColor" stroke-width="1" stroke-linecap="round"/>
        </svg>
        <input type="number" class="et__num" step="1" min="1" :value="(selected!.style as TextStyle).fontSize ?? 11" @input="(e) => updateStyle('fontSize', parseFloat((e.target as HTMLInputElement).value) || 11)" data-tip="Yazi Boyutu (pt)" />
      </div>

      <div class="et__sep" />

      <!-- Color -->
      <div class="et__group">
        <label class="et__color-wrap" data-tip="Renk">
          <input type="color" class="et__color" :value="(selected!.style as TextStyle).color ?? '#000000'" @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="2" y="11" width="10" height="2" rx="0.5" :fill="(selected!.style as TextStyle).color ?? '#000000'"/>
            <path d="M5 9L7 3l2 6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" fill="none"/>
            <line x1="5.5" y1="7.5" x2="8.5" y2="7.5" stroke="currentColor" stroke-width="1" stroke-linecap="round"/>
          </svg>
        </label>
      </div>
    </template>

    <!-- ===== Line ===== -->
    <template v-if="isLine">
      <!-- Stroke width -->
      <div class="et__group et__group--gap">
        <svg class="et__gap-icon" width="12" height="12" viewBox="0 0 12 12" fill="none">
          <line x1="1" y1="6" x2="11" y2="6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <input type="number" class="et__num" step="0.1" min="0.1" :value="(selected!.style as any).strokeWidth ?? 0.5" @input="(e) => updateStyle('strokeWidth', parseFloat((e.target as HTMLInputElement).value) || 0.5)" data-tip="Kalinlik (mm)" />
      </div>

      <div class="et__sep" />

      <!-- Color -->
      <div class="et__group">
        <label class="et__color-wrap" data-tip="Renk">
          <input type="color" class="et__color" :value="(selected!.style as any).strokeColor ?? '#000000'" @input="(e) => updateStyle('strokeColor', (e.target as HTMLInputElement).value)" />
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <line x1="2" y1="7" x2="12" y2="7" :stroke="(selected!.style as any).strokeColor ?? '#000000'" stroke-width="2.5" stroke-linecap="round"/>
          </svg>
        </label>
      </div>
    </template>
  </div>
</template>

<style scoped>
.et {
  display: flex;
  align-items: center;
  gap: 2px;
  background: #1e293b;
  border-radius: 6px;
  padding: 3px 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25), 0 0 0 1px rgba(255, 255, 255, 0.06);
  pointer-events: auto;
  white-space: nowrap;
}

.et__group {
  display: flex;
  align-items: center;
  gap: 1px;
}

.et__sep {
  width: 1px;
  height: 16px;
  background: #334155;
  margin: 0 2px;
  flex-shrink: 0;
}

/* Tooltip */
[data-tip] {
  position: relative;
}

[data-tip]::after {
  content: attr(data-tip);
  position: absolute;
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  background: #0f172a;
  color: #e2e8f0;
  font-size: 10px;
  padding: 3px 6px;
  border-radius: 4px;
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s;
  z-index: 10;
}

[data-tip]:hover::after,
[data-tip]:focus-within::after {
  opacity: 1;
}

/* Button */
.et__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: #94a3b8;
  cursor: pointer;
  padding: 0;
  transition: background 0.1s, color 0.1s;
}

.et__btn:hover {
  background: #334155;
  color: #e2e8f0;
}

.et__btn--active {
  background: #3b82f6;
  color: white;
}

.et__btn--active:hover {
  background: #2563eb;
}

/* Number input */
.et__group--gap {
  gap: 3px;
}

.et__gap-icon {
  color: #64748b;
  flex-shrink: 0;
}

.et__num {
  width: 32px;
  height: 22px;
  border: 1px solid #334155;
  border-radius: 4px;
  background: #0f172a;
  color: #e2e8f0;
  text-align: center;
  font-size: 11px;
  font-family: inherit;
  padding: 0;
  outline: none;
  -moz-appearance: textfield;
}

.et__num::-webkit-inner-spin-button,
.et__num::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.et__num:focus {
  border-color: #3b82f6;
}

/* Color */
.et__color-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  cursor: pointer;
  position: relative;
  color: #94a3b8;
  transition: background 0.1s;
}

.et__color-wrap:hover {
  background: #334155;
  color: #e2e8f0;
}

.et__color {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
  width: 100%;
  height: 100%;
}
</style>
