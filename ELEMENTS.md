# Eleman Tipleri — dreport

Bu belge, dreport toolbar'inda bulunan ve planlanmis tum eleman tiplerini aciklar.

---

## Mevcut Elemanlar

### `container` — Duzen Kutusu

CSS Flexbox mantiginda calisan layout container'i. Cocuk elemanlari `direction` (row/column) dogrultusunda dizer. Ic ice gecebilir. Tum diger elemanlar bir container icinde yer alir.

- **Binding:** Yok
- **Ozellikler:** `direction`, `gap`, `padding`, `align`, `justify`, `style`

---

### `static_text` — Sabit Metin

Veri baglantisi olmayan, kullanicinin dogrudan yazdigi metin. Fatura basliklari, etiketler, aciklama satirlari icin kullanilir.

- **Binding:** Yok
- **Ozellikler:** `content`, `style` (fontSize, fontWeight, color, align)

---

### `text` — Dinamik Metin

JSON schema'dan veri ceken metin elemani. Kullanici schema agacindan bir alani surukleyip bu elemana baglar.

- **Binding:** Scalar (`"binding": { "type": "scalar", "path": "firma.unvan" }`)
- **Ozellikler:** `binding`, `style`, `format` (currency, date, percentage)

---

### `repeating_table` — Tekrarlayan Tablo

Array verisinden tekrarlayan satirlar ureten tablo bileseni. Fatura kalemleri, stok listeleri gibi tekrarlayan veri icin kullanilir.

- **Binding:** Array (`"dataSource": "kalemler"`)
- **Ozellikler:** `columns` (alan, genislik, hizalama), `headerStyle`, `rowStyle`, `zebraStyle`

---

### `line` — Cizgi

Yatay veya dikey ayirici cizgi. Bolum ayirma, dekoratif amaclarla kullanilir.

- **Binding:** Yok
- **Ozellikler:** `style` (strokeColor, strokeWidth)

---

### `image` — Gorsel

