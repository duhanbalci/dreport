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
    pub font_style: Option<String>,
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

// --- Condition (v-if benzeri koşullu gösterim) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    /// Data JSON'daki alan yolu (ör: "toplamlar.iskonto")
    pub path: String,
    /// Karşılaştırma operatörü: eq, neq, gt, gte, lt, lte, empty, not_empty
    pub operator: String,
    /// Karşılaştırılacak değer (empty/not_empty için gerekmez)
    #[serde(default)]
    pub value: Option<serde_json::Value>,
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
    /// Show vertical grid lines at each category (line charts). Defaults to true.
    pub show_vertical_grid: Option<bool>,
    pub vertical_grid_color: Option<String>,
    #[serde(default)]
    pub reference_lines: Vec<ChartReferenceLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartReferenceLine {
    /// Category index (0-based) where the vertical line is drawn
    pub category_index: usize,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub dash: Option<bool>,
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
    #[serde(flatten)]
    pub base: ElementBase,
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

// --- Element Base (ortak alanlar) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementBase {
    pub id: String,
    #[serde(default)]
    pub condition: Option<Condition>,
    #[serde(default)]
    pub position: PositionMode,
    #[serde(default)]
    pub size: SizeConstraint,
}

impl ElementBase {
    /// Flow pozisyonlu, condition'sız, verilen size ile base oluştur
    pub fn flow(id: String, size: SizeConstraint) -> Self {
        Self {
            id,
            condition: None,
            position: PositionMode::Flow,
            size,
        }
    }
}

pub trait HasBase {
    fn base(&self) -> &ElementBase;
    fn base_mut(&mut self) -> &mut ElementBase;
}

macro_rules! impl_has_base {
    ($($t:ty),+ $(,)?) => {
        $(impl HasBase for $t {
            fn base(&self) -> &ElementBase { &self.base }
            fn base_mut(&mut self) -> &mut ElementBase { &mut self.base }
        })+
    };
}

impl_has_base!(
    ContainerElement,
    StaticTextElement,
    TextElement,
    LineElement,
    ImageElement,
    PageNumberElement,
    BarcodeElement,
    RepeatingTableElement,
    PageBreakElement,
    CurrentDateElement,
    ShapeElement,
    CheckboxElement,
    CalculatedTextElement,
    RichTextElement,
    ChartElement,
);

pub trait ElementTypeStr {
    fn type_str(&self) -> &'static str;
}

macro_rules! impl_type_str {
    ($($t:ty => $s:literal),+ $(,)?) => {
        $(impl ElementTypeStr for $t {
            fn type_str(&self) -> &'static str { $s }
        })+
    };
}

impl_type_str!(
    ContainerElement => "container",
    StaticTextElement => "static_text",
    TextElement => "text",
    LineElement => "line",
    ImageElement => "image",
    PageNumberElement => "page_number",
    BarcodeElement => "barcode",
    RepeatingTableElement => "repeating_table",
    PageBreakElement => "page_break",
    CurrentDateElement => "current_date",
    ShapeElement => "shape",
    CheckboxElement => "checkbox",
    CalculatedTextElement => "calculated_text",
    RichTextElement => "rich_text",
    ChartElement => "chart",
);

pub trait HasTextStyle {
    fn text_style(&self) -> &TextStyle;
}

macro_rules! impl_has_text_style {
    ($($t:ty),+ $(,)?) => {
        $(impl HasTextStyle for $t {
            fn text_style(&self) -> &TextStyle { &self.style }
        })+
    };
}

impl_has_text_style!(
    StaticTextElement,
    TextElement,
    PageNumberElement,
    CurrentDateElement,
    CalculatedTextElement,
    RichTextElement,
);

pub trait HasOptionalBinding {
    fn binding(&self) -> Option<&ScalarBinding>;
    fn static_value(&self) -> Option<&str>;
}

