use std::collections::HashMap;

use dreport_core::models::*;
use taffy::prelude::*;

use crate::data_resolve::ResolvedData;
use crate::sizing::{self, mm_to_pt, pt_to_mm};
use crate::table_layout::{self, TableExpandCache};
use crate::text_measure::TextMeasurer;
use crate::{ElementLayout, LayoutError, LayoutResult, ResolvedContent, ResolvedStyle};

/// Taffy node ile dreport element arasındaki mapping
struct NodeInfo {
    element_id: String,
    element_type: String,
    content: Option<ResolvedContent>,
    style: ResolvedStyle,
    children_ids: Vec<String>,
}

/// Taffy leaf node'lar için ölçüm context'i
struct MeasureContext {
    text: String,
    font_family: Option<String>,
    font_size_pt: f32,
    font_weight: Option<String>,
    /// Rich text span'ları (varsa text/font_family/font_size_pt/font_weight yok sayılır)
    rich_spans: Option<Vec<crate::text_measure::RichSpanMeasure>>,
}

/// Ana layout hesaplama fonksiyonu.
pub fn compute(
    template: &Template,
    resolved: &ResolvedData,
    measurer: &mut TextMeasurer,
) -> Result<LayoutResult, LayoutError> {
    let page_w_pt = mm_to_pt(template.page.width);
    let page_width_mm = template.page.width;

    // --- 1. Header layout (varsa) ---
    let (header_elements, header_height_mm) = if let Some(ref header) = template.header {
        compute_section(header, page_w_pt, page_width_mm, resolved, measurer)?
    } else {
        (vec![], 0.0)
    };

    // --- 2. Footer layout (varsa) ---
    let (footer_elements, footer_height_mm) = if let Some(ref footer) = template.footer {
        compute_section(footer, page_w_pt, page_width_mm, resolved, measurer)?
    } else {
        (vec![], 0.0)
    };

    // --- 3. Body layout — SINIRSIZ YÜKSEKLİK ---
    let mut taffy = TaffyTree::<MeasureContext>::new();
    taffy.disable_rounding();
    let mut node_map: HashMap<NodeId, NodeInfo> = HashMap::new();
    let mut table_cache = TableExpandCache::new();

    let page_width_mm = template.page.width;
    let root_node = build_container(
        &template.root,
        &mut taffy,
        &mut node_map,
        resolved,
        None,
        measurer,
        page_width_mm,
        &mut table_cache,
    )?;

    // Sayfa wrapper: sayfa genişliğinde ama yükseklik sınırsız (auto)
    let page_style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        size: Size {
            width: Dimension::length(page_w_pt),
            height: Dimension::auto(),
        },
        ..Default::default()
    };
    let page_node = taffy.new_with_children(page_style, &[root_node])?;

    taffy
        .compute_layout_with_measure(
            page_node,
            Size {
                width: AvailableSpace::Definite(page_w_pt),
                height: AvailableSpace::MaxContent,
            },
            |known_dimensions, available_space, _node_id, context, _style| {
                measure_leaf(known_dimensions, available_space, context, measurer)
            },
        )?;

    let body_elements = collect_layout(&taffy, root_node, &node_map, resolved, 0.0, 0.0)?;

    // --- 4. Container break modlarını topla ---
    let break_modes = collect_break_modes(&template.root);

    // --- 4b. repeat_header == false olan tablo ID'lerini topla ---
    let no_repeat_header_tables = collect_no_repeat_header_tables(&template.root);

    // --- 5. Sayfalara böl ---
    let input = crate::page_break::PageSplitInput {
        body_elements,
        page_height_mm: template.page.height,
        header_height_mm,
        footer_height_mm,
        header_elements,
        footer_elements,
        page_width_mm: template.page.width,
        break_modes,
        page_number_formats: resolved.page_number_formats.clone(),
        root_padding_top_mm: template.root.padding.top,
        no_repeat_header_tables,
    };

    let pages = crate::page_break::split_into_pages(input);

    Ok(LayoutResult { pages })
}

/// Header veya footer gibi bağımsız bir container section'ı hesapla.
/// Sayfa genişliğinde, auto yükseklikte layout yapar.
fn compute_section(
    container: &ContainerElement,
    page_w_pt: f32,
    page_width_mm: f64,
    resolved: &ResolvedData,
    measurer: &mut TextMeasurer,
) -> Result<(Vec<ElementLayout>, f64), LayoutError> {
    let mut taffy = TaffyTree::<MeasureContext>::new();
    taffy.disable_rounding();
    let mut node_map: HashMap<NodeId, NodeInfo> = HashMap::new();
    let mut table_cache = TableExpandCache::new();

    let section_node = build_container(container, &mut taffy, &mut node_map, resolved, None, measurer, page_width_mm, &mut table_cache)?;

    let wrapper_style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        size: Size {
            width: Dimension::length(page_w_pt),
            height: Dimension::auto(),
        },
        ..Default::default()
    };
    let wrapper_node = taffy.new_with_children(wrapper_style, &[section_node])?;

    taffy
        .compute_layout_with_measure(
            wrapper_node,
            Size {
                width: AvailableSpace::Definite(page_w_pt),
                height: AvailableSpace::MaxContent,
            },
            |known_dimensions, available_space, _node_id, context, _style| {
                measure_leaf(known_dimensions, available_space, context, measurer)
            },
        )?;

    let elements = collect_layout(&taffy, section_node, &node_map, resolved, 0.0, 0.0)?;

    // Section yüksekliği
    let section_layout = taffy.layout(section_node)?;
    let height_mm = pt_to_mm(section_layout.size.height);

    Ok((elements, height_mm))
}

