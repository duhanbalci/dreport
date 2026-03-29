<script setup lang="ts">
import { ref, watch } from 'vue'
import { useTemplateStore } from '../../stores/template'
import { useEditorStore } from '../../stores/editor'
import { useSchemaStore } from '../../stores/schema'
import type { BarcodeElement, BarcodeFormat, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: BarcodeElement }>()
const templateStore = useTemplateStore()
const editorStore = useEditorStore()
const schemaStore = useSchemaStore()

function update(updates: Partial<TemplateElement>) {
  const id = editorStore.selectedElementId
  if (!id) return
  templateStore.updateElement(id, updates)
}

function updateStyle(key: string, value: unknown) {
  update({ style: { ...props.element.style, [key]: value } } as Partial<TemplateElement>)
}

const barcodeDefaults: Record<BarcodeFormat, string> = {
  qr: 'https://example.com',
  ean13: '5901234123457',
  ean8: '96385074',
  code128: 'DREPORT-001',
  code39: 'DREPORT',
}

function eanCheckDigit(data: string): number {
  let sum = 0
  for (let i = 0; i < data.length; i++) {
    const d = parseInt(data[i])
    sum += d * (i % 2 === 0 ? 1 : 3)
  }
  return (10 - (sum % 10)) % 10
}

function validateBarcode(format: BarcodeFormat, value: string): boolean {
  if (!value) return false
  switch (format) {
    case 'ean13':
      if (!/^\d{13}$/.test(value)) return false
      return eanCheckDigit(value.slice(0, 12)) === parseInt(value[12])
    case 'ean8':
      if (!/^\d{8}$/.test(value)) return false
      return eanCheckDigit(value.slice(0, 7)) === parseInt(value[7])
    case 'code39':
      return /^[A-Z0-9\-. $/+%]+$/i.test(value)
    case 'code128':
      return value.length > 0 && [...value].every(c => c.charCodeAt(0) < 128)
    case 'qr':
      return value.length > 0
    default:
      return value.length > 0
  }
}

const barcodeInputValue = ref('')
const barcodeInputInvalid = ref(false)

watch(() => props.element.value ?? '', (val) => {
  barcodeInputValue.value = val
  barcodeInputInvalid.value = false
}, { immediate: true })

function onBarcodeValueInput(e: Event) {
  const val = (e.target as HTMLInputElement).value
  barcodeInputValue.value = val

  if (validateBarcode(props.element.format, val)) {
    barcodeInputInvalid.value = false
    update({ value: val } as any)
  } else {
    barcodeInputInvalid.value = true
  }
}

function onBarcodeFormatChange(newFormat: BarcodeFormat) {
  const currentValue = props.element.value ?? ''
  if (validateBarcode(newFormat, currentValue)) {
    update({ format: newFormat } as any)
  } else {
    const defaultVal = barcodeDefaults[newFormat]
    barcodeInputValue.value = defaultVal
    barcodeInputInvalid.value = false
    update({ format: newFormat, value: defaultVal } as any)
  }
}
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title">Barkod Ayarlari</div>
    <div class="prop-row">
      <label class="prop-label">Format</label>
      <select class="prop-input prop-select"
        :value="element.format"
        @change="(e) => onBarcodeFormatChange((e.target as HTMLSelectElement).value as BarcodeFormat)">
        <option value="qr">QR Kod</option>
        <option value="ean13">EAN-13</option>
        <option value="ean8">EAN-8</option>
        <option value="code128">Code 128</option>
        <option value="code39">Code 39</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="prop-label">Deger</label>
      <input class="prop-input" type="text"
        :class="{ 'prop-input--invalid': barcodeInputInvalid }"
        :value="barcodeInputValue"
        @input="onBarcodeValueInput" />
    </div>
    <div class="prop-row">
      <label class="prop-label">Renk</label>
      <div class="prop-row-inline">
        <input class="prop-input prop-color" type="color"
          :value="element.style.color ?? '#000000'"
          @input="(e) => updateStyle('color', (e.target as HTMLInputElement).value)" />
        <button v-if="element.style.color" class="prop-clear" @click="updateStyle('color', undefined)">x</button>
      </div>
    </div>
    <div v-if="element.format !== 'qr'" class="prop-row">
      <label class="prop-label">Metin Goster</label>
      <input type="checkbox"
        :checked="element.style.includeText ?? (element.format === 'ean13' || element.format === 'ean8')"
        @change="(e) => updateStyle('includeText', (e.target as HTMLInputElement).checked)" />
    </div>
    <div v-if="schemaStore.scalarFields.length > 0" class="prop-row">
      <label class="prop-label">Veri Baglama</label>
      <select class="prop-input prop-select"
        :value="element.binding?.path ?? ''"
        @change="(e) => {
          const val = (e.target as HTMLSelectElement).value
          if (val) {
            update({ binding: { type: 'scalar', path: val } } as any)
          } else {
            update({ binding: undefined } as any)
          }
        }">
        <option value="">Yok (statik deger)</option>
        <option
          v-for="field in schemaStore.scalarFields"
          :key="field.path"
          :value="field.path"
        >{{ field.title }} ({{ field.path }})</option>
      </select>
    </div>
  </div>
</template>
