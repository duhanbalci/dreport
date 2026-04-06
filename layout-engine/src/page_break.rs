use std::collections::{HashMap, HashSet};

use crate::{ElementLayout, PageLayout, ResolvedContent};

/// Sayfa bölme girdi yapısı
pub struct PageSplitInput {
    /// Body elemanları (sınırsız yükseklikte hesaplanmış, mutlak mm koordinatları)
    pub body_elements: Vec<ElementLayout>,
    /// Sayfa yüksekliği (mm)
    pub page_height_mm: f64,
    /// Header yüksekliği (mm) — body'nin başlangıç offset'i
    pub header_height_mm: f64,
    /// Footer yüksekliği (mm)
    pub footer_height_mm: f64,
    /// Header elemanları (klonlanacak, her sayfada tekrar)
    pub header_elements: Vec<ElementLayout>,
    /// Footer elemanları (klonlanacak, her sayfada tekrar)
    pub footer_elements: Vec<ElementLayout>,
    /// Sayfa genişliği (mm)
    pub page_width_mm: f64,
    /// Container break modları: element_id → "auto" | "avoid"
    pub break_modes: HashMap<String, String>,
    /// page_number format string'leri: element_id → format
    pub page_number_formats: HashMap<String, String>,
    /// Root container'ın üst padding'i (mm) — sayfa 2+ için body offset
    pub root_padding_top_mm: f64,
    /// Header tekrarı kapatılmış tablo ID'leri
    pub no_repeat_header_tables: HashSet<String>,
}

/// Body elemanlarını sayfalara böl, header/footer ekle, page number'ları çöz.
pub fn split_into_pages(input: PageSplitInput) -> Vec<PageLayout> {
    let content_height = input.page_height_mm - input.header_height_mm - input.footer_height_mm;

    if content_height <= 0.0 {
        // Header + footer sayfaya sığmıyor — tek sayfa döndür
        return vec![assemble_page(
            0,
            &input.body_elements,
            &input.header_elements,
            &input.footer_elements,
            input.page_width_mm,
            input.page_height_mm,
            input.header_height_mm,
            input.footer_height_mm,
            0.0,
            input.root_padding_top_mm,
        )];
    }

    // Parent lookup: element_id → parent_id (children alanından)
    let parent_map = build_parent_map(&input.body_elements);

    // "avoid" grupları: container_id → (top_mm, bottom_mm, tüm descendant id'leri)
    let avoid_groups = build_avoid_groups(&input.body_elements, &input.break_modes, &parent_map);

    // Tablo yapısı tespiti: table_id → header element id'leri
    // repeat_header == false olan tablolar hariç tutulur
    let mut table_info = detect_table_structure(&input.body_elements);
    for table_id in &input.no_repeat_header_tables {
        table_info.remove(table_id);
    }

    // Elemanları sayfalara böl
    let page_slices = split_elements(
        &input.body_elements,
        content_height,
        &avoid_groups,
        &parent_map,
        &table_info,
    );

    let total_pages = page_slices.len().max(1);

    let mut pages: Vec<PageLayout> = Vec::with_capacity(total_pages);

    for (page_idx, slice) in page_slices.iter().enumerate() {
        let page = assemble_page(
            page_idx,
            &slice.elements,
            &input.header_elements,
            &input.footer_elements,
            input.page_width_mm,
            input.page_height_mm,
            input.header_height_mm,
            input.footer_height_mm,
            slice.y_offset,
            input.root_padding_top_mm,
        );
        pages.push(page);
    }

    // Boş sayfa koruması
    if pages.is_empty() {
        pages.push(assemble_page(
            0,
            &[],
            &input.header_elements,
            &input.footer_elements,
            input.page_width_mm,
            input.page_height_mm,
            input.header_height_mm,
            input.footer_height_mm,
            0.0,
            input.root_padding_top_mm,
        ));
    }

    // Page number çözümleme
    let total = pages.len();
    for (page_idx, page) in pages.iter_mut().enumerate() {
        resolve_page_numbers(&mut page.elements, page_idx + 1, total, &input.page_number_formats);
    }

    pages
}

/// Bir avoid grubunun bilgisi
struct AvoidGroup {
    top_mm: f64,
    bottom_mm: f64,
    element_ids: HashSet<String>,
}