/// Template ağacındaki tüm container'ların break_inside modlarını topla.
fn collect_break_modes(root: &ContainerElement) -> HashMap<String, String> {
    let mut modes = HashMap::new();
    collect_break_modes_recursive(&TemplateElement::Container(root.clone()), &mut modes);
    modes
}

fn collect_break_modes_recursive(el: &TemplateElement, modes: &mut HashMap<String, String>) {
    if let TemplateElement::Container(c) = el {
        modes.insert(c.id.clone(), c.break_inside.clone());
        for child in &c.children {
            collect_break_modes_recursive(child, modes);
        }
    }
}

/// repeat_header == false olan tablo ID'lerini topla.
fn collect_no_repeat_header_tables(root: &ContainerElement) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    collect_no_repeat_recursive(&TemplateElement::Container(root.clone()), &mut set);
    set
}

fn collect_no_repeat_recursive(el: &TemplateElement, set: &mut std::collections::HashSet<String>) {
    match el {
        TemplateElement::Container(c) => {
            for child in &c.children {
                collect_no_repeat_recursive(child, set);
            }
        }
        TemplateElement::RepeatingTable(t) => {
            if t.repeat_header == Some(false) {
                set.insert(t.id.clone());
            }
        }
        _ => {}
    }
}

/// Container element'ini taffy node ağacına ekle (recursive)
fn build_container(
    el: &ContainerElement,
    taffy: &mut TaffyTree<MeasureContext>,
    node_map: &mut HashMap<NodeId, NodeInfo>,
    resolved: &ResolvedData,
    parent_direction: Option<&str>,
    measurer: &mut TextMeasurer,
    page_width_mm: f64,
    table_cache: &mut TableExpandCache,
) -> Result<NodeId, LayoutError> {
    let style = sizing::container_to_style(el, parent_direction);
    let direction = el.direction.as_str();

    // Child'lar için kullanılabilir genişliği hesapla
    // Container'ın kendi padding ve border'ını çıkar
    let border_w = el.style.border_width.unwrap_or(0.0);
    let container_own_width = match &el.size.width {
        SizeValue::Fixed { value } => *value,
        _ => page_width_mm, // Fr veya Auto ise parent'ın genişliğini kullan
    };
    let content_width_mm = container_own_width - el.padding.left - el.padding.right - border_w * 2.0;
    let content_width_mm = content_width_mm.max(0.0);

    let mut child_nodes = Vec::new();
    let mut children_ids = Vec::new();

    for child in &el.children {
        let child_node = build_element(child, taffy, node_map, resolved, Some(direction), measurer, content_width_mm, table_cache)?;
        child_nodes.push(child_node);
        children_ids.push(child.id().to_string());
    }

    let node = taffy.new_with_children(style, &child_nodes)?;

    node_map.insert(
        node,
        NodeInfo {
            element_id: el.id.clone(),
            element_type: "container".to_string(),
            content: None,
            style: ResolvedStyle {
                background_color: el.style.background_color.clone(),
                border_color: el.style.border_color.clone(),
                border_width: el.style.border_width,
                border_radius: el.style.border_radius,
                border_style: el.style.border_style.clone(),
                ..Default::default()
            },
            children_ids,
        },
    );

    Ok(node)
}

