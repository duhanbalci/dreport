<script setup lang="ts">
import { ref, watch } from 'vue'
import { DreportEditor } from './lib'
import type { Template, JsonSchema } from './lib'

// --- Full Invoice Schema ---

const defaultInvoiceSchema: JsonSchema = {
  $id: 'fatura-schema',
  type: 'object',
  properties: {
    firma: {
      type: 'object',
      title: 'Firma',
      properties: {
        unvan: { type: 'string', title: 'Firma Unvani' },
        vergiDairesi: { type: 'string', title: 'Vergi Dairesi' },
        vergiNo: { type: 'string', title: 'Vergi No' },
        adres: { type: 'string', title: 'Adres' },
        il: { type: 'string', title: 'Il' },
        telefon: { type: 'string', title: 'Telefon' },
        email: { type: 'string', title: 'E-posta' },
        logo: { type: 'string', title: 'Logo', format: 'image' },
      },
    },
    fatura: {
      type: 'object',
      title: 'Fatura',
      properties: {
        no: { type: 'string', title: 'Fatura No' },
        seri: { type: 'string', title: 'Seri' },
        tarih: { type: 'string', title: 'Duzenleme Tarihi', format: 'date' },
        vadeTarihi: { type: 'string', title: 'Vade Tarihi', format: 'date' },
      },
    },
    musteri: {
      type: 'object',
      title: 'Musteri',
      properties: {
        unvan: { type: 'string', title: 'Musteri Unvani' },
        vergiDairesi: { type: 'string', title: 'Vergi Dairesi' },
        vergiNo: { type: 'string', title: 'Vergi No' },
        adres: { type: 'string', title: 'Adres' },
        il: { type: 'string', title: 'Il' },
        telefon: { type: 'string', title: 'Telefon' },
      },
    },
    kalemler: {
      type: 'array',
      title: 'Fatura Kalemleri',
      items: {
        type: 'object',
        properties: {
          siraNo: { type: 'integer', title: 'Sira No' },
          adi: { type: 'string', title: 'Urun / Hizmet Adi' },
          miktar: { type: 'number', title: 'Miktar' },
          birim: { type: 'string', title: 'Birim' },
          birimFiyat: { type: 'number', title: 'Birim Fiyat', format: 'currency' },
          tutar: { type: 'number', title: 'Tutar', format: 'currency' },
        },
      },
    },
    toplamlar: {
      type: 'object',
      title: 'Toplamlar',
      properties: {
        araToplam: { type: 'number', title: 'Ara Toplam', format: 'currency' },
        kdvOrani: { type: 'number', title: 'KDV Orani', format: 'percentage' },
        kdv: { type: 'number', title: 'KDV', format: 'currency' },
        genelToplam: { type: 'number', title: 'Genel Toplam', format: 'currency' },
      },
    },
  },
}

const currentSchema = ref<JsonSchema>(structuredClone(defaultInvoiceSchema))

// --- Schema persistence ---

const SCHEMA_STORAGE_KEY = 'dreport-schema'

function loadSchemaFromLocalStorage(): JsonSchema | null {
  try {
    const raw = localStorage.getItem(SCHEMA_STORAGE_KEY)
    if (!raw) return null
    return JSON.parse(raw) as JsonSchema
  } catch {
    return null
  }
}

const savedSchema = loadSchemaFromLocalStorage()
if (savedSchema) {
  currentSchema.value = savedSchema
}

watch(currentSchema, (val) => {
  localStorage.setItem(SCHEMA_STORAGE_KEY, JSON.stringify(val))
}, { deep: true })

// --- Sample Invoice Data ---

