<script setup lang="ts">
import { ref, watch } from 'vue'
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import { useSchemaStore } from '../../stores/schema'
import PropSection from './shared/PropSection.vue'
import PropSelect from './shared/PropSelect.vue'
import PropColorInput from './shared/PropColorInput.vue'
import PropCheckbox from './shared/PropCheckbox.vue'
import PropFieldSelect from './shared/PropFieldSelect.vue'
import type { BarcodeElement, BarcodeFormat } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: BarcodeElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const schemaStore = useSchemaStore()

const formatOptions = [
  { value: 'qr', label: 'QR Kod' },
  { value: 'ean13', label: 'EAN-13' },
  { value: 'ean8', label: 'EAN-8' },
  { value: 'code128', label: 'Code 128' },
  { value: 'code39', label: 'Code 39' },
]

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
      return value.length > 0 && [...value].every((c) => c.charCodeAt(0) < 128)
    case 'qr':
      return value.length > 0
    default:
      return value.length > 0
  }
}

const barcodeInputValue = ref('')
const barcodeInputInvalid = ref(false)

watch(
  () => props.element.value ?? '',
  (val) => {
    barcodeInputValue.value = val
    barcodeInputInvalid.value = false
  },
  { immediate: true },
)

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

function onBarcodeFormatChange(newFormat: string) {
  const fmt = newFormat as BarcodeFormat
  const currentValue = props.element.value ?? ''
  if (validateBarcode(fmt, currentValue)) {
    update({ format: fmt } as any)
  } else {
    const defaultVal = barcodeDefaults[fmt]
    barcodeInputValue.value = defaultVal
    barcodeInputInvalid.value = false
    update({ format: fmt, value: defaultVal } as any)
  }
}
</script>

<template>
  <PropSection title="Barkod Ayarlari">
    <PropSelect
      label="Format"
      :model-value="element.format"
      :options="formatOptions"
      data-tip="Barkod formati"
      @update:model-value="onBarcodeFormatChange"
    />
    <div class="prop-row" data-tip="Barkod icerigi — formata uygun olmali">
      <label class="prop-label">Deger</label>
      <input
        class="prop-input"
        type="text"
        :class="{ 'prop-input--invalid': barcodeInputInvalid }"
        :value="barcodeInputValue"
        @input="onBarcodeValueInput"
      />
    </div>
    <PropColorInput
      label="Renk"
      :model-value="element.style.color ?? '#000000'"
      :clearable="true"
      data-tip="Barkod cizgi/modul rengi"
      @update:model-value="(v) => updateStyle('color', v)"
    />
    <PropCheckbox
      v-if="element.format !== 'qr'"
      label="Metin Goster"
      :model-value="
        element.style.includeText ?? (element.format === 'ean13' || element.format === 'ean8')
      "
      data-tip="Barkod altinda degeri metin olarak goster"
      @update:model-value="(v) => updateStyle('includeText', v)"
    />
    <PropFieldSelect
      v-if="schemaStore.scalarFields.length > 0"
      label="Veri Baglama"
      :model-value="element.binding?.path ?? ''"
      :fields="schemaStore.scalarFields"
      :allow-empty="true"
      empty-label="Yok (statik deger)"
      data-tip="Schema'dan dinamik veri baglama"
      @update:model-value="
        (v) => {
          if (v) update({ binding: { type: 'scalar', path: v } } as any)
          else update({ binding: undefined } as any)
        }
      "
    />
  </PropSection>
</template>