/// Herhangi bir element tipini taffy node'a çevir
fn build_element(
    el: &TemplateElement,
    taffy: &mut TaffyTree<MeasureContext>,
    node_map: &mut HashMap<NodeId, NodeInfo>,
    resolved: &ResolvedData,
    parent_direction: Option<&str>,
    measurer: &mut TextMeasurer,
    page_width_mm: f64,
    table_cache: &mut TableExpandCache,
) -> Result<NodeId, LayoutError> {
    match el {
        TemplateElement::Container(e) => {
            build_container(e, taffy, node_map, resolved, parent_direction, measurer, page_width_mm, table_cache)
        }
        TemplateElement::StaticText(e) => build_text_leaf(
            taffy,
            node_map,
            &e.id,
            "static_text",
            resolved
                .texts
                .get(&e.id)
                .map(|s| s.as_str())
                .unwrap_or(&e.content),
            &e.style,
            &e.size,
            &e.position,
            parent_direction,
        ),
        TemplateElement::Text(e) => {
            let text = resolved.texts.get(&e.id).map(|s| s.as_str()).unwrap_or("");
            build_text_leaf(
                taffy,
                node_map,
                &e.id,
                "text",
                text,
                &e.style,
                &e.size,
                &e.position,
                parent_direction,
            )
        }
        TemplateElement::PageNumber(e) => {
            let text = resolved
                .texts
                .get(&e.id)
                .map(|s| s.as_str())
                .unwrap_or("1 / 1");
            build_text_leaf(
                taffy,
                node_map,
                &e.id,
                "page_number",
                text,
                &e.style,
                &e.size,
                &e.position,
                parent_direction,
            )
        }
        TemplateElement::CurrentDate(e) => {
            let text = resolved
                .texts
                .get(&e.id)
                .map(|s| s.as_str())
                .unwrap_or("");
            build_text_leaf(
                taffy,
                node_map,
                &e.id,
                "current_date",
                text,
                &e.style,
                &e.size,
                &e.position,
                parent_direction,
            )
        }
        TemplateElement::CalculatedText(e) => {
            let text = resolved
                .texts
                .get(&e.id)
                .map(|s| s.as_str())
                .unwrap_or("");
            build_text_leaf(
                taffy,
                node_map,
                &e.id,
                "calculated_text",
                text,
                &e.style,
                &e.size,
                &e.position,
                parent_direction,
            )
        }
        TemplateElement::Line(e) => {
            let stroke_w = e.style.stroke_width.unwrap_or(0.5);
            let style = sizing::leaf_style(&e.size, &e.position, parent_direction);

            // Line: genişlik parent'tan, yükseklik stroke width
            let mut leaf_style = style;
            if matches!(e.size.height, SizeValue::Auto) {
                leaf_style.size.height = Dimension::length(mm_to_pt(stroke_w));
            }

            let node = taffy.new_leaf(leaf_style)?;
            node_map.insert(
                node,
                NodeInfo {
                    element_id: e.id.clone(),
                    element_type: "line".to_string(),
                    content: Some(ResolvedContent::Line),
                    style: ResolvedStyle {
                        stroke_color: e.style.stroke_color.clone(),
                        stroke_width: Some(stroke_w),
                        ..Default::default()
                    },
                    children_ids: vec![],
                },
            );
            Ok(node)
        }
        TemplateElement::Image(e) => {
            let style = sizing::leaf_style(&e.size, &e.position, parent_direction);
            let src = resolved.images.get(&e.id).cloned().unwrap_or_default();

            let node = taffy.new_leaf(style)?;
            node_map.insert(
                node,
                NodeInfo {
                    element_id: e.id.clone(),
                    element_type: "image".to_string(),
                    content: Some(ResolvedContent::Image { src }),
                    style: ResolvedStyle {
                        object_fit: e.style.object_fit.clone(),
                        ..Default::default()
                    },
                    children_ids: vec![],
                },
            );
            Ok(node)
        }
        TemplateElement::Barcode(e) => {
            let mut style = sizing::leaf_style(&e.size, &e.position, parent_direction);
            let value = resolved.barcodes.get(&e.id).cloned().unwrap_or_default();

            // Barcode leaf'e minimum boyut ver (MeasureFunc yok, Auto=0 olur)
            let is_qr = e.format == "qr";
            let default_h = if is_qr { 20.0 } else { 15.0 }; // mm
            let default_w = if is_qr { 20.0 } else { 40.0 }; // mm
            if matches!(e.size.height, SizeValue::Auto) {
                style.min_size.height = Dimension::length(mm_to_pt(default_h));
            }
            if matches!(e.size.width, SizeValue::Auto) {
                style.min_size.width = Dimension::length(mm_to_pt(default_w));
            }

            let node = taffy.new_leaf(style)?;
            node_map.insert(
                node,
                NodeInfo {
                    element_id: e.id.clone(),
                    element_type: "barcode".to_string(),
                    content: Some(ResolvedContent::Barcode {
                        format: e.format.clone(),
                        value,
                    }),
                    style: ResolvedStyle {
                        barcode_color: e.style.color.clone(),
                        barcode_include_text: e.style.include_text,
                        ..Default::default()
                    },
                    children_ids: vec![],
                },
            );
            Ok(node)
        }
        TemplateElement::RepeatingTable(e) => {
            // Tabloyu container ağacına expand et (cache ile)
            let expanded = table_layout::expand_table_cached(e, resolved, measurer, page_width_mm, table_cache);

            // Expand edilmiş tablo cell'lerinin text'lerini resolved'a ekle
            // (expand_table StaticText'ler üretir, bunların text'leri zaten content'te)
            let mut table_resolved = resolved.clone();
            register_expanded_texts(
                &TemplateElement::Container(expanded.clone()),
                &mut table_resolved,
            );

            // Container olarak build et
            build_container(
                &expanded,
                taffy,
                node_map,
                &table_resolved,
                parent_direction,
                measurer,
                page_width_mm,
                table_cache,
            )
        }
        TemplateElement::Shape(e) => {
            let style = sizing::leaf_style(&e.size, &e.position, parent_direction);
            let node = taffy.new_leaf(style)?;
            node_map.insert(
                node,
                NodeInfo {
                    element_id: e.id.clone(),
                    element_type: "shape".to_string(),
                    content: Some(ResolvedContent::Shape {
                        shape_type: e.shape_type.clone(),
                    }),
                    style: ResolvedStyle {
                        background_color: e.style.background_color.clone(),
                        border_color: e.style.border_color.clone(),
                        border_width: e.style.border_width,
                        border_radius: e.style.border_radius,
                        ..Default::default()
                    },
                    children_ids: vec![],
                },
            );
            Ok(node)
        }
        TemplateElement::Checkbox(e) => {
            let checked_str = resolved.texts.get(&e.id).map(|s| s.as_str()).unwrap_or("false");
            let checked = checked_str == "true";
            let box_size_mm = e.style.size.unwrap_or(4.0);
            let style = sizing::leaf_style(&e.size, &e.position, parent_direction);

            // Auto size → square based on style.size
            let mut leaf_style = style;
            if matches!(e.size.width, SizeValue::Auto) {
                leaf_style.size.width = Dimension::length(mm_to_pt(box_size_mm));
            }
            if matches!(e.size.height, SizeValue::Auto) {
                leaf_style.size.height = Dimension::length(mm_to_pt(box_size_mm));
            }

            let node = taffy.new_leaf(leaf_style)?;
            node_map.insert(
                node,
                NodeInfo {
                    element_id: e.id.clone(),
                    element_type: "checkbox".to_string(),
                    content: Some(ResolvedContent::Checkbox { checked }),
                    style: ResolvedStyle {
                        color: e.style.check_color.clone(),
                        border_color: e.style.border_color.clone(),
                        border_width: e.style.border_width,
                        ..Default::default()
                    },
                    children_ids: vec![],
                },
            );
            Ok(node)
        }
        TemplateElement::RichText(e) => {
            let spans = resolved.rich_texts.get(&e.id).cloned().unwrap_or_default();
            let rich_span_measures: Vec<crate::text_measure::RichSpanMeasure> = spans
                .iter()
                .map(|s| crate::text_measure::RichSpanMeasure {
                    text: s.text.clone(),
                    font_family: s.font_family.clone(),
                    font_size_pt: s.font_size.unwrap_or(11.0) as f32,
                    font_weight: s.font_weight.clone(),
                })
                .collect();

            let max_font_size_pt = rich_span_measures
                .iter()
                .map(|s| s.font_size_pt)
                .fold(11.0f32, f32::max);

            let style = sizing::leaf_style(&e.size, &e.position, parent_direction);

            let context = MeasureContext {
                text: String::new(),
                font_family: None,
                font_size_pt: max_font_size_pt,
                font_weight: None,
                rich_spans: Some(rich_span_measures),
            };

            let node = taffy.new_leaf_with_context(style, context)?;

            // ResolvedContent::RichText span'ları oluştur
            let resolved_spans: Vec<crate::ResolvedRichSpan> = spans
                .iter()
                .map(|s| crate::ResolvedRichSpan {
                    text: s.text.clone(),
                    font_size: s.font_size,
                    font_weight: s.font_weight.clone(),
                    font_family: s.font_family.clone(),
                    color: s.color.clone(),
                })
                .collect();

            node_map.insert(
                node,
                NodeInfo {
                    element_id: e.id.clone(),
                    element_type: "rich_text".to_string(),
                    content: Some(ResolvedContent::RichText { spans: resolved_spans }),
                    style: ResolvedStyle {
                        font_size: e.style.font_size,
                        font_weight: e.style.font_weight.clone(),
                        font_family: e.style.font_family.clone(),
                        color: e.style.color.clone(),
                        text_align: e.style.align.clone(),
                        ..Default::default()
                    },
                    children_ids: vec![],
                },
            );
            Ok(node)
        }
        TemplateElement::Chart(e) => {
            let mut style = sizing::leaf_style(&e.size, &e.position, parent_direction);
            // Default minimum boyut — Auto ise chart cok kucuk olmasin
            if matches!(e.size.width, SizeValue::Auto) {
                style.min_size.width = Dimension::length(mm_to_pt(80.0));
            }
            if matches!(e.size.height, SizeValue::Auto) {
                style.min_size.height = Dimension::length(mm_to_pt(60.0));
            }
            let node = taffy.new_leaf(style)?;
            node_map.insert(
                node,
                NodeInfo {
                    element_id: e.id.clone(),
                    element_type: "chart".to_string(),
                    content: None, // SVG collect_layout'ta uretilecek
                    style: ResolvedStyle::default(),
                    children_ids: vec![],
                },
            );
            Ok(node)
        }
        TemplateElement::PageBreak(e) => {
            // Küçük yükseklik — editörde görünür olması için (0.5mm ≈ 1.4pt)
            let style = Style {
                size: Size {
                    width: Dimension::auto(),
                    height: Dimension::length(mm_to_pt(0.5)),
                },
                ..Default::default()
            };
            let node = taffy.new_leaf(style)?;
            node_map.insert(
                node,
                NodeInfo {
                    element_id: e.id.clone(),
                    element_type: "page_break".to_string(),
                    content: None,
                    style: ResolvedStyle::default(),
                    children_ids: vec![],
                },
            );
            Ok(node)
        }
    }
}

