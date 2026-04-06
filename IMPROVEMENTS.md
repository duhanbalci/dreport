# dreport - Improvement & Feature Tracker

> Bu dosya projenin kapsamli analizinden elde edilen bulgu, iyilestirme ve yeni ozellik onerilerini icerir.
> Her basligin yanindaki durum etiketi, ilgili madde tamamlandiginda `[IMPLEMENTE EDILDI]` olarak guncellenecektir.

---

## 1. Kritik Buglar

### 1.1 Undo/Redo `Object.assign` Hatasi `[IMPLEMENTE EDILDI]`

**Dosya:** `frontend/src/composables/useUndoRedo.ts` (satir 52)

**Sorun:**
`applySnapshot` fonksiyonu snapshot'i geri yuklerken `Object.assign(source.value, JSON.parse(snap))` kullaniyor. `Object.assign` shallow merge yapar — mevcut objede olan ama snapshot'ta olmayan key'leri **silmez**. Bu, ozellikle `header` ve `footer` toggle islemlerinde ciddi bir bug olusturur.

**Senaryo:**
1. Kullanici template'e header ekler (`template.header` olusur)
2. Ctrl+Z ile geri alir
3. Snapshot header eklenmeden onceki state'i icerir ama `Object.assign` `header` key'ini silemez
4. Header hala template'te kalir — undo calismamis olur

**Cozum:**
```typescript
// YANLIS (mevcut)
Object.assign(source.value as object, JSON.parse(snap))

// DOGRU
source.value = JSON.parse(snap)
```
Vue'nun reactivity sistemi ref degeri tamamen degistirildiginde dogru calisiyor. Reference replacement ile tum key'ler (silinen dahil) dogru sekilde geri yuklenir.

**Ek Sorun — Debounce Race Condition:**
Undo/redo watcher'da 300ms debounce var. Kullanici hizli bir edit yapip 300ms icinde Ctrl+Z basarsa, snapshot henuz push edilmemis olabilir ve undo onceki-onceki state'e doner. Debounce yerine `requestIdleCallback` veya edit sonrasi aninda flush mekanizmasi dusunulmeli.

---

### 1.2 PDF'te Text Wrapping Yok `[IMPLEMENTE EDILDI]`

**Dosya:** `layout-engine/src/pdf_render.rs` (satir ~487)

**Sorun:**
`render_text()` fonksiyonu metni tek bir `draw_text()` cagrisiyla ciziyor. Taffy layout engine'i text olcum sirasinda cosmic-text uzerinden line-break hesapliyor ve yuksekligi buna gore belirliyor. Ancak PDF render asamasinda bu line-break bilgisi kullanilmiyor — metin tek satirda, kutudan tasarak ciziliyor.

**Etki:**
Bu, projenin temel vaadi olan "editorde gordugum = PDF'te aldigim" WYSIWYG garantisini kiran en buyuk bug. Editorde birden fazla satira sarilan bir text elemani, PDF'te tek satir olarak kutudan tasar.

**Cozum Yaklasimi:**
1. `text_measure.rs`'deki cosmic-text `Buffer`'dan line-break pozisyonlarini `LayoutResult`'a tasimak
2. `pdf_render.rs`'de her satiri ayri `draw_text()` cagrisiyla, dogru y-offset ile cizmek
3. Alternatif: PDF render sirasinda cosmic-text'i tekrar calistirip line layout almak (daha basit ama daha yavas)

---

### 1.3 Image objectFit Hardcoded `[IMPLEMENTE EDILDI]`

**Dosya:** `frontend/src/components/editor/LayoutRenderer.vue` (satir ~229)

**Sorun:**
Image render sirasinda `objectFit` degeri sabit `'fill'` olarak ataniyor:
```typescript
objectFit: 'fill',  // el.style.objectFit degerini yok sayiyor
```

`ImageStyle` tipi, `ImageElement` ve `ImageProperties.vue` hepsi `contain | cover | stretch` destekliyor. `ResolvedStyle` interface'inde `objectFit` alani var. Ancak `LayoutRenderer` bunu okumuyor.

**Etki:**
Editor onizlemede tum gorseller her zaman `fill` modunda gosteriliyor. Kullanici `contain` veya `cover` secse bile editorde fark gormuyor.

**Cozum:**
```typescript
objectFit: el.style.objectFit || 'fill',
```

---

### 1.4 PDF'te Italic Font Secilmiyor `[IMPLEMENTE EDILDI]`

**Dosya:** `layout-engine/src/pdf_render.rs` (satir ~104)