const sampleData: Record<string, unknown> = {
  firma: {
    unvan: 'Teknova Yazilim A.S.',
    vergiDairesi: 'Besiktas',
    vergiNo: '1234567890',
    adres: 'Levent Mah. Inovasyon Sk. No:42 Kat:5',
    il: 'Istanbul',
    telefon: '+90 212 555 0042',
    email: 'info@teknova.com.tr',
  },
  fatura: {
    no: 'FTR-2026-001547',
    seri: 'A',
    tarih: '2026-03-29',
    vadeTarihi: '2026-04-28',
  },
  musteri: {
    unvan: 'Anadolu Lojistik Ltd. Sti.',
    vergiDairesi: 'Kadikoy',
    vergiNo: '9876543210',
    adres: 'Caferaga Mah. Moda Cd. No:18',
    il: 'Istanbul',
    telefon: '+90 216 444 0018',
  },
  kalemler: [
    { siraNo: 1, adi: 'Web Uygulama Gelistirme', miktar: 1, birim: 'Adet', birimFiyat: 45000, tutar: 45000 },
    { siraNo: 2, adi: 'Mobil Uygulama Gelistirme', miktar: 1, birim: 'Adet', birimFiyat: 35000, tutar: 35000 },
    { siraNo: 3, adi: 'UI/UX Tasarim Hizmeti', miktar: 40, birim: 'Saat', birimFiyat: 750, tutar: 30000 },
    { siraNo: 4, adi: 'Sunucu Bakim Sozlesmesi (Yillik)', miktar: 1, birim: 'Adet', birimFiyat: 12000, tutar: 12000 },
    { siraNo: 5, adi: 'SSL Sertifikasi', miktar: 3, birim: 'Adet', birimFiyat: 500, tutar: 1500 },
  ],
  toplamlar: {
    araToplam: 123500,
    kdvOrani: 20,
    kdv: 24700,
    genelToplam: 148200,
  },
}

// --- Default Invoice Template ---

const sz = {
  fixed: (v: number) => ({ type: 'fixed' as const, value: v }),
  auto: () => ({ type: 'auto' as const }),
  fr: (v = 1) => ({ type: 'fr' as const, value: v }),
}