/// Expand edilmiş tablo cell'lerinin text'lerini ResolvedData'ya kaydet
fn register_expanded_texts(el: &TemplateElement, resolved: &mut ResolvedData) {
    match el {
        TemplateElement::StaticText(e) => {
            resolved.texts.insert(e.id.clone(), e.content.clone());
        }
        TemplateElement::Container(e) => {
            for child in &e.children {
                register_expanded_texts(child, resolved);
            }
        }
        _ => {}
    }
}

/// Text leaf node oluştur (static_text, text, page_number için ortak)
fn build_text_leaf(
    taffy: &mut TaffyTree<MeasureContext>,
    node_map: &mut HashMap<NodeId, NodeInfo>,
    id: &str,
    element_type: &str,
    text: &str,
    text_style: &TextStyle,
    size: &SizeConstraint,
    position: &PositionMode,
    parent_direction: Option<&str>,
) -> Result<NodeId, LayoutError> {
    let style = sizing::leaf_style(size, position, parent_direction);
    let font_size_pt = text_style.font_size.unwrap_or(11.0) as f32;

    let context = MeasureContext {
        text: text.to_string(),
        font_family: text_style.font_family.clone(),
        font_size_pt,
        font_weight: text_style.font_weight.clone(),
        rich_spans: None,
    };

    let node = taffy.new_leaf_with_context(style, context)?;

    node_map.insert(
        node,
        NodeInfo {
            element_id: id.to_string(),
            element_type: element_type.to_string(),
            content: Some(ResolvedContent::Text {
                value: text.to_string(),
            }),
            style: ResolvedStyle {
                font_size: text_style.font_size,
                font_weight: text_style.font_weight.clone(),
                font_style: text_style.font_style.clone(),
                font_family: text_style.font_family.clone(),
                color: text_style.color.clone(),
                text_align: text_style.align.clone(),
                ..Default::default()
            },
            children_ids: vec![],
        },
    );

    Ok(node)
}