/// Tablo yapısı bilgisi
struct TableInfo {
    /// table_id → header satırının eleman id'leri
    _header_element_ids: Vec<String>,
    /// table_id → header satırındaki elemanların klonları
    header_elements: Vec<ElementLayout>,
    /// Header yüksekliği (mm)
    header_height_mm: f64,
}

/// Sayfa dilimi
struct PageSlice {
    elements: Vec<ElementLayout>,
    y_offset: f64, // Bu sayfanın strip'teki başlangıç y koordinatı
}

fn build_parent_map(elements: &[ElementLayout]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for el in elements {
        for child_id in &el.children {
            map.insert(child_id.clone(), el.id.clone());
        }
    }
    map
}

fn build_avoid_groups(
    elements: &[ElementLayout],
    break_modes: &HashMap<String, String>,
    _parent_map: &HashMap<String, String>,
) -> Vec<AvoidGroup> {
    // Hangi container'lar avoid?
    let avoid_ids: HashSet<&String> = break_modes
        .iter()
        .filter(|(_, mode)| mode.as_str() == "avoid")
        .map(|(id, _)| id)
        .collect();

    if avoid_ids.is_empty() {
        return vec![];
    }

    let element_map: HashMap<&str, &ElementLayout> =
        elements.iter().map(|e| (e.id.as_str(), e)).collect();

    let mut groups = Vec::new();

    for avoid_id in &avoid_ids {
        if let Some(container) = element_map.get(avoid_id.as_str()) {
            // Bu container'ın tüm descendant'larını bul
            let mut descendant_ids = HashSet::new();
            descendant_ids.insert(container.id.clone());
            collect_descendants(container.id.as_str(), elements, &mut descendant_ids);

            // Grubun top/bottom'ını hesapla
            let mut top = container.y_mm;
            let mut bottom = container.y_mm + container.height_mm;
            for el in elements {
                if descendant_ids.contains(&el.id) {
                    top = top.min(el.y_mm);
                    bottom = bottom.max(el.y_mm + el.height_mm);
                }
            }

            groups.push(AvoidGroup {
                top_mm: top,
                bottom_mm: bottom,
                element_ids: descendant_ids,
            });
        }
    }

    groups
}

fn collect_descendants(
    parent_id: &str,
    elements: &[ElementLayout],
    result: &mut HashSet<String>,
) {
    // children alanından recursive olarak topla
    for el in elements {
        if el.id == parent_id {
            for child_id in &el.children {
                result.insert(child_id.clone());
                collect_descendants(child_id, elements, result);
            }
            break;
        }
    }
}

/// Bir elemanın en yakın avoid ancestor'ı var mı?
fn find_avoid_group<'a>(
    element_id: &str,
    avoid_groups: &'a [AvoidGroup],
) -> Option<&'a AvoidGroup> {
    avoid_groups
        .iter()
        .find(|g| g.element_ids.contains(element_id))
}

fn detect_table_structure(elements: &[ElementLayout]) -> HashMap<String, TableInfo> {
    // Tablo yapısını ID pattern'inden tespit et:
    // {table_id}_header → header satırı container
    // {table_id}_hdr_{N} → header hücreleri
    // {table_id}_row_{N} → veri satırı container
    // {table_id}_r{N}c{M} → veri hücreleri

    let mut tables: HashMap<String, TableInfo> = HashMap::new();

    // Önce header container'ları bul
    for el in elements {
        if el.id.ends_with("_header") && el.element_type == "container" {
            let table_id = el.id.trim_end_matches("_header").to_string();
            // Bu table_id ile başlayan row'lar var mı kontrol et
            let has_rows = elements
                .iter()
                .any(|e| e.id.starts_with(&format!("{}_row_", table_id)));
            if has_rows {
                // Header elemanlarını topla (header container + children)
                let mut header_ids = vec![el.id.clone()];
                for child_id in &el.children {
                    header_ids.push(child_id.clone());
                }

                let header_elements: Vec<ElementLayout> = elements
                    .iter()
                    .filter(|e| header_ids.contains(&e.id))
                    .cloned()
                    .collect();

                let header_height = el.height_mm;

                tables.insert(
                    table_id,
                    TableInfo {
                        _header_element_ids: header_ids,
                        header_elements,
                        header_height_mm: header_height,
                    },
                );
            }
        }
    }

    tables
}