**Sorun:**
`FontCollection::get()` metodu her zaman `is_italic: false` gonderiyor. Italic font variant'lari collection'a yukleniyor ama hicbir zaman secilemiyorlar.

**Etki:**
Template'te `fontStyle: "italic"` olarak ayarlanmis metin, PDF ciktisinda normal (regular) olarak goruntulenir. Editor tarafinda HTML/CSS italic destekledigi icin sorun gorunmuyor, ama PDF farkli cikiyor.

**Cozum:**
`FontCollection::get()` metoduna `is_italic` parametresi ekleyip, `ResolvedStyle.fontStyle` degerine gore italic font secimi yapmak.

---

## 2. Onemli Teknik Sorunlar

### 2.1 `repeat_header` Flag'i Kontrol Edilmiyor `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/table_layout.rs`, `layout-engine/src/page_break.rs`

**Sorun:**
`RepeatingTableElement` uzerinde `repeat_header: Option<bool>` alani tanimli ve default degeri `true`. Ancak `table_layout.rs`'deki tablo genisletme kodu bu flag'i hic kontrol etmiyor — header her zaman tekrarlaniyor.

**Etki:**
Kullanici tablo header tekrarini kapatamaz. Bazi belge tasarimlarinda (ornegin ozetlerde) header tekrari istenmeyebilir.

**Cozum:**
`page_break.rs`'deki header klonlama mantigi `repeat_header` flag'ini kontrol etmeli. `false` ise yeni sayfada header eklememeli.

---

### 2.2 TableColumn.format Uygulanmiyor `[IMPLEMENTE EDILMEDI]`

**Dosya:** `core/src/models.rs`, `layout-engine/src/table_layout.rs`, `layout-engine/src/data_resolve.rs`

**Sorun:**
`TableColumn` struct'inda `format: Option<String>` alani tanimli ama pipeline boyunca hic kullanilmiyor. Sutun bazinda currency, date veya percentage formatlama calismaz.

**Etki:**
Kullanici bir tablo sutununu `currency` formatinda tanimlarsa, hucrelerdeki sayilar ham haliyle gosterilir (ornegin `15000` yerine `15.000,00 ₺` olmaz).

**Cozum:**
`data_resolve.rs`'de tablo satir verisi cozumlenirken, ilgili sutunun `format` degerini `expr_eval::apply_format()` fonksiyonuna gecirerek formatlama uygulamak.

---

### 2.3 rounded_rectangle Shape PDF'te Duz Dikdortgen `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/pdf_render.rs` — `render_shape()`

**Sorun:**
Shape render fonksiyonu `ellipse` disindaki tum shape tiplerini duz dikdortgen olarak ciziyor. `rounded_rectangle` tipi ve `border_radius` stili yok sayiliyor.

**Cozum:**
`border_radius > 0` kontrolu ile krilla'nin rounded rectangle API'sini kullanmak.

---

### 2.4 Chart Render Kod Tekrari (~400 Satir) `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/chart_render.rs` (SVG), `layout-engine/src/pdf_render.rs` (PDF chart bolumu)

**Sorun:**
Chart rendering iki ayri yerde uygulanmis: SVG icin `chart_render.rs`, PDF icin `pdf_render.rs`. Margin hesaplama, eksen cizimi, etiket yerlesimi, legend mantigi gibi ~400 satirlik logic her iki dosyada tekrarlaniyor.

**Etki:**
Bir chart ozelligindeki degisiklik iki dosyada ayri ayri yapilmak zorunda. Senkronizasyon unutuldugunda SVG ve PDF chart'lar farkli gorunur.

**Cozum:**
Ortak bir `ChartLayout` struct'i ile hesaplama mantigi tek yerde yapilip, SVG ve PDF renderer'lara sadece cizim primitive'leri gecirilmeli. Strategy/trait pattern ile:
```rust
trait ChartRenderer {
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: &LineStyle);
    fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64, style: &FillStyle);
    fn draw_text(&mut self, x: f64, y: f64, text: &str, style: &TextStyle);
}
```

---

### 2.5 Taffy unwrap() Kullanimi — Panic Riski `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/tree.rs` (satirlar: 80, 93, 143, 156, 215, 345, 366, 397)

**Sorun:**
Taffy'nin `new_with_children()`, `compute_layout_with_measure()`, `layout()` gibi metodlari `Result` donduruyor ama tumu `.unwrap()` ile cagiriliyor. Taffy internal hatasi durumunda (bellek yetersizligi, invalid tree state) program panic yapar.