Statik (base64/URL) veya dinamik (schema'dan) gorsel. Logo, imza, urun gorseli gibi kullanim alanlari.

- **Binding:** Opsiyonel scalar (dinamik gorsel icin)
- **Ozellikler:** `src` (statik), `binding`, `style` (objectFit)

---

### `page_number` — Sayfa Numarasi

Cok sayfali belgelerde otomatik sayfa numarasi. Format sablonu destekler (or: "Sayfa {current} / {total}").

- **Binding:** Otomatik
- **Ozellikler:** `format`, `style`

---

### `barcode` — Barkod / QR Kod

1D ve 2D barkod ureteci. e-Fatura, e-Arsiv, urun etiketleri icin kullanilir.

- **Binding:** Scalar (barkod verisi icin)
- **Desteklenen formatlar:** QR, EAN-13, EAN-8, CODE128, CODE39
- **Ozellikler:** `barcodeType`, `binding`, `style`

---

## Planlanmis Elemanlar

### `rich_text` — Zengin Metin [Henuz implemente edilmedi]

Tek bir metin blogu icinde karisik formatlama destekleyen eleman. Kalin, italik, farkli font boyutu, renk gibi stilleri ayni paragraf icinde kullanmayi saglar.

- **Kullanim alanlari:** Fatura aciklama alanlari, sozlesme maddeleri, rapor notlari, uzun formlu metin icerikleri
- **Binding:** Opsiyonel scalar (dinamik icerik icin)
- **Yaklasim:** Inline span'lar ile zengin metin. cosmic-text attributed text destekledigi icin layout engine tarafinda uyumlu.

```jsonc
{
  "type": "rich_text",
  "content": [
    { "text": "Odeme vadesi: ", "style": {} },
    { "text": "30 gun", "style": { "fontWeight": "bold", "color": "#e00" } }
  ]
}
```

**Referans:** Telerik (HtmlTextBox), DevExpress (Rich Text), Stimulsoft, FastReport, CraftMyPDF — hepsinde mevcut. Belge tasarim araclarinda standart bir beklenti.

---

### `shape` — Sekil (Dikdortgen / Elips) [Henuz implemente edilmedi]

Cocuk eleman barindirmayan sade gorsel element. Vurgu kutulari, dekoratif cerceveler, arka plan alanlari icin kullanilir. Container'dan farki: layout'a katilmaz, sadece gorsel amaclidir.

- **Kullanim alanlari:** Toplam kutusunun arka plani, raporlarda highlight alanlari, dekoratif cerceveler
- **Binding:** Yok
- **Sekil tipleri:** `rectangle`, `ellipse`, `rounded_rectangle`

```jsonc
{
  "type": "shape",
  "shapeType": "rectangle",
  "style": {
    "backgroundColor": "#f0f0f0",
    "borderColor": "#333",
    "borderWidth": 0.5,
    "borderRadius": 2
  }
}
```

**Referans:** JasperReports, Telerik, DevExpress, Stimulsoft, FastReport, CraftMyPDF — neredeyse tum araclarda var.

---

### `checkbox` — Onay Kutusu [Henuz implemente edilmedi]

Boolean deger gosteren isaret kutusu. Isaretsiz kare veya isaretli (checkmark) kare olarak render edilir. Veri baglantisi ile dinamik calisan veya statik olarak kullanilabilen basit bir element.

- **Kullanim alanlari:** Irsaliyelerde "teslim edildi / edilmedi", faturalarda odeme durumu, raporlarda checklist, form benzeri belgeler
- **Binding:** Scalar (boolean alan)

```jsonc
{
  "type": "checkbox",
  "binding": { "type": "scalar", "path": "fatura.odpiendi" },
  "style": { "size": 4, "checkColor": "#000", "borderColor": "#333" }
}
```

**Referans:** DevExpress, Telerik, Stimulsoft, FastReport, CraftMyPDF.

---

### `calculated_text` — Hesaplanmis Alan [Henuz implemente edilmedi]

Basit ifadeler (expression) ile hesaplanmis deger gosteren metin elemani. Aritmetik islemler, string birlestirme ve kosullu metin destekler.

- **Kullanim alanlari:** Ara toplam hesaplari (`araToplam * 0.20`), string birlestirme (`"Fatura No: " + fatura.no`), kosullu metin, rapor ozetleri
- **Binding:** Expression-based (birden fazla alana referans verebilir)
- **Format:** currency, date, percentage, number destegi

```jsonc
{
  "type": "calculated_text",
  "expression": "toplamlar.araToplam * 0.20",
  "format": "currency",
  "style": { "fontSize": 10 }
}
```

**Referans:** Crystal Reports (Formula Field), JasperReports (Variable), Stimulsoft (Expression).

---

### `current_date` — Tarih / Zaman [Henuz implemente edilmedi]

Belgenin basilma/render anindaki tarihi otomatik gosteren element. `page_number` gibi otomatik deger uretir, veri baglantisi gerektirmez.

- **Kullanim alanlari:** Fatura basim tarihi, rapor olusturma zamani, belge altbilgisi
- **Binding:** Otomatik
- **Format:** Konfigurasyon ile (or: `DD.MM.YYYY`, `DD MMMM YYYY`, `DD.MM.YYYY HH:mm`)

```jsonc
{
  "type": "current_date",
  "format": "DD.MM.YYYY",
  "style": { "fontSize": 8, "color": "#666" }
}
```

**Referans:** Crystal Reports (Print Date), JasperReports (Current Date), BIRT (AutoText).

---

### `page_break` — Sayfa Sonu [Henuz implemente edilmedi]

Kullanicinin belirli bir noktada yeni sayfaya gecmesini saglayan kontrol elemani. Otomatik sayfa sonu (page_break.rs) zaten mevcut, bu element manuel kontrol saglar.

- **Kullanim alanlari:** Rapor ozet sayfasi + detay sayfasi ayrimi, faturada ek bilgi sayfasi, belirli bolumlerin ayri sayfada baslamasi
- **Binding:** Yok
- **Gorsel:** Editorde kesikli cizgi olarak gosterilir, PDF'te sayfa gecisi uretir.

```jsonc
{
  "type": "page_break"
}
```

**Referans:** DevExpress (Page Break kontrol), Stimulsoft.

---

### `chart` — Grafik [Henuz implemente edilmedi]

Veri gorselIestirme icin basit grafik elemani. Rapor ciktilari icin degerli, fatura/irsaliye icin genellikle gereksiz.

- **Kullanim alanlari:** Satis raporlari, performans ozetleri, karsilastirmali veriler
- **Binding:** Array veya multiple scalar
- **Grafik tipleri:** `bar`, `pie`, `line` (baslangic seti)
- **Yaklasim:** Backend'de SVG olarak render edilip PDF'e image olarak gomulur.

```jsonc
{
  "type": "chart",
  "chartType": "bar",
  "dataSource": "aylik_satislar",
  "labelField": "ay",
  "valueField": "tutar",
  "style": { "width": 120, "height": 80 }
}
```

**Referans:** JasperReports, Crystal Reports, Telerik, DevExpress, Stimulsoft, CraftMyPDF — enterprise araclarin tamami destekler.

---

## Toolbar Organizasyonu

```
Toolbar
├── Duzen
│   ├── Container             (mevcut)
│   └── Page Break            (planlanmis)
├── Metin
│   ├── Statik Metin          (mevcut)
│   ├── Rich Text             (planlanmis)
│   └── Hesaplanmis Alan      (planlanmis)
├── Veri
│   ├── Tekrarlayan Tablo     (mevcut)
│   └── Checkbox              (planlanmis)
├── Gorsel
│   ├── Gorsel                (mevcut)
│   ├── Cizgi                 (mevcut)
│   ├── Sekil                 (planlanmis)
│   └── Barkod / QR           (mevcut)
├── Otomatik
│   ├── Sayfa No              (mevcut)
│   └── Tarih                 (planlanmis)
└── Rapor
    └── Grafik                (planlanmis)
```

---

## Oncelik Sirasi

| Oncelik | Element | Gerekce |
|---------|---------|---------|
| 1 | `rich_text` | Karisik formatlama en cok talep edilen ozellik, cosmic-text uyumlu |
| 2 | `shape` | Basit implementasyon, gorsel zenginlik katiyor |
| 3 | `checkbox` | Boolean gosterim, form/irsaliye icin onemli |
| 4 | `calculated_text` | Hesaplama ihtiyaci fatura/rapor icin kritik |
| 5 | `current_date` | Kucuk ama kullanisli, hizli implemente edilir |
| 6 | `page_break` | Manuel sayfa kontrolu, rapor senaryolari icin |
| 7 | `chart` | En karmasik, rapor fazinda ele alinabilir |
