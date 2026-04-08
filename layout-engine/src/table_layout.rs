use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use dreport_core::models::*;

use crate::data_resolve::ResolvedData;
use crate::text_measure::TextMeasurer;

/// Cache for expanded table containers.
/// Key: hash of (table JSON + resolved rows + available width).
pub type TableExpandCache = HashMap<u64, ContainerElement>;

fn table_cache_key(
    table: &RepeatingTableElement,
    rows: &[Vec<String>],
    available_width_mm: f64,
) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    // Serialize table definition (id, columns, style, etc.)
    if let Ok(json) = serde_json::to_string(table) {
        json.hash(&mut hasher);
    }
    // Hash resolved row data
    for row in rows {
        for cell in row {
            cell.hash(&mut hasher);
        }
        row.len().hash(&mut hasher);
    }
    rows.len().hash(&mut hasher);
    // Hash available width (as bits to avoid float hashing issues)
    available_width_mm.to_bits().hash(&mut hasher);
    hasher.finish()
}

/// Her auto sütun için header + tüm data satırlarındaki en geniş text'i ölç,
/// doğal genişliklerini Fixed olarak ata.
/// Fr sütunları olduğu gibi bırak (kalan alanı taffy dağıtır).
/// Sadece auto sütunlar varsa (fr/fixed yoksa) kalan alanı oransal dağıt.
fn compute_auto_column_widths(
    table: &RepeatingTableElement,
    rows: &[Vec<String>],
    measurer: &mut TextMeasurer,
    available_width_mm: f64,
) -> Vec<SizeValue> {
    let num_cols = table.columns.len();
    let font_size = table.style.font_size.unwrap_or(10.0);
    let header_font_size = table.style.header_font_size.unwrap_or(font_size);
    let cell_pad_h = table.style.cell_padding_h.unwrap_or(2.0);
    let header_pad_h = table.style.header_padding_h.unwrap_or(cell_pad_h);
    // Ölçüme dahil edilecek max yatay padding (header ve cell'den büyük olanı)
    let max_pad_h = cell_pad_h.max(header_pad_h);

    // Hangi sütunlar auto?
    let is_auto: Vec<bool> = table
        .columns
        .iter()
        .map(|c| matches!(c.width, SizeValue::Auto))
        .collect();

    // Hiç auto yoksa olduğu gibi dön
    if !is_auto.iter().any(|&a| a) {
        return table.columns.iter().map(|c| c.width.clone()).collect();
    }

    // Fr sütun var mı?
    let has_fr = table
        .columns
        .iter()
        .any(|c| matches!(c.width, SizeValue::Fr { .. }));

    // Her auto sütun için max içerik genişliğini ölç (mm cinsinden)
    let mut max_widths_mm = vec![0.0_f64; num_cols];

    for (col_idx, col) in table.columns.iter().enumerate() {
        if !is_auto[col_idx] {
            continue;
        }

        // Header text ölçümü (font_size zaten pt cinsinden)
        let (header_w_pt, _) = measurer.measure(
            &col.title,
            None,
            header_font_size as f32,
            Some("bold"),
            None,
        );
        let header_w_mm = header_w_pt as f64 / (72.0 / 25.4);
        max_widths_mm[col_idx] = header_w_mm;

        // Data row text ölçümü
        for row in rows {
            let text = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            if text.is_empty() {
                continue;
            }
            let (w_pt, _) = measurer.measure(text, None, font_size as f32, None, None);
            let w_mm = w_pt as f64 / (72.0 / 25.4);
            if w_mm > max_widths_mm[col_idx] {
                max_widths_mm[col_idx] = w_mm;
            }
        }

        // Yatay padding ekle (sol + sağ)
        max_widths_mm[col_idx] += max_pad_h * 2.0;
    }

    // Fixed sütunların kapladığı alanı hesapla
    let mut fixed_total_mm = 0.0_f64;
    for (col_idx, col) in table.columns.iter().enumerate() {
        if !is_auto[col_idx]
            && let SizeValue::Fixed { value } = &col.width
        {
            fixed_total_mm += value;
        }
    }

    // Auto sütunların toplam doğal genişliği
    let auto_natural_total: f64 = max_widths_mm.iter().sum();
    let remaining_mm = available_width_mm - fixed_total_mm;

    // Sonuç genişlikleri
    let mut result: Vec<SizeValue> = Vec::with_capacity(num_cols);

    if has_fr {
        // Fr sütunlar var — auto sütunlara doğal genişliklerini ver,
        // kalan alanı Fr sütunlarına bırak (taffy flex ile dağıtır).

        // Fr sütunları için minimum alan ayır (en az padding kadar)
        let fr_count = table
            .columns
            .iter()
            .filter(|c| matches!(c.width, SizeValue::Fr { .. }))
            .count();
        let fr_min_space = fr_count as f64 * max_pad_h * 2.0;
        let auto_budget = (remaining_mm - fr_min_space).max(0.0);

        for (col_idx, col) in table.columns.iter().enumerate() {
            if !is_auto[col_idx] {
                result.push(col.width.clone());
            } else if auto_natural_total <= auto_budget {
                // Sığıyor — doğal genişliği kullan
                result.push(SizeValue::Fixed {
                    value: max_widths_mm[col_idx],
                });
            } else if auto_budget > 0.0 && auto_natural_total > 0.0 {
                // Sığmıyor — budget'a oransal küçült
                let ratio = max_widths_mm[col_idx] / auto_natural_total;
                let width_mm = auto_budget * ratio;
                result.push(SizeValue::Fixed { value: width_mm });
            } else {
                result.push(SizeValue::Fixed {
                    value: max_widths_mm[col_idx],
                });
            }
        }
    } else {
        // Fr sütun yok — kalan alanı auto sütunlar arasında oransal dağıt
        for (col_idx, col) in table.columns.iter().enumerate() {
            if !is_auto[col_idx] {
                result.push(col.width.clone());
            } else if auto_natural_total > 0.0 {
                let ratio = max_widths_mm[col_idx] / auto_natural_total;
                let width_mm = remaining_mm * ratio;
                result.push(SizeValue::Fixed { value: width_mm });
            } else {
                // Tüm auto sütunlar boş — eşit dağıt
                let auto_count = is_auto.iter().filter(|&&a| a).count();
                let width_mm = remaining_mm / auto_count as f64;
                result.push(SizeValue::Fixed { value: width_mm });
            }
        }
    }

    result
}

