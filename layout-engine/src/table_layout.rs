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
        break_inside: "auto".to_string(),
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
            break_inside: "auto".to_string(),
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
        break_inside: "auto".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_resolve::{ResolvedData, ResolvedTable};
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
            id: "tbl".to_string(),
            position: PositionMode::Flow,
            size: SizeConstraint {
                width: SizeValue::Fr { value: 1.0 },
                height: SizeValue::Auto,
                ..Default::default()
            },
            data_source: ArrayBinding { path: "items".to_string() },
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
        }
    }

    #[test]
    fn test_expand_table_structure() {
        let table = make_table(2);
        let resolved = make_resolved("tbl", vec![
            vec!["A".to_string(), "1".to_string()],
            vec!["B".to_string(), "2".to_string()],
        ]);

        let container = expand_table(&table, &resolved);

        // Wrapper container properties
        assert_eq!(container.id, "tbl");
        assert_eq!(container.direction, "column");

        // Children: header row + 2 data rows (no border_color so no separator line)
        assert_eq!(container.children.len(), 3);

        // First child is header row container
        match &container.children[0] {
            TemplateElement::Container(c) => {
                assert_eq!(c.id, "tbl_header");
                assert_eq!(c.direction, "row");
                assert_eq!(c.children.len(), 2); // 2 columns
                // Check header cell text
                match &c.children[0] {
                    TemplateElement::StaticText(t) => assert_eq!(t.content, "Column 0"),
                    _ => panic!("Expected StaticText for header cell"),
                }
            }
            _ => panic!("Expected Container for header row"),
        }

        // Data rows
        for (row_idx, child) in container.children[1..].iter().enumerate() {
            match child {
                TemplateElement::Container(c) => {
                    assert_eq!(c.id, format!("tbl_row_{}", row_idx));
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

        let container = expand_table(&table, &resolved);

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
        let resolved = make_resolved("tbl", vec![
            vec!["a".into(), "b".into(), "c".into(), "d".into()],
        ]);

        let container = expand_table(&table, &resolved);

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
        let resolved = make_resolved("tbl", vec![
            vec!["Hello".to_string(), "42".to_string()],
        ]);

        let container = expand_table(&table, &resolved);

        // Data row cells should contain the resolved text
        match &container.children[1] {
            TemplateElement::Container(c) => {
                match &c.children[0] {
                    TemplateElement::StaticText(t) => assert_eq!(t.content, "Hello"),
                    _ => panic!("Expected StaticText"),
                }
                match &c.children[1] {
                    TemplateElement::StaticText(t) => assert_eq!(t.content, "42"),
                    _ => panic!("Expected StaticText"),
                }
            }
            _ => panic!("Expected Container"),
        }
    }

    #[test]
    fn test_expand_table_with_border_adds_separator() {
        let mut table = make_table(2);
        table.style.border_color = Some("#000000".to_string());
        let resolved = make_resolved("tbl", vec![
            vec!["A".to_string(), "1".to_string()],
        ]);

        let container = expand_table(&table, &resolved);

        // header + separator line + 1 data row = 3
        assert_eq!(container.children.len(), 3);

        // Second child should be a Line
        match &container.children[1] {
            TemplateElement::Line(l) => {
                assert_eq!(l.id, "tbl_header_line");
            }
            _ => panic!("Expected Line separator after header"),
        }
    }

    #[test]
    fn test_expand_table_zebra_stripes() {
        let mut table = make_table(1);
        table.style.zebra_odd = Some("#f0f0f0".to_string());
        table.style.zebra_even = Some("#ffffff".to_string());
        let resolved = make_resolved("tbl", vec![
            vec!["row0".into()],
            vec!["row1".into()],
            vec!["row2".into()],
        ]);

        let container = expand_table(&table, &resolved);

        // header + 3 data rows
        assert_eq!(container.children.len(), 4);

        // row_0 (even index) => zebra_odd
        match &container.children[1] {
            TemplateElement::Container(c) => {
                assert_eq!(c.style.background_color, Some("#f0f0f0".to_string()));
            }
            _ => panic!("Expected Container"),
        }
        // row_1 (odd index) => zebra_even
        match &container.children[2] {
            TemplateElement::Container(c) => {
                assert_eq!(c.style.background_color, Some("#ffffff".to_string()));
            }
            _ => panic!("Expected Container"),
        }
        // row_2 (even index) => zebra_odd
        match &container.children[3] {
            TemplateElement::Container(c) => {
                assert_eq!(c.style.background_color, Some("#f0f0f0".to_string()));
            }
            _ => panic!("Expected Container"),
        }
    }
}
