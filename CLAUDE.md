# CLAUDE.md — dreport

## Proje Özeti

**dreport**, kullanıcıların fatura, irsaliye, rapor gibi belge şablonlarını görsel bir drag & drop editör ile tasarlayıp, JSON veri ile birleştirerek PDF çıktı almasını sağlayan bir belge tasarım aracıdır.

Temel fark: Editördeki önizleme doğrudan Typst render çıktısıdır. Canvas üzerinde ayrı bir render engine çalışmaz — kullanıcı her zaman gerçek Typst çıktısını görür. Bu sayede "editörde gördüğüm ile PDF'te aldığım farklı" sorunu ortadan kalkar.

---

## Teknoloji Kararları

| Katman            | Teknoloji                            | Gerekçe                                                   |
| ----------------- | ------------------------------------ | --------------------------------------------------------- |
| Frontend          | Vue 3 (Composition API) + TypeScript | Kullanıcı tercihi                                         |
| Editör Render     | Typst WASM → SVG                     | Editör çıktısı = PDF çıktısı tutarlılığı                  |
| Typst WASM        | `@myriaddreamin/typst.ts`            | Tarayıcıda Typst derleme; SVG çıktı üretimi               |
| Etkileşim Katmanı | SVG overlay (Vue bileşenleri)        | Typst SVG üzerine seçim, sürükleme, yeniden boyutlandırma |
| Backend           | Rust + Axum                          | Typst crate'lerini doğrudan kullanabilme; performans      |
| PDF Render        | `typst` Rust crate (server-side)     | Nihai PDF üretimi sunucuda; font tutarlılığı garantisi    |
| Veri Formatı      | JSON (şablon tanımı + veri)          | Evrensel, kolay serialize/deserialize                     |
| Paket Yönetimi    | bun (frontend), cargo (backend)      | —                                                         |

---

## Mimari Genel Bakış

```
┌─────────────────────────────────────────────────────┐
│                   VUE FRONTEND                       │
│                                                     │
│  ┌───────────────┐    ┌──────────────────────────┐  │
│  │ Sol Panel      │    │ Editör Canvas             │  │
│  │ - Bileşenler   │    │                          │  │
│  │ - Schema Tree  │    │  ┌────────────────────┐  │  │
│  │ - Özellikler   │    │  │ Typst WASM → SVG   │  │  │
│  │                │    │  │ (gerçek render)     │  │  │
│  │                │    │  └────────────────────┘  │  │
│  │                │    │  ┌────────────────────┐  │  │
│  │                │    │  │ SVG Overlay (Vue)   │  │  │
│  │                │    │  │ seçim / drag / resize│  │  │
│  │                │    │  └────────────────────┘  │  │
│  └───────────────┘    └──────────────────────────┘  │
│                                                     │
│  Template JSON ←→ Typst Markup ←→ SVG Render        │
└──────────────────────────┬──────────────────────────┘
                           │ POST /api/render
                           ▼
┌─────────────────────────────────────────────────────┐
│                   RUST BACKEND (Axum)                │
│                                                     │
│  Template JSON + Data JSON → Typst Markup → PDF     │
│  (typst crate ile doğrudan derleme)                 │
└─────────────────────────────────────────────────────┘
```

---

## Editör Render Stratejisi: Typst WASM Full Render

### Temel Prensip

Editörde ayrı bir canvas render engine (fabric.js, konva.js vb.) KULLANILMAZ. Bunun yerine:

1. Kullanıcının tasarımı bir **Template JSON** olarak tutulur.
2. Template JSON'dan **Typst markup** üretilir (frontend'de, saf fonksiyon).
3. Typst markup, **typst.ts WASM** modülü ile tarayıcıda derlenir → **SVG** çıktı üretilir.
4. SVG, editör alanında gösterilir.
5. SVG üzerine **Vue bileşenleriyle bir interaction overlay** yerleştirilir (seçim kutuları, drag handle'lar, resize köşeleri).
6. Kullanıcı bir elemanı sürüklediğinde → Template JSON güncellenir → Typst yeniden derlenir → SVG güncellenir.

### Performans Stratejisi

Typst incremental compilation destekler, ancak her fare hareketi için full render döngüsü ağır olabilir. Bu yüzden:

- **Drag sırasında:** Overlay katmanında sadece CSS transform ile görsel geri bildirim ver (hafif, anlık). Typst derleme YAPMA.
- **Drag bittiğinde (mouseup/pointerup):** Template JSON'ı güncelle, Typst derle, SVG'yi yenile.
- **Debounce:** Özellik panelinden yapılan değişikliklerde (font boyutu, renk vs.) 150-300ms debounce ile derle.
- **Web Worker:** Typst WASM derlemeyi bir Web Worker içinde çalıştır. Ana thread'i ASLA bloklamayacak.

### typst.ts Entegrasyonu

```typescript
// Temel kullanım şeması
import { $typst } from "@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs";

// Worker içinde:
async function compile(typstMarkup: string): Promise<string> {
  const svg = await $typst.svg({ mainContent: typstMarkup });
  return svg;
}
```

- WASM modülleri: `typst-ts-web-compiler` (~7.6 MB) + `typst-ts-renderer` (~350 KB)
- Fontlar: Projeye gömülü font seti gerekecek (~4.4 MB). Başlangıçta Noto Sans / Inter gibi bir set yeterli.
- İlk yükleme ağır olabilir — lazy loading ve cache stratejisi gerekli (Service Worker ile WASM cache'leme).

---

## Veri Modeli

### Layout Sistemi: Container-Based

Eski model (her eleman absolute `place()`) yerine, CSS Flexbox mantığına benzeyen container-based layout kullanılır:

- **Sayfa = kök container.** `page.margins` → kök container'ın `padding`'i olur.
- **Container'lar iç içe geçebilir.** `direction: "row" | "column"` ile yatay/dikey dizilim.
- **Elemanlar varsayılan olarak flow içindedir** — otomatik pozisyonlanır.
- **Opsiyonel absolute positioning:** Kullanıcı isterse bir elemanı `position: "absolute"` yapabilir. Bu durumda eleman parent container içinde absolute konumlanır (`place()` ile).

Bu sayede:
- Tablo satırları artarsa alttaki elemanlar otomatik kayar.
- Aynı satıra iki kolon koymak için iç içe container yeterlidir.
- Absolute mod ile serbest pozisyonlama da mümkündür.

### Boyut Sistemi (SizeValue)

Her eleman ve container için `width` ve `height` şu tiplerden biri olabilir:

| Tip     | Açıklama                              | Typst karşılığı |
| ------- | ------------------------------------- | --------------- |
| `fixed` | Sabit boyut (mm)                      | `80mm`          |
| `auto`  | İçeriğe göre otomatik                 | `auto`          |
| `fr`    | Kalan alanı oransal doldur            | `1fr`, `2fr`    |

Ek olarak `minWidth`, `maxWidth`, `minHeight`, `maxHeight` (mm) desteklenir.

### Template JSON (Şablon Tanımı)

```jsonc
{
  "id": "tpl_fatura_001",
  "name": "Standart Fatura",
  "page": { "width": 210, "height": 297 },
  "fonts": ["Noto Sans", "Noto Sans Mono"],
  "root": {
    "id": "root",
    "type": "container",
    "position": { "type": "flow" },
    "size": { "width": { "type": "auto" }, "height": { "type": "auto" } },
    "direction": "column",
    "gap": 5,
    "padding": { "top": 15, "right": 15, "bottom": 15, "left": 15 },
    "align": "stretch",
    "justify": "start",
    "style": {},
    "children": [
      {
        "id": "c_header",
        "type": "container",
        "position": { "type": "flow" },
        "size": { "width": { "type": "fr", "value": 1 }, "height": { "type": "auto" } },
        "direction": "row",
        "gap": 5,
        "padding": { "top": 0, "right": 0, "bottom": 0, "left": 0 },
        "align": "start",
        "justify": "start",
        "style": {},
        "children": [
          {
            "id": "el_firma",
            "type": "text",
            "position": { "type": "flow" },
            "size": { "width": { "type": "fr", "value": 1 }, "height": { "type": "auto" } },
            "style": { "fontSize": 14, "fontWeight": "bold" },
            "binding": { "type": "scalar", "path": "firma.unvan" }
          },
          {
            "id": "el_fatura_baslik",
            "type": "static_text",
            "position": { "type": "flow" },
            "size": { "width": { "type": "auto" }, "height": { "type": "auto" } },
            "style": { "fontSize": 12, "fontWeight": "bold", "align": "right" },
            "content": "FATURA"
          }
        ]
      },
      {
        "id": "el_cizgi",
        "type": "line",
        "position": { "type": "flow" },
        "size": { "width": { "type": "fr", "value": 1 }, "height": { "type": "auto" } },
        "style": { "strokeColor": "#000000", "strokeWidth": 0.5 }
      }
    ]
  }
}
```

### Eleman Tipleri

| Tip               | Açıklama                              | Binding          |
| ----------------- | ------------------------------------- | ---------------- |
| `container`       | Düzen kutusu, çocuk elemanları barındırır | Yok          |
| `static_text`     | Sabit metin, veri bağlantısı yok      | Yok              |
| `text`            | Dinamik metin, schema'dan veri çeker  | Scalar           |
| `repeating_table` | Array verisinden tekrarlayan tablo    | Array            |
| `line`            | Yatay/dikey çizgi                     | Yok              |
| `image`           | Statik veya dinamik görsel            | Opsiyonel scalar |
| `page_number`     | Sayfa numarası (çok sayfalı belgeler) | Otomatik         |

### Container Özellikleri

| Özellik     | Tip                                      | Açıklama                          |
| ----------- | ---------------------------------------- | --------------------------------- |
| `direction` | `"row"` \| `"column"`                   | Çocukları yatay mı dikey mi diz   |
| `gap`       | number (mm)                              | Çocuklar arası boşluk             |
| `padding`   | `{ top, right, bottom, left }` (mm)     | İç boşluk                         |
| `align`     | `"start"` \| `"center"` \| `"end"` \| `"stretch"` | Cross-axis hizalama   |
| `justify`   | `"start"` \| `"center"` \| `"end"` \| `"space-between"` | Main-axis dağılım |
| `style`     | `{ backgroundColor, borderColor, borderWidth, borderRadius }` | Görsel stil |

### Positioning Modları

| Mod        | Açıklama                                    | Typst karşılığı         |
| ---------- | ------------------------------------------- | ----------------------- |
| `flow`     | Parent container'ın flow'una katıl (default)| `stack` / `box` içinde  |
| `absolute` | Parent container içinde sabit konum         | `place(dx, dy)`         |

### Fatura Örneği — Container Ağacı

```
Sayfa (kök container, column, padding: 15mm)
├── Header (container, row, gap: 5mm)
│   ├── Logo alanı (container, column, width: 60mm)
│   │   ├── image (logo)
│   │   └── text (firma ünvanı)
│   └── Fatura bilgi (container, column, width: fill, align: end)
│       ├── static_text ("FATURA")
│       ├── text (fatura no)
│       └── text (tarih)
├── line (ayırıcı çizgi)
├── repeating_table (kalemler)
└── Footer (container, row)
    ├── boş alan (width: fill)
    └── Toplamlar (container, column, width: 80mm)
        ├── text (ara toplam)
        ├── text (KDV)
        └── text (genel toplam)
```

### Data JSON (Gerçek Veri)

Render zamanında şablonla birleştirilen veri:

```jsonc
{
  "firma": {
    "unvan": "Acme Teknoloji A.Ş.",
    "vergiNo": "1234567890",
    "logo": "data:image/png;base64,...",
  },
  "fatura": {
    "no": "FTR-2026-001",
    "tarih": "2026-03-29",
  },
  "kalemler": [
    {
      "siraNo": 1,
      "adi": "Web Geliştirme Hizmeti",
      "miktar": 1,
      "birim": "Adet",
      "birimFiyat": 15000,
      "tutar": 15000,
    },
    {
      "siraNo": 2,
      "adi": "SSL Sertifikası",
      "miktar": 2,
      "birim": "Adet",
      "birimFiyat": 500,
      "tutar": 1000,
    },
  ],
  "toplamlar": {
    "araToplam": 16000,
    "kdv": 2880,
    "genelToplam": 18880,
  },
}
```

### JSON Schema (Veri Yapısı Tanımı)

Editörün sol panelinde kullanıcıya sunulan, bağlanabilir alanların ağaç yapısı. Kullanıcı bu ağaçtan sürükleyerek elemanları bağlar.

```jsonc
{
  "$id": "fatura-schema",
  "type": "object",
  "properties": {
    "firma": {
      "type": "object",
      "properties": {
        "unvan": { "type": "string", "title": "Firma Ünvanı" },
        "vergiNo": { "type": "string", "title": "Vergi No" },
        "logo": { "type": "string", "title": "Logo", "format": "image" },
      },
    },
    "fatura": {
      "type": "object",
      "properties": {
        "no": { "type": "string", "title": "Fatura No" },
        "tarih": { "type": "string", "title": "Tarih", "format": "date" },
      },
    },
    "kalemler": {
      "type": "array",
      "title": "Fatura Kalemleri",
      "items": {
        "type": "object",
        "properties": {
          "siraNo": { "type": "integer", "title": "Sıra No" },
          "adi": { "type": "string", "title": "Ürün / Hizmet Adı" },
          "miktar": { "type": "number", "title": "Miktar" },
          "birim": { "type": "string", "title": "Birim" },
          "birimFiyat": {
            "type": "number",
            "title": "Birim Fiyat",
            "format": "currency",
          },
          "tutar": { "type": "number", "title": "Tutar", "format": "currency" },
        },
      },
    },
    "toplamlar": {
      "type": "object",
      "properties": {
        "araToplam": {
          "type": "number",
          "title": "Ara Toplam",
          "format": "currency",
        },
        "kdv": { "type": "number", "title": "KDV", "format": "currency" },
        "genelToplam": {
          "type": "number",
          "title": "Genel Toplam",
          "format": "currency",
        },
      },
    },
  },
}
```

---

## Binding Mekanizması

### Scalar Binding

Basit alan bağlama — bir eleman, JSON'daki tek bir değere bağlanır.

- Editörde: Kullanıcı schema ağacından bir alanı sürükleyip text elemanına bırakır.
- Template JSON'da: `"binding": { "type": "scalar", "path": "firma.vergiNo" }`
- Typst çıktısında: `#data.firma.vergiNo`

### Array Binding (Tekrarlayan Tablo)

Array verisi için özel tablo bileşeni. Kullanıcı:

1. Araç çubuğundan "Tekrarlayan Tablo" bileşenini sürükler.
2. `dataSource` olarak schema'daki bir array alanı seçer (ör: `kalemler`).
3. Sütun tanımlarında array'in alt alanlarını seçer (ör: `kalemler[].adi`).
4. Tablo stili (header rengi, zebra satırlar vs.) ayarlar.

Typst çıktısında:

```typst
#let kalemler = data.kalemler
#table(
  columns: (8%, 40%, 12%, 10%, 15%, 15%),
  align: (center, left, right, center, right, right),
  fill: (_, row) => if row == 0 { rgb("#f0f0f0") } else if calc.odd(row) { rgb("#fafafa") } else { none },
  [*\#*], [*Ürün / Hizmet*], [*Miktar*], [*Birim*], [*Birim Fiyat*], [*Tutar*],
  ..kalemler.map(k => (
    [#k.siraNo],
    [#k.adi],
    [#k.miktar],
    [#k.birim],
    [#format-currency(k.birimFiyat)],
    [#format-currency(k.tutar)],
  )).flatten()
)
```

### Format Fonksiyonları

Schema'daki `format` alanına göre Typst helper fonksiyonları üretilir:

- `currency` → para birimi formatlama (binlik ayracı, kuruş, ₺ sembolü)
- `date` → tarih formatlama (gün.ay.yıl)
- `percentage` → yüzde formatlama

---

## Template JSON → Typst Markup Dönüşümü

Bu dönüşüm hem frontend'de (WASM önizleme için) hem backend'de (PDF üretimi için) çalışır. Aynı saf fonksiyon her iki tarafta da kullanılmalıdır:

- **Frontend:** TypeScript'te yazılır (`core/template-to-typst.ts`).
- **Backend:** Aynı mantık Rust'ta implemente edilir.

Her iki implementasyon da birebir aynı Typst çıktısını üretmelidir. Tutarlılık testleri yazılmalıdır.

### Dönüşüm Kuralları (Container-Based)

1. **Sayfa ayarları** — kök container'ın padding'i = sayfa margin:

   ```typst
   #set page(width: 210mm, height: 297mm, margin: (top: 15mm, right: 15mm, bottom: 15mm, left: 15mm))
   ```

2. **Veri enjeksiyonu:**

   ```typst
   #let data = ( firma: ( unvan: "Acme A.Ş.", ... ), ... )
   ```

3. **Container (column) → `stack(dir: ttb)`:**

   ```typst
   #stack(dir: ttb, spacing: 5mm,
     [#text(size: 18pt, weight: "bold")[dreport]],
     [#text(size: 11pt, fill: rgb("#666666"))[Alt başlık]],
   )
   ```

4. **Container (row) → `stack(dir: ltr)`:**

   ```typst
   #stack(dir: ltr, spacing: 5mm,
     [#box(width: 1fr)[Sol kolon]],
     [#box(width: 1fr)[Sağ kolon]],
   )
   ```

5. **İç içe container → `box` + `stack`:**

   ```typst
   #box(width: 1fr, inset: (top: 5mm, bottom: 5mm))[
     #stack(dir: ttb, spacing: 3mm,
       [#text[Eleman 1]],
       [#text[Eleman 2]],
     )
   ]
   ```

6. **Absolute eleman → `place()` (sadece absolute positioning seçilmişse):**

   ```typst
   #place(top + left, dx: 130mm, dy: 30mm)[
     #text(size: 12pt, weight: "bold")[FATURA]
   ]
   ```

7. **Çizgi → `line()`:**

   ```typst
   #line(length: 1fr, stroke: 0.5pt + rgb("#000000"))
   ```

8. **Önizleme vs. nihai render:**
   - Önizleme: `data` mock veri ile doldurulur.
   - Nihai render: `data` gerçek JSON verisi ile doldurulur.

---

## SVG Overlay — Etkileşim Katmanı

Typst SVG'si read-only bir render çıktısıdır — tıklama veya sürükleme algılamaz. SVG'nin üstüne Vue bileşenleriyle bir etkileşim katmanı (overlay) koyulur.

### Overlay Nasıl Çalışır (Container Layout)

1. Overlay, template JSON'un ağaç yapısını yansıtır (recursive `ElementHandle` bileşenleri).
2. Kök overlay, sayfa padding'ini CSS padding olarak uygular.
3. Flow elemanlar `position: relative` ile doğal akışta durur.
4. Absolute elemanlar `position: absolute` ile parent container içinde konumlanır.
5. Tıklama ile seçim → mavi kenarlık (container ise mor kenarlık) + resize handle'lar.
6. Absolute elemanlar sürüklenebilir — drag sırasında CSS transform, bırakınca Typst re-render.
7. Flow elemanlar sürüklenemez — sıra değişikliği drag-to-reorder ile yapılır (ilerideki fazda).

### Overlay ↔ SVG Koordinat Eşleştirme

- Overlay container'ı, sayfa CSS variable'ları ile margin'leri eşler.
- Zoom yapıldığında hem SVG hem overlay aynı oranda scale edilir.
- Koordinat dönüşümü: `px = mm * (containerWidthPx / pageWidthMm) * zoomLevel`

---

## Proje Yapısı (Monorepo)

```
dreport/
├── CLAUDE.md
├── README.md
├── frontend/                       # Vue 3 + TypeScript
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── public/
│   │   └── fonts/                  # Typst WASM için gömülü fontlar
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.vue
│   │   ├── stores/                 # Pinia
│   │   │   ├── template.ts         # Template JSON state
│   │   │   ├── schema.ts           # JSON Schema state
│   │   │   └── editor.ts           # Editör UI state (seçili eleman, zoom vs.)
│   │   ├── components/
│   │   │   ├── editor/
│   │   │   │   ├── EditorCanvas.vue        # Ana editör alanı (SVG + overlay container)
│   │   │   │   ├── TypstSvgLayer.vue       # Typst SVG render katmanı
│   │   │   │   ├── InteractionOverlay.vue  # Etkileşim katmanı (tüm handle'ların parent'ı)
│   │   │   │   ├── ElementHandle.vue       # Tekil eleman seçim/drag/resize handle
│   │   │   │   ├── SnapGuides.vue          # Hizalama çizgileri
│   │   │   │   └── RulerBar.vue            # Cetvel
│   │   │   ├── panels/
│   │   │   │   ├── ToolboxPanel.vue        # Sol: bileşen araç kutusu
│   │   │   │   ├── SchemaTreePanel.vue     # Sol: JSON schema ağacı (drag source)
│   │   │   │   └── PropertiesPanel.vue     # Sağ: seçili elemanın özellikleri
│   │   │   ├── properties/
│   │   │   │   ├── TextProperties.vue
│   │   │   │   ├── TableProperties.vue
│   │   │   │   ├── ImageProperties.vue
│   │   │   │   └── StyleProperties.vue
│   │   │   └── common/
│   │   │       ├── ColorPicker.vue
│   │   │       ├── FontSelector.vue
│   │   │       └── UnitInput.vue           # mm/pt girişi
│   │   ├── composables/
│   │   │   ├── useTypstCompiler.ts         # Typst WASM yönetimi (Web Worker iletişimi)
│   │   │   ├── useDragDrop.ts              # Sürükle-bırak mantığı
│   │   │   ├── useElementSelection.ts      # Eleman seçimi
│   │   │   ├── useSnapGuides.ts            # Mıknatıslı hizalama
│   │   │   ├── useUndoRedo.ts              # Geri al / yinele
│   │   │   └── useZoomPan.ts               # Zoom ve kaydırma
│   │   ├── workers/
│   │   │   └── typst.worker.ts             # Typst WASM Web Worker
│   │   ├── core/
│   │   │   ├── template-to-typst.ts        # Template JSON → Typst markup dönüşümü
│   │   │   ├── schema-parser.ts            # JSON Schema → ağaç yapısı (panel için)
│   │   │   ├── mock-data-generator.ts      # Schema'dan örnek veri üretme
│   │   │   └── types.ts                    # Ortak TypeScript tip tanımları
│   │   └── styles/
│   │       └── editor.css
│   └── tests/
│       ├── template-to-typst.test.ts
│       └── schema-parser.test.ts
│
├── backend/                        # Rust + Axum
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── render.rs           # POST /api/render → PDF
│   │   │   └── health.rs           # GET /api/health
│   │   ├── typst_engine/
│   │   │   ├── mod.rs
│   │   │   ├── compiler.rs         # typst crate wrapper (World impl)
│   │   │   ├── template_to_typst.rs # Template JSON → Typst markup (Rust)
│   │   │   └── fonts.rs            # Font yönetimi ve yükleme
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── template.rs         # Template JSON serde modelleri
│   │       └── schema.rs           # JSON Schema modelleri
│   ├── fonts/                      # Gömülü font dosyaları
│   │   ├── NotoSans-Regular.ttf
│   │   ├── NotoSans-Bold.ttf
│   │   ├── NotoSans-Italic.ttf
│   │   └── NotoSansMono-Regular.ttf
│   └── tests/
│       ├── render_test.rs
│       └── template_to_typst_test.rs
│
└── shared/                         # Ortak şema tanımları
    └── schemas/
        ├── fatura.schema.json
        └── irsaliye.schema.json
```

---

## API Endpoints

İlk aşamada minimal API:

### `POST /api/render`

Template JSON + Data JSON alır, PDF döner.

**Request:**

```json
{
  "template": {},
  "data": {}
}
```

**Response:** `Content-Type: application/pdf` — binary PDF

### `GET /api/health`

Sunucu sağlık kontrolü.

---

## Editör UI/UX Davranışları

### Eleman Ekleme

- **Araç kutusundan sürükle-bırak:** Container, statik metin, çizgi, görsel, tekrarlayan tablo.
- **Container'a bırakma:** Eleman hedef container'ın flow'una eklenir. Drop pozisyonuna göre sıra belirlenir.
- **Schema ağacından sürükle-bırak:** Scalar alan container'a bırakılınca `text` elemanı oluşur, binding ayarlanır.
- **Schema'dan array alanı sürükle-bırak:** `repeating_table` oluşur, sütunları ayarlama diyaloğu açılır.

### Eleman Seçimi ve Manipülasyon

- Tıklama ile seçim → mavi kenarlık (container ise mor kenarlık) + resize handle'lar.
- **Flow elemanlar:** Sürüklenemez (pozisyon otomatik). Sıra değişikliği drag-to-reorder ile.
- **Absolute elemanlar:** Drag ile taşınır. Sürükleme sırasında CSS transform, bırakınca Typst re-render.
- **Container seçimi:** Tıklayınca sağ panelde direction, gap, align, padding ayarları.
- **Positioning modu değişikliği:** Sağ panelde flow ↔ absolute geçişi.
- Delete/Backspace ile silme.
- Shift+tıklama ile çoklu seçim.

### Undo/Redo

- Template JSON üzerinde immutable snapshot stack.
- Ctrl+Z / Ctrl+Shift+Z.

### Zoom ve Pan

- Ctrl+scroll ile zoom.
- Space+drag veya orta fare tuşu ile pan.
- Zoom aralığı: %25 – %400.

---

## Geliştirme Öncelikleri (Roadmap)

### Faz 1: Temel Altyapı ✓

- [x] Proje iskeleti kurulumu (Vue + Vite + Pinia, Axum boilerplate)
- [x] Typst WASM entegrasyonu — Web Worker'da Typst markup → SVG
- [x] Template JSON → Typst markup dönüşümü (static_text, text, line)
- [x] Container-based layout sistemi (tree yapı, flow + absolute positioning)
- [x] EditorCanvas: Typst SVG + recursive overlay + seçim
- [x] Absolute elemanlar için drag ile taşıma
- [x] Resize handle'lar
- [x] Backend iskeleti (Axum, health endpoint, render placeholder)
- [x] Font dosyaları (Noto Sans ailesi)

### Faz 2: Editör Temelleri

- [ ] `text` (dinamik binding'li) eleman tipi (Typst dönüşümü var, UI eksik)
- [ ] Schema tree paneli — JSON schema'dan ağaç oluşturma
- [ ] Schema'dan drag ile binding oluşturma
- [ ] Properties paneli — seçili elemanın stillerini düzenleme (font, renk, boyut, hizalama)
- [ ] Container properties paneli — direction, gap, padding, align ayarları
- [ ] Mock data generator — schema'dan örnek veri üretip önizlemede kullanma
- [ ] Undo/redo
- [ ] Toolbox paneli — eleman/container ekleme

### Faz 3: Tablo ve Array Binding

- [ ] `repeating_table` bileşeni ve Typst markup üretimi
- [ ] Sütun tanımlama UI'ı (alan seçimi, genişlik, hizalama)
- [ ] Array field'larına binding
- [ ] Tablo stili ayarları (header, zebra, border)
- [ ] Format fonksiyonları (currency, date)

### Faz 4: PDF Render Backend

- [ ] Axum server setup + `POST /api/render`
- [ ] Rust'ta template-to-typst dönüşümü (TypeScript versiyonuyla tutarlı)
- [ ] `typst` crate ile `World` trait implementasyonu
- [ ] PDF derleme ve response olarak dönme
- [ ] Font embed (frontend ile aynı font seti)
- [ ] Frontend'den "PDF İndir" butonu

### Faz 5: Polish

- [ ] Snap guides ve hizalama
- [ ] Zoom / pan
- [ ] `line`, `rect` eleman tipleri
- [ ] `image` eleman tipi (statik + dinamik)
- [ ] Sayfa numarası
- [ ] Çoklu sayfa desteği
- [ ] Template kaydetme / yükleme (JSON dosyası export/import)

---

## Önemli Teknik Notlar

### Typst WASM Font Stratejisi

Typst tarayıcıda çalışırken sistem fontlarına erişemez. Font dosyaları (`*.ttf` / `*.otf`) projeye dahil edilmeli ve WASM'a yüklenmelidir. Başlangıçta minimal bir set:

- Noto Sans (Regular, Bold, Italic, Bold Italic) — genel metin
- Noto Sans Mono (Regular) — tablo sayıları, monospace ihtiyaçları
- Toplam ~4-5 MB

**Kritik:** Backend'de (Rust) ve frontend'de (WASM) birebir aynı font dosyaları kullanılmalıdır. Farklı font = farklı metrik = render uyumsuzluğu.

### Koordinat Sistemi

- Tüm pozisyonlar **milimetre (mm)** cinsindendir.
- Template JSON'daki değerler mm, Typst'e `Xmm` olarak yazılır.
- Editör canvas'ta mm → px dönüşümü: `px = mm * (containerWidthPx / pageWidthMm) * zoomLevel`
- Referans: A4 = 210mm × 297mm.

### Typst Özel Karakter Escape

Template JSON → Typst dönüşümünde kullanıcı verisindeki özel karakterler escape edilmelidir:

- `#`, `$`, `@`, `*`, `_`, `<`, `>`, `\` Typst'te özel anlam taşır.
- Kullanıcı verisi `[...]` content block'a sarılarak büyük ölçüde güvenli hale gelir.
- İçerideki `[`, `]` karakterleri ise `\[`, `\]` olarak escape edilmelidir.

### Hata Yönetimi

- Typst derleme hatası olursa → editörde kırmızı banner ile hata mesajı göster.
- Derleme başarısız olduğunda son başarılı SVG'yi koru, kullanıcının çalışmasını bozma.
- Web Worker crash olursa → yeniden başlat, state'i koru.

### Eleman Sırası (Z-Order)

- Template JSON'daki `elements` dizisinin sırası = çizim sırası (sonraki üstte).
- Kullanıcı "Öne Getir" / "Arkaya Gönder" yapabilmeli → dizi sırası değişir.

---

## Rust Backend — Typst World Implementasyonu

Typst crate ile PDF üretmek için `World` trait'i implement etmek gerekir. Bu trait, Typst'e dosya sistemi, fontlar ve zaman bilgisi sağlar.

```rust
use typst::World;

struct DreportWorld {
    /// Ana .typ dosyasının içeriği (dinamik üretilen markup)
    main_source: String,
    /// Yüklenmiş font dosyaları
    fonts: Vec<typst::text::Font>,
    /// Data JSON (json dosyası olarak erişilebilir)
    data_json: String,
}

impl World for DreportWorld {
    // file(), font(), main(), source(), ... implementasyonları
}
```

Derleme akışı:

1. HTTP request gelir (template + data JSON).
2. Template JSON → Typst markup string üretilir.
3. Data JSON, sanal dosya sistemi üzerinden `data.json` olarak erişilebilir yapılır.
4. `DreportWorld` oluşturulur.
5. `typst::compile(&world)` → `Document` elde edilir.
6. `typst_pdf::pdf(&document, ...)` → PDF bytes.
7. Response olarak döndürülür.

---

## Kod Stili ve Konvansiyonlar

### Frontend (TypeScript / Vue)

- Composition API + `<script setup>` kullan, Options API KULLANMA.
- Pinia store'lar `defineStore` ile.
- Tip güvenliği: `strict: true` tsconfig'de. `any` kullanma, gerekirse `unknown` + type guard.
- Composable isimlendirme: `useXxx` pattern.
- Bileşen isimleri: PascalCase, en az iki kelime (ör: `EditorCanvas`, `SchemaTreePanel`).
- CSS: Scoped styles veya CSS modules. Global CSS minimum.

### Backend (Rust)

- Axum handler'lar async.
- Serde ile JSON serialize/deserialize (`#[derive(Serialize, Deserialize)]`).
- Hata yönetimi: `thiserror` ile typed errors, handler'larda `anyhow` kabul edilebilir.
- Typst crate dependency: `typst`, `typst-pdf`.
- Clippy uyarıları temiz tutulacak.

### Genel

- Commit mesajları: conventional commits (`feat:`, `fix:`, `refactor:`, `docs:` vs.).
- Türkçe yorum yazılabilir, kod ve değişken isimleri İngilizce.
- Template JSON field isimleri İngilizce (ör: `position`, `size`, `binding`).
- UI etiketleri ve kullanıcıya gösterilen metinler Türkçe.

---

## Kısıtlamalar ve Bilinçli Tercihler

1. **Veritabanı yok (ilk aşama).** Template'ler JSON dosyası olarak import/export edilir.
2. **Kullanıcı auth yok.** Tek kullanıcılı yerel kullanım senaryosu.
3. **Sadece PDF çıktı.** Typst'in SVG/PNG/HTML çıktıları ileride eklenebilir.
4. **Tekrarlayan bölge (repeating region) yok — sadece tekrarlayan tablo.** Array binding yalnızca tablo bileşeni ile yapılır. Serbest form repeating region ilerideki fazlarda değerlendirilir.
5. **WYSIWYG garantisi Typst üzerinden.** Editörde kendi render engine'imiz yok — Typst ne üretiyorsa kullanıcı onu görür.
6. **Canvas kütüphanesi (fabric.js / konva.js) kullanılmıyor.** Etkileşim katmanı saf Vue bileşenleri + pointer event'ler ile yapılır. Render zaten Typst SVG'sidir.