**Etki:**
Backend'de bir template render istegi panic'e yol acarsa, o Tokio task sonlanir. WASM tarafinda panic tum worker'i oldurur.

**Cozum:**
`unwrap()` yerine `map_err` ile `LayoutError` tipine donusturmek ve `compute_layout` fonksiyonundan `Result<LayoutResult, LayoutError>` dondurmek.

---

### 2.6 Backend PDF Render Async Thread Blocking `[IMPLEMENTE EDILMEDI]`

**Dosya:** `backend/src/routes/render.rs` (satir ~25)

**Sorun:**
`compute_layout()` ve `render_pdf()` senkron, CPU-intensive islemler. Axum async handler icinde dogrudan cagiriliyorlar — bu Tokio async thread'ini bloklar.

**Etki:**
Yogun yuklenme altinda veya buyuk template'lerde diger HTTP isteklerinin islenmesi gecikir. Tokio'nun async avantaji kaybolur.

**Cozum:**
```rust
let pdf_bytes = tokio::task::spawn_blocking(move || {
    let layout = compute_layout(&template, &data, &fonts);
    render_pdf(&layout, &fonts)
}).await??;
```

---

### 2.7 Currency Formatting Hardcoded Turkce `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/expr_eval.rs` (satir ~86)

**Sorun:**
`format_currency()` fonksiyonu Turk Lirasi formati icin hardcoded:
- `.` binlik ayiraci
- `,` ondalik ayiraci
- `₺` para birimi sembolu

Chart render'daki `format_value()` ise `.` ondalik ayirici ve `K/M` kisaltma kullaniyor — iki farkli lokalizasyon.

**Cozum:**
Bir `Locale` veya `FormatConfig` struct'i olusturup, template seviyesinde veya global config ile para birimi, ondalik ayiraci ve binlik ayiraci belirlenebilir hale getirmek.

---

### 2.8 Worker Font Fetch Hata Yakalama Yok `[IMPLEMENTE EDILMEDI]`

**Dosya:** `frontend/src/workers/layout.worker.ts` (satirlar 25-33)

**Sorun:**
Font dosyalari `await fetch(...)` ile yukleniyor, hic `try/catch` veya response status kontrolu yok. Font dosyasi 404 donerse `Promise.all` bos/kirik buffer ile resolve olur ve WASM `loadFonts` yanlis metriklerle sessizce devam eder.

**Etki:**
Font yuklenemezse layout engine kirik metriklerle calisir — text boyutlari yanlis hesaplanir, WYSIWYG bozulur, hata mesaji gorulmez.

**Cozum:**
```typescript
const res = await fetch(url)
if (!res.ok) throw new Error(`Font yuklenemedi: ${url} (${res.status})`)
const buffer = await res.arrayBuffer()
```

---

### 2.9 importTemplate Validasyon Eksikligi `[IMPLEMENTE EDILMEDI]`

**Dosya:** `frontend/src/stores/template.ts` (satir ~195)

**Sorun:**
`importTemplate` metodu `JSON.parse` sonucunu hic dogrulamadan store'a yaziyor. Bozuk veya eksik alanli JSON, store'u ara durumda birakir.

**Cozum:**
1. `try/catch` ile parse hatalarini yakalamak
2. Minimum schema dogrulamasi: `root` alani var mi, `root.type === 'container'` mi, `page` alani gecerli mi
3. Basarisiz durumda onceki state'i korumak

---

### 2.10 Barcode Promise Timeout Yok `[IMPLEMENTE EDILMEDI]`

**Dosya:** `frontend/src/composables/useLayoutEngine.ts`

**Sorun:**
`generateBarcode()` bir Promise donduruyor ama timeout mekanizmasi yok. Worker crash olursa veya takilirsa, promise sonsuza kadar pending kalir. `dispose()` metodu da bekleyen promise'leri resolve/reject etmiyor.

**Cozum:**
```typescript
const timeout = setTimeout(() => {
    barcodeCallbacks.delete(id)
    resolve(null)
}, 5000)
```
`dispose()` icinde tum pending callback'leri `null` ile resolve etmek.

---

### 2.11 moveElement Cift Layout Recompute `[IMPLEMENTE EDILMEDI]`

**Dosya:** `frontend/src/stores/template.ts`

**Sorun:**
`moveElement` fonksiyonu `removeElement()` + `addChild()` cagiriyor, her biri `layoutVersion++` yapiyor. Tek bir mantiksal islem icin iki layout recompute tetikleniyor.

