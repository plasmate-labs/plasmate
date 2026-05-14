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
  | 'group'
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
  target?: string;
  rel?: string;
  download?: boolean | string;
  input_type?: string;
  value?: string;
  placeholder?: string;
  required?: boolean;
  readonly?: boolean;
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
  legend?: string;
  open?: boolean;
  summary?: string;
  contenteditable?: boolean | string;
  tabindex?: number | string;
  accesskey?: string;
  name?: string;
  autocomplete?: string;
  inputmode?: string;
  enterkeyhint?: string;
  form?: string;
  list?: string;
  popovertarget?: string;
  popovertargetaction?: string;
  commandfor?: string;
  command?: string;
  popover?: string;
  minlength?: number | string;
  maxlength?: number | string;
  min?: number | string;
  max?: number | string;
  step?: string;
  pattern?: string;
  description?: string;
  aria?: AriaState;
  has_srcdoc?: boolean;
  srcdoc_preview?: string;
  sandbox?: string;
  allow?: string;
  width?: string;
  height?: string;
}

export interface AriaState {
  expanded?: boolean;
  selected?: boolean;
  checked?: boolean | string;
  readonly?: boolean;
  disabled?: boolean;
  current?: boolean | string;
  pressed?: boolean;
  hidden?: boolean;
  controls?: string;
  haspopup?: boolean | string;
  invalid?: boolean | string;
  autocomplete?: string;
  active_descendant?: string;
  errormessage?: string;
  keyshortcuts?: string;
  roledescription?: string;
  busy?: boolean;
  live?: string;
  atomic?: boolean;
  relevant?: string;
  owns?: string;
  flowto?: string;
  details?: string;
  multiline?: boolean;
  multiselectable?: boolean;
  orientation?: string;
  sort?: string;
  valuemin?: string;
  valuemax?: string;
  valuenow?: string;
  valuetext?: string;
}

export interface SomShadowRoot {
  mode: 'open' | 'closed' | string;
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
  text?: string;
  label?: string;
  actions?: ElementAction[];
  attrs?: SomElementAttrs;
  children?: SomElement[];
  hints?: SemanticHint[];
  shadow?: SomShadowRoot;
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
