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
        SizeValue::Fixed { value } => Dimension::length(mm_to_pt(*value)),
        SizeValue::Auto => Dimension::auto(),
        // Fr için dimension Auto, flex_grow ayrıca set edilir
        SizeValue::Fr { .. } => Dimension::auto(),
    }
}

/// SizeValue → taffy LengthPercentage (min/max constraint'ler için)
fn mm_to_length(mm: f64) -> Dimension {
    Dimension::length(mm_to_pt(mm))
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
        _ => fr_value(&size.height),
    };

    // Cross axis fr: row'da height fr, column'da width fr
    let cross_fr = match parent_direction {
        Some("row") => fr_value(&size.height),
        _ => fr_value(&size.width),
    };

    // Eğer main axis fr ise, flex_grow ayarla ve flex_basis 0 yap
    if main_fr > 0.0 {
        style.flex_grow = main_fr;
        style.flex_shrink = 1.0;
        style.flex_basis = Dimension::length(0.0);

        // min-width: 0 (row) veya min-height: 0 (column) ayarla —
        // taffy'de min_size default Auto = içerik boyutunun altına küçülemez.
        // Fr elemanların içerik taşırması engellemek için min_size 0 olmalı.
        match parent_direction {
            Some("row") => {
                if size.min_width.is_none() {
                    style.min_size.width = Dimension::length(0.0);
                }
            }
            _ => {
                if size.min_height.is_none() {
                    style.min_size.height = Dimension::length(0.0);
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
            width: LengthPercentage::length(mm_to_pt(el.gap)),
            height: LengthPercentage::length(mm_to_pt(el.gap)),
        },
        padding: Rect {
            top: LengthPercentage::length(mm_to_pt(el.padding.top)),
            right: LengthPercentage::length(mm_to_pt(el.padding.right)),
            bottom: LengthPercentage::length(mm_to_pt(el.padding.bottom)),
            left: LengthPercentage::length(mm_to_pt(el.padding.left)),
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
                top: LengthPercentageAuto::length(mm_to_pt(*y)),
                left: LengthPercentageAuto::length(mm_to_pt(*x)),
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
            top: LengthPercentage::length(bpt),
            right: LengthPercentage::length(bpt),
            bottom: LengthPercentage::length(bpt),
            left: LengthPercentage::length(bpt),
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
                top: LengthPercentageAuto::length(mm_to_pt(*y)),
                left: LengthPercentageAuto::length(mm_to_pt(*x)),
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
    use dreport_core::models::{ContainerStyle, Padding};

    #[test]
    fn test_mm_to_pt_conversion() {
        let pt = mm_to_pt(210.0);
        // A4 width = 210mm ≈ 595.28pt
        assert!((pt - 595.28).abs() < 0.1);
    }

    #[test]
    fn test_mm_to_pt_one_inch() {
        // 1 inch = 25.4mm = 72pt
        let pt = mm_to_pt(25.4);
        assert!(
            (pt - 72.0).abs() < 0.01,
            "25.4mm should be ~72pt, got {}",
            pt
        );
    }

    #[test]
    fn test_pt_to_mm_conversion() {
        // 72pt = 25.4mm (1 inch)
        let mm = pt_to_mm(72.0);
        assert!(
            (mm - 25.4).abs() < 0.01,
            "72pt should be ~25.4mm, got {}",
            mm
        );
    }

    #[test]
    fn test_roundtrip_mm_pt_mm() {
        // mm → pt → mm should preserve value within tolerance
        let original = 100.0_f64;
        let pt = mm_to_pt(original);
        let back = pt_to_mm(pt);
        assert!(
            (back - original).abs() < 0.01,
            "Roundtrip failed: {} → {}pt → {}",
            original,
            pt,
            back
        );
    }

    #[test]
    fn test_mm_to_pt_zero() {
        assert_eq!(mm_to_pt(0.0), 0.0);
    }

    #[test]
    fn test_pt_to_mm_zero() {
        assert!((pt_to_mm(0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fixed_size() {
        let sv = SizeValue::Fixed { value: 50.0 };
        assert_eq!(
            size_value_to_dimension(&sv),
            Dimension::length(mm_to_pt(50.0))
        );
    }

    #[test]
    fn test_auto_size() {
        let sv = SizeValue::Auto;
        assert_eq!(size_value_to_dimension(&sv), Dimension::auto());
    }

    #[test]
    fn test_fr_maps_to_auto_dimension() {
        let sv = SizeValue::Fr { value: 2.0 };
        assert_eq!(size_value_to_dimension(&sv), Dimension::auto());
    }

    #[test]
    fn test_fr_value_extraction() {
        assert_eq!(fr_value(&SizeValue::Fr { value: 3.0 }), 3.0);
        assert_eq!(fr_value(&SizeValue::Auto), 0.0);
        assert_eq!(fr_value(&SizeValue::Fixed { value: 10.0 }), 0.0);
    }

    #[test]
    fn test_apply_size_fr_sets_flex_grow() {
        let size = SizeConstraint {
            width: SizeValue::Fr { value: 2.0 },
            height: SizeValue::Auto,
            ..Default::default()
        };
        let mut style = Style::default();
        apply_size_to_style(&mut style, &size, Some("row"));
        assert_eq!(style.flex_grow, 2.0);
        assert_eq!(style.flex_basis, Dimension::length(0.0));
    }

    #[test]
    fn test_apply_size_fixed_no_flex_grow() {
        let size = SizeConstraint {
            width: SizeValue::Fixed { value: 50.0 },
            height: SizeValue::Fixed { value: 30.0 },
            ..Default::default()
        };
        let mut style = Style::default();
        apply_size_to_style(&mut style, &size, Some("row"));
        assert_eq!(style.flex_grow, 0.0);
    }

    #[test]
    fn test_apply_size_min_max_constraints() {
        let size = SizeConstraint {
            width: SizeValue::Auto,
            height: SizeValue::Auto,
            min_width: Some(20.0),
            max_width: Some(100.0),
            min_height: Some(10.0),
            max_height: Some(50.0),
        };
        let mut style = Style::default();
        apply_size_to_style(&mut style, &size, None);
        assert_eq!(style.min_size.width, Dimension::length(mm_to_pt(20.0)));
        assert_eq!(style.max_size.width, Dimension::length(mm_to_pt(100.0)));
        assert_eq!(style.min_size.height, Dimension::length(mm_to_pt(10.0)));
        assert_eq!(style.max_size.height, Dimension::length(mm_to_pt(50.0)));
    }

    #[test]
    fn test_container_to_style_direction() {
        let el = ContainerElement {
            id: "test".to_string(),
            position: PositionMode::Flow,
            size: SizeConstraint::default(),
            direction: "row".to_string(),
            gap: 5.0,
            padding: Padding {
                top: 10.0,
                right: 10.0,
                bottom: 10.0,
                left: 10.0,
            },
            align: "center".to_string(),
            justify: "space-between".to_string(),
            style: ContainerStyle::default(),
            children: vec![],
            break_inside: "auto".to_string(),
        };
        let style = container_to_style(&el, None);
        assert_eq!(style.flex_direction, FlexDirection::Row);
        assert_eq!(style.align_items, Some(AlignItems::Center));
        assert_eq!(style.justify_content, Some(JustifyContent::SpaceBetween));
    }

    #[test]
    fn test_container_to_style_absolute() {
        let el = ContainerElement {
            id: "test".to_string(),
            position: PositionMode::Absolute { x: 20.0, y: 30.0 },
            size: SizeConstraint::default(),
            direction: "column".to_string(),
            gap: 0.0,
            padding: Padding::default(),
            align: "stretch".to_string(),
            justify: "start".to_string(),
            style: ContainerStyle::default(),
            children: vec![],
            break_inside: "auto".to_string(),
        };
        let style = container_to_style(&el, None);
        assert_eq!(style.position, Position::Absolute);
    }

    #[test]
    fn test_leaf_style_flow() {
        let size = SizeConstraint {
            width: SizeValue::Fixed { value: 60.0 },
            height: SizeValue::Auto,
            ..Default::default()
        };
        let style = leaf_style(&size, &PositionMode::Flow, Some("column"));
        assert_eq!(style.position, Position::Relative);
        assert_eq!(style.size.width, Dimension::length(mm_to_pt(60.0)));
    }

    #[test]
    fn test_leaf_style_absolute() {
        let size = SizeConstraint {
            width: SizeValue::Fixed { value: 40.0 },
            height: SizeValue::Fixed { value: 20.0 },
            ..Default::default()
        };
        let style = leaf_style(&size, &PositionMode::Absolute { x: 10.0, y: 15.0 }, None);
        assert_eq!(style.position, Position::Absolute);
    }
}