/// Hangi tablo'ya ait bir satır elemanı mı?
fn detect_table_row(element_id: &str) -> Option<(String, usize)> {
    // Pattern: {table_id}_row_{N}
    if let Some(pos) = element_id.rfind("_row_") {
        let table_id = element_id[..pos].to_string();
        let row_str = &element_id[pos + 5..];
        if let Ok(row_idx) = row_str.parse::<usize>() {
            return Some((table_id, row_idx));
        }
    }
    None
}

fn split_elements(
    elements: &[ElementLayout],
    content_height: f64,
    avoid_groups: &[AvoidGroup],
    _parent_map: &HashMap<String, String>,
    table_info: &HashMap<String, TableInfo>,
) -> Vec<PageSlice> {
    if elements.is_empty() {
        return vec![PageSlice {
            elements: vec![],
            y_offset: 0.0,
        }];
    }

    let mut pages: Vec<PageSlice> = vec![PageSlice {
        elements: Vec::new(),
        y_offset: 0.0,
    }];

    // Yapısal container'ları tespit et: çocukları arasında container olan container'lar.
    // Bu container'lar sayfa sınırında bölünebilir (çocukları bireysel sayfa bölmesi yapar).
    // Aksine, çocukları sadece leaf olan container'lar (ör. tablo satırı) atomik kalır.
    let element_type_map: HashMap<&str, &str> = elements
        .iter()
        .map(|e| (e.id.as_str(), e.element_type.as_str()))
        .collect();
    let splittable_containers: HashSet<&str> = elements
        .iter()
        .filter(|e| e.element_type == "container")
        .filter(|e| {
            e.children
                .iter()
                .any(|child_id| element_type_map.get(child_id.as_str()) == Some(&"container"))
        })
        .map(|e| e.id.as_str())
        .collect();

    let mut page_top = 0.0; // Mevcut sayfanın strip'teki başlangıç y'si
    let mut processed: HashSet<String> = HashSet::new();
    // Hangi tablo'ların header'ı bu sayfada zaten var?
    let mut table_header_on_page: HashSet<String> = HashSet::new();

    for el in elements {
        if processed.contains(&el.id) {
            continue;
        }

        // page_break elemanı → mevcut sayfaya ekle, sonra yeni sayfa zorla
        if el.element_type == "page_break" {
            pages.last_mut().unwrap().elements.push(el.clone());
            processed.insert(el.id.clone());
            page_top = el.y_mm + el.height_mm;
            pages.push(PageSlice {
                elements: Vec::new(),
                y_offset: page_top,
            });
            table_header_on_page.clear();
            continue;
        }

        let el_top = el.y_mm;
        let el_bottom = el.y_mm + el.height_mm;
        let relative_bottom = el_bottom - page_top;

        // Avoid group kontrolü
        if let Some(group) = find_avoid_group(&el.id, avoid_groups) {
            let group_relative_bottom = group.bottom_mm - page_top;
            let group_height = group.bottom_mm - group.top_mm;

            if group_relative_bottom > content_height && group_height <= content_height {
                // Grup mevcut sayfaya sığmıyor ama tek sayfaya sığar → yeni sayfa
                page_top = group.top_mm;
                pages.push(PageSlice {
                    elements: Vec::new(),
                    y_offset: page_top,
                });
                table_header_on_page.clear();
            }
            // Grup sayfadan büyükse → normal akışa devam (bölünemez ama mecbur)
        }

        // Eleman mevcut sayfaya sığıyor mu?
        if relative_bottom > content_height && el_top > page_top {
            // Yapısal container (çocukları container olan) → bölünebilir.
            // Komple yeni sayfaya atmak yerine mevcut sayfada bırak,
            // çocuk elemanlar bireysel olarak sayfa bölmesini halledecek.
            if splittable_containers.contains(el.id.as_str()) {
                pages.last_mut().unwrap().elements.push(el.clone());
                processed.insert(el.id.clone());
                continue;
            }

            // Sığmıyor → yeni sayfa

            // Tablo satırı mı? Header tekrarı gerekebilir
            let mut table_header_to_add: Option<(String, Vec<ElementLayout>, f64)> = None;
            if let Some((table_id, _row_idx)) = detect_table_row(&el.id) {
                if let Some(info) = table_info.get(&table_id) {
                    // Yeni sayfada bu tablonun header'ını tekrarla
                    table_header_to_add =
                        Some((table_id.clone(), info.header_elements.clone(), info.header_height_mm));
                }
            }

            page_top = el_top;

            // Tablo header tekrarı varsa, header yüksekliği kadar offset
            if let Some((ref table_id, ref header_els, header_h)) = table_header_to_add {
                // Header'ı yeni sayfanın başına koy (offset'li)
                page_top = el_top - header_h;
                let new_page_idx = pages.len();
                pages.push(PageSlice {
                    elements: Vec::new(),
                    y_offset: page_top,
                });
                table_header_on_page.clear();

                // Header elemanlarını klonla ve y pozisyonlarını yeni sayfaya taşı.
                // Orijinal header elemanları tablonun ilk konumundaki y değerlerine sahip.
                // Yeni sayfada page_top'tan başlamaları gerekir.
                let orig_header_y = header_els
                    .iter()
                    .map(|e| e.y_mm)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(page_top);
                let y_shift = page_top - orig_header_y;

                for hdr_el in header_els {
                    let mut cloned = hdr_el.clone();
                    cloned.y_mm += y_shift;
                    cloned.id = format!("{}_p{}", hdr_el.id, new_page_idx);
                    pages.last_mut().unwrap().elements.push(cloned);
                }
                table_header_on_page.insert(table_id.clone());
            } else {
                pages.push(PageSlice {
                    elements: Vec::new(),
                    y_offset: page_top,
                });
                table_header_on_page.clear();
            }
        }

        // Elemanı mevcut sayfaya ekle
        pages.last_mut().unwrap().elements.push(el.clone());
        processed.insert(el.id.clone());
    }

    pages
}

