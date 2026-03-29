use dreport_core::models::{ContainerElement, PositionMode, SizeConstraint, SizeValue};
use taffy::prelude::*;

/// 1mm = 72/25.4 pt (kesin değer)
const MM_TO_PT_F64: f64 = 72.0 / 25.4;

pub fn mm_to_pt(mm: f64) -> f32 {
    (mm * MM_TO_PT_F64) as f32
}

pub fn pt_to_mm(pt: f32) -> f64 {
    (pt as f64) / MM_TO_PT_F64
}

/// SizeValue → taffy Dimension (width veya height için)
fn size_value_to_dimension(sv: &SizeValue) -> Dimension {
    match sv {
        SizeValue::Fixed { value } => Dimension::Length(mm_to_pt(*value)),
        SizeValue::Auto => Dimension::Auto,
        // Fr için dimension Auto, flex_grow ayrıca set edilir
        SizeValue::Fr { .. } => Dimension::Auto,
    }
}

/// SizeValue → taffy LengthPercentage (min/max constraint'ler için)
fn mm_to_length(mm: f64) -> Dimension {
    Dimension::Length(mm_to_pt(mm))
}

/// Fr değerini döndür (yoksa 0)
fn fr_value(sv: &SizeValue) -> f32 {
    match sv {
        SizeValue::Fr { value } => *value as f32,
        _ => 0.0,
    }
}

/// SizeConstraint'ten taffy Style'a flex_grow ayarını da dahil ederek dönüştür.
/// `main_axis` parametresi parent container'ın direction'ına göre
/// hangi eksenin flex_grow kullanacağını belirler.
pub fn apply_size_to_style(
    style: &mut Style,
    size: &SizeConstraint,
    parent_direction: Option<&str>,
) {
    style.size = Size {
        width: size_value_to_dimension(&size.width),
        height: size_value_to_dimension(&size.height),
    };

    // Min/max constraint'ler
    if let Some(min_w) = size.min_width {
        style.min_size.width = mm_to_length(min_w);
    }
    if let Some(min_h) = size.min_height {
        style.min_size.height = mm_to_length(min_h);
    }
    if let Some(max_w) = size.max_width {
        style.max_size.width = mm_to_length(max_w);
    }
    if let Some(max_h) = size.max_height {
        style.max_size.height = mm_to_length(max_h);
    }

    // Fr → flex_grow (main axis'e göre)
    let main_fr = match parent_direction {
        Some("row") => fr_value(&size.width),
        Some("column") | _ => fr_value(&size.height),
    };

    // Cross axis fr: row'da height fr, column'da width fr
    let cross_fr = match parent_direction {
        Some("row") => fr_value(&size.height),
        Some("column") | _ => fr_value(&size.width),
    };

    // Eğer main axis fr ise, flex_grow ayarla ve flex_basis 0 yap
    if main_fr > 0.0 {
        style.flex_grow = main_fr;
        style.flex_shrink = 1.0;
        style.flex_basis = Dimension::Length(0.0);

        // min-width: 0 (row) veya min-height: 0 (column) ayarla —
        // taffy'de min_size default Auto = içerik boyutunun altına küçülemez.
        // Fr elemanların içerik taşırması engellemek için min_size 0 olmalı.
        match parent_direction {
            Some("row") => {
                if size.min_width.is_none() {
                    style.min_size.width = Dimension::Length(0.0);
                }
            }
            _ => {
                if size.min_height.is_none() {
                    style.min_size.height = Dimension::Length(0.0);
                }
            }
        }
    }

    // Cross axis fr ise, align_self stretch yeterli
    // (taffy'de cross axis flex_grow doğrudan yok, stretch ile çözülür)
    if cross_fr > 0.0 {
        style.align_self = Some(AlignSelf::Stretch);
    }
}

/// ContainerElement → taffy Style
pub fn container_to_style(el: &ContainerElement, parent_direction: Option<&str>) -> Style {
    let mut style = Style {
        display: Display::Flex,
        flex_direction: match el.direction.as_str() {
            "row" => FlexDirection::Row,
            _ => FlexDirection::Column,
        },
        gap: Size {
            width: LengthPercentage::Length(mm_to_pt(el.gap)),
            height: LengthPercentage::Length(mm_to_pt(el.gap)),
        },
        padding: Rect {
            top: LengthPercentage::Length(mm_to_pt(el.padding.top)),
            right: LengthPercentage::Length(mm_to_pt(el.padding.right)),
            bottom: LengthPercentage::Length(mm_to_pt(el.padding.bottom)),
            left: LengthPercentage::Length(mm_to_pt(el.padding.left)),
        },
        align_items: Some(match el.align.as_str() {
            "center" => AlignItems::Center,
            "end" => AlignItems::FlexEnd,
            "stretch" => AlignItems::Stretch,
            _ => AlignItems::FlexStart,
        }),
        justify_content: Some(match el.justify.as_str() {
            "center" => JustifyContent::Center,
            "end" => JustifyContent::FlexEnd,
            "space-between" => JustifyContent::SpaceBetween,
            _ => JustifyContent::FlexStart,
        }),
        ..Default::default()
    };

    // Pozisyon moduna göre
    match &el.position {
        PositionMode::Absolute { x, y } => {
            style.position = Position::Absolute;
            style.inset = Rect {
                top: LengthPercentageAuto::Length(mm_to_pt(*y)),
                left: LengthPercentageAuto::Length(mm_to_pt(*x)),
                right: auto(),
                bottom: auto(),
            };
        }
        PositionMode::Flow => {}
    }

    // Boyut
    apply_size_to_style(&mut style, &el.size, parent_direction);

    // Container border
    if let Some(bw) = el.style.border_width {
        let bpt = mm_to_pt(bw);
        style.border = Rect {
            top: LengthPercentage::Length(bpt),
            right: LengthPercentage::Length(bpt),
            bottom: LengthPercentage::Length(bpt),
            left: LengthPercentage::Length(bpt),
        };
    }

    style
}

/// Leaf element (text, line, image vs.) için taffy Style
pub fn leaf_style(
    size: &SizeConstraint,
    position: &PositionMode,
    parent_direction: Option<&str>,
) -> Style {
    let mut style = Style::default();

    match position {
        PositionMode::Absolute { x, y } => {
            style.position = Position::Absolute;
            style.inset = Rect {
                top: LengthPercentageAuto::Length(mm_to_pt(*y)),
                left: LengthPercentageAuto::Length(mm_to_pt(*x)),
                right: auto(),
                bottom: auto(),
            };
        }
        PositionMode::Flow => {}
    }

    apply_size_to_style(&mut style, size, parent_direction);

    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mm_to_pt_conversion() {
        let pt = mm_to_pt(210.0);
        // A4 width = 210mm ≈ 595.28pt
        assert!((pt - 595.28).abs() < 0.1);
    }

    #[test]
    fn test_fixed_size() {
        let sv = SizeValue::Fixed { value: 50.0 };
        match size_value_to_dimension(&sv) {
            Dimension::Length(pt) => assert!((pt - mm_to_pt(50.0)).abs() < 0.01),
            _ => panic!("Expected Length"),
        }
    }

    #[test]
    fn test_fr_maps_to_auto_dimension() {
        let sv = SizeValue::Fr { value: 2.0 };
        assert!(matches!(size_value_to_dimension(&sv), Dimension::Auto));
    }
}