impl HasOptionalBinding for ImageElement {
    fn binding(&self) -> Option<&ScalarBinding> {
        self.binding.as_ref()
    }
    fn static_value(&self) -> Option<&str> {
        self.src.as_deref()
    }
}

impl HasOptionalBinding for BarcodeElement {
    fn binding(&self) -> Option<&ScalarBinding> {
        self.binding.as_ref()
    }
    fn static_value(&self) -> Option<&str> {
        self.value.as_deref()
    }
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
    Chart(Box<ChartElement>),
}

impl TemplateElement {
    fn inner_base(&self) -> &ElementBase {
        match self {
            Self::Container(e) => e.base(),
            Self::StaticText(e) => e.base(),
            Self::Text(e) => e.base(),
            Self::Line(e) => e.base(),
            Self::RepeatingTable(e) => e.base(),
            Self::Image(e) => e.base(),
            Self::PageNumber(e) => e.base(),
            Self::Barcode(e) => e.base(),
            Self::PageBreak(e) => e.base(),
            Self::CurrentDate(e) => e.base(),
            Self::Shape(e) => e.base(),
            Self::Checkbox(e) => e.base(),
            Self::CalculatedText(e) => e.base(),
            Self::RichText(e) => e.base(),
            Self::Chart(e) => e.base(),
        }
    }

    pub fn id(&self) -> &str {
        &self.inner_base().id
    }

    pub fn position(&self) -> &PositionMode {
        &self.inner_base().position
    }

    pub fn condition(&self) -> Option<&Condition> {
        self.inner_base().condition.as_ref()
    }