fn assemble_page(
    page_index: usize,
    body_elements: &[ElementLayout],
    header_elements: &[ElementLayout],
    footer_elements: &[ElementLayout],
    page_width_mm: f64,
    page_height_mm: f64,
    header_height_mm: f64,
    footer_height_mm: f64,
    body_y_offset: f64,
    root_padding_top_mm: f64,
) -> PageLayout {
    let mut elements = Vec::new();

    // Header elemanları (y = orijinal y, sayfa başında)
    for el in header_elements {
        let mut cloned = el.clone();
        if page_index > 0 {
            // Sonraki sayfalarda ID'yi unique yap
            cloned.id = format!("{}_p{}", el.id, page_index);
        }
        elements.push(cloned);
    }

    // Body elemanları (y offset'li — strip y'den sayfa-relative y'ye)
    // Sayfa 2+ için root padding tekrar eklenir (root container sadece sayfa 1'de var)
    let extra_top = if page_index > 0 { root_padding_top_mm } else { 0.0 };
    for el in body_elements {
        let mut adjusted = el.clone();
        adjusted.y_mm = el.y_mm - body_y_offset + header_height_mm + extra_top;
        elements.push(adjusted);
    }

    // Footer elemanları (sayfanın altında)
    let footer_y_offset = page_height_mm - footer_height_mm;
    for el in footer_elements {
        let mut cloned = el.clone();
        // Footer elemanlarının y'si footer container'ın başlangıcına relative
        // Footer'ın orijinal y'si 0'dan başlıyor (ayrı hesaplanıyor)
        // Sayfa içi pozisyon: footer_y_offset + orijinal y
        cloned.y_mm = el.y_mm + footer_y_offset;
        if page_index > 0 {
            cloned.id = format!("{}_p{}", el.id, page_index);
        }
        elements.push(cloned);
    }

    PageLayout {
        page_index,
        width_mm: page_width_mm,
        height_mm: page_height_mm,
        elements,
    }
}

