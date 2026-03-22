/**
 * TypeScript types for the SOM (Semantic Object Model) specification v1.0.
 *
 * Generated from specs/som-schema.json — do not edit by hand.
 */
type RegionRole = 'navigation' | 'main' | 'aside' | 'header' | 'footer' | 'form' | 'dialog' | 'content';
type ElementRole = 'link' | 'button' | 'text_input' | 'textarea' | 'select' | 'checkbox' | 'radio' | 'heading' | 'image' | 'list' | 'table' | 'paragraph' | 'section' | 'separator';
type ElementAction = 'click' | 'type' | 'clear' | 'select' | 'toggle';
type SemanticHint = 'active' | 'badge' | 'card' | 'collapsed' | 'danger' | 'disabled' | 'error' | 'expanded' | 'hero' | 'hidden' | 'large' | 'loading' | 'modal' | 'notification' | 'primary' | 'required' | 'secondary' | 'selected' | 'small' | 'sticky' | 'success' | 'warning';
interface SelectOption {
    value: string;
    text: string;
    selected?: boolean;
}
interface ListItem {
    text: string;
}
interface SomElementAttrs {
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
}
interface LinkElement {
    rel: string;
    href: string;
    type?: string;
    hreflang?: string;
}
interface SomElement {
    id: string;
    role: ElementRole;
    text?: string;
    label?: string;
    actions?: ElementAction[];
    attrs?: SomElementAttrs;
    children?: SomElement[];
    hints?: SemanticHint[];
}
interface SomRegion {
    id: string;
    role: RegionRole;
    label?: string;
    action?: string;
    method?: string;
    elements: SomElement[];
}
interface SomMeta {
    html_bytes: number;
    som_bytes: number;
    element_count: number;
    interactive_count: number;
}
interface StructuredData {
    json_ld?: Record<string, unknown>[];
    open_graph?: Record<string, string>;
    twitter_card?: Record<string, string>;
    meta?: Record<string, string>;
    links?: LinkElement[];
}
interface Som {
    som_version: string;
    url: string;
    title: string;
    lang: string;
    regions: SomRegion[];
    meta: SomMeta;
    structured_data?: StructuredData;
}

/**
 * Parse a JSON string or plain object into a typed Som.
 * Throws if the input is not valid SOM.
 */
declare function parseSom(input: string | object): Som;
/**
 * Type guard that checks whether an unknown value conforms to the SOM schema.
 */
declare function isValidSom(input: unknown): input is Som;
/**
 * Parse raw Plasmate CLI JSON output into a typed Som.
 * Handles cases where the CLI may emit extra text before or after the JSON.
 */
declare function fromPlasmate(jsonOutput: string): Som;

/** Flatten all elements from all regions into a single array. */
declare function getAllElements(som: Som): SomElement[];
/** Find all elements with a specific role. */
declare function findByRole(som: Som, role: ElementRole): SomElement[];
/** Find an element by its SOM id. */
declare function findById(som: Som, id: string): SomElement | undefined;
/** Find elements containing text (case-insensitive substring by default). */
declare function findByText(som: Som, text: string, options?: {
    exact?: boolean;
}): SomElement[];
/** Get all elements that have actions (clickable, typeable, etc.). */
declare function getInteractiveElements(som: Som): SomElement[];
/** Extract all links with their text and URLs. */
declare function getLinks(som: Som): Array<{
    text: string;
    href: string;
    id: string;
}>;
/** Get all form regions. */
declare function getForms(som: Som): SomRegion[];
/** Get all input-type elements. */
declare function getInputs(som: Som): SomElement[];
/** Extract heading hierarchy. */
declare function getHeadings(som: Som): Array<{
    level: number;
    text: string;
    id: string;
}>;
/** Extract all visible text content, joined with newlines. */
declare function getText(som: Som): string;
/** Extract text grouped by region. */
declare function getTextByRegion(som: Som): Array<{
    region: string;
    role: string;
    text: string;
}>;
/** Return html_bytes / som_bytes from meta. */
declare function getCompressionRatio(som: Som): number;
/** Convert SOM to a readable markdown representation. */
declare function toMarkdown(som: Som): string;
/** Generic filter across all elements. */
declare function filter(som: Som, predicate: (el: SomElement) => boolean): SomElement[];

export { type ElementAction, type ElementRole, type LinkElement, type ListItem, type RegionRole, type SelectOption, type SemanticHint, type Som, type SomElement, type SomElementAttrs, type SomMeta, type SomRegion, type StructuredData, filter, findById, findByRole, findByText, fromPlasmate, getAllElements, getCompressionRatio, getForms, getHeadings, getInputs, getInteractiveElements, getLinks, getText, getTextByRegion, isValidSom, parseSom, toMarkdown };