/// Taffy MeasureFunc: text leaf node'ları ölç
fn measure_leaf(
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    context: Option<&mut MeasureContext>,
    measurer: &mut TextMeasurer,
) -> Size<f32> {
    let Some(ctx) = context else {
        // Context yoksa (line, image vs.) → taffy style'daki boyutu kullan
        return Size {
            width: known_dimensions.width.unwrap_or(0.0),
            height: known_dimensions.height.unwrap_or(0.0),
        };
    };

    // Bilinen boyutlar varsa onları kullan
    if let (Some(w), Some(h)) = (known_dimensions.width, known_dimensions.height) {
        return Size {
            width: w,
            height: h,
        };
    }

    let available_width = match available_space.width {
        AvailableSpace::Definite(w) => Some(w),
        AvailableSpace::MaxContent => None,
        AvailableSpace::MinContent => Some(0.0),
    };

    let (measured_w, measured_h) = if let Some(ref rich_spans) = ctx.rich_spans {
        measurer.measure_rich_text(rich_spans, available_width)
    } else {
        measurer.measure(
            &ctx.text,
            ctx.font_family.as_deref(),
            ctx.font_size_pt,
            ctx.font_weight.as_deref(),
            available_width,
        )
    };

    Size {
        width: known_dimensions.width.unwrap_or(measured_w),
        height: known_dimensions.height.unwrap_or(measured_h),
    }
}