/// Cache-aware table expansion.
/// Verilen cache'e bakar, hit varsa clone döner. Miss'te expand edip cache'e yazar.
pub fn expand_table_cached(
    table: &RepeatingTableElement,
    resolved: &ResolvedData,
    measurer: &mut TextMeasurer,
    available_width_mm: f64,
    cache: &mut TableExpandCache,
) -> ContainerElement {
    let rows = resolved
        .tables
        .get(&table.base.id)
        .map(|t| t.rows.as_slice())
        .unwrap_or(&[]);
    let key = table_cache_key(table, rows, available_width_mm);

    if let Some(cached) = cache.get(&key) {
        return cached.clone();
    }

    let result = expand_table(table, resolved, measurer, available_width_mm);
    cache.insert(key, result.clone());
    result
}

/// RepeatingTable element'ini bir container ağacına expand eder.
/// Tablo → column container (header row + data rows)
/// Her row → row container (cell'ler → static_text)
///
/// Bu sayede tablo, normal container layout'u ile hesaplanır.
pub fn expand_table(
    table: &RepeatingTableElement,
    resolved: &ResolvedData,
    measurer: &mut TextMeasurer,
    available_width_mm: f64,
) -> ContainerElement {
    let resolved_table = resolved.tables.get(&table.base.id);
    let rows = resolved_table.map(|t| t.rows.as_slice()).unwrap_or(&[]);

    // Auto sütunlar için içerik bazlı genişlik hesapla
    let effective_widths = compute_auto_column_widths(table, rows, measurer, available_width_mm);

    // Padding değerleri (mm)
    let cell_pad_h = table.style.cell_padding_h.unwrap_or(2.0);
    let cell_pad_v = table.style.cell_padding_v.unwrap_or(1.0);
    let header_pad_h = table.style.header_padding_h.unwrap_or(cell_pad_h);
    let header_pad_v = table.style.header_padding_v.unwrap_or(cell_pad_v);

    let mut children: Vec<TemplateElement> = Vec::new();

    // Header row — her hücre padding container'ı içinde
    let header_cells: Vec<TemplateElement> = table
        .columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let text = TemplateElement::StaticText(StaticTextElement {
                base: ElementBase::flow(
                    format!("{}_hdr_{}", table.base.id, i),
                    SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        ..Default::default()
                    },
                ),
                style: TextStyle {
                    font_size: table.style.header_font_size.or(table.style.font_size),
                    font_weight: Some("bold".to_string()),
                    font_style: None,
                    font_family: None,
                    color: table.style.header_color.clone(),
                    align: Some(col.align.clone()),
                },
                content: col.title.clone(),
            });
            TemplateElement::Container(ContainerElement {
                base: ElementBase::flow(
                    format!("{}_hdr_{}_wrap", table.base.id, i),
                    SizeConstraint { width: effective_widths[i].clone(), ..Default::default() },
                ),
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding { top: header_pad_v, right: header_pad_h, bottom: header_pad_v, left: header_pad_h },
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                children: vec![text],
                break_inside: "auto".to_string(),
            })
        })
        .collect();

    children.push(TemplateElement::Container(ContainerElement {
        base: ElementBase::flow(
            format!("{}_header", table.base.id),
            SizeConstraint { width: SizeValue::Fr { value: 1.0 }, ..Default::default() },
        ),
        direction: "row".to_string(),
        gap: 0.0,
        padding: Padding::default(),
        align: "stretch".to_string(),
        justify: "start".to_string(),
        style: ContainerStyle { background_color: table.style.header_bg.clone(), ..Default::default() },
        children: header_cells,
        break_inside: "auto".to_string(),
    }));

    // Header altına ayırıcı çizgi
    if table.style.border_color.is_some() {
        children.push(TemplateElement::Line(LineElement {
            base: ElementBase::flow(
                format!("{}_header_line", table.base.id),
                SizeConstraint { width: SizeValue::Fr { value: 1.0 }, ..Default::default() },
            ),
            style: LineStyle {
                stroke_color: table.style.border_color.clone(),
                stroke_width: table.style.border_width,
            },
        }));
    }

    // Data rows — her hücre padding container'ı içinde
    for (row_idx, row_data) in rows.iter().enumerate() {
        let cells: Vec<TemplateElement> = table
            .columns
            .iter()
            .enumerate()
            .map(|(col_idx, col)| {
                let text_content = row_data.get(col_idx).cloned().unwrap_or_default();

                let text = TemplateElement::StaticText(StaticTextElement {
                    base: ElementBase::flow(
                        format!("{}_r{}c{}", table.base.id, row_idx, col_idx),
                        SizeConstraint { width: SizeValue::Fr { value: 1.0 }, ..Default::default() },
                    ),
                    style: TextStyle {
                        font_size: table.style.font_size,
                        font_weight: None,
                        font_style: None,
                        font_family: None,
                        color: None,
                        align: Some(col.align.clone()),
                    },
                    content: text_content,
                });
                TemplateElement::Container(ContainerElement {
                    base: ElementBase::flow(
                        format!("{}_r{}c{}_wrap", table.base.id, row_idx, col_idx),
                        SizeConstraint { width: effective_widths[col_idx].clone(), ..Default::default() },
                    ),
                    direction: "column".to_string(),
                    gap: 0.0,
                    padding: Padding { top: cell_pad_v, right: cell_pad_h, bottom: cell_pad_v, left: cell_pad_h },
                    align: "stretch".to_string(),
                    justify: "start".to_string(),
                    style: ContainerStyle::default(),
                    children: vec![text],
                    break_inside: "auto".to_string(),
                })
            })
            .collect();

        // row_idx 0-based: çift index (0,2,4) renksiz, tek index (1,3,5) zebra rengi
        let bg = if row_idx % 2 == 1 {
            table.style.zebra_odd.clone()
        } else {
            table.style.zebra_even.clone()
        };

        children.push(TemplateElement::Container(ContainerElement {
            base: ElementBase::flow(
                format!("{}_row_{}", table.base.id, row_idx),
                SizeConstraint { width: SizeValue::Fr { value: 1.0 }, ..Default::default() },
            ),
            direction: "row".to_string(),
            gap: 0.0,
            padding: Padding::default(),
            align: "stretch".to_string(),
            justify: "start".to_string(),
            style: ContainerStyle { background_color: bg, ..Default::default() },
            children: cells,
            break_inside: "auto".to_string(),
        }));
    }

    // Wrapper container (column direction, tüm tablo)
    ContainerElement {
        base: ElementBase {
            id: table.base.id.clone(),
            condition: None,
            position: table.base.position.clone(),
            size: table.base.size.clone(),
        },
        direction: "column".to_string(),
        gap: 0.0,
        padding: Padding::default(),
        align: "stretch".to_string(),
        justify: "start".to_string(),
        style: ContainerStyle {
            border_color: table.style.border_color.clone(),
            border_width: table.style.border_width,
            ..Default::default()
        },
        children,
        break_inside: "auto".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FontData;
    use crate::data_resolve::{ResolvedData, ResolvedTable};
    use crate::text_measure::TextMeasurer;
    use std::collections::HashMap;

    fn make_table(num_columns: usize) -> RepeatingTableElement {
        let columns: Vec<TableColumn> = (0..num_columns)
            .map(|i| TableColumn {
                id: format!("col_{}", i),
                field: format!("field_{}", i),
                title: format!("Column {}", i),
                width: SizeValue::Fr { value: 1.0 },
                align: "left".to_string(),
                format: None,
            })
            .collect();

        RepeatingTableElement {
            base: ElementBase::flow(
                "tbl".to_string(),
                SizeConstraint { width: SizeValue::Fr { value: 1.0 }, ..Default::default() },
            ),
            data_source: ArrayBinding {
                path: "items".to_string(),
            },
            columns,
            style: TableStyle::default(),
            repeat_header: Some(true),
        }
    }

    fn make_resolved(table_id: &str, rows: Vec<Vec<String>>) -> ResolvedData {
        let mut tables = HashMap::new();
        tables.insert(table_id.to_string(), ResolvedTable { rows });
        ResolvedData {
            texts: HashMap::new(),
            tables,
            barcodes: HashMap::new(),
            images: HashMap::new(),
            page_number_formats: HashMap::new(),
            rich_texts: HashMap::new(),
            charts: HashMap::new(),
            hidden_elements: std::collections::HashSet::new(),
        }
    }

    fn make_measurer() -> TextMeasurer {
        // Font dosyasını yükle
        let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("backend/fonts/NotoSans-Regular.ttf");
        let font_bytes = std::fs::read(&font_path).expect("Font file not found");
        let font_data = vec![FontData::from_bytes(font_bytes).expect("Font parse failed")];
        TextMeasurer::new(&font_data)
    }

    /// Hücre wrapper container'ından içindeki StaticText'i çıkar
    fn unwrap_cell_text(cell: &TemplateElement) -> &StaticTextElement {
        match cell {
            TemplateElement::Container(c) => {
                assert_eq!(
                    c.children.len(),
                    1,
                    "Cell wrapper should have exactly 1 child"
                );
                match &c.children[0] {
                    TemplateElement::StaticText(t) => t,
                    _ => panic!("Expected StaticText inside cell wrapper"),
                }
            }
            _ => panic!("Expected Container wrapper for cell"),
        }
    }

    #[test]
    fn test_expand_table_structure() {
        let table = make_table(2);
        let resolved = make_resolved(
            "tbl",
            vec![
                vec!["A".to_string(), "1".to_string()],
                vec!["B".to_string(), "2".to_string()],
            ],
        );
        let mut measurer = make_measurer();

        let container = expand_table(&table, &resolved, &mut measurer, 180.0);

        // Wrapper container properties
        assert_eq!(container.base.id, "tbl");
        assert_eq!(container.direction, "column");

        // Children: header row + 2 data rows (no border_color so no separator line)
        assert_eq!(container.children.len(), 3);

        // First child is header row container
        match &container.children[0] {
            TemplateElement::Container(c) => {
                assert_eq!(c.base.id, "tbl_header");
                assert_eq!(c.direction, "row");
                assert_eq!(c.children.len(), 2); // 2 columns
                // Check header cell text (inside wrapper container)
                let text = unwrap_cell_text(&c.children[0]);
                assert_eq!(text.content, "Column 0");
            }
            _ => panic!("Expected Container for header row"),
        }

        // Data rows
        for (row_idx, child) in container.children[1..].iter().enumerate() {
            match child {
                TemplateElement::Container(c) => {
                    assert_eq!(c.base.id, format!("tbl_row_{}", row_idx));
                    assert_eq!(c.direction, "row");
                    assert_eq!(c.children.len(), 2);
                }
                _ => panic!("Expected Container for data row"),
            }
        }
    }

    #[test]
    fn test_expand_table_empty_data() {
        let table = make_table(3);
        let resolved = make_resolved("tbl", vec![]);
        let mut measurer = make_measurer();

        let container = expand_table(&table, &resolved, &mut measurer, 180.0);

        // Only header row, no data rows
        assert_eq!(container.children.len(), 1);

        // Header should still have all 3 columns
        match &container.children[0] {
            TemplateElement::Container(c) => {
                assert_eq!(c.children.len(), 3);
            }
            _ => panic!("Expected Container for header row"),
        }
    }

    #[test]
    fn test_expand_table_column_count() {
        let table = make_table(4);
        let resolved = make_resolved(
            "tbl",
            vec![vec!["a".into(), "b".into(), "c".into(), "d".into()]],
        );
        let mut measurer = make_measurer();

        let container = expand_table(&table, &resolved, &mut measurer, 180.0);

        // header + 1 data row
        assert_eq!(container.children.len(), 2);

        // Both header and data row should have 4 cells
        match &container.children[0] {
            TemplateElement::Container(c) => assert_eq!(c.children.len(), 4),
            _ => panic!("Expected Container"),
        }
        match &container.children[1] {
            TemplateElement::Container(c) => assert_eq!(c.children.len(), 4),
            _ => panic!("Expected Container"),
        }
    }

    #[test]
    fn test_expand_table_data_cell_content() {
        let table = make_table(2);
        let resolved = make_resolved("tbl", vec![vec!["Hello".to_string(), "42".to_string()]]);
        let mut measurer = make_measurer();

        let container = expand_table(&table, &resolved, &mut measurer, 180.0);

        // Data row cells should contain the resolved text (inside wrapper containers)
        match &container.children[1] {
            TemplateElement::Container(c) => {
                let t0 = unwrap_cell_text(&c.children[0]);
                assert_eq!(t0.content, "Hello");
                let t1 = unwrap_cell_text(&c.children[1]);
                assert_eq!(t1.content, "42");
            }
            _ => panic!("Expected Container"),
        }
    }

    #[test]
    fn test_expand_table_with_border_adds_separator() {
        let mut table = make_table(2);
        table.style.border_color = Some("#000000".to_string());
        let resolved = make_resolved("tbl", vec![vec!["A".to_string(), "1".to_string()]]);
        let mut measurer = make_measurer();

        let container = expand_table(&table, &resolved, &mut measurer, 180.0);

        // header + separator line + 1 data row = 3
        assert_eq!(container.children.len(), 3);

        // Second child should be a Line
        match &container.children[1] {
            TemplateElement::Line(l) => {
                assert_eq!(l.base.id, "tbl_header_line");
            }
            _ => panic!("Expected Line separator after header"),
        }
    }

    #[test]
    fn test_expand_table_zebra_stripes() {
        let mut table = make_table(1);
        table.style.zebra_odd = Some("#f0f0f0".to_string());
        table.style.zebra_even = Some("#ffffff".to_string());
        let resolved = make_resolved(
            "tbl",
            vec![
                vec!["row0".into()],
                vec!["row1".into()],
                vec!["row2".into()],
            ],
        );
        let mut measurer = make_measurer();

        let container = expand_table(&table, &resolved, &mut measurer, 180.0);

        // header + 3 data rows
        assert_eq!(container.children.len(), 4);

        // row_0 (even index) => zebra_even (no stripe)
        match &container.children[1] {
            TemplateElement::Container(c) => {
                assert_eq!(c.style.background_color, Some("#ffffff".to_string()));
            }
            _ => panic!("Expected Container"),
        }
        // row_1 (odd index) => zebra_odd (striped)
        match &container.children[2] {
            TemplateElement::Container(c) => {
                assert_eq!(c.style.background_color, Some("#f0f0f0".to_string()));
            }
            _ => panic!("Expected Container"),
        }
        // row_2 (even index) => zebra_even (no stripe)
        match &container.children[3] {
            TemplateElement::Container(c) => {
                assert_eq!(c.style.background_color, Some("#ffffff".to_string()));
            }
            _ => panic!("Expected Container"),
        }
    }

    #[test]
    fn test_auto_columns_get_content_based_widths() {
        // Auto sütunlu tablo: genişlikler içeriğe göre hesaplanmalı
        let columns = vec![
            TableColumn {
                id: "col_0".into(),
                field: "short".into(),
                title: "No".into(),
                width: SizeValue::Auto,
                align: "right".into(),
                format: None,
            },
            TableColumn {
                id: "col_1".into(),
                field: "long".into(),
                title: "Urun / Hizmet Adi".into(),
                width: SizeValue::Auto,
                align: "left".into(),
                format: None,
            },
        ];

        let table = RepeatingTableElement {
            base: ElementBase::flow(
                "tbl".to_string(),
                SizeConstraint { width: SizeValue::Fr { value: 1.0 }, ..Default::default() },
            ),
            data_source: ArrayBinding {
                path: "items".to_string(),
            },
            columns,
            style: TableStyle::default(),
            repeat_header: Some(true),
        };

        let resolved = make_resolved(
            "tbl",
            vec![
                vec!["1".into(), "Web Uygulama Gelistirme".into()],
                vec!["2".into(), "SSL Sertifikasi".into()],
            ],
        );
        let mut measurer = make_measurer();

        let container = expand_table(&table, &resolved, &mut measurer, 180.0);

        // Header row'daki ilk hücre wrapper (kısa: "No") ikinciden (uzun: "Urun / Hizmet Adi") dar olmalı
        match &container.children[0] {
            TemplateElement::Container(c) => {
                let w0 = match &c.children[0] {
                    TemplateElement::Container(wrap) => match &wrap.base.size.width {
                        SizeValue::Fixed { value } => *value,
                        _ => panic!("Expected Fixed width for auto column wrapper"),
                    },
                    _ => panic!("Expected Container wrapper"),
                };
                let w1 = match &c.children[1] {
                    TemplateElement::Container(wrap) => match &wrap.base.size.width {
                        SizeValue::Fixed { value } => *value,
                        _ => panic!("Expected Fixed width for auto column wrapper"),
                    },
                    _ => panic!("Expected Container wrapper"),
                };
                assert!(
                    w1 > w0,
                    "Long column ({w1}mm) should be wider than short column ({w0}mm)"
                );
                // Her iki sütun toplamı available_width'e eşit olmalı
                let total = w0 + w1;
                assert!(
                    (total - 180.0).abs() < 0.1,
                    "Total width ({total}mm) should equal available width (180mm)"
                );
            }
            _ => panic!("Expected Container"),
        }
    }

    #[test]
    fn test_table_cache_hit() {
        let table = make_table(2);
        let resolved = make_resolved("tbl", vec![vec!["A".to_string(), "1".to_string()]]);
        let mut measurer = make_measurer();
        let mut cache = TableExpandCache::new();

        // First call — cache miss
        let result1 = expand_table_cached(&table, &resolved, &mut measurer, 180.0, &mut cache);
        assert_eq!(cache.len(), 1);

        // Second call — same inputs — cache hit
        let result2 = expand_table_cached(&table, &resolved, &mut measurer, 180.0, &mut cache);
        assert_eq!(cache.len(), 1); // no new entry
        assert_eq!(result1.base.id, result2.base.id);
        assert_eq!(result1.children.len(), result2.children.len());
    }

    #[test]
    fn test_table_cache_miss_on_data_change() {
        let table = make_table(2);
        let resolved1 = make_resolved("tbl", vec![vec!["A".to_string(), "1".to_string()]]);
        let resolved2 = make_resolved("tbl", vec![vec!["B".to_string(), "2".to_string()]]);
        let mut measurer = make_measurer();
        let mut cache = TableExpandCache::new();

        expand_table_cached(&table, &resolved1, &mut measurer, 180.0, &mut cache);
        assert_eq!(cache.len(), 1);

        // Different data — cache miss
        expand_table_cached(&table, &resolved2, &mut measurer, 180.0, &mut cache);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_table_cache_miss_on_width_change() {
        let table = make_table(2);
        let resolved = make_resolved("tbl", vec![vec!["A".to_string(), "1".to_string()]]);
        let mut measurer = make_measurer();
        let mut cache = TableExpandCache::new();

        expand_table_cached(&table, &resolved, &mut measurer, 180.0, &mut cache);
        assert_eq!(cache.len(), 1);

        // Different available width — cache miss
        expand_table_cached(&table, &resolved, &mut measurer, 160.0, &mut cache);
        assert_eq!(cache.len(), 2);
    }
}
