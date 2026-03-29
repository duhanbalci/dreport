use serde::{Deserialize, Serialize};

// --- Boyut sistemi ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SizeValue {
    #[serde(rename = "fixed")]
    Fixed { value: f64 },
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "fr")]
    Fr { value: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeConstraint {
    pub width: SizeValue,
    pub height: SizeValue,
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
    pub max_width: Option<f64>,
    pub max_height: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSettings {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Padding {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

// --- Positioning ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PositionMode {
    #[serde(rename = "flow")]
    Flow,
    #[serde(rename = "absolute")]
    Absolute { x: f64, y: f64 },
}

// --- Stil ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TextStyle {
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub color: Option<String>,
    pub align: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LineStyle {
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ContainerStyle {
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<f64>,
    pub border_radius: Option<f64>,
    pub border_style: Option<String>,
}

// --- Binding ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarBinding {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayBinding {
    pub path: String,
}

// --- Tablo ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumn {
    pub id: String,
    pub field: String,
    pub title: String,
    pub width: SizeValue,
    pub align: String,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TableStyle {
    pub header_bg: Option<String>,
    pub header_color: Option<String>,
    pub zebra_odd: Option<String>,
    pub zebra_even: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<f64>,
    pub font_size: Option<f64>,
    pub header_font_size: Option<f64>,
}

// --- Barcode ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BarcodeStyle {
    pub color: Option<String>,
}

// --- Element tipleri ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ImageStyle {
    pub object_fit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TemplateElement {
    #[serde(rename = "container")]
    Container(ContainerElement),
    #[serde(rename = "static_text")]
    StaticText(StaticTextElement),
    #[serde(rename = "text")]
    Text(TextElement),
    #[serde(rename = "line")]
    Line(LineElement),
    #[serde(rename = "repeating_table")]
    RepeatingTable(RepeatingTableElement),
    #[serde(rename = "image")]
    Image(ImageElement),
    #[serde(rename = "page_number")]
    PageNumber(PageNumberElement),
    #[serde(rename = "barcode")]
    Barcode(BarcodeElement),
}

impl TemplateElement {
    pub fn id(&self) -> &str {
        match self {
            Self::Container(e) => &e.id,
            Self::StaticText(e) => &e.id,
            Self::Text(e) => &e.id,
            Self::Line(e) => &e.id,
            Self::RepeatingTable(e) => &e.id,
            Self::Image(e) => &e.id,
            Self::PageNumber(e) => &e.id,
            Self::Barcode(e) => &e.id,
        }
    }

    pub fn position(&self) -> &PositionMode {
        match self {
            Self::Container(e) => &e.position,
            Self::StaticText(e) => &e.position,
            Self::Text(e) => &e.position,
            Self::Line(e) => &e.position,
            Self::RepeatingTable(e) => &e.position,
            Self::Image(e) => &e.position,
            Self::PageNumber(e) => &e.position,
            Self::Barcode(e) => &e.position,
        }
    }

    pub fn size(&self) -> &SizeConstraint {
        match self {
            Self::Container(e) => &e.size,
            Self::StaticText(e) => &e.size,
            Self::Text(e) => &e.size,
            Self::Line(e) => &e.size,
            Self::RepeatingTable(e) => &e.size,
            Self::Image(e) => &e.size,
            Self::PageNumber(e) => &e.size,
            Self::Barcode(e) => &e.size,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub direction: String,
    pub gap: f64,
    pub padding: Padding,
    pub align: String,
    pub justify: String,
    pub style: ContainerStyle,
    pub children: Vec<TemplateElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticTextElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub style: TextStyle,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub style: TextStyle,
    pub content: Option<String>,
    pub binding: ScalarBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub style: LineStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub src: Option<String>,
    pub binding: Option<ScalarBinding>,
    pub style: ImageStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageNumberElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub style: TextStyle,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarcodeElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub format: String, // qr, ean13, ean8, code128, code39
    pub value: Option<String>,
    pub binding: Option<ScalarBinding>,
    pub style: BarcodeStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepeatingTableElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub data_source: ArrayBinding,
    pub columns: Vec<TableColumn>,
    pub style: TableStyle,
}

// --- Template ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub page: PageSettings,
    pub fonts: Vec<String>,
    pub root: ContainerElement,
}
