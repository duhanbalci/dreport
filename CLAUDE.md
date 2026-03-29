# CLAUDE.md — dreport

## Proje Ozeti

**dreport**, kullanicilarin fatura, irsaliye, rapor gibi belge sablonlarini gorsel bir drag & drop editor ile tasarlayip, JSON veri ile birlestirerek PDF cikti almasini saglayan bir belge tasarim aracidir.

Temel fark: Editorde ayri bir canvas render engine (fabric.js, konva.js vb.) KULLANILMAZ. Bunun yerine custom bir layout engine (taffy + cosmic-text) kullanilir. Ayni layout engine hem editorde onizleme (HTML div'ler) hem backend'de PDF uretimi (krilla) icin calisir. Bu sayede "editorde gordugum ile PDF'te aldigim farkli" sorunu ortadan kalkar.

---

## Teknoloji Kararlari

| Katman            | Teknoloji                            | Gerekce                                                        |
| ----------------- | ------------------------------------ | -------------------------------------------------------------- |
| Frontend          | Vue 3 (Composition API) + TypeScript | Kullanici tercihi                                              |
| Layout Engine     | taffy (flexbox) + cosmic-text        | Template JSON → hesaplanmis pozisyonlar; hem WASM hem native   |
| Editor Render     | HTML div'ler (LayoutRenderer.vue)    | Layout engine sonuclarina gore CSS ile render                  |
| Etkilesim Katmani | DOM overlay (Vue bilesenleri)        | Layout sonuclari uzerine secim, surekleme, yeniden boyutlandirma |
| Backend           | Rust + Axum                          | Layout engine'i dogrudan kullanabilme; performans              |
| PDF Render        | krilla (server-side)                 | LayoutResult → PDF; font tutarliligi garantisi                 |
| Veri Formati      | JSON (sablon tanimi + veri)          | Evrensel, kolay serialize/deserialize                          |
| Paket Yonetimi    | bun (frontend), cargo (backend)      | —                                                              |

---

## Mimari Genel Bakis

```
┌──────────────────────────────────────────────────────┐
│                   VUE FRONTEND                        │
│                                                      │
│  ┌───────────────┐    ┌───────────────────────────┐  │
│  │ Sol Panel      │    │ Editor Canvas              │  │
│  │ - Bilesenler   │    │                           │  │
│  │ - Schema Tree  │    │  ┌─────────────────────┐  │  │
│  │ - Ozellikler   │    │  │ LayoutRenderer.vue  │  │  │
│  │                │    │  │ (HTML div render)   │  │  │
│  │                │    │  └─────────────────────┘  │  │
│  │                │    │  ┌─────────────────────┐  │  │
│  │                │    │  │ InteractionOverlay  │  │  │
│  │                │    │  │ secim / drag / resize│  │  │
│  │                │    │  └─────────────────────┘  │  │
│  └───────────────┘    └───────────────────────────┘  │
│                                                      │
│  Template JSON → layout-engine WASM → LayoutResult   │
│                → LayoutRenderer (HTML) + Overlay      │
└──────────────────────────┬───────────────────────────┘
                           │ POST /api/render
                           ▼
┌──────────────────────────────────────────────────────┐
│                   RUST BACKEND (Axum)                  │
│                                                      │
│  Template JSON + Data JSON                           │
│    → compute_layout() (taffy + cosmic-text)          │
│    → render_pdf() (krilla)                           │
│    → PDF bytes                                       │
└──────────────────────────────────────────────────────┘
```

---

## Render Pipeline

### Temel Prensip

Typst KULLANILMAZ. Bunun yerine custom layout engine:

1. Kullanicinin tasarimi bir **Template JSON** olarak tutulur.
2. Template JSON + Data JSON, **layout-engine** WASM modulu ile tarayicide islenir → **LayoutResult** uretilir.
3. LayoutResult, her elemanin mutlak pozisyonunu (x, y, width, height mm cinsinden) icerir.
4. **LayoutRenderer.vue** bu sonuclari HTML div'ler olarak render eder.
5. **InteractionOverlay.vue** ayni pozisyon bilgisiyle secim, drag, resize handle'lar koyar.
6. Kullanici bir elemani suruklediqinde → Template JSON guncellenir → layout-engine yeniden calisir → HTML guncellenir.

### Performans Stratejisi

- **Drag sirasinda:** Overlay katmaninda sadece CSS transform ile gorsel geri bildirim ver. Layout engine calistirma.
- **Drag bittiginde (mouseup/pointerup):** Template JSON'i guncelle, layout engine calistir, HTML'i yenile.
- **Debounce:** Ozellik panelinden yapilan degisikliklerde 150-300ms debounce ile hesapla.
- **Web Worker:** Layout engine WASM'i bir Web Worker icinde calistir. Ana thread'i ASLA bloklamayacak.

### Layout Engine (layout-engine crate)

```rust
// Temel kullanim
use dreport_layout::{compute_layout, LayoutResult};

let layout: LayoutResult = compute_layout(&template, &data, &fonts);
// layout.pages[0].elements → her elemanin x, y, width, height (mm)
```

WASM tarafinda (frontend):
```typescript
// layout.worker.ts icinde
import init, { computeLayout, loadFonts } from 'dreport-layout-wasm';

await init();
await loadFonts(fontBytes);
const layoutJson = computeLayout(templateJson, dataJson);
```

- WASM modulu: ~1-2 MB (vs eski Typst 8MB)
- Fontlar: Ayni Noto Sans seti (~4-5 MB), font olcum icin gerekli.

---

## Veri Modeli

### Layout Sistemi: Container-Based

CSS Flexbox mantigina benzeyen container-based layout:

- **Sayfa = kok container.** `page.margins` → kok container'in `padding`'i olur.
- **Container'lar ic ice gecebilir.** `direction: "row" | "column"` ile yatay/dikey dizilim.
- **Elemanlar varsayilan olarak flow icindedir** — otomatik pozisyonlanir.
- **Opsiyonel absolute positioning:** Kullanici isterse bir elemani `position: "absolute"` yapabilir.

Bu sayede:
- Tablo satirlari artarsa alttaki elemanlar otomatik kayar.
- Ayni satira iki kolon koymak icin ic ice container yeterlidir.
- Absolute mod ile serbest pozisyonlama da mumkundur.

### Boyut Sistemi (SizeValue)

Her eleman ve container icin `width` ve `height` su tiplerden biri olabilir:

| Tip     | Aciklama                              | Taffy karsiligi                |
| ------- | ------------------------------------- | ------------------------------ |
| `fixed` | Sabit boyut (mm)                      | `Dimension::Length(pt)`        |
| `auto`  | Iceriqe gore otomatik                 | `Dimension::Auto`              |
| `fr`    | Kalan alani oransal doldur            | `flex_grow: n, flex_basis: 0`  |

Ek olarak `minWidth`, `maxWidth`, `minHeight`, `maxHeight` (mm) desteklenir.

### Template JSON (Sablon Tanimi)

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

| Tip               | Aciklama                              | Binding          |
| ----------------- | ------------------------------------- | ---------------- |
| `container`       | Duzen kutusu, cocuk elemanlari barindirir | Yok          |
| `static_text`     | Sabit metin, veri baglantisi yok      | Yok              |
| `text`            | Dinamik metin, schema'dan veri ceker  | Scalar           |
| `repeating_table` | Array verisinden tekrarlayan tablo    | Array            |
| `line`            | Yatay/dikey cizgi                     | Yok              |
| `image`           | Statik veya dinamik gorsel            | Opsiyonel scalar |
| `page_number`     | Sayfa numarasi (cok sayfali belgeler) | Otomatik         |

### Container Ozellikleri

| Ozellik     | Tip                                      | Aciklama                          |
| ----------- | ---------------------------------------- | --------------------------------- |
| `direction` | `"row"` \| `"column"`                   | Cocuklari yatay mi dikey mi diz   |
| `gap`       | number (mm)                              | Cocuklar arasi bosluk             |
| `padding`   | `{ top, right, bottom, left }` (mm)     | Ic bosluk                         |
| `align`     | `"start"` \| `"center"` \| `"end"` \| `"stretch"` | Cross-axis hizalama   |
| `justify`   | `"start"` \| `"center"` \| `"end"` \| `"space-between"` | Main-axis dagilim |
| `style`     | `{ backgroundColor, borderColor, borderWidth, borderRadius }` | Gorsel stil |

### Positioning Modlari

| Mod        | Aciklama                                    | Taffy karsiligi                    |
| ---------- | ------------------------------------------- | ---------------------------------- |
| `flow`     | Parent container'in flow'una katil (default)| `Position::Relative`               |
| `absolute` | Parent container icinde sabit konum         | `Position::Absolute, inset: top/left` |

### Fatura Ornegi — Container Agaci

```
Sayfa (kok container, column, padding: 15mm)
├── Header (container, row, gap: 5mm)
│   ├── Logo alani (container, column, width: 60mm)
│   │   ├── image (logo)
│   │   └── text (firma unvani)
│   └── Fatura bilgi (container, column, width: fill, align: end)
│       ├── static_text ("FATURA")
│       ├── text (fatura no)
│       └── text (tarih)
├── line (ayirici cizgi)
├── repeating_table (kalemler)
└── Footer (container, row)
    ├── bos alan (width: fill)
    └── Toplamlar (container, column, width: 80mm)
        ├── text (ara toplam)
        ├── text (KDV)
        └── text (genel toplam)
```

### Data JSON (Gercek Veri)

Render zamaninda sablonla birlestirilen veri:

```jsonc
{
  "firma": {
    "unvan": "Acme Teknoloji A.S.",
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
      "adi": "Web Gelistirme Hizmeti",
      "miktar": 1,
      "birim": "Adet",
      "birimFiyat": 15000,
      "tutar": 15000,
    },
    {
      "siraNo": 2,
      "adi": "SSL Sertifikasi",
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

### JSON Schema (Veri Yapisi Tanimi)

Editorun sol panelinde kullaniciya sunulan, baglanabilir alanlarin agac yapisi. Kullanici bu agactan surukleyerek elemanlari baglar.

```jsonc
{
  "$id": "fatura-schema",
  "type": "object",
  "properties": {
    "firma": {
      "type": "object",
      "properties": {
        "unvan": { "type": "string", "title": "Firma Unvani" },
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
          "siraNo": { "type": "integer", "title": "Sira No" },
          "adi": { "type": "string", "title": "Urun / Hizmet Adi" },
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

## Binding Mekanizmasi

### Scalar Binding

Basit alan baglama — bir eleman, JSON'daki tek bir degere baglanir.

- Editorde: Kullanici schema agacindan bir alani surukleyip text elemanina birakir.
- Template JSON'da: `"binding": { "type": "scalar", "path": "firma.vergiNo" }`
- Layout engine'de: `data_resolve.rs` JSON path'i cozumler, text icerigini uretir.

### Array Binding (Tekrarlayan Tablo)

Array verisi icin ozel tablo bileseni. Kullanici:

1. Arac kutusundan "Tekrarlayan Tablo" bilesenini surukler.
2. `dataSource` olarak schema'daki bir array alani secer (or: `kalemler`).
3. Sutun tanimlarinda array'in alt alanlarini secer (or: `kalemler[].adi`).
4. Tablo stili (header rengi, zebra satirlar vs.) ayarlar.

Layout engine'de: `table_layout.rs` repeating_table'i satir/sutun container agacina acar, taffy ile layout hesaplar.

### Format Fonksiyonlari

Schema'daki `format` alanina gore formatlama yapilir:

- `currency` → para birimi formatlama (binlik ayiraci, kurus, ₺ sembolu)
- `date` → tarih formatlama (gun.ay.yil)
- `percentage` → yuzde formatlama

---

## Layout Engine Detaylari

### LayoutResult (engine ciktisi)

```rust
pub struct LayoutResult {
    pub pages: Vec<PageLayout>,
}

pub struct PageLayout {
    pub width_mm: f64,
    pub height_mm: f64,
    pub elements: Vec<ElementLayout>,
}

pub struct ElementLayout {
    pub id: String,
    pub x_mm: f64,           // Sayfa sol ustten mutlak pozisyon
    pub y_mm: f64,
    pub width_mm: f64,
    pub height_mm: f64,
    pub element_type: String,
    pub content: Option<ResolvedContent>,
    pub style: ResolvedStyle,
}
```

### Taffy Mapping

| dreport                         | taffy                              |
| ------------------------------- | ---------------------------------- |
| `container(direction: row)`     | `FlexDirection::Row`               |
| `container(direction: column)`  | `FlexDirection::Column`            |
| `gap`                           | `gap: Size { width, height }`     |
| `padding`                       | `padding: Rect { top, right, bottom, left }` |
| `align: start/center/end/stretch` | `align_items`                   |
| `justify: start/center/end/space-between` | `justify_content`       |
| `SizeValue::Fixed(mm)`          | `Dimension::Length(pt)`            |
| `SizeValue::Auto`               | `Dimension::Auto`                  |
| `SizeValue::Fr(n)`              | `flex_grow: n, flex_basis: 0`      |
| `PositionMode::Absolute`        | `Position::Absolute, inset: top/left` |

Text leaf node'lari → taffy `MeasureFunc` callback'i ile cosmic-text'ten olcum alir.

### PDF Render (Backend)

```rust
// render.rs akisi
let layout = dreport_layout::compute_layout(&template, &data, &fonts);
let pdf_bytes = dreport_layout::pdf_render::render_pdf(&layout, &fonts)?;
// → Response olarak dondurulur
```

krilla kutuphanesi ile her ElementLayout'u PDF sayfasina cizer: text, line, rect, image, table.

---

## Interaction Overlay

Layout engine pozisyon bilgisi uretiyor — bu pozisyonlar hem gorsel render (LayoutRenderer.vue) hem etkilesim katmani (InteractionOverlay.vue) tarafindan kullanilir.

### Overlay Nasil Calisir

1. LayoutResult'taki her eleman icin pozisyon ve boyut bilgisi (mm) alinir.
2. mm → px donusumu yapilir: `px = mm * scale * zoomLevel`
3. Her eleman icin CSS `position: absolute` + `left/top/width/height` ile handle yerlestirilir.
4. Tikla ile secim → mavi kenarlik (container ise mor kenarlik) + resize handle'lar.
5. Absolute elemanlar suruklenebilir — drag sirasinda CSS transform, birakinca layout engine re-compute.
6. Flow elemanlar suruklenemez — sira degisikligi drag-to-reorder ile yapilir.

### En Buyuk Kazanim

Eski mimari: Typst SVG (opak) + Vue overlay (pozisyon sync'i sorunlu)
Yeni mimari: Layout engine pozisyon verir → DOM div'ler hem gorsel hem etkilesim katmani. Ayri SVG/overlay senkronizasyonu sorunu yok.

---

## Proje Yapisi (Monorepo)

```
dreport/
├── CLAUDE.md
├── Cargo.toml                      # Workspace: core, backend, layout-engine
├── justfile
├── frontend/                       # Vue 3 + TypeScript
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── public/
│   │   └── fonts/                  # Layout engine WASM icin gomulu fontlar
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.vue
│   │   ├── stores/                 # Pinia
│   │   │   ├── template.ts         # Template JSON state
│   │   │   ├── schema.ts           # JSON Schema state
│   │   │   └── editor.ts           # Editor UI state (secili eleman, zoom vs.)
│   │   ├── components/
│   │   │   ├── editor/
│   │   │   │   ├── EditorCanvas.vue        # Ana editor alani (LayoutRenderer + overlay)
│   │   │   │   ├── LayoutRenderer.vue      # LayoutResult → HTML div render
│   │   │   │   ├── InteractionOverlay.vue  # Etkilesim katmani (secim/drag/resize)
│   │   │   │   ├── ElementHandle.vue       # Tekil eleman secim/drag/resize handle
│   │   │   │   ├── SnapGuides.vue          # Hizalama cizgileri
│   │   │   │   └── RulerBar.vue            # Cetvel
│   │   │   ├── panels/
│   │   │   │   ├── ToolboxPanel.vue        # Sol: bilesen arac kutusu
│   │   │   │   ├── SchemaTreePanel.vue     # Sol: JSON schema agaci (drag source)
│   │   │   │   └── PropertiesPanel.vue     # Sag: secili elemanin ozellikleri
│   │   │   ├── properties/
│   │   │   │   ├── TextProperties.vue
│   │   │   │   ├── TableProperties.vue
│   │   │   │   ├── ImageProperties.vue
│   │   │   │   └── StyleProperties.vue
│   │   │   └── common/
│   │   │       ├── ColorPicker.vue
│   │   │       ├── FontSelector.vue
│   │   │       └── UnitInput.vue           # mm/pt girisi
│   │   ├── composables/
│   │   │   ├── useLayoutEngine.ts          # Layout engine WASM yonetimi (Web Worker iletisimi)
│   │   │   ├── useDragDrop.ts              # Surukle-birak mantigi
│   │   │   ├── useElementSelection.ts      # Eleman secimi
│   │   │   ├── useSnapGuides.ts            # Miknatisli hizalama
│   │   │   ├── useUndoRedo.ts              # Geri al / yinele
│   │   │   └── useZoomPan.ts               # Zoom ve kaydirma
│   │   ├── workers/
│   │   │   └── layout.worker.ts            # Layout engine WASM Web Worker
│   │   ├── core/
│   │   │   ├── types.ts                    # Ortak TypeScript tip tanimlari
│   │   │   ├── layout-types.ts             # LayoutResult TypeScript tipleri
│   │   │   ├── schema-parser.ts            # JSON Schema → agac yapisi (panel icin)
│   │   │   └── mock-data-generator.ts      # Schema'dan ornek veri uretme
│   │   └── styles/
│   │       └── editor.css
│   └── tests/
│       └── schema-parser.test.ts
│
├── core/                           # Ortak Rust modelleri
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── models.rs               # Template JSON serde modelleri (tum crate'ler kullanir)
│
├── layout-engine/                  # Custom layout engine (taffy + cosmic-text)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Public API: compute_layout()
│       ├── tree.rs                 # Template → taffy node tree
│       ├── sizing.rs               # SizeValue → taffy Style mapping
│       ├── text_measure.rs         # cosmic-text ile text olcum
│       ├── table_layout.rs         # RepeatingTable → container agacina expand
│       ├── data_resolve.rs         # Binding'leri cozumle (gercek text content uret)
│       ├── page_break.rs           # Cok sayfali belgeler icin icerik bolme
│       ├── pdf_render.rs           # LayoutResult → PDF (krilla, sadece native)
│       ├── wasm_api.rs             # wasm_bindgen exports (loadFonts, computeLayout)
│       └── font.rs                 # Font yukleme (WASM fetch vs native file read)
│
├── backend/                        # Rust + Axum
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── render.rs           # POST /api/render → PDF (layout-engine kullanir)
│   │   │   └── health.rs           # GET /api/health
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── template.rs         # Template JSON serde modelleri
│   │       └── schema.rs           # JSON Schema modelleri
│   ├── fonts/                      # Gomulu font dosyalari
│   │   ├── NotoSans-Regular.ttf
│   │   ├── NotoSans-Bold.ttf
│   │   ├── NotoSans-Italic.ttf
│   │   └── NotoSansMono-Regular.ttf
│   └── tests/
│       └── render_test.rs
│
└── shared/                         # Ortak sema tanimlari
    └── schemas/
        ├── fatura.schema.json
        └── irsaliye.schema.json
```

---

## API Endpoints

### `POST /api/render`

Template JSON + Data JSON alir, PDF doner.

**Request:**

```json
{
  "template": {},
  "data": {}
}
```

**Response:** `Content-Type: application/pdf` — binary PDF

**Akis:**
1. Template + Data JSON parse edilir.
2. `compute_layout(template, data, fonts)` → `LayoutResult`
3. `render_pdf(layout_result, fonts)` → PDF bytes
4. Response olarak dondurulur.

### `GET /api/health`

Sunucu saglik kontrolu.

---

## Editor UI/UX Davranislari

### Eleman Ekleme

- **Arac kutusundan surukle-birak:** Container, statik metin, cizgi, gorsel, tekrarlayan tablo.
- **Container'a birakma:** Eleman hedef container'in flow'una eklenir. Drop pozisyonuna gore sira belirlenir.
- **Schema agacindan surukle-birak:** Scalar alan container'a birakilinca `text` elemani olusur, binding ayarlanir.
- **Schema'dan array alani surukle-birak:** `repeating_table` olusur, sutunlari ayarlama diyalogu acilir.

### Eleman Secimi ve Manipulasyon

- Tiklama ile secim → mavi kenarlik (container ise mor kenarlik) + resize handle'lar.
- **Flow elemanlar:** Suruklenemez (pozisyon otomatik). Sira degisikligi drag-to-reorder ile.
- **Absolute elemanlar:** Drag ile tasinir. Surukleme sirasinda CSS transform, birakinca layout engine re-compute.
- **Container secimi:** Tiklayinca sag panelde direction, gap, align, padding ayarlari.
- **Positioning modu degisikligi:** Sag panelde flow ↔ absolute gecisi.
- Delete/Backspace ile silme.
- Shift+tiklama ile coklu secim.

### Undo/Redo

- Template JSON uzerinde immutable snapshot stack.
- Ctrl+Z / Ctrl+Shift+Z.

### Zoom ve Pan

- Ctrl+scroll ile zoom.
- Space+drag veya orta fare tusu ile pan.
- Zoom araligi: %25 – %400.

---

## Gelistirme Oncelikleri (Roadmap)

### Faz 1: Temel Altyapi ✓

- [x] Proje iskeleti kurulumu (Vue + Vite + Pinia, Axum boilerplate)
- [x] Container-based layout sistemi (tree yapi, flow + absolute positioning)
- [x] Font dosyalari (Noto Sans ailesi)

### Faz 2: Custom Layout Engine ✓

- [x] layout-engine crate olusturma (taffy + cosmic-text)
- [x] Template → taffy node tree donusumu (tree.rs)
- [x] SizeValue mapping (sizing.rs)
- [x] Text olcum (text_measure.rs, cosmic-text)
- [x] Binding cozumleme (data_resolve.rs)
- [x] Tablo expansion (table_layout.rs)
- [x] WASM bindings (wasm_api.rs)
- [x] Frontend entegrasyonu (layout.worker.ts, useLayoutEngine.ts, LayoutRenderer.vue)
- [x] InteractionOverlay adaptasyonu
- [x] Typst bagimliliklarinin kaldirilmasi (backend)

### Faz 3: PDF Render ✓

- [x] pdf_render.rs — krilla ile PDF uretimi
- [x] Backend route guncelleme (POST /api/render)
- [x] Page break desteqi (page_break.rs)

### Faz 4: Editor Temelleri

- [ ] Schema tree paneli — JSON schema'dan agac olusturma
- [ ] Schema'dan drag ile binding olusturma
- [ ] Properties paneli — secili elemanin stillerini duzenleme (font, renk, boyut, hizalama)
- [ ] Container properties paneli — direction, gap, padding, align ayarlari
- [ ] Mock data generator — schema'dan ornek veri uretip onizlemede kullanma
- [ ] Undo/redo
- [ ] Toolbox paneli — eleman/container ekleme

### Faz 5: Tablo ve Array Binding

- [ ] Sutun tanimlama UI'i (alan secimi, genislik, hizalama)
- [ ] Array field'larina binding
- [ ] Tablo stili ayarlari (header, zebra, border)
- [ ] Format fonksiyonlari (currency, date)

### Faz 6: Polish

- [ ] Snap guides ve hizalama
- [ ] Zoom / pan
- [ ] `image` eleman tipi (statik + dinamik)
- [ ] Sayfa numarasi
- [ ] Template kaydetme / yukleme (JSON dosyasi export/import)

---

## Onemli Teknik Notlar

### Font Stratejisi

Layout engine (hem WASM hem native) cosmic-text ile text olcum yapar — bu nedenle font dosyalarina ihtiyac duyar. Font dosyalari projeye dahil edilmeli ve hem WASM'a hem backend'e yuklenmelidir. Baslangicta minimal bir set:

- Noto Sans (Regular, Bold, Italic, Bold Italic) — genel metin
- Noto Sans Mono (Regular) — tablo sayilari, monospace ihtiyaclari
- Toplam ~4-5 MB

**Kritik:** Backend'de (native Rust) ve frontend'de (WASM) birebir ayni font dosyalari kullanilmalidir. Farkli font = farkli metrik = layout uyumsuzlugu.

### Koordinat Sistemi

- Tum pozisyonlar **milimetre (mm)** cinsindendir.
- Template JSON'daki degerler mm, taffy'ye point'e cevrilerek verilir.
- LayoutResult'taki degerler mm cinsindendir.
- Editor canvas'ta mm → px donusumu: `px = mm * scale * zoomLevel`
- Referans: A4 = 210mm × 297mm.

### Hata Yonetimi

- Layout engine hatasi olursa → editorde kirmizi banner ile hata mesaji goster.
- Hesaplama basarisiz oldugunda son basarili LayoutResult'i koru, kullanicinin calsimasini bozma.
- Web Worker crash olursa → yeniden baslat, state'i koru.

### Eleman Sirasi (Z-Order)

- Template JSON'daki `children` dizisinin sirasi = cizim sirasi (sonraki ustte).
- Kullanici "One Getir" / "Arkaya Gonder" yapabilmeli → dizi sirasi degisir.

---

## Kod Stili ve Konvansiyonlar

### Frontend (TypeScript / Vue)

- Composition API + `<script setup>` kullan, Options API KULLANMA.
- Pinia store'lar `defineStore` ile.
- Tip guvenligi: `strict: true` tsconfig'de. `any` kullanma, gerekirse `unknown` + type guard.
- Composable isimlendirme: `useXxx` pattern.
- Bilesen isimleri: PascalCase, en az iki kelime (or: `EditorCanvas`, `SchemaTreePanel`).
- CSS: Scoped styles veya CSS modules. Global CSS minimum.

### Backend (Rust)

- Axum handler'lar async.
- Serde ile JSON serialize/deserialize (`#[derive(Serialize, Deserialize)]`).
- Hata yonetimi: `thiserror` ile typed errors, handler'larda `anyhow` kabul edilebilir.
- Layout engine dependency: `dreport-layout` crate.
- Clippy uyarilari temiz tutulacak.

### Genel

- Commit mesajlari: conventional commits (`feat:`, `fix:`, `refactor:`, `docs:` vs.).
- Turkce yorum yazilabilir, kod ve degisken isimleri Ingilizce.
- Template JSON field isimleri Ingilizce (or: `position`, `size`, `binding`).
- UI etiketleri ve kullaniciya gosterilen metinler Turkce.

---

## Kisitlamalar ve Bilincli Tercihler

1. **Veritabani yok (ilk asama).** Template'ler JSON dosyasi olarak import/export edilir.
2. **Kullanici auth yok.** Tek kullanicili yerel kullanim senaryosu.
3. **Sadece PDF cikti.** Ileride PNG/SVG eklenebilir.
4. **Tekrarlayan bolge (repeating region) yok — sadece tekrarlayan tablo.** Array binding yalnizca tablo bileseni ile yapilir. Serbest form repeating region ilerideki fazlarda degerlendirilir.
5. **WYSIWYG garantisi layout engine uzerinden.** Ayni layout engine (taffy + cosmic-text) hem editorde hem PDF'te kullanilir. Editor HTML div render, PDF krilla render — ama pozisyonlar ayni engine'den gelir.
6. **Canvas kutuphanesi (fabric.js / konva.js) kullanilmiyor.** Etkilesim katmani saf Vue bilesenleri + pointer event'ler ile yapilir. Render LayoutRenderer.vue ile HTML div'lerdir.