const defaultInvoiceTemplate: Template = {
  id: 'tpl_fatura_demo',
  name: 'Standart Fatura',
  page: { width: 210, height: 297 },
  fonts: ['Noto Sans'],
  root: {
    id: 'root',
    type: 'container',
    position: { type: 'flow' },
    size: { width: sz.auto(), height: sz.auto() },
    direction: 'column',
    gap: 5,
    padding: { top: 15, right: 15, bottom: 15, left: 15 },
    align: 'stretch',
    justify: 'start',
    style: {},
    children: [
      // --- Header Row ---
      {
        id: 'c_header',
        type: 'container',
        position: { type: 'flow' },
        size: { width: sz.fr(), height: sz.auto() },
        direction: 'row',
        gap: 5,
        padding: { top: 0, right: 0, bottom: 0, left: 0 },
        align: 'start',
        justify: 'space-between',
        style: {},
        children: [
          // Firma bilgileri (sol)
          {
            id: 'c_firma',
            type: 'container',
            position: { type: 'flow' },
            size: { width: sz.fr(), height: sz.auto() },
            direction: 'column',
            gap: 1,
            padding: { top: 0, right: 0, bottom: 0, left: 0 },
            align: 'start',
            justify: 'start',
            style: {},
            children: [
              {
                id: 'el_firma_unvan',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 14, fontWeight: 'bold', color: '#1a1a1a' },
                binding: { type: 'scalar', path: 'firma.unvan' },
              },
              {
                id: 'el_firma_adres',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 9, color: '#555555' },
                binding: { type: 'scalar', path: 'firma.adres' },
              },
              {
                id: 'el_firma_il',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 9, color: '#555555' },
                binding: { type: 'scalar', path: 'firma.il' },
              },
              {
                id: 'el_firma_telefon',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 9, color: '#555555' },
                content: 'Tel: ',
                binding: { type: 'scalar', path: 'firma.telefon' },
              },
              {
                id: 'el_firma_vd',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 9, color: '#555555' },
                content: 'VD: ',
                binding: { type: 'scalar', path: 'firma.vergiDairesi' },
              },
              {
                id: 'el_firma_vn',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 9, color: '#555555' },
                content: 'VN: ',
                binding: { type: 'scalar', path: 'firma.vergiNo' },
              },
            ],
          },
          // Fatura basligi (sag)
          {
            id: 'c_fatura_baslik',
            type: 'container',
            position: { type: 'flow' },
            size: { width: sz.auto(), height: sz.auto() },
            direction: 'column',
            gap: 2,
            padding: { top: 0, right: 0, bottom: 0, left: 0 },
            align: 'end',
            justify: 'start',
            style: {},
            children: [
              {
                id: 'el_fatura_baslik',
                type: 'static_text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 18, fontWeight: 'bold', color: '#1a1a1a', align: 'right' },
                content: 'FATURA',
              },
              {
                id: 'el_fatura_no',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 10, color: '#333333', align: 'right' },
                content: 'No: ',
                binding: { type: 'scalar', path: 'fatura.no' },
              },
              {
                id: 'el_fatura_tarih',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 10, color: '#333333', align: 'right' },
                content: 'Tarih: ',
                binding: { type: 'scalar', path: 'fatura.tarih' },
              },
              {
                id: 'el_fatura_vade',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 10, color: '#333333', align: 'right' },
                content: 'Vade: ',
                binding: { type: 'scalar', path: 'fatura.vadeTarihi' },
              },
            ],
          },
        ],
      },
      // --- Separator ---
      {
        id: 'el_cizgi_1',
        type: 'line',
        position: { type: 'flow' },
        size: { width: sz.fr(), height: sz.auto() },
        style: { strokeColor: '#cccccc', strokeWidth: 0.5 },
      },
      // --- Musteri Bilgileri ---
      {
        id: 'c_musteri',
        type: 'container',
        position: { type: 'flow' },
        size: { width: sz.fr(), height: sz.auto() },
        direction: 'column',
        gap: 1,
        padding: { top: 3, right: 5, bottom: 3, left: 5 },
        align: 'start',
        justify: 'start',
        style: { backgroundColor: '#f8f9fa', borderColor: '#e9ecef', borderWidth: 0.5 },
        children: [
          {
            id: 'el_musteri_baslik',
            type: 'static_text',
            position: { type: 'flow' },
            size: { width: sz.auto(), height: sz.auto() },
            style: { fontSize: 9, fontWeight: 'bold', color: '#666666' },
            content: 'MUSTERI BILGILERI',
          },
          {
            id: 'el_musteri_unvan',
            type: 'text',
            position: { type: 'flow' },
            size: { width: sz.auto(), height: sz.auto() },
            style: { fontSize: 11, fontWeight: 'bold', color: '#1a1a1a' },
            binding: { type: 'scalar', path: 'musteri.unvan' },
          },
          {
            id: 'el_musteri_adres',
            type: 'text',
            position: { type: 'flow' },
            size: { width: sz.auto(), height: sz.auto() },
            style: { fontSize: 9, color: '#555555' },
            binding: { type: 'scalar', path: 'musteri.adres' },
          },
          {
            id: 'el_musteri_vd',
            type: 'text',
            position: { type: 'flow' },
            size: { width: sz.auto(), height: sz.auto() },
            style: { fontSize: 9, color: '#555555' },
            content: 'VD: ',
            binding: { type: 'scalar', path: 'musteri.vergiDairesi' },
          },
          {
            id: 'el_musteri_vn',
            type: 'text',
            position: { type: 'flow' },
            size: { width: sz.auto(), height: sz.auto() },
            style: { fontSize: 9, color: '#555555' },
            content: 'VN: ',
            binding: { type: 'scalar', path: 'musteri.vergiNo' },
          },
        ],
      },
      // --- Kalemler Tablosu ---
      {
        id: 'el_tablo',
        type: 'repeating_table',
        position: { type: 'flow' },
        size: { width: sz.fr(), height: sz.auto() },
        dataSource: { type: 'array', path: 'kalemler' },
        columns: [
          { id: 'col_sira', field: 'siraNo', title: '#', width: sz.fixed(10), align: 'center' },
          { id: 'col_adi', field: 'adi', title: 'Urun / Hizmet', width: sz.fr(), align: 'left' },
          { id: 'col_miktar', field: 'miktar', title: 'Miktar', width: sz.fixed(18), align: 'right' },
          { id: 'col_birim', field: 'birim', title: 'Birim', width: sz.fixed(18), align: 'center' },
          { id: 'col_fiyat', field: 'birimFiyat', title: 'Birim Fiyat', width: sz.fixed(28), align: 'right', format: 'currency' as const },
          { id: 'col_tutar', field: 'tutar', title: 'Tutar', width: sz.fixed(28), align: 'right', format: 'currency' as const },
        ],
        style: {
          fontSize: 9,
          headerFontSize: 9,
          headerBg: '#1e293b',
          headerColor: '#ffffff',
          zebraOdd: '#ffffff',
          zebraEven: '#f8fafc',
          borderColor: '#e2e8f0',
          borderWidth: 0.5,
        },
      },
      // --- Kalem Tutarlari Grafik ---
      {
        id: 'el_chart_bar',
        type: 'chart',
        position: { type: 'flow' },
        size: { width: sz.fr(), height: sz.fixed(60) },
        chartType: 'bar',
        dataSource: { type: 'array', path: 'kalemler' },
        categoryField: 'adi',
        valueField: 'tutar',
        title: { text: 'Kalem Tutarlari', fontSize: 3.5, color: '#1e293b', align: 'center' },
        legend: { show: false },
        labels: { show: true, fontSize: 2.2, color: '#333' },
        axis: { showGrid: true },
        style: { colors: ['#4F46E5', '#10B981', '#F59E0B', '#EF4444'] },
      },
      // --- Toplamlar ---
      {
        id: 'c_toplamlar_row',
        type: 'container',
        position: { type: 'flow' },
        size: { width: sz.fr(), height: sz.auto() },
        direction: 'row',
        gap: 0,
        padding: { top: 3, right: 0, bottom: 0, left: 0 },
        align: 'start',
        justify: 'end',
        style: {},
        children: [
          {
            id: 'c_toplamlar',
            type: 'container',
            position: { type: 'flow' },
            size: { width: sz.fixed(80), height: sz.auto() },
            direction: 'column',
            gap: 2,
            padding: { top: 3, right: 5, bottom: 3, left: 5 },
            align: 'stretch',
            justify: 'start',
            style: { borderColor: '#e2e8f0', borderWidth: 0.5 },
            children: [
              {
                id: 'el_ara_toplam',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 10, color: '#333333', align: 'right' },
                content: 'Ara Toplam: ',
                binding: { type: 'scalar', path: 'toplamlar.araToplam' },
              },
              {
                id: 'el_kdv',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 10, color: '#333333', align: 'right' },
                content: 'KDV (%20): ',
                binding: { type: 'scalar', path: 'toplamlar.kdv' },
              },
              {
                id: 'el_cizgi_2',
                type: 'line',
                position: { type: 'flow' },
                size: { width: sz.fr(), height: sz.auto() },
                style: { strokeColor: '#1e293b', strokeWidth: 1 },
              },
              {
                id: 'el_genel_toplam',
                type: 'text',
                position: { type: 'flow' },
                size: { width: sz.auto(), height: sz.auto() },
                style: { fontSize: 12, fontWeight: 'bold', color: '#1a1a1a', align: 'right' },
                content: 'GENEL TOPLAM: ',
                binding: { type: 'scalar', path: 'toplamlar.genelToplam' },
              },
            ],
          },
        ],
      },
    ],
  },
}