**Cozum:**
`moveElement` icinde `layoutVersion` bump'ini tek seferde yapmak:
- `removeElement` ve `addChild`'in internal versiyonlarini olustur (version bump'siz)
- Islemin sonunda tek bir `layoutVersion++` yap

---

### 2.12 Barcode ID Collision Riski `[IMPLEMENTE EDILMEDI]`

**Dosya:** `frontend/src/composables/useLayoutEngine.ts`

**Sorun:**
Barcode request ID'leri `barcodeReqId + 100000` offset'i ile layout request ID'lerinden ayristiriliyor. Uzun sureli oturumlarda `requestId > 100000` olursa (dusuk ihtimal ama mumkun) ID'ler carpisabilir.

**Cozum:**
Ayri bir message type namespace kullanmak — `msg.type` alani ile ayristirma zaten yapiliyor, ID offset'ine gerek yok. Veya barcode icin ayri bir counter kullanmak.

---

## 3. Eksik Ozellikler (CLAUDE.md'de Tanimli)

### 3.1 Coklu Secim (Multi-Selection) `[IMPLEMENTE EDILDI]`

**Referans:** CLAUDE.md — "Shift+tiklama ile coklu secim"

**Mevcut Durum:**
`selectedElementId` tek bir `string | null` olarak tanimli. Coklu secim icin hicbir state, UI veya islem mantigi yok.

**Gerekli Degisiklikler:**
1. `stores/editor.ts`'de `selectedElementIds: Set<string>` eklemek
2. `InteractionOverlay.vue`'da Shift+click ile set'e ekleme/cikarma
3. Coklu secimde toplu tasima (absolute elemanlar icin)
4. Coklu secimde toplu ozellik degistirme (ortak alanlar icin)
5. Coklu secimde toplu silme

---

### 3.2 Z-Order Kontrolleri `[IMPLEMENTE EDILDI]`

**Referans:** CLAUDE.md — "One Getir / Arkaya Gonder"

**Mevcut Durum:**
`reorderChild` metodu var ve drag-to-reorder icin kullaniyor. Ancak "One Getir" / "Arkaya Gonder" / "En One Getir" / "En Arkaya Gonder" icin UI bulunmuyor.

**Gerekli Degisiklikler:**
1. `ElementToolbar.vue`'ya z-order butonlari eklemek
2. Store'da `bringForward`, `sendBackward`, `bringToFront`, `sendToBack` action'lari
3. Klavye kisayollari (ornegin Ctrl+] / Ctrl+[)

---

### 3.3 Dinamik Image Binding UI `[IMPLEMENTE EDILDI]`

**Referans:** CLAUDE.md — "image: Statik veya dinamik gorsel, Opsiyonel scalar binding"

**Mevcut Durum:**
`ImageElement` tipinde `binding: Option<ScalarBinding>` tanimli. Backend veri cozumlemesi destekliyor. Ancak `ImageProperties.vue`'da sadece statik dosya yukleme UI'i var — binding secim arayuzu yok.

**Gerekli Degisiklikler:**
1. `ImageProperties.vue`'ya "Statik / Dinamik" toggle eklemek
2. Dinamik modda schema agacindan alan secimi (format: image)
3. `mock-data-generator.ts`'de image binding'leri icin placeholder gorsel uretmek

---

### 3.4 RulerBar (Cetvel) `[IMPLEMENTE EDILDI]`

**Referans:** CLAUDE.md proje yapisi — `components/editor/RulerBar.vue`

**Mevcut Durum:**
Component dosyasi olusturulmamis, hicbir yerde import edilmiyor.

**Gerekli Ozellikler:**
1. Yatay (ust) ve dikey (sol) cetvel
2. mm olcek birimi ile isaretleme
3. Zoom seviyesiyle senkron olcekleme
4. Secili elemanin pozisyonunu cetvel uzerinde isaretleme
5. Sayfa kenarliklari (margin) gostergesi

---

### 3.5 Format Fonksiyonlari (Tablo Sutunlari) `[IMPLEMENTE EDILDI]`

**Referans:** CLAUDE.md roadmap — "Format fonksiyonlari (currency, date)"

**Mevcut Durum:**
`expr_eval.rs`'de `apply_format()` fonksiyonu var ve `currency`, `percentage`, `number` formatlarini destekliyor. Ancak `TableColumn.format` alani pipeline'da hic kullanilmiyor (2.2 ile ayni sorun).

**Gerekli Degisiklikler:**
1. `data_resolve.rs`'de tablo hucre verisi cozumlenirken sutun formatini uygulamak
2. `RepeatingTableProperties.vue`'da sutun bazinda format secimi UI'i
3. Schema'daki `format` alanina gore otomatik format onerisi

