/**
 * TypeScript types for the SOM (Semantic Object Model) specification v1.0.
 *
 * Generated from specs/som-schema.json — do not edit by hand.
 */

// ---- Enums ----

export type RegionRole =
  | 'navigation'
  | 'main'
  | 'aside'
  | 'header'
  | 'footer'
  | 'form'
  | 'dialog'
  | 'content';

export type ElementRole =
  | 'link'
  | 'button'
  | 'text_input'
  | 'textarea'
  | 'select'
  | 'checkbox'
  | 'radio'
  | 'heading'
  | 'image'
  | 'list'
  | 'table'
  | 'paragraph'
  | 'section'
  | 'separator'
  | 'details'
  | 'iframe';

export type ElementAction = 'click' | 'type' | 'clear' | 'select' | 'toggle';

export type SemanticHint =
  | 'active'
  | 'badge'
  | 'card'
  | 'collapsed'
  | 'danger'
  | 'disabled'
  | 'error'
  | 'expanded'
  | 'hero'
  | 'hidden'
  | 'large'
  | 'loading'
  | 'modal'
  | 'notification'
  | 'primary'
  | 'required'
  | 'secondary'
  | 'selected'
  | 'small'
  | 'sticky'
  | 'success'
  | 'warning';

// ---- Sub-types ----

export interface SelectOption {
  value: string;
  text: string;
  selected?: boolean;
}

export interface ListItem {
  text: string;
}

export interface SomElementAttrs {
  href?: string;
  input_type?: string;
  value?: string;
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  checked?: boolean;
  group?: string;
  multiple?: boolean;
  options?: SelectOption[];
  level?: number;
  alt?: string;
  src?: string;
  ordered?: boolean;
  items?: ListItem[];
  headers?: string[];
  rows?: string[][];
  section_label?: string;
  open?: boolean;
  summary?: string;
  has_srcdoc?: boolean;
  srcdoc_preview?: string;
  name?: string;
  sandbox?: string;
  allow?: string;
  width?: string;
  height?: string;
}

export interface ShadowRoot {
  mode: string;
  elements: SomElement[];
}

export interface LinkElement {
  rel: string;
  href: string;
  type?: string;
  hreflang?: string;
}

// ---- Core types ----

export interface SomElement {
  id: string;
  role: ElementRole;
  html_id?: string;
  text?: string;
  label?: string;
  actions?: ElementAction[];
  attrs?: SomElementAttrs;
  children?: SomElement[];
  hints?: SemanticHint[];
  shadow?: ShadowRoot;
}

export interface SomRegion {
  id: string;
  role: RegionRole;
  label?: string;
  action?: string;
  method?: string;
  elements: SomElement[];
}

export interface SomMeta {
  html_bytes: number;
  som_bytes: number;
  element_count: number;
  interactive_count: number;
}

export interface StructuredData {
  json_ld?: Record<string, unknown>[];
  open_graph?: Record<string, string>;
  twitter_card?: Record<string, string>;
  meta?: Record<string, string>;
  links?: LinkElement[];
}

export interface Som {
  som_version: string;
  url: string;
  title: string;
  lang: string;
  regions: SomRegion[];
  meta: SomMeta;
  structured_data?: StructuredData;
}