    pub fn size(&self) -> &SizeConstraint {
        &self.inner_base().size
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            Self::Container(e) => e.type_str(),
            Self::StaticText(e) => e.type_str(),
            Self::Text(e) => e.type_str(),
            Self::Line(e) => e.type_str(),
            Self::RepeatingTable(e) => e.type_str(),
            Self::Image(e) => e.type_str(),
            Self::PageNumber(e) => e.type_str(),
            Self::Barcode(e) => e.type_str(),
            Self::PageBreak(e) => e.type_str(),
            Self::CurrentDate(e) => e.type_str(),
            Self::Shape(e) => e.type_str(),
            Self::Checkbox(e) => e.type_str(),
            Self::CalculatedText(e) => e.type_str(),
            Self::RichText(e) => e.type_str(),
            Self::Chart(e) => e.type_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichTextElement {
    #[serde(flatten)]
    pub base: ElementBase,
    #[serde(default)]
    pub style: TextStyle, // varsayilan stil (span'lar override edebilir)
    pub content: Vec<RichTextSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerElement {
    #[serde(flatten)]
    pub base: ElementBase,
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

fn default_auto() -> String {
    "auto".to_string()
}

fn default_column() -> String {
    "column".to_string()
}
fn default_stretch() -> String {
    "stretch".to_string()
}
fn default_start() -> String {
    "start".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticTextElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub style: TextStyle,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub style: TextStyle,
    pub content: Option<String>,
    pub binding: ScalarBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub style: LineStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub src: Option<String>,
    pub binding: Option<ScalarBinding>,
    pub style: ImageStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageNumberElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub style: TextStyle,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarcodeElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub format: String, // qr, ean13, ean8, code128, code39
    pub value: Option<String>,
    pub binding: Option<ScalarBinding>,
    pub style: BarcodeStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepeatingTableElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub data_source: ArrayBinding,
    pub columns: Vec<TableColumn>,
    pub style: TableStyle,
    #[serde(default = "default_true")]
    pub repeat_header: Option<bool>,
}

fn default_true() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBreakElement {
    #[serde(flatten)]
    pub base: ElementBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentDateElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub style: TextStyle,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub shape_type: String, // rectangle, ellipse, rounded_rectangle
    pub style: ContainerStyle,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CheckboxStyle {
    pub size: Option<f64>,            // mm — kare boyutu
    pub check_color: Option<String>,  // checkmark rengi
    pub border_color: Option<String>, // kare kenar rengi
    pub border_width: Option<f64>,    // kenar kalınlığı
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckboxElement {
    #[serde(flatten)]
    pub base: ElementBase,
    pub checked: Option<bool>,          // statik değer
    pub binding: Option<ScalarBinding>, // dinamik boolean binding
    pub style: CheckboxStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculatedTextElement {
    #[serde(flatten)]
    pub base: ElementBase,
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
    #[serde(default)]
    pub format_config: Option<FormatConfig>,
    /// Lokalizasyon: "tr-TR", "en-US", "de-DE", "fr-FR" vb.
    /// Belirtilirse ve format_config yoksa, locale'den FormatConfig türetilir.
    #[serde(default)]
    pub locale: Option<String>,
}

/// Sayı/para birimi formatlama ayarları.
/// Belirtilmezse Türk Lirası varsayılan (. binlik, , ondalık, ₺ sembol).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormatConfig {
    /// Binlik ayırıcı (varsayılan ".")
    #[serde(default = "FormatConfig::default_thousands_sep")]
    pub thousands_separator: String,
    /// Ondalık ayırıcı (varsayılan ",")
    #[serde(default = "FormatConfig::default_decimal_sep")]
    pub decimal_separator: String,
    /// Para birimi sembolü (varsayılan "₺")
    #[serde(default = "FormatConfig::default_currency_symbol")]
    pub currency_symbol: String,
    /// Para birimi sembolü pozisyonu: "suffix" (varsayılan) veya "prefix"
    #[serde(default = "FormatConfig::default_currency_position")]
    pub currency_position: String,
}

impl FormatConfig {
    fn default_thousands_sep() -> String {
        ".".to_string()
    }
    fn default_decimal_sep() -> String {
        ",".to_string()
    }
    fn default_currency_symbol() -> String {
        "₺".to_string()
    }
    fn default_currency_position() -> String {
        "suffix".to_string()
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            thousands_separator: Self::default_thousands_sep(),
            decimal_separator: Self::default_decimal_sep(),
            currency_symbol: Self::default_currency_symbol(),
            currency_position: Self::default_currency_position(),
        }
    }
}

impl FormatConfig {
    /// Locale string'inden FormatConfig türet.
    /// Desteklenen locale'ler: tr-TR, en-US, de-DE, fr-FR.
    /// Bilinmeyen locale → Türk formatı (varsayılan).
    pub fn from_locale(locale: &str) -> Self {
        match locale {
            "en-US" | "en" => Self {
                thousands_separator: ",".to_string(),
                decimal_separator: ".".to_string(),
                currency_symbol: "$".to_string(),
                currency_position: "prefix".to_string(),
            },
            "de-DE" | "de" => Self {
                thousands_separator: ".".to_string(),
                decimal_separator: ",".to_string(),
                currency_symbol: "€".to_string(),
                currency_position: "suffix".to_string(),
            },
            "fr-FR" | "fr" => Self {
                thousands_separator: " ".to_string(),
                decimal_separator: ",".to_string(),
                currency_symbol: "€".to_string(),
                currency_position: "suffix".to_string(),
            },
            "en-GB" | "gb" => Self {
                thousands_separator: ",".to_string(),
                decimal_separator: ".".to_string(),
                currency_symbol: "£".to_string(),
                currency_position: "prefix".to_string(),
            },
            // tr-TR veya bilinmeyen → Türk formatı
            _ => Self::default(),
        }
    }
}

impl Template {
    /// Template'in etkin FormatConfig'ini döndür.
    /// Öncelik: format_config > locale > varsayılan (tr-TR).
    pub fn effective_format_config(&self) -> FormatConfig {
        if let Some(ref fc) = self.format_config {
            fc.clone()
        } else if let Some(ref locale) = self.locale {
            FormatConfig::from_locale(locale)
        } else {
            FormatConfig::default()
        }
    }
}