---

## 4. Mimari Iyilestirmeler

### 4.1 Worker Message Type Safety `[TAMAMLANDI]`

**Dosya:** `frontend/src/composables/useLayoutEngine.ts` (satir 27)

**Sorun:**
Worker mesajlari `MessageEvent<any>` olarak aliniyor. `msg.type` string kontrolleri ile ayristiriliyor — yeni bir mesaj tipi eklendiyse TypeScript uyarmaz.

**Cozum:**
```typescript
type WorkerMessage =
  | { type: 'compiled'; id: number; result: string; error?: string }
  | { type: 'barcode'; id: number; imageData?: ImageData; error?: string }
  | { type: 'error'; error: string }

worker.onmessage = (e: MessageEvent<WorkerMessage>) => {
    const msg = e.data
    switch (msg.type) {
        case 'compiled': ...
        case 'barcode': ...
        case 'error': ...
    }
}
```

---

### 4.2 Image Re-Encoding Optimizasyonu `[TAMAMLANDI]`

**Dosya:** `layout-engine/src/pdf_render.rs` (satir ~712)

**Sorun:**
`render_image()` tum gorselleri format ne olursa olsun RGBA PNG'ye decode/re-encode ediyor. Neden: "krilla JPEG destegi sinirli" (satir ~666). Ancak PNG input'lari da gereksiz yere decode edilip tekrar encode ediliyor.

**Etki:**
1MB JPEG → ~4MB RGBA decode → PNG re-encode. Bellek ve CPU israfi.

**Cozum:**
- PNG input kontrolu (magic bytes `\x89PNG`): decode etmeden dogrudan embed
- JPEG icin: krilla'nin guncel JPEG destegini kontrol et, mumkunse dogrudan embed
- Fallback: sadece tanilmayan formatlar icin decode/re-encode

---

### 4.3 Tablo Genisletme Cache `[TAMAMLANDI]`

**Dosya:** `layout-engine/src/table_layout.rs`

**Sorun:**
`expand_table()` her layout hesaplamasinda tum tablo satirlarini yeni container agacina klonluyor. 1000 satirlik bir tabloda binlerce `StaticTextElement` ve `ContainerElement` struct'i olusturuluyor.

**Etki:**
Buyuk tablolarda layout hesaplama suresi ve bellek kullanimi artar. Editorde her degisiklikte tum tablo yeniden genisletilir.

**Cozum:**
- Tablo verisinin hash'i uzerinden cache: veri degismemisse onceki genisletilmis agaci tekrar kullan
- Incremental update: sadece degisen satirlari guncelle (daha karmasik)

---

### 4.4 Font Loader Iyilestirmesi (Backend) `[TAMAMLANDI]`

**Dosya:** `backend/src/main.rs` (satirlar 44-53)

**Sorun:**
Font ailesi tespiti dosya adinda `"Mono"` string'i aranarak yapiliyor. `"Mono"` icermeyen tum fontlar `"Noto Sans"` olarak etiketleniyor. Yeni font aileleri eklendikce bu mantik bozulur.

**Cozum:**
TTF/OTF `name` tablosunu okuyarak font ailesini (family name) metadata'dan almak. `cosmic-text`'in `fontdb`'si bunu zaten yapiyor — ayni yaklasiM kullanilabilir.

---

### 4.5 Floating-Point Currency Formatlama Hatasi `[TAMAMLANDI]`

**Dosya:** `layout-engine/src/expr_eval.rs` (satir ~82)

**Sorun:**
```rust
((abs - abs.floor()) * 100.0).round() as i64
```
`1.005` gibi degerler icin floating-point representation kaybi nedeniyle kusurat 0 veya 1 olarak yanlis yuvarlanabilir.

**Cozum:**
`Decimal` arithmetic kullanmak.

---

## 5. Altyapi ve Developer Experience

### 5.1 CI/CD Pipeline `[TAMAMLANDI]`

**Mevcut Durum:**
Hicbir CI/CD konfigurasyonu yok (`.github/`, `.gitea/`, vb.).

**Onerilen Pipeline (Gitea Actions):**
```yaml
# .gitea/workflows/ci.yml
jobs:
  rust:
    steps:
      - cargo fmt --check
      - cargo clippy -- -D warnings
      - cargo test --workspace
  frontend:
    steps:
      - bun install
      - bun run type-check
      - bun run test
  wasm:
    steps:
      - wasm-pack build (verify WASM compile)
```

---