// --- LocalStorage persistence ---

const STORAGE_KEY = 'dreport-template'

function loadFromLocalStorage(): Template | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return null
    return JSON.parse(raw) as Template
  } catch {
    return null
  }
}

const template = ref<Template>(loadFromLocalStorage() ?? structuredClone(defaultInvoiceTemplate))

let saveTimeout: ReturnType<typeof setTimeout> | null = null
watch(template, (val) => {
  if (saveTimeout) clearTimeout(saveTimeout)
  saveTimeout = setTimeout(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(val))
  }, 500)
}, { deep: true })

// --- Editor ref ---

const editorRef = ref<InstanceType<typeof DreportEditor> | null>(null)
const pdfLoading = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)
const schemaFileInputRef = ref<HTMLInputElement | null>(null)

function triggerImport() {
  fileInputRef.value?.click()
}

function onImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    try {
      const parsed = JSON.parse(reader.result as string)
      // Detect bundle (has both 'template' and 'schema' keys)
      if (parsed.template && parsed.schema) {
        editorRef.value?.importTemplate(JSON.stringify(parsed.template))
        currentSchema.value = parsed.schema
        return
      }
      // Detect standalone template (has 'root' key)
      if (parsed.root) {
        editorRef.value?.importTemplate(reader.result as string)
        return
      }
      // Fallback: try as template
      editorRef.value?.importTemplate(reader.result as string)
    } catch {
      alert('Gecersiz sablon dosyasi')
    }
  }
  reader.readAsText(file)
  input.value = ''
}

function exportTemplate() {
  const json = editorRef.value?.exportTemplate()
  if (!json) return
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${template.value.name || 'sablon'}.json`
  a.click()
  URL.revokeObjectURL(url)
}

// --- Schema import/export ---

function triggerSchemaImport() {
  schemaFileInputRef.value?.click()
}

function onSchemaImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    try {
      const schema = JSON.parse(reader.result as string)
      currentSchema.value = schema
    } catch {
      alert('Gecersiz schema dosyasi')
    }
  }
  reader.readAsText(file)
  input.value = ''
}

function exportSchema() {
  const json = JSON.stringify(currentSchema.value, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'schema.json'
  a.click()
  URL.revokeObjectURL(url)
}

// --- Bundle export (template + schema) ---

function exportBundle() {
  const templateJson = editorRef.value?.exportTemplate()
  if (!templateJson) return
  const bundle = {
    template: JSON.parse(templateJson),
    schema: currentSchema.value,
  }
  const json = JSON.stringify(bundle, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${template.value.name || 'sablon'}-bundle.json`
  a.click()
  URL.revokeObjectURL(url)
}