/// Taffy layout sonuçlarını ElementLayout listesine dönüştür (recursive).
/// Pozisyon biriktirmesi f64 (mm) cinsinde yapılır — f32'de toplama hassasiyet kaybına yol açar.
fn collect_layout(
    taffy: &TaffyTree<MeasureContext>,
    node: NodeId,
    node_map: &HashMap<NodeId, NodeInfo>,
    resolved: &ResolvedData,
    parent_x_mm: f64,
    parent_y_mm: f64,
) -> Result<Vec<ElementLayout>, LayoutError> {
    let mut elements = Vec::new();

    let Some(info) = node_map.get(&node) else {
        return Ok(elements);
    };

    let layout = taffy.layout(node)?;
    let x_mm = parent_x_mm + pt_to_mm(layout.location.x);
    let y_mm = parent_y_mm + pt_to_mm(layout.location.y);
    let w_mm = pt_to_mm(layout.size.width);
    let h_mm = pt_to_mm(layout.size.height);

    // Chart elementleri icin SVG uret (boyutlar artik belli)
    let content = if info.element_type == "chart" {
        resolved.charts.get(&info.element_id).map(|cd| {
            use crate::{ChartRenderData, ChartSeriesData};
            use crate::chart_layout::DEFAULT_COLORS;

            // Renk paleti olustur
            let n_colors = cd.categories.len().max(cd.series.len()).max(1);
            let colors: Vec<String> = (0..n_colors)
                .map(|i| {
                    cd.style.colors.as_ref()
                        .and_then(|c| c.get(i).cloned())
                        .unwrap_or_else(|| DEFAULT_COLORS[i % DEFAULT_COLORS.len()].to_string())
                })
                .collect();

            ResolvedContent::Chart {
                svg: crate::chart_render::render_svg(cd, w_mm, h_mm),
                chart_data: ChartRenderData {
                    chart_type: cd.chart_type.clone(),
                    categories: cd.categories.clone(),
                    series: cd.series.iter().map(|s| ChartSeriesData {
                        name: s.name.clone(),
                        values: s.values.clone(),
                    }).collect(),
                    title_text: cd.title.as_ref().map(|t| t.text.clone()),
                    title_font_size: cd.title.as_ref().and_then(|t| t.font_size),
                    title_color: cd.title.as_ref().and_then(|t| t.color.clone()),
                    colors,
                    show_labels: cd.labels.as_ref().is_some_and(|l| l.show),
                    label_font_size: cd.labels.as_ref().and_then(|l| l.font_size),
                    show_grid: cd.axis.as_ref().and_then(|a| a.show_grid).unwrap_or(true),
                    grid_color: cd.axis.as_ref().and_then(|a| a.grid_color.clone()),
                    bar_gap: cd.style.bar_gap,
                    stacked: matches!(cd.group_mode, Some(dreport_core::models::GroupMode::Stacked)),
                    inner_radius: cd.style.inner_radius,
                    show_points: cd.style.show_points,
                    line_width: cd.style.line_width,
                    background_color: cd.style.background_color.clone(),
                    label_color: cd.labels.as_ref().and_then(|l| l.color.clone()),
                    legend_show: cd.legend.as_ref().is_some_and(|l| l.show),
                    legend_position: cd.legend.as_ref().and_then(|l| l.position.clone()),
                    legend_font_size: cd.legend.as_ref().and_then(|l| l.font_size),
                    x_label: cd.axis.as_ref().and_then(|a| a.x_label.clone()),
                    y_label: cd.axis.as_ref().and_then(|a| a.y_label.clone()),
                    title_align: cd.title.as_ref().and_then(|t| t.align.clone()),
                },
            }
        })
    } else {
        info.content.clone()
    };

    elements.push(ElementLayout {
        id: info.element_id.clone(),
        x_mm,
        y_mm,
        width_mm: w_mm,
        height_mm: h_mm,
        element_type: info.element_type.clone(),
        content,
        style: info.style.clone(),
        children: info.children_ids.clone(),
    });

    // Child node'ları da topla
    let children = taffy.children(node)?;
    for child_node in children {
        let child_elements = collect_layout(taffy, child_node, node_map, resolved, x_mm, y_mm)?;
        elements.extend(child_elements);
    }

    Ok(elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn simple_template() -> Template {
        Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            page: PageSettings {
                width: 210.0,
                height: 297.0,
            },
            fonts: vec!["Noto Sans".to_string()],
            header: None,
            footer: None,
            format_config: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Auto,
                    height: SizeValue::Auto,
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                },
                direction: "column".to_string(),
                gap: 5.0,
                padding: Padding {
                    top: 15.0,
                    right: 15.0,
                    bottom: 15.0,
                    left: 15.0,
                },
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    TemplateElement::StaticText(StaticTextElement {
                        id: "title".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None,
                            min_height: None,
                            max_width: None,
                            max_height: None,
                        },
                        style: TextStyle {
                            font_size: Some(18.0),
                            font_weight: Some("bold".to_string()),
                            ..Default::default()
                        },
                        content: "FATURA".to_string(),
                    }),
                    TemplateElement::Line(LineElement {
                        id: "line1".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None,
                            min_height: None,
                            max_width: None,
                            max_height: None,
                        },
                        style: LineStyle {
                            stroke_color: Some("#000000".to_string()),
                            stroke_width: Some(0.5),
                        },
                    }),
                    TemplateElement::StaticText(StaticTextElement {
                        id: "body".to_string(),
                        position: PositionMode::Flow,
                        size: SizeConstraint {
                            width: SizeValue::Fr { value: 1.0 },
                            height: SizeValue::Auto,
                            min_width: None,
                            min_height: None,
                            max_width: None,
                            max_height: None,
                        },
                        style: TextStyle {
                            font_size: Some(11.0),
                            ..Default::default()
                        },
                        content: "Bu bir test belgesidir.".to_string(),
                    }),
                ],
            },
        }
    }

    #[test]
    fn test_basic_layout() {
        let template = simple_template();
        let data = json!({});
        let resolved = crate::data_resolve::resolve_template(&template, &data);
        let fonts = crate::text_measure::load_test_fonts();
        let mut measurer = TextMeasurer::new(&fonts);

        let result = compute(&template, &resolved, &mut measurer).unwrap();

        assert_eq!(result.pages.len(), 1);
        let page = &result.pages[0];
        assert_eq!(page.width_mm, 210.0);
        assert_eq!(page.height_mm, 297.0);

        println!("Layout sonuçları:");
        for el in &page.elements {
            println!(
                "  {} ({}): x={:.1}mm y={:.1}mm w={:.1}mm h={:.1}mm",
                el.id, el.element_type, el.x_mm, el.y_mm, el.width_mm, el.height_mm
            );
        }

        // Root container + 3 children = en az 4 element
        assert!(page.elements.len() >= 4);

        // Root container pozisyonu: (0, 0)
        let root = page.elements.iter().find(|e| e.id == "root").unwrap();
        assert!(root.x_mm.abs() < 0.1);
        assert!(root.y_mm.abs() < 0.1);

        // Title: padding'in içinde olmalı (x ≈ 15mm, y ≈ 15mm)
        let title = page.elements.iter().find(|e| e.id == "title").unwrap();
        assert!((title.x_mm - 15.0).abs() < 1.0);
        assert!((title.y_mm - 15.0).abs() < 1.0);
        assert!(
            title.width_mm > 100.0,
            "title width={:.1}mm, expected > 100mm",
            title.width_mm
        );

        // Line: title'dan sonra (y > title.y)
        let line = page.elements.iter().find(|e| e.id == "line1").unwrap();
        assert!(line.y_mm > title.y_mm);
        assert!(line.height_mm < 2.0);

        // Body: line'dan sonra
        let body = page.elements.iter().find(|e| e.id == "body").unwrap();
        assert!(body.y_mm > line.y_mm);
    }

    #[test]
    fn test_row_container() {
        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            page: PageSettings {
                width: 210.0,
                height: 297.0,
            },
            fonts: vec![],
            header: None,
            footer: None,
            format_config: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Auto,
                    height: SizeValue::Auto,
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                },
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding {
                    top: 10.0,
                    right: 10.0,
                    bottom: 10.0,
                    left: 10.0,
                },
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![TemplateElement::Container(ContainerElement {
                    id: "row".to_string(),
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: SizeValue::Fr { value: 1.0 },
                        height: SizeValue::Auto,
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                    },
                    direction: "row".to_string(),
                    gap: 5.0,
                    padding: Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    },
                    align: "start".to_string(),
                    justify: "start".to_string(),
                    style: ContainerStyle::default(),
                    break_inside: "auto".to_string(),
                    children: vec![
                        TemplateElement::StaticText(StaticTextElement {
                            id: "left".to_string(),
                            position: PositionMode::Flow,
                            size: SizeConstraint {
                                width: SizeValue::Fr { value: 1.0 },
                                height: SizeValue::Auto,
                                min_width: None,
                                min_height: None,
                                max_width: None,
                                max_height: None,
                            },
                            style: TextStyle {
                                font_size: Some(11.0),
                                ..Default::default()
                            },
                            content: "Sol".to_string(),
                        }),
                        TemplateElement::StaticText(StaticTextElement {
                            id: "right".to_string(),
                            position: PositionMode::Flow,
                            size: SizeConstraint {
                                width: SizeValue::Fr { value: 1.0 },
                                height: SizeValue::Auto,
                                min_width: None,
                                min_height: None,
                                max_width: None,
                                max_height: None,
                            },
                            style: TextStyle {
                                font_size: Some(11.0),
                                ..Default::default()
                            },
                            content: "Sağ".to_string(),
                        }),
                    ],
                })],
            },
        };

        let data = json!({});
        let resolved = crate::data_resolve::resolve_template(&template, &data);
        let fonts = crate::text_measure::load_test_fonts();
        let mut measurer = TextMeasurer::new(&fonts);
        let result = compute(&template, &resolved, &mut measurer).unwrap();

        let page = &result.pages[0];
        let left = page.elements.iter().find(|e| e.id == "left").unwrap();
        let right = page.elements.iter().find(|e| e.id == "right").unwrap();

        // İki eleman eşit genişlikte olmalı (fr: 1 + fr: 1)
        assert!((left.width_mm - right.width_mm).abs() < 1.0);

        // Right, left'in sağında olmalı
        assert!(right.x_mm > left.x_mm + left.width_mm - 1.0);

        // İkisinin toplam genişliği ≈ 190mm (210 - 10 - 10 padding, - 5mm gap)
        let total = left.width_mm + right.width_mm + 5.0; // gap
        assert!((total - 190.0).abs() < 2.0);

        println!("Row layout:");
        for el in &page.elements {
            println!(
                "  {} ({}): x={:.1}mm y={:.1}mm w={:.1}mm h={:.1}mm",
                el.id, el.element_type, el.x_mm, el.y_mm, el.width_mm, el.height_mm
            );
        }
    }

    #[test]
    fn test_absolute_positioning() {
        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            page: PageSettings {
                width: 210.0,
                height: 297.0,
            },
            fonts: vec![],
            header: None,
            footer: None,
            format_config: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: SizeValue::Auto,
                    height: SizeValue::Auto,
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                },
                direction: "column".to_string(),
                gap: 0.0,
                padding: Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                },
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![TemplateElement::StaticText(StaticTextElement {
                    id: "abs_text".to_string(),
                    position: PositionMode::Absolute { x: 50.0, y: 80.0 },
                    size: SizeConstraint {
                        width: SizeValue::Fixed { value: 100.0 },
                        height: SizeValue::Auto,
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                    },
                    style: TextStyle {
                        font_size: Some(14.0),
                        ..Default::default()
                    },
                    content: "Absolute".to_string(),
                })],
            },
        };

        let data = json!({});
        let resolved = crate::data_resolve::resolve_template(&template, &data);
        let fonts = crate::text_measure::load_test_fonts();
        let mut measurer = TextMeasurer::new(&fonts);
        let result = compute(&template, &resolved, &mut measurer).unwrap();

        let page = &result.pages[0];
        let abs = page.elements.iter().find(|e| e.id == "abs_text").unwrap();

        // Absolute pozisyon: x ≈ 50mm, y ≈ 80mm
        assert!((abs.x_mm - 50.0).abs() < 1.0);
        assert!((abs.y_mm - 80.0).abs() < 1.0);
        assert!((abs.width_mm - 100.0).abs() < 1.0);

        println!(
            "Absolute: x={:.1}mm y={:.1}mm w={:.1}mm h={:.1}mm",
            abs.x_mm, abs.y_mm, abs.width_mm, abs.height_mm
        );
    }

    #[test]
    fn test_invoice_header_overflow() {
        // App.vue'daki fatura şablonunun header kısmını birebir test et
        let sz_auto = SizeConstraint {
            width: SizeValue::Auto,
            height: SizeValue::Auto,
            ..Default::default()
        };
        let sz_fr_auto = SizeConstraint {
            width: SizeValue::Fr { value: 1.0 },
            height: SizeValue::Auto,
            ..Default::default()
        };
        let p0 = Padding::default();

        let template = Template {
            id: "test".to_string(),
            name: "Test".to_string(),
            page: PageSettings {
                width: 210.0,
                height: 297.0,
            },
            fonts: vec!["Noto Sans".to_string()],
            header: None,
            footer: None,
            format_config: None,
            root: ContainerElement {
                id: "root".to_string(),
                position: PositionMode::Flow,
                size: sz_auto.clone(),
                direction: "column".to_string(),
                gap: 5.0,
                padding: Padding {
                    top: 15.0,
                    right: 15.0,
                    bottom: 15.0,
                    left: 15.0,
                },
                align: "stretch".to_string(),
                justify: "start".to_string(),
                style: ContainerStyle::default(),
                break_inside: "auto".to_string(),
                children: vec![
                    // Header row
                    TemplateElement::Container(ContainerElement {
                        id: "c_header".to_string(),
                        position: PositionMode::Flow,
                        size: sz_fr_auto.clone(),
                        direction: "row".to_string(),
                        gap: 5.0,
                        padding: p0.clone(),
                        align: "start".to_string(),
                        justify: "space-between".to_string(),
                        style: ContainerStyle::default(),
                        break_inside: "auto".to_string(),
                        children: vec![
                            // Sol: firma bilgileri
                            TemplateElement::Container(ContainerElement {
                                id: "c_firma".to_string(),
                                position: PositionMode::Flow,
                                size: sz_fr_auto.clone(),
                                direction: "column".to_string(),
                                gap: 1.0,
                                padding: p0.clone(),
                                align: "start".to_string(),
                                justify: "start".to_string(),
                                style: ContainerStyle::default(),
                                break_inside: "auto".to_string(),
                                children: vec![
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_firma_unvan".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(14.0),
                                            font_weight: Some("bold".to_string()),
                                            ..Default::default()
                                        },
                                        content: "Teknova Yazılım ve Danışmanlık A.Ş.".to_string(),
                                    }),
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_firma_adres".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(9.0),
                                            ..Default::default()
                                        },
                                        content: "Levent Mah. Inovasyon Sk. No:42 Kat:5"
                                            .to_string(),
                                    }),
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_firma_il".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(9.0),
                                            ..Default::default()
                                        },
                                        content: "Istanbul".to_string(),
                                    }),
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_firma_tel".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(9.0),
                                            ..Default::default()
                                        },
                                        content: "Tel: +90 212 555 0042".to_string(),
                                    }),
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_firma_vd".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(9.0),
                                            ..Default::default()
                                        },
                                        content: "VD: Levent VD".to_string(),
                                    }),
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_firma_vn".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(9.0),
                                            ..Default::default()
                                        },
                                        content: "VN: 1234567890".to_string(),
                                    }),
                                ],
                            }),
                            // Sağ: fatura başlığı
                            TemplateElement::Container(ContainerElement {
                                id: "c_fatura_baslik".to_string(),
                                position: PositionMode::Flow,
                                size: sz_auto.clone(),
                                direction: "column".to_string(),
                                gap: 2.0,
                                padding: p0.clone(),
                                align: "end".to_string(),
                                justify: "start".to_string(),
                                style: ContainerStyle::default(),
                                break_inside: "auto".to_string(),
                                children: vec![
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_fatura_baslik".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(18.0),
                                            font_weight: Some("bold".to_string()),
                                            ..Default::default()
                                        },
                                        content: "FATURA".to_string(),
                                    }),
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_fatura_no".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(10.0),
                                            ..Default::default()
                                        },
                                        content: "No: FTR-2026-001547".to_string(),
                                    }),
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_fatura_tarih".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(10.0),
                                            ..Default::default()
                                        },
                                        content: "Tarih: 2026-03-29".to_string(),
                                    }),
                                    TemplateElement::StaticText(StaticTextElement {
                                        id: "el_fatura_vade".to_string(),
                                        position: PositionMode::Flow,
                                        size: sz_auto.clone(),
                                        style: TextStyle {
                                            font_size: Some(10.0),
                                            ..Default::default()
                                        },
                                        content: "Vade: 2026-04-28".to_string(),
                                    }),
                                ],
                            }),
                        ],
                    }),
                ],
            },
        };

        let data = json!({});
        let resolved = crate::data_resolve::resolve_template(&template, &data);
        let fonts = crate::text_measure::load_test_fonts();
        let mut measurer = TextMeasurer::new(&fonts);
        let result = compute(&template, &resolved, &mut measurer).unwrap();

        let page = &result.pages[0];
        println!("\n=== FATURA HEADER LAYOUT ===");
        for el in &page.elements {
            println!(
                "  {:20} ({:12}): x={:9.4}mm y={:9.4}mm w={:9.4}mm h={:9.4}mm",
                el.id, el.element_type, el.x_mm, el.y_mm, el.width_mm, el.height_mm
            );
        }

        // c_header sınırlarını kontrol et
        let header = page.elements.iter().find(|e| e.id == "c_header").unwrap();
        let fatura_baslik = page
            .elements
            .iter()
            .find(|e| e.id == "c_fatura_baslik")
            .unwrap();

        let header_right = header.x_mm + header.width_mm;
        let header_bottom = header.y_mm + header.height_mm;
        let fb_right = fatura_baslik.x_mm + fatura_baslik.width_mm;
        let fb_bottom = fatura_baslik.y_mm + fatura_baslik.height_mm;

        println!(
            "\n  c_header     sağ kenar: {:.1}mm, alt kenar: {:.1}mm",
            header_right, header_bottom
        );
        println!(
            "  c_fatura_baslik sağ kenar: {:.1}mm, alt kenar: {:.1}mm",
            fb_right, fb_bottom
        );
        println!("  Yatay taşma: {:.1}mm", fb_right - header_right);
        println!("  Dikey taşma: {:.1}mm", fb_bottom - header_bottom);

        // c_fatura_baslik, c_header'ın dışına taşmamalı
        assert!(
            fb_right <= header_right + 0.5,
            "c_fatura_baslik sağ kenarı ({:.1}mm) c_header sağ kenarını ({:.1}mm) aşıyor!",
            fb_right,
            header_right
        );
        assert!(
            fb_bottom <= header_bottom + 0.5,
            "c_fatura_baslik alt kenarı ({:.1}mm) c_header alt kenarını ({:.1}mm) aşıyor!",
            fb_bottom,
            header_bottom
        );
    }
}