### 5.2 justfile Test/Lint/Fmt Recipe'leri `[TAMAMLANDI]`

**Dosya:** `justfile`

**Mevcut Durum:**
Sadece `front`, `back`, `dev`, `wasm`, `wasm-watch`, `publish-*` recipe'leri var.

**Eklenecek Recipe'ler:**
```just
test:
    cargo test --workspace
    cd frontend && bun run test

lint:
    cargo clippy --workspace -- -D warnings
    cd frontend && bun run lint

fmt:
    cargo fmt --workspace
    cd frontend && bun run format

build:
    cd frontend && bun run build
    cargo build --release -p dreport-backend

check:
    cargo check --workspace
    cd frontend && bun run type-check
```

---

### 5.3 rust-toolchain.toml `[TAMAMLANDI]`

**Sorun:**
Proje Rust edition 2024 kullaniyor (Rust 1.85+) ama toolchain pinlenmemis. Farkli gelistirici ortamlarinda farkli Rust versiyonlari derleme hatalarina yol acabilir.

**Cozum:**
```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

---

### 5.4 WASM Binary Git'te Tracked `[TAMAMLANDI]`

**Dosya:** `frontend/public/wasm/dreport_layout_bg.wasm`

**Sorun:**
`.gitignore`'da `*.wasm` var ama dosya onceden commit edilmis — ignore kurali gecersiz. ~2MB binary her commit'te diff'te gorunuyor.

**Cozum:**
```bash
git rm --cached frontend/public/wasm/dreport_layout_bg.wasm
```
WASM dosyasini build artifact olarak ele almak. CI/CD veya README'de build adimini belgelemek.

---

### 5.5 codemirror-lang-dexpr Dis Bagimlilik `[TAMAMLANDI]`

**Dosya:** `frontend/package.json`

**Sorun:**
```json
"codemirror-lang-dexpr": "file:../../rust-expr/editor"
```
Repo disinda, ust dizinde `rust-expr` projesinin checkout edilmis olmasini gerektiriyor. Bu bagimlilik belgelenmemis — baska bir gelistirici veya CI `bun install` yapinca sessizce kirilir.

**Cozum Secenekleri:**
1. `rust-expr` paketini Gitea registry'ye publish edip npm/bun dependency olarak eklemek
2. Git submodule olarak eklemek
3. En azindan README'de belgelemek ve `bun install` basarisiz oldugunda anlasilir hata mesaji vermek

---

### 5.6 ESLint / oxfmt Kurulumu `[TAMAMLANDI]`

**Mevcut Durum:**
Frontend'de hicbir linter veya formatter konfigurasyonu yok. TypeScript strict mode tip hatalarini yakalasa da, AST-level linting (unused imports, Vue-specific patterns, tutarli stil kurallari) bulunmuyor.

**Master Thougs**
oxfmt kullanalım prettier yerine, eslint kullanmaya devam edebiliriz oxlint vue için yeterince olgun değil.

**Onerilen Yaklasim:**
- `eslint` + `@vue/eslint-config-typescript` + `eslint-plugin-vue`
- `prettier` + `.prettierrc`
- `package.json`'a `lint` ve `format` script'leri

---

### 5.7 Test Helper Duplikasyonu `[TAMAMLANDI]`

**Dosyalar:**
- `layout-engine/tests/layout_integration.rs`
- `layout-engine/tests/pdf_render_test.rs`
- `layout-engine/tests/visual_test.rs`

**Sorun:**
`load_test_fonts()` fonksiyonu uc test dosyasinda birebir ayni sekilde copy-paste edilmis.

**Cozum:**
`layout-engine/tests/common/mod.rs` olusturup ortak test utility'lerini buraya tasimak:
```rust
// tests/common/mod.rs
pub fn load_test_fonts() -> Vec<FontData> { ... }
```

---

### 5.8 Test Artifact Temizligi `[TAMAMLANDI]`

**Dosya:** `layout-engine/tests/pdf_render_test.rs`

**Sorun:**
`test_page_break_produces_multiple_pages` testi workspace root'a `test_page_break.pdf` yaziyor. Her test calistirmada kalir.

**Cozum:**
`tempfile` crate'i ile gecici dosya olusturmak veya `tests/output/` dizinine yazip `.gitignore`'a eklemek.

---

## 6. Test Coverage Bosluklari

### 6.1 page_break.rs Test Eksikligi `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/page_break.rs`

**Durum:** Projenin en karmasik mantik parcalarindan biri — sifir dedicated test. Entegrasyon testlerinde dolayli olarak test ediliyor ama edge case'ler (break_inside: avoid, tablo header tekrari, sayfa tasmasi sinirlari) test edilmemis.

**Gerekli Testler:**
1. Basit sayfa tasmasi — icerik tek sayfaya sigmadigi durum
2. `break_inside: avoid` ile grup tasma ve yeni sayfaya gecis
3. Tablo header tekrari — cok sayfali tablo
4. `page_break` elemani ile zorunlu sayfa gecisi
5. Edge: tam sayfa sinirina denk gelen eleman
6. Edge: sayfaya sigmayan tek eleman (sayfadan buyuk)

---

### 6.2 chart_render.rs Test Eksikligi `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/chart_render.rs`

**Durum:** Sadece visual snapshot testi var. SVG ciktisi icin unit test yok.

**Gerekli Testler:**
1. Bar chart SVG structure (dogru sayida rect, label)
2. Line chart data point koordinatlari
3. Pie chart dilim acilari (360 derece toplami)
4. Legend render kosullari (tek seri vs coklu seri)
5. Bos veri seti edge case

---

### 6.3 pdf_render.rs Unit Test Eksikligi `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/pdf_render.rs`

**Durum:** Sadece entegrasyon testleri var (PDF magic bytes kontrolu). `render_text`, `render_image`, `render_barcode`, `render_chart` gibi fonksiyonlar icin unit test yok.

---

### 6.4 Frontend Component Testleri `[IMPLEMENTE EDILMEDI]`

**Durum:** `vitest` ve `playwright` devDependency olarak yuklu ama test dosyasi yok (schema-parser testi haric).

**Oncelikli Test Hedefleri:**
1. `useUndoRedo` — snapshot push, undo, redo, stack limitleri
2. `useSnapGuides` — snap hesaplama, threshold davranisi
3. Template store — CRUD islemleri, tree traversal
4. `InteractionOverlay` — drag/resize event handling (component test)

---

## 7. Yeni Ozellik Onerileri

### 7.1 Conditional Rendering `[IMPLEMENTE EDILMEDI]`

**Aciklama:**
Template'te `v-if` benzeri kosullu gosterim. Data'daki bir alana gore eleman goster/gizle.

**Ornek:**
```json
{
  "id": "el_iskonto",
  "type": "text",
  "condition": {
    "path": "toplamlar.iskonto",
    "operator": "gt",
    "value": 0
  },
  "binding": { "type": "scalar", "path": "toplamlar.iskonto" }
}
```

**Etki:** Kullanici tek bir template ile farkli veri durumlarini karsilayabilir (ornegin iskonto varsa goster, yoksa gizle).

---

### 7.2 Template Versiyonlama `[IMPLEMENTE EDILMEDI]`

**Aciklama:**
Template JSON uzerinde degisiklik gecmisi. Her kayit/export'ta versiyon numarasi arttirilir, onceki versiyonlara donulebilir.

**Yaklasim:**
- Template JSON'a `version: number` ve `history: ChangeEntry[]` alani
- JSON diff-bazli degisiklik kaydi (tam snapshot degil, sadece delta)
- UI'da versiyon gecmisi paneli

---

### 7.3 Tekrarlayan Bolge (Repeating Region) `[IMPLEMENTE EDILMEDI]`

**Referans:** CLAUDE.md kisitlamalar — "Serbest form repeating region ilerideki fazlarda degerlendirilir"

**Aciklama:**
Tablo disinda array verisiyle tekrarlayan serbest-form container. Ornegin bir kart tasarimi array'deki her kayit icin tekrarlanir.

**Karmasiklik:** Yuksek — layout engine'de container agacinin dinamik genisletilmesi, sayfa tasma mantigi, editor'da tekrar onizlemesi.

---

### 7.4 PNG/SVG Export `[IMPLEMENTE EDILMEDI]`

**Referans:** CLAUDE.md — "Sadece PDF cikti. Ileride PNG/SVG eklenebilir."

**Yaklasim:**
- SVG: LayoutResult → SVG element'leri (chart_render.rs'deki pattern'e benzer)
- PNG: SVG render + rasterize (resvg crate) veya dogrudan image crate ile pixel render
- Backend'e `/api/render?format=png|svg|pdf` parametresi

---

### 7.5 Coklu Dil / Lokalizasyon Destegi `[IMPLEMENTE EDILMEDI]`

**Aciklama:**
Currency, date ve sayi formatlama icin lokalizasyon. Su an Turk lokali hardcoded.

**Yaklasim:**
- Template JSON'a `locale: "tr-TR"` alani
- `expr_eval.rs`'de locale-aware formatlama
- UI etiketleri icin i18n framework (vue-i18n)

---

### 7.6 Sayfa Basligi/Altligi Kosullari `[IMPLEMENTE EDILMEDI]`

**Aciklama:**
Farkli sayfalar icin farkli header/footer:
- Ilk sayfa farkli (ornegin firma logosu sadece ilk sayfada)
- Son sayfa farkli (ornegin toplam ve imza sadece son sayfada)
- Cift/tek sayfa farkli (kitap/katalog baski icin)

**Yaklasim:**
Template'te header/footer tanimi icin `condition` alani:
```json
"header": {
  "condition": "first_page",
  "children": [...]
}
```

---

### 7.7 QR Code Eleman Tipi `[IMPLEMENTE EDILMEDI]`

**Mevcut Durum:**
`rxing` crate'i barcode uretimi icin zaten kullaniliyor ve QR Code destegi var. Ancak UI tarafinda ayri bir QR Code eleman tipi tanimlanmamis.

**Gerekli Degisiklikler:**
1. `core/models.rs`'e `QrCodeElement` tipi
2. Barcode element'ten farkli olarak kare aspect ratio zorunlulugu
3. Editor'da QR Code onizlemesi
4. Properties panelinde QR icerik ve boyut ayarlari

---

### 7.8 Template Marketplace / Galeri `[IMPLEMENTE EDILMEDI]`

**Aciklama:**
Hazir sablon galerisi — kullanici sifirdan tasarlamak yerine bir sablon secip uzerine duzenleyebilir.

**Yaklasim:**
- `shared/templates/` dizininde kategorize edilmis JSON sablonlar
- UI'da "Sablonlardan Baslat" modali
- Kategoriler: Fatura, Irsaliye, Rapor, Sertifika, Makbuz
- Her sablon icin thumbnail onizleme

---

## 8. Kucuk Ama Degerli Iyilestirmeler

### 8.1 Chart Legend Tek Seri Durumu `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/chart_render.rs`

**Sorun:** Legend yalnizca `series.len() > 1` oldugunda render ediliyor. Tek serili bar chart'ta `legend: { show: true }` sessizce yok sayiliyor.

---

### 8.2 Pie Chart Label Kontrolu `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/chart_render.rs` (satirlar 521-551)

**Sorun:** Pie chart'ta kategori isimleri ve leader line'lar her zaman render ediliyor. `labels.show` flag'i sadece dilim icindeki yuzde etiketini kontrol ediyor.

---

### 8.3 Data Path'te Nokta Kisitlamasi `[IMPLEMENTE EDILMEDI]`

**Dosya:** `layout-engine/src/data_resolve.rs` (satir ~117)

**Sorun:** `resolve_path()` `.` karakteri ile split yapiyor. Alan isimleri nokta iceriyorsa (`firma.adres.il` vs `firma."adres.il"`) dogru cozumlenmiyor. Bu kisitlama belgelenmemis.

**Cozum:** Bracket notation destegi (`firma["adres.il"]`) veya en azindan dokumantasyon.

---

### 8.4 DreportEditor Prop-Store Sync Fragility `[IMPLEMENTE EDILMEDI]`

**Dosya:** `frontend/src/lib/DreportEditor.vue`

**Sorun:** `let syncing = false` boolean'i ile prop↔store dongusu engelleniyor. `nextTick` arasinda gelen store mutation'lari (klavye kisayolu vb.) sessizce yutulabilir.

**Cozum:** `syncing` flag'ini reactive yapmak ve watcher'da condition check yerine `watchEffect` kullanmak, veya store event bazli uni-directional data flow'a gecmek.

---

### 8.5 CORS Konfigurasyonu `[IMPLEMENTE EDILMEDI]`

**Dosya:** `backend/src/main.rs`

**Sorun:** `CorsLayer` tamamen acik (`allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)`). Yerel gelistirme icin sorun degil ama production icin kisitlanmali.

**Cozum:** Environment variable ile origin kisitlamasi: `CORS_ORIGIN=http://localhost:5173` (dev), `CORS_ORIGIN=https://app.dreport.com` (prod).

---

### 8.6 Request Size Limit `[IMPLEMENTE EDILMEDI]`

**Dosya:** `backend/src/main.rs`

**Sorun:** HTTP body boyut limiti yok. Buyuk JSON payload'lari tamamen belleqe alinir.

**Cozum:** Axum'un `DefaultBodyLimit` middleware'i ile makul bir limit (ornegin 10MB) koymak.