fn resolve_page_numbers(
    elements: &mut [ElementLayout],
    current_page: usize,
    total_pages: usize,
    formats: &HashMap<String, String>,
) {
    for el in elements.iter_mut() {
        if el.element_type != "page_number" {
            continue;
        }

        // ID'den orijinal format ID'sini çıkar (sayfa klonları _p{N} ile biter)
        let original_id = if let Some(pos) = el.id.rfind("_p") {
            let suffix = &el.id[pos + 2..];
            if suffix.parse::<usize>().is_ok() {
                &el.id[..pos]
            } else {
                &el.id
            }
        } else {
            &el.id
        };

        let fmt = formats
            .get(original_id)
            .map(|s| s.as_str())
            .unwrap_or("{current} / {total}");

        let text = fmt
            .replace("{current}", &current_page.to_string())
            .replace("{total}", &total_pages.to_string());

        el.content = Some(ResolvedContent::PageNumber {
            current: current_page,
            total: total_pages,
        });
        // Ayrıca text content'i de güncelle (LayoutRenderer text olarak render ediyor)
        // PageNumber render'da content.type === "text" kontrolü var, text olarak da ekle
        el.content = Some(ResolvedContent::Text { value: text });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResolvedStyle;

    fn make_element(id: &str, y: f64, height: f64, element_type: &str) -> ElementLayout {
        ElementLayout {
            id: id.to_string(),
            x_mm: 0.0,
            y_mm: y,
            width_mm: 180.0,
            height_mm: height,
            element_type: element_type.to_string(),
            content: None,
            style: ResolvedStyle::default(),
            children: vec![],
        }
    }

    #[test]
    fn test_single_page_no_split() {
        let input = PageSplitInput {
            body_elements: vec![
                make_element("el1", 0.0, 50.0, "text"),
                make_element("el2", 50.0, 50.0, "text"),
            ],
            page_height_mm: 297.0,
            header_height_mm: 0.0,
            footer_height_mm: 0.0,
            header_elements: vec![],
            footer_elements: vec![],
            page_width_mm: 210.0,
            break_modes: HashMap::new(),
            page_number_formats: HashMap::new(),
            root_padding_top_mm: 0.0,
            no_repeat_header_tables: HashSet::new(),
        };

        let pages = split_into_pages(input);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].elements.len(), 2);
    }

    #[test]
    fn test_auto_page_break() {
        let input = PageSplitInput {
            body_elements: vec![
                make_element("el1", 0.0, 200.0, "text"),
                make_element("el2", 200.0, 200.0, "text"),
            ],
            page_height_mm: 297.0,
            header_height_mm: 0.0,
            footer_height_mm: 0.0,
            header_elements: vec![],
            footer_elements: vec![],
            page_width_mm: 210.0,
            break_modes: HashMap::new(),
            page_number_formats: HashMap::new(),
            root_padding_top_mm: 0.0,
            no_repeat_header_tables: HashSet::new(),
        };

        let pages = split_into_pages(input);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].elements.len(), 1);
        assert_eq!(pages[0].elements[0].id, "el1");
        assert_eq!(pages[1].elements.len(), 1);
        assert_eq!(pages[1].elements[0].id, "el2");
    }

    #[test]
    fn test_manual_page_break() {
        let input = PageSplitInput {
            body_elements: vec![
                make_element("el1", 0.0, 50.0, "text"),
                make_element("pb1", 50.0, 0.0, "page_break"),
                make_element("el2", 50.0, 50.0, "text"),
            ],
            page_height_mm: 297.0,
            header_height_mm: 0.0,
            footer_height_mm: 0.0,
            header_elements: vec![],
            footer_elements: vec![],
            page_width_mm: 210.0,
            break_modes: HashMap::new(),
            page_number_formats: HashMap::new(),
            root_padding_top_mm: 0.0,
            no_repeat_header_tables: HashSet::new(),
        };

        let pages = split_into_pages(input);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].elements.len(), 2); // el1 + pb1
        assert_eq!(pages[1].elements.len(), 1); // el2
    }

    #[test]
    fn test_header_footer_on_all_pages() {
        let header = vec![make_element("hdr", 0.0, 15.0, "text")];
        let footer = vec![make_element("ftr", 0.0, 10.0, "text")];

        let input = PageSplitInput {
            body_elements: vec![
                make_element("el1", 0.0, 200.0, "text"),
                make_element("el2", 200.0, 200.0, "text"),
            ],
            page_height_mm: 297.0,
            header_height_mm: 15.0,
            footer_height_mm: 10.0,
            header_elements: header,
            footer_elements: footer,
            page_width_mm: 210.0,
            break_modes: HashMap::new(),
            page_number_formats: HashMap::new(),
            root_padding_top_mm: 0.0,
            no_repeat_header_tables: HashSet::new(),
        };

        let pages = split_into_pages(input);
        assert_eq!(pages.len(), 2);
        // Her sayfada header + body + footer var
        // Sayfa 1: hdr + el1 + ftr = 3
        assert!(pages[0].elements.iter().any(|e| e.id == "hdr"));
        assert!(pages[0].elements.iter().any(|e| e.id == "el1"));
        assert!(pages[0].elements.iter().any(|e| e.id == "ftr"));
        // Sayfa 2: hdr_p1 + el2 + ftr_p1 = 3
        assert!(pages[1].elements.iter().any(|e| e.id == "hdr_p1"));
        assert!(pages[1].elements.iter().any(|e| e.id == "el2"));
        assert!(pages[1].elements.iter().any(|e| e.id == "ftr_p1"));
    }

    #[test]
    fn test_page_numbers_resolved() {
        let mut formats = HashMap::new();
        formats.insert("pn".to_string(), "{current} / {total}".to_string());

        let input = PageSplitInput {
            body_elements: vec![
                make_element("el1", 0.0, 200.0, "text"),
                make_element("el2", 200.0, 200.0, "text"),
            ],
            page_height_mm: 297.0,
            header_height_mm: 15.0,
            footer_height_mm: 10.0,
            header_elements: vec![{
                let mut el = make_element("pn", 0.0, 10.0, "page_number");
                el.content = Some(ResolvedContent::Text {
                    value: "1 / 1".to_string(),
                });
                el
            }],
            footer_elements: vec![],
            page_width_mm: 210.0,
            break_modes: HashMap::new(),
            page_number_formats: formats,
            root_padding_top_mm: 0.0,
            no_repeat_header_tables: HashSet::new(),
        };

        let pages = split_into_pages(input);
        assert_eq!(pages.len(), 2);

        // Sayfa 1: pn → "1 / 2"
        let pn1 = pages[0].elements.iter().find(|e| e.id == "pn").unwrap();
        if let Some(ResolvedContent::Text { value }) = &pn1.content {
            assert_eq!(value, "1 / 2");
        } else {
            panic!("page_number content should be text");
        }

        // Sayfa 2: pn_p1 → "2 / 2"
        let pn2 = pages[1]
            .elements
            .iter()
            .find(|e| e.id == "pn_p1")
            .unwrap();
        if let Some(ResolvedContent::Text { value }) = &pn2.content {
            assert_eq!(value, "2 / 2");
        } else {
            panic!("page_number content should be text");
        }
    }

    #[test]
    fn test_table_splits_across_pages_not_jumps() {
        // Tablo wrapper container sayfa yüksekliğinden büyük olduğunda,
        // komple yeni sayfaya atlamak yerine satırları sayfalara bölmeli.
        //
        // Senaryo: sayfa 200mm, content 200mm (header/footer yok).
        // Tablonun öncesinde 50mm'lik bir eleman var.
        // Tablo wrapper: y=50, h=300 (sayfaya sığmaz).
        // Tablo satırları: her biri 30mm.
        // Beklenen: ilk sayfa = el1 + tbl wrapper + header + ilk ~5 satır,
        //           ikinci sayfa = kalan satırlar.

        let mut tbl_wrapper = make_element("tbl", 50.0, 300.0, "container");
        tbl_wrapper.children = vec![
            "tbl_header".to_string(),
            "tbl_row_0".to_string(),
            "tbl_row_1".to_string(),
            "tbl_row_2".to_string(),
            "tbl_row_3".to_string(),
            "tbl_row_4".to_string(),
            "tbl_row_5".to_string(),
            "tbl_row_6".to_string(),
            "tbl_row_7".to_string(),
            "tbl_row_8".to_string(),
            "tbl_row_9".to_string(),
        ];

        let tbl_header = {
            let mut el = make_element("tbl_header", 50.0, 20.0, "container");
            el.children = vec!["tbl_hdr_0".to_string()];
            el
        };
        let tbl_hdr_0 = make_element("tbl_hdr_0", 50.0, 20.0, "static_text");

        // 10 satır, her biri 28mm (gap dahil), y=70'ten başlıyor
        let rows: Vec<ElementLayout> = (0..10)
            .flat_map(|i| {
                let y = 70.0 + (i as f64) * 28.0;
                let mut row = make_element(&format!("tbl_row_{}", i), y, 28.0, "container");
                row.children = vec![format!("tbl_r{}c0", i)];
                let cell = make_element(&format!("tbl_r{}c0", i), y, 28.0, "static_text");
                vec![row, cell]
            })
            .collect();

        let mut body_elements = vec![
            make_element("el1", 0.0, 50.0, "text"),
            tbl_wrapper,
            tbl_header,
            tbl_hdr_0,
        ];
        body_elements.extend(rows);

        let input = PageSplitInput {
            body_elements,
            page_height_mm: 200.0,
            header_height_mm: 0.0,
            footer_height_mm: 0.0,
            header_elements: vec![],
            footer_elements: vec![],
            page_width_mm: 210.0,
            break_modes: HashMap::new(),
            page_number_formats: HashMap::new(),
            root_padding_top_mm: 0.0,
            no_repeat_header_tables: HashSet::new(),
        };

        let pages = split_into_pages(input);

        // Tablo komple 2. sayfaya atlamamalı!
        // Sayfa 1'de el1 + tablo başlangıcı olmalı
        assert!(
            pages[0].elements.iter().any(|e| e.id == "el1"),
            "el1 should be on page 1"
        );
        assert!(
            pages[0].elements.iter().any(|e| e.id == "tbl"),
            "table wrapper should start on page 1 (not jump to page 2)"
        );
        assert!(
            pages[0].elements.iter().any(|e| e.id == "tbl_header"),
            "table header should be on page 1"
        );
        assert!(
            pages[0].elements.iter().any(|e| e.id == "tbl_row_0"),
            "first table row should be on page 1"
        );

        // En az 2 sayfa olmalı (tablo bölünmeli)
        assert!(
            pages.len() >= 2,
            "table should split across at least 2 pages, got {}",
            pages.len()
        );

        // Son satırlar sonraki sayfa(lar)da olmalı
        let last_row_id = "tbl_row_9";
        let last_row_page = pages
            .iter()
            .position(|p| p.elements.iter().any(|e| e.id == last_row_id))
            .expect("last row should exist somewhere");
        assert!(
            last_row_page > 0,
            "last table row should be on a later page"
        );
    }

    #[test]
    fn test_table_header_repeats_on_new_page() {
        // Tablo satırı yeni sayfaya geçtiğinde, header tekrar edilmeli.
        //
        // Senaryo: sayfa 150mm, tablo y=0'dan başlıyor.
        // Header: 20mm, her satır 30mm → 4 satır = 120mm + header 20mm = 140mm (1. sayfa)
        // 5. satır sığmaz → 2. sayfaya geçer, header tekrar olmalı.

        let mut tbl_wrapper = make_element("tbl", 0.0, 200.0, "container");
        tbl_wrapper.children = vec![
            "tbl_header".to_string(),
            "tbl_row_0".to_string(),
            "tbl_row_1".to_string(),
            "tbl_row_2".to_string(),
            "tbl_row_3".to_string(),
            "tbl_row_4".to_string(),
            "tbl_row_5".to_string(),
        ];

        let tbl_header = {
            let mut el = make_element("tbl_header", 0.0, 20.0, "container");
            el.children = vec!["tbl_hdr_0".to_string()];
            el
        };
        let tbl_hdr_0 = make_element("tbl_hdr_0", 0.0, 20.0, "static_text");

        let rows: Vec<ElementLayout> = (0..6)
            .flat_map(|i| {
                let y = 20.0 + (i as f64) * 30.0;
                let mut row = make_element(&format!("tbl_row_{}", i), y, 30.0, "container");
                row.children = vec![format!("tbl_r{}c0", i)];
                let cell = make_element(&format!("tbl_r{}c0", i), y, 30.0, "static_text");
                vec![row, cell]
            })
            .collect();

        let mut body_elements = vec![tbl_wrapper, tbl_header, tbl_hdr_0];
        body_elements.extend(rows);

        let input = PageSplitInput {
            body_elements,
            page_height_mm: 150.0,
            header_height_mm: 0.0,
            footer_height_mm: 0.0,
            header_elements: vec![],
            footer_elements: vec![],
            page_width_mm: 210.0,
            break_modes: HashMap::new(),
            page_number_formats: HashMap::new(),
            root_padding_top_mm: 0.0,
            no_repeat_header_tables: HashSet::new(),
        };

        let pages = split_into_pages(input);

        assert!(pages.len() >= 2, "should split into at least 2 pages");

        // Sayfa 2'de tablo header'ının tekrar edilmiş kopyası olmalı
        let page2_has_header = pages[1]
            .elements
            .iter()
            .any(|e| e.id.starts_with("tbl_header"));
        assert!(
            page2_has_header,
            "table header should be repeated on page 2. Page 2 elements: {:?}",
            pages[1].elements.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_repeated_header_no_gap_with_rows() {
        // Tekrarlanan header ile ilk satır arasında boşluk olmamalı.
        // Header'ın y pozisyonu yeni sayfanın başlangıcına relocate edilmeli.
        //
        // Senaryo: tablo y=100'de başlıyor, header 10mm, satırlar 8mm.
        // Sayfa content_height=80mm.
        // Satırlar: y=110, 118, 126, ... → relative_bottom kontrolü.

        let mut tbl_wrapper = make_element("tbl", 100.0, 120.0, "container");
        tbl_wrapper.children = vec![
            "tbl_header".to_string(),
            "tbl_row_0".to_string(),
            "tbl_row_1".to_string(),
            "tbl_row_2".to_string(),
            "tbl_row_3".to_string(),
            "tbl_row_4".to_string(),
            "tbl_row_5".to_string(),
            "tbl_row_6".to_string(),
            "tbl_row_7".to_string(),
            "tbl_row_8".to_string(),
            "tbl_row_9".to_string(),
        ];

        let tbl_header = {
            let mut el = make_element("tbl_header", 100.0, 10.0, "container");
            el.children = vec!["tbl_hdr_0".to_string()];
            el
        };
        let tbl_hdr_0 = make_element("tbl_hdr_0", 100.0, 10.0, "static_text");

        let rows: Vec<ElementLayout> = (0..10)
            .flat_map(|i| {
                let y = 110.0 + (i as f64) * 12.0;
                let mut row = make_element(&format!("tbl_row_{}", i), y, 12.0, "container");
                row.children = vec![format!("tbl_r{}c0", i)];
                let cell = make_element(&format!("tbl_r{}c0", i), y, 12.0, "static_text");
                vec![row, cell]
            })
            .collect();

        let mut body_elements = vec![
            make_element("el1", 0.0, 50.0, "text"), // 50mm metin
            make_element("el2", 50.0, 50.0, "text"), // 50mm metin (toplam 100mm)
            tbl_wrapper,
            tbl_header,
            tbl_hdr_0,
        ];
        body_elements.extend(rows);

        // content_height = 200 - 15 - 10 = 175
        let doc_header = vec![make_element("doc_hdr", 0.0, 15.0, "text")];
        let doc_footer = vec![make_element("doc_ftr", 0.0, 10.0, "text")];

        let input = PageSplitInput {
            body_elements,
            page_height_mm: 200.0,
            header_height_mm: 15.0,
            footer_height_mm: 10.0,
            header_elements: doc_header,
            footer_elements: doc_footer,
            page_width_mm: 210.0,
            break_modes: HashMap::new(),
            page_number_formats: HashMap::new(),
            root_padding_top_mm: 5.0,
            no_repeat_header_tables: HashSet::new(),
        };

        let pages = split_into_pages(input);
        assert!(pages.len() >= 2, "should have at least 2 pages");

        // Sayfa 2'deki elemanlar
        let page2 = &pages[1];

        // Tekrarlanan header'ı bul
        let repeated_header = page2
            .elements
            .iter()
            .find(|e| e.id.starts_with("tbl_header") && e.id != "tbl_header")
            .expect("repeated table header should exist on page 2");

        // Header'dan sonraki ilk satırı bul
        let first_row_on_page2 = page2
            .elements
            .iter()
            .find(|e| e.id.starts_with("tbl_row_"))
            .expect("at least one table row should be on page 2");

        // Header'ın alt kenarı ile satırın üst kenarı arasında boşluk olmamalı (veya çok az)
        let header_bottom = repeated_header.y_mm + repeated_header.height_mm;
        let row_top = first_row_on_page2.y_mm;
        let gap = (row_top - header_bottom).abs();

        assert!(
            gap < 1.0,
            "gap between repeated header (bottom={:.1}) and first row (top={:.1}) should be < 1mm, got {:.1}mm",
            header_bottom,
            row_top,
            gap
        );

        // Header y değeri negatif olmamalı
        assert!(
            repeated_header.y_mm >= 0.0,
            "repeated header y should be non-negative, got {:.1}",
            repeated_header.y_mm
        );

        // Header, document header'dan sonra gelmeli
        assert!(
            repeated_header.y_mm >= 15.0,
            "repeated header should be after doc header (15mm), got {:.1}",
            repeated_header.y_mm
        );
    }
}
