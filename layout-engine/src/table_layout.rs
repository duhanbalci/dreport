use dreport_core::models::*;

use crate::data_resolve::ResolvedData;

/// RepeatingTable element'ini bir container ağacına expand eder.
/// Tablo → column container (header row + data rows)
/// Her row → row container (cell'ler → static_text)
///
/// Bu sayede tablo, normal container layout'u ile hesaplanır.
pub fn expand_table(
    table: &RepeatingTableElement,
    resolved: &ResolvedData,
) -> ContainerElement {
    let resolved_table = resolved.tables.get(&table.id);
    let rows = resolved_table
        .map(|t| t.rows.as_slice())
        .unwrap_or(&[]);

    let mut children: Vec<TemplateElement> = Vec::new();

    // Header row
    let header_cells: Vec<TemplateElement> = table
        .columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            TemplateElement::StaticText(StaticTextElement {
                id: format!("{}_hdr_{}", table.id, i),
                position: PositionMode::Flow,
                size: SizeConstraint {
                    width: col.width.clone(),
                    height: SizeValue::Auto,
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                },
                style: TextStyle {
                    font_size: table.style.header_font_size.or(table.style.font_size),
                    font_weight: Some("bold".to_string()),
                    font_family: None,
                    color: table.style.header_color.clone(),
                    align: Some(col.align.clone()),
                },
                content: col.title.clone(),
            })
        })
        .collect();

    children.push(TemplateElement::Container(ContainerElement {
        id: format!("{}_header", table.id),
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
        gap: 0.0,
        padding: Padding {
            top: 1.0,
            right: 0.0,
            bottom: 1.0,
            left: 0.0,
        },
        align: "center".to_string(),
        justify: "start".to_string(),
        style: ContainerStyle {
            background_color: table.style.header_bg.clone(),
            ..Default::default()
        },
        children: header_cells,
    }));

    // Header altına ayırıcı çizgi
    if table.style.border_color.is_some() {
        children.push(TemplateElement::Line(LineElement {
            id: format!("{}_header_line", table.id),
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
                stroke_color: table.style.border_color.clone(),
                stroke_width: table.style.border_width,
            },
        }));
    }

    // Data rows
    for (row_idx, row_data) in rows.iter().enumerate() {
        let cells: Vec<TemplateElement> = table
            .columns
            .iter()
            .enumerate()
            .map(|(col_idx, col)| {
                let text = row_data
                    .get(col_idx)
                    .cloned()
                    .unwrap_or_default();

                TemplateElement::StaticText(StaticTextElement {
                    id: format!("{}_r{}c{}", table.id, row_idx, col_idx),
                    position: PositionMode::Flow,
                    size: SizeConstraint {
                        width: col.width.clone(),
                        height: SizeValue::Auto,
                        min_width: None,
                        min_height: None,
                        max_width: None,
                        max_height: None,
                    },
                    style: TextStyle {
                        font_size: table.style.font_size,
                        font_weight: None,
                        font_family: None,
                        color: None,
                        align: Some(col.align.clone()),
                    },
                    content: text,
                })
            })
            .collect();

        // row_idx 0-based: 0. satır görsel olarak 1. (tek/odd), 1. satır 2. (çift/even)
        let bg = if row_idx % 2 == 0 {
            table.style.zebra_odd.clone()
        } else {
            table.style.zebra_even.clone()
        };

        children.push(TemplateElement::Container(ContainerElement {
            id: format!("{}_row_{}", table.id, row_idx),
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
            gap: 0.0,
            padding: Padding {
                top: 0.5,
                right: 0.0,
                bottom: 0.5,
                left: 0.0,
            },
            align: "center".to_string(),
            justify: "start".to_string(),
            style: ContainerStyle {
                background_color: bg,
                ..Default::default()
            },
            children: cells,
        }));
    }

    // Wrapper container (column direction, tüm tablo)
    ContainerElement {
        id: table.id.clone(),
        position: table.position.clone(),
        size: table.size.clone(),
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
        style: ContainerStyle {
            border_color: table.style.border_color.clone(),
            border_width: table.style.border_width,
            ..Default::default()
        },
        children,
    }
}