async function downloadPdf() {
  pdfLoading.value = true
  try {
    const blob = await editorRef.value?.exportPdf()
    if (!blob) return
    const url = URL.createObjectURL(blob)
    window.open(url, '_blank')
  } catch (err) {
    alert(err instanceof Error ? err.message : 'PDF olusturulamadi')
  } finally {
    pdfLoading.value = false
  }
}

function resetTemplate() {
  template.value = structuredClone(defaultInvoiceTemplate)
  currentSchema.value = structuredClone(defaultInvoiceSchema)
  localStorage.removeItem(STORAGE_KEY)
  localStorage.removeItem(SCHEMA_STORAGE_KEY)
}
</script>

<template>
  <div class="app-layout">
    <header class="app-header">
      <h1>dreport</h1>
      <span class="app-header__subtitle">Belge Tasarim Araci</span>
      <div style="flex: 1"></div>
      <input ref="fileInputRef" type="file" accept=".json" style="display: none" @change="onImportFile" />
      <input ref="schemaFileInputRef" type="file" accept=".json" style="display: none" @change="onSchemaImportFile" />

      <!-- Template operations -->
      <button class="header-btn header-btn--secondary" @click="resetTemplate" title="Sifirla">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M2 8a6 6 0 0 1 10.2-4.3L14 2v4h-4l1.7-1.7A4.5 4.5 0 1 0 12.5 8" /><path d="M12.5 8a4.5 4.5 0 0 1-8.2 2.5" /></svg>
        Sifirla
      </button>
      <button class="header-btn header-btn--secondary" @click="triggerImport" title="Sablon Yukle">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 10V2m0 0L5 5m3-3 3 3" /><path d="M2 10v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-2" /></svg>
        Yukle
      </button>
      <button class="header-btn header-btn--secondary" @click="exportTemplate" title="Sablon Kaydet">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 2v8m0 0 3-3m-3 3L5 7" /><path d="M2 10v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-2" /></svg>
        Kaydet
      </button>
      <button class="header-btn header-btn--secondary" @click="exportBundle" title="Sablon + Schema Birlikte Kaydet">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="1" width="12" height="14" rx="1.5" /><path d="M5 4h6M5 7h6M5 10h4" /></svg>
        Paket
      </button>

      <div class="header-divider"></div>

      <!-- Schema operations -->
      <button class="header-btn header-btn--secondary" @click="triggerSchemaImport" title="Schema Yukle">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 10V2m0 0L5 5m3-3 3 3" /><path d="M2 10v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-2" /></svg>
        Schema
      </button>
      <button class="header-btn header-btn--secondary" @click="exportSchema" title="Schema Kaydet">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M8 2v8m0 0 3-3m-3 3L5 7" /><path d="M2 10v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-2" /></svg>
        Schema
      </button>

      <div class="header-divider"></div>

      <!-- Output -->
      <button class="header-btn" :disabled="pdfLoading" @click="downloadPdf">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="1" width="10" height="14" rx="1.5" /><path d="M6 5h4M6 8h4M6 11h2" /></svg>
        {{ pdfLoading ? 'Hazirlaniyor...' : 'PDF Onizle' }}
      </button>
    </header>
    <DreportEditor
      ref="editorRef"
      v-model="template"
      :schema="currentSchema"
      :data="sampleData"
      :config="{ apiBaseUrl: 'http://localhost:3001/api' }"
    />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.app-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: #1e293b;
  color: white;
  flex-shrink: 0;
}

.app-header h1 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.app-header__subtitle {
  font-size: 13px;
  color: #94a3b8;
}

.header-btn {
  padding: 6px 16px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s;
}

.header-btn:hover:not(:disabled) {
  background: #2563eb;
}

.header-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.header-btn--secondary {
  background: transparent;
  border: 1px solid #475569;
  color: #cbd5e1;
}

.header-btn--secondary:hover {
  background: #334155;
  color: white;
}

.header-btn svg {
  width: 14px;
  height: 14px;
  vertical-align: -2px;
  margin-right: 4px;
  flex-shrink: 0;
}

.header-divider {
  width: 1px;
  height: 20px;
  background: #475569;
  margin: 0 4px;
  flex-shrink: 0;
}
</style>
