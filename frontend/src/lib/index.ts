export { default as DreportEditor } from './DreportEditor.vue'
export type { DreportEditorConfig } from './DreportEditor.vue'

// Core types
export type {
  Template,
  TemplateElement,
  ContainerElement,
  LeafElement,
  StaticTextElement,
  TextElement,
  LineElement,
  ImageElement,
  PageNumberElement,
  BarcodeElement,
  RepeatingTableElement,
  SizeValue,
  SizeConstraint,
  PositionMode,
  ScalarBinding,
  ArrayBinding,
  ElementBinding,
  TextStyle,
  LineStyle,
  ContainerStyle,
  ImageStyle,
  BarcodeStyle,
  BarcodeFormat,
  TableColumn,
  TableStyle,
  FormatType,
} from '../core/types'

// Schema types
export type { JsonSchema, SchemaNode } from '../core/schema-parser'
