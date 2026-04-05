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
#[serde(rename_all = "camelCase", default)]
pub struct SizeConstraint {
    pub width: SizeValue,
    pub height: SizeValue,
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
    pub max_width: Option<f64>,
    pub max_height: Option<f64>,
}

impl Default for SizeConstraint {
    fn default() -> Self {
        Self {
            width: SizeValue::Auto,
            height: SizeValue::Auto,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSettings {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Padding {
    #[serde(default)]
    pub top: f64,
    #[serde(default)]
    pub right: f64,
    #[serde(default)]
    pub bottom: f64,
    #[serde(default)]
    pub left: f64,
}

// --- Positioning ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PositionMode {
    #[default]
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
    /// Hücre iç boşluğu — yatay (sol+sağ), mm cinsinden. Default: 2.0
    pub cell_padding_h: Option<f64>,
    /// Hücre iç boşluğu — dikey (üst+alt), mm cinsinden. Default: 1.0
    pub cell_padding_v: Option<f64>,
    /// Header hücre iç boşluğu — yatay (sol+sağ), mm cinsinden. Default: cell_padding_h
    pub header_padding_h: Option<f64>,
    /// Header hücre iç boşluğu — dikey (üst+alt), mm cinsinden. Default: cell_padding_v
    pub header_padding_v: Option<f64>,
}

// --- Barcode ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BarcodeStyle {
    pub color: Option<String>,
    pub include_text: Option<bool>,
}

// --- Rich Text ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichTextSpan {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub binding: Option<ScalarBinding>,
    #[serde(default)]
    pub style: TextStyle,
}

// --- Chart ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Bar,
    Line,
    Pie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupMode {
    Grouped,
    Stacked,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChartTitle {
    pub text: String,
    pub font_size: Option<f64>,
    pub color: Option<String>,
    pub align: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChartLegend {
    pub show: bool,
    pub position: Option<String>,
    pub font_size: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChartLabels {
    pub show: bool,
    pub font_size: Option<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChartAxis {
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub show_grid: Option<bool>,
    pub grid_color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChartStyle {
    pub colors: Option<Vec<String>>,
    pub background_color: Option<String>,
    pub bar_gap: Option<f64>,
    pub line_width: Option<f64>,
    pub show_points: Option<bool>,
    pub curve_type: Option<String>,
    pub inner_radius: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub chart_type: ChartType,
    pub data_source: ArrayBinding,
    pub category_field: String,
    pub value_field: String,
    #[serde(default)]
    pub group_field: Option<String>,
    #[serde(default)]
    pub group_mode: Option<GroupMode>,
    #[serde(default)]
    pub title: Option<ChartTitle>,
    #[serde(default)]
    pub legend: Option<ChartLegend>,
    #[serde(default)]
    pub labels: Option<ChartLabels>,
    #[serde(default)]
    pub axis: Option<ChartAxis>,
    #[serde(default)]
    pub style: ChartStyle,
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
    #[serde(rename = "page_break")]
    PageBreak(PageBreakElement),
    #[serde(rename = "current_date")]
    CurrentDate(CurrentDateElement),
    #[serde(rename = "shape")]
    Shape(ShapeElement),
    #[serde(rename = "checkbox")]
    Checkbox(CheckboxElement),
    #[serde(rename = "calculated_text")]
    CalculatedText(CalculatedTextElement),
    #[serde(rename = "rich_text")]
    RichText(RichTextElement),
    #[serde(rename = "chart")]
    Chart(ChartElement),
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
            Self::PageBreak(e) => &e.id,
            Self::CurrentDate(e) => &e.id,
            Self::Shape(e) => &e.id,
            Self::Checkbox(e) => &e.id,
            Self::CalculatedText(e) => &e.id,
            Self::RichText(e) => &e.id,
            Self::Chart(e) => &e.id,
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
            Self::PageBreak(_) => &PositionMode::Flow,
            Self::CurrentDate(e) => &e.position,
            Self::Shape(e) => &e.position,
            Self::Checkbox(e) => &e.position,
            Self::CalculatedText(e) => &e.position,
            Self::RichText(e) => &e.position,
            Self::Chart(e) => &e.position,
        }
    }

    pub fn size(&self) -> &SizeConstraint {
        static DEFAULT_SIZE: SizeConstraint = SizeConstraint {
            width: SizeValue::Auto,
            height: SizeValue::Auto,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        };
        match self {
            Self::Container(e) => &e.size,
            Self::StaticText(e) => &e.size,
            Self::Text(e) => &e.size,
            Self::Line(e) => &e.size,
            Self::RepeatingTable(e) => &e.size,
            Self::Image(e) => &e.size,
            Self::PageNumber(e) => &e.size,
            Self::Barcode(e) => &e.size,
            Self::PageBreak(_) => &DEFAULT_SIZE,
            Self::CurrentDate(e) => &e.size,
            Self::Shape(e) => &e.size,
            Self::Checkbox(e) => &e.size,
            Self::CalculatedText(e) => &e.size,
            Self::RichText(e) => &e.size,
            Self::Chart(e) => &e.size,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichTextElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    #[serde(default)]
    pub style: TextStyle, // varsayilan stil (span'lar override edebilir)
    pub content: Vec<RichTextSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerElement {
    pub id: String,
    #[serde(default)]
    pub position: PositionMode,
    #[serde(default)]
    pub size: SizeConstraint,
    #[serde(default = "default_column")]
    pub direction: String,
    #[serde(default)]
    pub gap: f64,
    #[serde(default)]
    pub padding: Padding,
    #[serde(default = "default_stretch")]
    pub align: String,
    #[serde(default = "default_start")]
    pub justify: String,
    #[serde(default)]
    pub style: ContainerStyle,
    #[serde(default)]
    pub children: Vec<TemplateElement>,
    #[serde(default = "default_auto")]
    pub break_inside: String,
}

fn default_auto() -> String { "auto".to_string() }

fn default_column() -> String { "column".to_string() }
fn default_stretch() -> String { "stretch".to_string() }
fn default_start() -> String { "start".to_string() }

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
    #[serde(default = "default_true")]
    pub repeat_header: Option<bool>,
}

fn default_true() -> Option<bool> { Some(true) }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBreakElement {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentDateElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub style: TextStyle,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub shape_type: String, // rectangle, ellipse, rounded_rectangle
    pub style: ContainerStyle,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckboxStyle {
    pub size: Option<f64>,          // mm — kare boyutu
    pub check_color: Option<String>,  // checkmark rengi
    pub border_color: Option<String>, // kare kenar rengi
    pub border_width: Option<f64>,    // kenar kalınlığı
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckboxElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub checked: Option<bool>,           // statik değer
    pub binding: Option<ScalarBinding>,  // dinamik boolean binding
    pub style: CheckboxStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculatedTextElement {
    pub id: String,
    pub position: PositionMode,
    pub size: SizeConstraint,
    pub style: TextStyle,
    pub expression: String,
    pub format: Option<String>,
}

// --- Template ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub page: PageSettings,
    pub fonts: Vec<String>,
    #[serde(default)]
    pub header: Option<ContainerElement>,
    #[serde(default)]
    pub footer: Option<ContainerElement>,
    pub root: ContainerElement,
}
