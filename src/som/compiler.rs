use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use serde_json::json;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use super::css_visibility::VisibilityRules;
use super::element_id::{generate_element_id, generate_region_id, ElementIdTracker};
use super::heuristics::{self, ContentConfig};
use super::metadata;
use super::types::*;

/// Context for content summarization during compilation.
struct CompileContext {
    config: ContentConfig,
    paragraph_count: Cell<usize>,
    is_main_region: Cell<bool>,
    /// CSS visibility rules for filtering hidden elements.
    css_rules: VisibilityRules,
}

/// Tracks heading hierarchy for a region.
struct HeadingTracker {
    /// Stack of (level, id) representing the current heading hierarchy.
    stack: Vec<(u8, String)>,
}

impl HeadingTracker {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Record a heading and determine if it's redundant.
    /// Returns true if the heading adds value (not a duplicate or same text as parent).
    fn track(&mut self, level: u8, text: &str, id: &str) -> bool {
        // Pop headings at same or lower level
        while let Some((l, _)) = self.stack.last() {
            if *l >= level {
                self.stack.pop();
            } else {
                break;
            }
        }
        self.stack.push((level, id.to_string()));
        true
    }

    /// Build a breadcrumb path from the heading stack.
    /// e.g. "Introduction > History > Early development"
    fn breadcrumb(&self) -> Option<String> {
        if self.stack.len() <= 1 {
            None
        } else {
            None // For now, don't emit breadcrumbs; the hierarchy is implicit from level attrs
        }
    }
}

/// Errors that can occur during SOM compilation.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("HTML parse error: {0}")]
    ParseError(String),
}

/// Compile HTML source into a SOM snapshot.
pub fn compile(html: &str, page_url: &str) -> Result<Som, CompileError> {
    let origin = url::Url::parse(page_url)
        .map(|u| u.origin().unicode_serialization())
        .unwrap_or_else(|_| page_url.to_string());

    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .map_err(|e| CompileError::ParseError(e.to_string()))?;

    let html_bytes = html.len();
    let mut id_tracker = ElementIdTracker::new();
    let mut region_counts: HashMap<String, usize> = HashMap::new();
    let css_rules = VisibilityRules::from_html(html);
    let ctx = CompileContext {
        config: ContentConfig::default(),
        paragraph_count: Cell::new(0),
        is_main_region: Cell::new(false),
        css_rules,
    };

    // Extract structured data (JSON-LD, OpenGraph, Twitter Cards, meta)
    let structured = metadata::extract_structured_data(html);
    let structured_data = if structured.is_empty() {
        None
    } else {
        Some(structured)
    };

    // Extract page title and lang
    let title = extract_title(&dom.document);
    let lang = extract_lang(&dom.document);

    // Find the body node
    let body = find_body(&dom.document);

    let regions = match body {
        Some(body_handle) => extract_regions(
            &body_handle,
            &origin,
            &mut id_tracker,
            &mut region_counts,
            &ctx,
        ),
        None => vec![],
    };

    // Count elements
    let element_count = Cell::new(0usize);
    let interactive_count = Cell::new(0usize);
    count_elements(&regions, &element_count, &interactive_count);

    // Calculate SOM bytes by serializing once
    let mut som = Som {
        som_version: "0.1".to_string(),
        url: page_url.to_string(),
        title,
        lang,
        regions,
        meta: SomMeta {
            html_bytes,
            som_bytes: 0,
            element_count: element_count.get(),
            interactive_count: interactive_count.get(),
        },
        structured_data,
    };

    // Serialize once for byte count
    let som_json = serde_json::to_string(&som).unwrap_or_default();
    som.meta.som_bytes = som_json.len();

    Ok(som)
}

fn count_elements(regions: &[Region], total: &Cell<usize>, interactive: &Cell<usize>) {
    for region in regions {
        count_elements_vec(&region.elements, total, interactive);
    }
}

fn count_elements_vec(elements: &[Element], total: &Cell<usize>, interactive: &Cell<usize>) {
    for el in elements {
        total.set(total.get() + 1);
        if el.role.is_interactive() {
            interactive.set(interactive.get() + 1);
        }
        if let Some(children) = &el.children {
            count_elements_vec(children, total, interactive);
        }
    }
}

fn extract_title(node: &Handle) -> String {
    if let NodeData::Element { name, .. } = &node.data {
        if name.local.as_ref() == "title" {
            return get_text_content(node);
        }
    }
    for child in node.children.borrow().iter() {
        let t = extract_title(child);
        if !t.is_empty() {
            return t;
        }
    }
    String::new()
}

fn extract_lang(node: &Handle) -> String {
    if let NodeData::Element { name, attrs, .. } = &node.data {
        if name.local.as_ref() == "html" {
            let attrs = attrs.borrow();
            for attr in attrs.iter() {
                if attr.name.local.as_ref() == "lang" {
                    return attr.value.to_string();
                }
            }
        }
    }
    for child in node.children.borrow().iter() {
        let l = extract_lang(child);
        if !l.is_empty() {
            return l;
        }
    }
    "en".to_string()
}

fn find_body(node: &Handle) -> Option<Handle> {
    if let NodeData::Element { name, .. } = &node.data {
        if name.local.as_ref() == "body" {
            return Some(node.clone());
        }
    }
    for child in node.children.borrow().iter() {
        if let Some(body) = find_body(child) {
            return Some(body);
        }
    }
    None
}

/// Extract regions from the body node.
fn extract_regions(
    body: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    region_counts: &mut HashMap<String, usize>,
    ctx: &CompileContext,
) -> Vec<Region> {
    let mut regions = Vec::new();
    let mut unassigned_elements = Vec::new();

    // First pass: find landmark regions
    collect_regions(
        body,
        origin,
        id_tracker,
        region_counts,
        &mut regions,
        &mut unassigned_elements,
        "0",
        ctx,
    );

    // If no landmarks found, wrap everything in a single content region
    if regions.is_empty() && !unassigned_elements.is_empty() {
        let count = region_counts.entry("content".into()).or_insert(0);
        let rid = generate_region_id("content", *count);
        *count += 1;
        regions.push(Region {
            id: rid,
            role: RegionRole::Content,
            label: None,
            action: None,
            method: None,
            elements: summarize_elements(unassigned_elements, ctx, false),
        });
    } else if !unassigned_elements.is_empty() {
        // Add remaining elements to a content region
        let count = region_counts.entry("content".into()).or_insert(0);
        let rid = generate_region_id("content", *count);
        *count += 1;
        regions.push(Region {
            id: rid,
            role: RegionRole::Content,
            label: None,
            action: None,
            method: None,
            elements: summarize_elements(unassigned_elements, ctx, false),
        });
    }

    // Filter out empty regions
    regions.retain(|r| !r.elements.is_empty());
    regions
}

/// Summarize elements by enforcing per-region budgets.
///
/// v0.1 goal: preserve intent-relevant structure while staying token-efficient.
/// This function:
/// - deduplicates links by href (same URL = keep only first occurrence)
/// - limits paragraphs, links, and total element counts
/// - preserves heading hierarchy (never drops headings)
/// - enforces a max elements budget
/// - appends a summary paragraph when items are dropped
fn summarize_elements(
    elements: Vec<Element>,
    ctx: &CompileContext,
    is_navigation: bool,
) -> Vec<Element> {
    let mut result = Vec::new();
    let mut seen_hrefs: HashSet<String> = HashSet::new();
    let mut heading_tracker = HeadingTracker::new();

    let mut kept_paras = 0usize;
    let mut kept_links = 0usize;

    let mut dropped_paras = 0usize;
    let mut dropped_links = 0usize;
    let mut dropped_dupes = 0usize;
    let mut dropped_other = 0usize;
    let mut dropped_chars = 0usize;

    let link_limit = if is_navigation {
        ctx.config.max_navigation_links
    } else {
        ctx.config.max_links
    };

    for el in elements {
        // Never drop form controls, they are high-signal for agents.
        let is_form_control = matches!(
            el.role,
            ElementRole::TextInput
                | ElementRole::Textarea
                | ElementRole::Select
                | ElementRole::Checkbox
                | ElementRole::Radio
                | ElementRole::Button
        );

        // Never drop headings - they provide structure for agent navigation
        let is_heading = el.role == ElementRole::Heading;

        // Track heading hierarchy
        if is_heading {
            let level = el
                .attrs
                .as_ref()
                .and_then(|a| a.get("level"))
                .and_then(|v| v.as_u64())
                .unwrap_or(6) as u8;
            let text = el.text.as_deref().unwrap_or("");
            heading_tracker.track(level, text, &el.id);
        }

        // Enforce max elements budget, but keep form controls and headings.
        if result.len() >= ctx.config.max_elements && !is_form_control && !is_heading {
            dropped_other += 1;
            if let Some(text) = &el.text {
                dropped_chars += text.len();
            }
            continue;
        }

        // Link deduplication: if same href already seen, skip
        if el.role == ElementRole::Link {
            if let Some(attrs) = &el.attrs {
                if let Some(href) = attrs.get("href").and_then(|v| v.as_str()) {
                    // Normalize: strip trailing slash and fragment for dedup
                    let normalized = normalize_href(href);
                    if !seen_hrefs.insert(normalized) {
                        dropped_dupes += 1;
                        if let Some(text) = &el.text {
                            dropped_chars += text.len();
                        }
                        continue;
                    }
                }
            }

            // Link budget
            kept_links += 1;
            if kept_links > link_limit {
                dropped_links += 1;
                if let Some(text) = &el.text {
                    dropped_chars += text.len();
                }
                continue;
            }
        }

        // Paragraph budget
        if el.role == ElementRole::Paragraph {
            kept_paras += 1;
            if kept_paras > ctx.config.max_paragraphs {
                dropped_paras += 1;
                if let Some(text) = &el.text {
                    dropped_chars += text.len();
                }
                continue;
            }
        }

        result.push(el);
    }

    let total_dropped = dropped_paras + dropped_links + dropped_dupes + dropped_other;
    if total_dropped > 0 {
        let mut parts = Vec::new();
        if dropped_dupes > 0 {
            parts.push(format!("{} duplicate links", dropped_dupes));
        }
        if dropped_links > 0 {
            parts.push(format!("{} more links", dropped_links));
        }
        if dropped_paras > 0 {
            parts.push(format!("{} more paragraphs", dropped_paras));
        }
        if dropped_other > 0 {
            parts.push(format!("{} more elements", dropped_other));
        }
        let summary_text = format!("[{} dropped, ~{} chars]", parts.join(", "), dropped_chars);

        result.push(Element {
            id: format!("e_summary_{}", total_dropped),
            role: ElementRole::Paragraph,
            html_id: None,
            text: Some(summary_text),
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
        });
    }

    result
}

/// Normalize an href for deduplication purposes.
fn normalize_href(href: &str) -> String {
    let mut s = href.to_string();
    // Strip fragment
    if let Some(pos) = s.find('#') {
        s.truncate(pos);
    }
    // Strip trailing slash (but keep root "/")
    if s.len() > 1 && s.ends_with('/') {
        s.pop();
    }
    // Lowercase for case-insensitive dedup
    s.to_lowercase()
}

fn collect_regions(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    region_counts: &mut HashMap<String, usize>,
    regions: &mut Vec<Region>,
    unassigned: &mut Vec<Element>,
    dom_path: &str,
    ctx: &CompileContext,
) {
    if heuristics::should_strip(node) {
        return;
    }

    // Check CSS visibility rules
    if let NodeData::Element { name, attrs, .. } = &node.data {
        let tag = name.local.as_ref();
        let attrs_borrow = attrs.borrow();
        let class = attrs_borrow
            .iter()
            .find(|a| a.name.local.as_ref() == "class")
            .map(|a| a.value.to_string())
            .unwrap_or_default();
        let id = attrs_borrow
            .iter()
            .find(|a| a.name.local.as_ref() == "id")
            .map(|a| a.value.to_string())
            .unwrap_or_default();
        drop(attrs_borrow);
        if ctx.css_rules.is_hidden(tag, &class, &id) {
            return;
        }
    }

    if let NodeData::Element { name, .. } = &node.data {
        let tag = name.local.as_ref();
        let attr_pairs = get_attr_pairs(node);

        // Check for landmark / form region (HTML5 and ARIA)
        if let Some(role_str) = heuristics::landmark_role(tag, &attr_pairs) {
            create_landmark_region(
                node,
                role_str,
                origin,
                id_tracker,
                region_counts,
                regions,
                dom_path,
                ctx,
                &attr_pairs,
            );
            return;
        }

        // Check for class/id-based region hints (when no explicit landmarks)
        if matches!(tag, "div" | "section" | "article" | "ul" | "ol") {
            if let Some(role_str) = heuristics::class_id_region_hint(&attr_pairs) {
                create_landmark_region(
                    node,
                    role_str,
                    origin,
                    id_tracker,
                    region_counts,
                    regions,
                    dom_path,
                    ctx,
                    &attr_pairs,
                );
                return;
            }
        }

        // Check for form regions
        if heuristics::is_form_region(tag) {
            let count = region_counts.entry("form".to_string()).or_insert(0);
            let rid = generate_region_id("form", *count);
            *count += 1;
            let label = heuristics::get_accessible_label(&attr_pairs).or_else(|| {
                attr_pairs
                    .iter()
                    .find(|(n, _)| n == "name" || n == "id")
                    .map(|(_, v)| v.clone())
            });
            let form_action = attr_pairs
                .iter()
                .find(|(n, _)| n == "action")
                .map(|(_, v)| v.clone());
            let form_method = attr_pairs
                .iter()
                .find(|(n, _)| n == "method")
                .map(|(_, v)| v.to_uppercase());
            let mut elements = Vec::new();
            extract_elements(node, origin, id_tracker, &mut elements, dom_path, ctx);
            if !elements.is_empty() {
                regions.push(Region {
                    id: rid,
                    role: RegionRole::Form,
                    label,
                    action: form_action,
                    method: form_method,
                    elements: summarize_elements(elements, ctx, false),
                });
            }
            return;
        }

        // Check for layout tables - decompose them instead of treating as data tables
        if tag == "table" && heuristics::is_layout_table(node) {
            // Decompose layout table: extract children as if the table weren't there
            extract_layout_table_contents(
                node,
                origin,
                id_tracker,
                region_counts,
                regions,
                unassigned,
                dom_path,
                ctx,
            );
            return;
        }

        // Check for improved wrapper divs - collapse them
        if heuristics::is_collapsible_wrapper(tag, node) {
            let children = node.children.borrow();
            for (i, child) in children.iter().enumerate() {
                let child_path = format!("{}/{}", dom_path, i);
                collect_regions(
                    child,
                    origin,
                    id_tracker,
                    region_counts,
                    regions,
                    unassigned,
                    &child_path,
                    ctx,
                );
            }
            return;
        }

        // Check if this subtree looks like navigation (heuristic)
        if matches!(tag, "div" | "ul" | "ol") {
            let link_count = count_links(node);
            let children = node.children.borrow();
            let direct_children = children.len();
            drop(children);
            if heuristics::looks_like_navigation(link_count, direct_children)
                && !contains_descendant_tag(node, &["main", "article"], 6)
            {
                let count = region_counts.entry("navigation".to_string()).or_insert(0);
                let rid = generate_region_id("navigation", *count);
                *count += 1;
                let mut elements = Vec::new();
                extract_elements(node, origin, id_tracker, &mut elements, dom_path, ctx);
                if !elements.is_empty() {
                    regions.push(Region {
                        id: rid,
                        role: RegionRole::Navigation,
                        label: None,
                        action: None,
                        method: None,
                        elements: summarize_elements(elements, ctx, true),
                    });
                }
                return;
            }
        }

        // Check for footer heuristic (last block with copyright/terms/privacy)
        if matches!(tag, "div" | "section")
            && heuristics::looks_like_footer(node)
            && !contains_descendant_tag(node, &["main", "article"], 6)
        {
            let count = region_counts.entry("footer".to_string()).or_insert(0);
            let rid = generate_region_id("footer", *count);
            *count += 1;
            let mut elements = Vec::new();
            extract_elements(node, origin, id_tracker, &mut elements, dom_path, ctx);
            if !elements.is_empty() {
                regions.push(Region {
                    id: rid,
                    role: RegionRole::Footer,
                    label: None,
                    action: None,
                    method: None,
                    elements: summarize_elements(elements, ctx, false),
                });
            }
            return;
        }

        // Not a region - try to convert this element to a SOM element
        // (e.g., <a>, <button>, <input>, <h1>, <p>, <img>, etc.)
        if let Some(el) = node_to_element(node, origin, id_tracker, dom_path, ctx) {
            // For non-interactive elements, also extract interactive children
            if !el.role.is_interactive() {
                let mut child_interactive = Vec::new();
                let children = node.children.borrow();
                for (i, child) in children.iter().enumerate() {
                    let child_path = format!("{}/{}", dom_path, i);
                    extract_interactive_children(
                        child,
                        origin,
                        id_tracker,
                        &mut child_interactive,
                        &child_path,
                    );
                }
                if !child_interactive.is_empty() {
                    unassigned.push(el);
                    unassigned.extend(child_interactive);
                    return;
                }
            }
            unassigned.push(el);
            return;
        }

        // If it's not a SOM element, recurse into children
        let children = node.children.borrow();
        for (i, child) in children.iter().enumerate() {
            let child_path = format!("{}/{}", dom_path, i);
            collect_regions(
                child,
                origin,
                id_tracker,
                region_counts,
                regions,
                unassigned,
                &child_path,
                ctx,
            );
        }
        return;
    }

    // For non-element nodes (text, etc.), try to extract
    if let Some(el) = node_to_element(node, origin, id_tracker, dom_path, ctx) {
        unassigned.push(el);
    }
}

/// Helper to create a landmark region from detected role.
fn create_landmark_region(
    node: &Handle,
    role_str: &str,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    region_counts: &mut HashMap<String, usize>,
    regions: &mut Vec<Region>,
    dom_path: &str,
    ctx: &CompileContext,
    attr_pairs: &[(String, String)],
) {
    let region_role = match role_str {
        "navigation" => RegionRole::Navigation,
        "main" => RegionRole::Main,
        "aside" => RegionRole::Aside,
        "header" => RegionRole::Header,
        "footer" => RegionRole::Footer,
        "dialog" => RegionRole::Dialog,
        "search" => RegionRole::Navigation, // Search is a form of navigation
        _ => RegionRole::Content,
    };
    let count = region_counts.entry(role_str.to_string()).or_insert(0);
    let rid = generate_region_id(role_str, *count);
    *count += 1;
    let label = heuristics::get_accessible_label(attr_pairs);

    // Track if we're in main region for content summarization
    let was_main = ctx.is_main_region.get();
    if region_role == RegionRole::Main {
        ctx.is_main_region.set(true);
        ctx.paragraph_count.set(0); // Reset paragraph count for main region
    }

    // Recursively collect sub-regions (forms, nested landmarks) and elements
    let mut sub_elements = Vec::new();
    let children = node.children.borrow();
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{}/{}", dom_path, i);
        collect_regions(
            child,
            origin,
            id_tracker,
            region_counts,
            regions,
            &mut sub_elements,
            &child_path,
            ctx,
        );
    }

    ctx.is_main_region.set(was_main);

    if !sub_elements.is_empty() {
        regions.push(Region {
            id: rid,
            role: region_role.clone(),
            label,
            action: None,
            method: None,
            elements: summarize_elements(sub_elements, ctx, region_role == RegionRole::Navigation),
        });
    }
}

/// Extract contents from a layout table, treating it as a container rather than data.
fn extract_layout_table_contents(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    region_counts: &mut HashMap<String, usize>,
    regions: &mut Vec<Region>,
    unassigned: &mut Vec<Element>,
    dom_path: &str,
    ctx: &CompileContext,
) {
    // Recursively process table contents, extracting semantic elements
    fn visit_layout_table(
        node: &Handle,
        origin: &str,
        id_tracker: &mut ElementIdTracker,
        region_counts: &mut HashMap<String, usize>,
        regions: &mut Vec<Region>,
        unassigned: &mut Vec<Element>,
        dom_path: &str,
        ctx: &CompileContext,
    ) {
        let children = node.children.borrow();
        for (i, child) in children.iter().enumerate() {
            let child_path = format!("{}/{}", dom_path, i);
            if let NodeData::Element { name, .. } = &child.data {
                let tag = name.local.as_ref();
                match tag {
                    "table" | "tbody" | "thead" | "tfoot" | "tr" | "td" | "th" => {
                        // Continue recursing through table structure
                        visit_layout_table(
                            child,
                            origin,
                            id_tracker,
                            region_counts,
                            regions,
                            unassigned,
                            &child_path,
                            ctx,
                        );
                    }
                    _ => {
                        // Found non-table element, process normally
                        collect_regions(
                            child,
                            origin,
                            id_tracker,
                            region_counts,
                            regions,
                            unassigned,
                            &child_path,
                            ctx,
                        );
                    }
                }
            } else {
                // Text or other nodes
                collect_regions(
                    child,
                    origin,
                    id_tracker,
                    region_counts,
                    regions,
                    unassigned,
                    &child_path,
                    ctx,
                );
            }
        }
    }

    visit_layout_table(
        node,
        origin,
        id_tracker,
        region_counts,
        regions,
        unassigned,
        dom_path,
        ctx,
    );
}

fn extract_elements(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    elements: &mut Vec<Element>,
    dom_path: &str,
    ctx: &CompileContext,
) {
    if heuristics::should_strip(node) {
        return;
    }

    // Check CSS visibility
    if let NodeData::Element { name, attrs, .. } = &node.data {
        let tag = name.local.as_ref();
        let attrs_borrow = attrs.borrow();
        let class = attrs_borrow
            .iter()
            .find(|a| a.name.local.as_ref() == "class")
            .map(|a| a.value.to_string())
            .unwrap_or_default();
        let id = attrs_borrow
            .iter()
            .find(|a| a.name.local.as_ref() == "id")
            .map(|a| a.value.to_string())
            .unwrap_or_default();
        drop(attrs_borrow);
        if ctx.css_rules.is_hidden(tag, &class, &id) {
            return;
        }
    }

    // Check for layout tables within regions
    if let NodeData::Element { name, .. } = &node.data {
        if name.local.as_ref() == "table" && heuristics::is_layout_table(node) {
            // Decompose layout table
            extract_layout_table_elements(node, origin, id_tracker, elements, dom_path, ctx);
            return;
        }
    }

    // Try to convert this node into an element
    if let Some(el) = node_to_element(node, origin, id_tracker, dom_path, ctx) {
        // For non-interactive container elements (paragraph, section, list),
        // also extract any interactive children they contain
        if !el.role.is_interactive() {
            let mut child_interactive = Vec::new();
            let children = node.children.borrow();
            for (i, child) in children.iter().enumerate() {
                let child_path = format!("{}/{}", dom_path, i);
                extract_interactive_children(
                    child,
                    origin,
                    id_tracker,
                    &mut child_interactive,
                    &child_path,
                );
            }
            if !child_interactive.is_empty() {
                // Add the paragraph/container first, then the interactive children
                elements.push(el);
                elements.extend(child_interactive);
                return;
            }
        }
        elements.push(el);
        return;
    }

    // Otherwise recurse into children
    let children = node.children.borrow();
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{}/{}", dom_path, i);
        extract_elements(child, origin, id_tracker, elements, &child_path, ctx);
    }
}

/// Extract elements from a layout table, treating it as a container.
fn extract_layout_table_elements(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    elements: &mut Vec<Element>,
    dom_path: &str,
    ctx: &CompileContext,
) {
    fn visit(
        node: &Handle,
        origin: &str,
        id_tracker: &mut ElementIdTracker,
        elements: &mut Vec<Element>,
        dom_path: &str,
        ctx: &CompileContext,
    ) {
        let children = node.children.borrow();
        for (i, child) in children.iter().enumerate() {
            let child_path = format!("{}/{}", dom_path, i);
            if let NodeData::Element { name, .. } = &child.data {
                let tag = name.local.as_ref();
                match tag {
                    "table" | "tbody" | "thead" | "tfoot" | "tr" | "td" | "th" => {
                        visit(child, origin, id_tracker, elements, &child_path, ctx);
                    }
                    _ => {
                        extract_elements(child, origin, id_tracker, elements, &child_path, ctx);
                    }
                }
            }
        }
    }
    visit(node, origin, id_tracker, elements, dom_path, ctx);
}

/// Extract only interactive elements from a subtree (for finding links inside paragraphs, etc.)
fn extract_interactive_children(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    elements: &mut Vec<Element>,
    dom_path: &str,
) {
    if heuristics::should_strip(node) {
        return;
    }
    if let NodeData::Element { name, .. } = &node.data {
        let tag = name.local.as_ref();
        let attr_pairs = get_attr_pairs(node);
        if let Some(role) = tag_to_role(tag, &attr_pairs) {
            if role.is_interactive() {
                if let Some(el) = interactive_node_to_element(node, origin, id_tracker, dom_path) {
                    elements.push(el);
                    return;
                }
            }
        }
    }
    let children = node.children.borrow();
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{}/{}", dom_path, i);
        extract_interactive_children(child, origin, id_tracker, elements, &child_path);
    }
}

/// Convert an interactive node to an element (without content summarization context).
fn interactive_node_to_element(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    dom_path: &str,
) -> Option<Element> {
    if let NodeData::Element { name, .. } = &node.data {
        let tag = name.local.as_ref();
        let attr_pairs = get_attr_pairs(node);
        let role = tag_to_role(tag, &attr_pairs)?;
        let text_content = get_text_content(node);
        let text = if text_content.is_empty() {
            None
        } else {
            Some(heuristics::normalize_text(&text_content))
        };
        let label = resolve_label(tag, &attr_pairs, &text);
        let accessible_name = label.as_deref().or(text.as_deref()).unwrap_or("");
        let raw_id = generate_element_id(origin, role.as_str(), accessible_name, dom_path);
        let id = id_tracker.register(raw_id);
        let actions = role.default_actions();
        let actions = if actions.is_empty() {
            None
        } else {
            Some(actions)
        };
        // Use a dummy context for attrs (interactive elements don't need list/para summarization)
        let dummy_ctx = CompileContext {
            config: ContentConfig::default(),
            paragraph_count: Cell::new(0),
            is_main_region: Cell::new(false),
            css_rules: VisibilityRules::default(),
        };
        let element_attrs = build_element_attrs(tag, &attr_pairs, node, &dummy_ctx);
        let children = build_children(node, origin, id_tracker, dom_path, &role);
        let hints = heuristics::infer_class_hints(&attr_pairs);
        let html_id = extract_html_id(&attr_pairs);

        return Some(Element {
            id,
            role,
            html_id,
            text,
            label,
            actions,
            attrs: element_attrs,
            children,
            hints,
        });
    }
    None
}

fn node_to_element(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    dom_path: &str,
    ctx: &CompileContext,
) -> Option<Element> {
    match &node.data {
        NodeData::Element { name, .. } => {
            let tag = name.local.as_ref();
            let attr_pairs = get_attr_pairs(node);

            // Treat layout tables as containers, not data.
            if tag == "table" && heuristics::is_layout_table(node) {
                return None;
            }

            let role = tag_to_role(tag, &attr_pairs)?;
            let text_content = get_text_content(node);

            // Apply summarization for paragraphs based on position
            let mut text = if text_content.is_empty() {
                None
            } else if role == ElementRole::Paragraph && ctx.is_main_region.get() {
                let para_num = ctx.paragraph_count.get();
                ctx.paragraph_count.set(para_num + 1);
                let max_len = if para_num == 0 {
                    ctx.config.first_para_max
                } else {
                    ctx.config.subsequent_para_max
                };
                Some(heuristics::truncate_text(&text_content, max_len))
            } else {
                Some(heuristics::normalize_text(&text_content))
            };

            // Avoid duplicating massive string content for structured container roles.
            if matches!(role, ElementRole::Table | ElementRole::List) {
                text = None;
            }

            let label = resolve_label(tag, &attr_pairs, &text);
            let accessible_name = label.as_deref().or(text.as_deref()).unwrap_or("");
            let raw_id = generate_element_id(origin, role.as_str(), accessible_name, dom_path);
            let id = id_tracker.register(raw_id);
            let actions = role.default_actions();
            let actions = if actions.is_empty() {
                None
            } else {
                Some(actions)
            };
            let element_attrs = build_element_attrs(tag, &attr_pairs, node, ctx);
            let children = build_children(node, origin, id_tracker, dom_path, &role);
            let hints = heuristics::infer_class_hints(&attr_pairs);
            let html_id = extract_html_id(&attr_pairs);

            Some(Element {
                id,
                role,
                html_id,
                text,
                label,
                actions,
                attrs: element_attrs,
                children,
                hints,
            })
        }
        NodeData::Text { contents } => {
            let text = heuristics::normalize_text(&contents.borrow());
            if text.is_empty() {
                return None;
            }
            let raw_id = generate_element_id(origin, "paragraph", &text, dom_path);
            let id = id_tracker.register(raw_id);
            Some(Element {
                id,
                role: ElementRole::Paragraph,
                html_id: None,
                text: Some(text),
                label: None,
                actions: None,
                attrs: None,
                children: None,
                hints: None,
            })
        }
        _ => None,
    }
}

fn tag_to_role(tag: &str, attrs: &[(String, String)]) -> Option<ElementRole> {
    // Check ARIA role attribute first
    for (name, value) in attrs {
        if name == "role" {
            return match value.as_str() {
                "button" => Some(ElementRole::Button),
                "link" => Some(ElementRole::Link),
                "checkbox" => Some(ElementRole::Checkbox),
                "radio" => Some(ElementRole::Radio),
                "img" => Some(ElementRole::Image),
                _ => None,
            };
        }
    }

    match tag {
        "a" => {
            // Only count as link if it has href
            if attrs.iter().any(|(n, _)| n == "href") {
                Some(ElementRole::Link)
            } else {
                None
            }
        }
        "button" => Some(ElementRole::Button),
        "input" => {
            let input_type = attrs
                .iter()
                .find(|(n, _)| n == "type")
                .map(|(_, v)| v.as_str())
                .unwrap_or("text");
            match input_type {
                "submit" | "button" | "reset" => Some(ElementRole::Button),
                "checkbox" => Some(ElementRole::Checkbox),
                "radio" => Some(ElementRole::Radio),
                "hidden" => None,
                "text" | "email" | "password" | "search" | "tel" | "url" | "number" | "date"
                | "time" | "datetime-local" | "month" | "week" | "color" => {
                    Some(ElementRole::TextInput)
                }
                _ => Some(ElementRole::TextInput),
            }
        }
        "textarea" => Some(ElementRole::Textarea),
        "select" => Some(ElementRole::Select),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some(ElementRole::Heading),
        "img" | "picture" => Some(ElementRole::Image),
        "ul" | "ol" => Some(ElementRole::List),
        "table" => Some(ElementRole::Table),
        "p" => Some(ElementRole::Paragraph),
        "section" | "article" => Some(ElementRole::Section),
        "hr" => Some(ElementRole::Separator),
        "details" => Some(ElementRole::Details),
        "iframe" => Some(ElementRole::Iframe),
        _ => None,
    }
}

fn resolve_label(_tag: &str, attrs: &[(String, String)], text: &Option<String>) -> Option<String> {
    // aria-label takes priority
    if let Some(label) = attrs.iter().find(|(n, _)| n == "aria-label") {
        if !label.1.is_empty() {
            let normalized = heuristics::normalize_text(&label.1);
            // Only return if different from text content
            if text.as_deref() != Some(&normalized) {
                return Some(normalized);
            }
        }
    }
    if let Some(title) = attrs.iter().find(|(n, _)| n == "title") {
        if !title.1.is_empty() {
            let normalized = heuristics::normalize_text(&title.1);
            if text.as_deref() != Some(&normalized) {
                return Some(normalized);
            }
        }
    }
    if let Some(ph) = attrs.iter().find(|(n, _)| n == "placeholder") {
        if !ph.1.is_empty() {
            return Some(heuristics::normalize_text(&ph.1));
        }
    }
    None
}

fn build_element_attrs(
    tag: &str,
    attrs: &[(String, String)],
    node: &Handle,
    ctx: &CompileContext,
) -> Option<serde_json::Value> {
    let mut map = serde_json::Map::new();

    match tag {
        "a" => {
            if let Some(href) = attrs.iter().find(|(n, _)| n == "href") {
                map.insert("href".into(), json!(href.1));
            }
        }
        "input" => {
            let input_type = attrs
                .iter()
                .find(|(n, _)| n == "type")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "text".to_string());

            if matches!(
                input_type.as_str(),
                "text" | "email" | "password" | "search" | "tel" | "url" | "number"
            ) {
                map.insert("input_type".into(), json!(input_type));
            }
            if let Some(v) = attrs.iter().find(|(n, _)| n == "value") {
                map.insert("value".into(), json!(v.1));
            }
            if let Some(ph) = attrs.iter().find(|(n, _)| n == "placeholder") {
                map.insert("placeholder".into(), json!(ph.1));
            }
            if attrs.iter().any(|(n, _)| n == "required") {
                map.insert("required".into(), json!(true));
            }
            if attrs.iter().any(|(n, _)| n == "disabled") {
                map.insert("disabled".into(), json!(true));
            }
            if input_type == "checkbox" || input_type == "radio" {
                if attrs.iter().any(|(n, _)| n == "checked") {
                    map.insert("checked".into(), json!(true));
                }
            }
            if input_type == "radio" {
                if let Some(name) = attrs.iter().find(|(n, _)| n == "name") {
                    map.insert("group".into(), json!(name.1));
                }
            }
        }
        "textarea" => {
            if let Some(ph) = attrs.iter().find(|(n, _)| n == "placeholder") {
                map.insert("placeholder".into(), json!(ph.1));
            }
            if attrs.iter().any(|(n, _)| n == "required") {
                map.insert("required".into(), json!(true));
            }
        }
        "button" => {
            if attrs.iter().any(|(n, _)| n == "disabled") {
                map.insert("disabled".into(), json!(true));
            }
        }
        "select" => {
            let options = extract_select_options(node);
            if !options.is_empty() {
                map.insert("options".into(), json!(options));
            }
            if attrs.iter().any(|(n, _)| n == "multiple") {
                map.insert("multiple".into(), json!(true));
            }
            if attrs.iter().any(|(n, _)| n == "required") {
                map.insert("required".into(), json!(true));
            }
        }
        "h1" => {
            map.insert("level".into(), json!(1));
        }
        "h2" => {
            map.insert("level".into(), json!(2));
        }
        "h3" => {
            map.insert("level".into(), json!(3));
        }
        "h4" => {
            map.insert("level".into(), json!(4));
        }
        "h5" => {
            map.insert("level".into(), json!(5));
        }
        "h6" => {
            map.insert("level".into(), json!(6));
        }
        "img" | "picture" => {
            if let Some(alt) = attrs.iter().find(|(n, _)| n == "alt") {
                map.insert("alt".into(), json!(alt.1));
            }
            if let Some(src) = attrs.iter().find(|(n, _)| n == "src") {
                map.insert("src".into(), json!(src.1));
            }
        }
        "ul" => {
            map.insert("ordered".into(), json!(false));
            let items = extract_list_items_with_limit(node, ctx.config.max_list_items);
            if !items.is_empty() {
                map.insert("items".into(), json!(items));
            }
        }
        "ol" => {
            map.insert("ordered".into(), json!(true));
            let items = extract_list_items_with_limit(node, ctx.config.max_list_items);
            if !items.is_empty() {
                map.insert("items".into(), json!(items));
            }
        }
        "table" => {
            if let Some(caption) = extract_table_caption(node) {
                map.insert("caption".into(), json!(caption));
            }
            let (headers, rows) = extract_table_data(node, ctx.config.max_table_cell_chars);
            if !headers.is_empty() {
                map.insert("headers".into(), json!(headers));
            }
            if !rows.is_empty() {
                map.insert("rows".into(), json!(rows));
            }
        }
        "section" | "article" => {
            if let Some(label) = attrs.iter().find(|(n, _)| n == "aria-label") {
                map.insert("section_label".into(), json!(label.1));
            }
        }
        "details" => {
            let open = attrs.iter().any(|(n, _)| n == "open");
            map.insert("open".into(), json!(open));
            // Extract summary text from the first <summary> child
            let summary_text = extract_summary_text(node);
            if let Some(st) = summary_text {
                map.insert("summary".into(), json!(st));
            }
        }
        "iframe" => {
            // Core iframe attributes for agents
            if let Some(src) = attrs.iter().find(|(n, _)| n == "src") {
                map.insert("src".into(), json!(src.1));
            }
            if let Some(srcdoc) = attrs.iter().find(|(n, _)| n == "srcdoc") {
                // For srcdoc, we just note it exists (content is inline HTML)
                map.insert("has_srcdoc".into(), json!(true));
                // Optionally extract a preview of the srcdoc content
                let preview: String = srcdoc.1.chars().take(200).collect();
                if !preview.is_empty() {
                    map.insert("srcdoc_preview".into(), json!(preview));
                }
            }
            if let Some(name) = attrs.iter().find(|(n, _)| n == "name") {
                map.insert("name".into(), json!(name.1));
            }
            if let Some(sandbox) = attrs.iter().find(|(n, _)| n == "sandbox") {
                map.insert("sandbox".into(), json!(sandbox.1));
            }
            if let Some(allow) = attrs.iter().find(|(n, _)| n == "allow") {
                map.insert("allow".into(), json!(allow.1));
            }
            // Dimensions can be useful for understanding iframe purpose
            if let Some(width) = attrs.iter().find(|(n, _)| n == "width") {
                map.insert("width".into(), json!(width.1));
            }
            if let Some(height) = attrs.iter().find(|(n, _)| n == "height") {
                map.insert("height".into(), json!(height.1));
            }
        }
        _ => {}
    }

    // ARIA state preservation: capture common ARIA state attributes
    let aria_states: &[(&str, &str)] = &[
        ("aria-expanded", "expanded"),
        ("aria-selected", "selected"),
        ("aria-checked", "checked"),
        ("aria-disabled", "disabled"),
        ("aria-current", "current"),
        ("aria-pressed", "pressed"),
        ("aria-hidden", "hidden"),
    ];
    let mut aria_map = serde_json::Map::new();
    for (html_attr, som_key) in aria_states {
        if let Some((_, val)) = attrs.iter().find(|(n, _)| n == *html_attr) {
            // Boolean ARIA attrs: "true"/"false" -> bool; others kept as string
            match val.as_str() {
                "true" => {
                    aria_map.insert((*som_key).into(), json!(true));
                }
                "false" => {
                    aria_map.insert((*som_key).into(), json!(false));
                }
                other => {
                    // e.g. aria-current="page", aria-checked="mixed"
                    aria_map.insert((*som_key).into(), json!(other));
                }
            }
        }
    }
    if !aria_map.is_empty() {
        map.insert("aria".into(), serde_json::Value::Object(aria_map));
    }

    if map.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(map))
    }
}

fn build_children(
    _node: &Handle,
    _origin: &str,
    _id_tracker: &mut ElementIdTracker,
    _dom_path: &str,
    parent_role: &ElementRole,
) -> Option<Vec<Element>> {
    // Only build children for certain roles
    if !matches!(
        parent_role,
        ElementRole::List | ElementRole::Table | ElementRole::Section
    ) {
        return None;
    }
    // For lists and tables, we include items/rows in attrs instead of children
    None
}

/// Extract text from the first `<summary>` child of a `<details>` element.
fn extract_summary_text(node: &Handle) -> Option<String> {
    for child in node.children.borrow().iter() {
        if let NodeData::Element { name, .. } = &child.data {
            if name.local.as_ref() == "summary" {
                let text = get_text_content(child);
                let trimmed = heuristics::normalize_text(&text);
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
        }
    }
    None
}

fn extract_select_options(node: &Handle) -> Vec<serde_json::Value> {
    let mut options = Vec::new();
    let children = node.children.borrow();
    for child in children.iter() {
        if let NodeData::Element { name, attrs, .. } = &child.data {
            if name.local.as_ref() == "option" {
                let attrs = attrs.borrow();
                let value = attrs
                    .iter()
                    .find(|a| a.name.local.as_ref() == "value")
                    .map(|a| a.value.to_string())
                    .unwrap_or_default();
                let text = get_text_content(child);
                let selected = attrs.iter().any(|a| a.name.local.as_ref() == "selected");
                let mut opt = serde_json::Map::new();
                opt.insert("value".into(), json!(value));
                opt.insert("text".into(), json!(heuristics::normalize_text(&text)));
                if selected {
                    opt.insert("selected".into(), json!(true));
                }
                options.push(serde_json::Value::Object(opt));
            } else if name.local.as_ref() == "optgroup" {
                // Recurse into optgroup
                let group_opts = extract_select_options(child);
                options.extend(group_opts);
            }
        }
    }
    options
}

fn extract_list_items_with_limit(node: &Handle, max_items: usize) -> Vec<serde_json::Value> {
    let mut items = Vec::new();
    let mut total_count = 0;
    let children = node.children.borrow();

    for child in children.iter() {
        if let NodeData::Element { name, .. } = &child.data {
            if name.local.as_ref() == "li" {
                let text = get_text_content(child);
                if !text.trim().is_empty() {
                    total_count += 1;
                    if items.len() < max_items {
                        let mut item = serde_json::Map::new();
                        item.insert("text".into(), json!(heuristics::normalize_text(&text)));
                        items.push(serde_json::Value::Object(item));
                    }
                }
            }
        }
    }

    // Add summary if items were truncated
    if total_count > max_items {
        let remaining = total_count - max_items;
        let mut summary = serde_json::Map::new();
        summary.insert("text".into(), json!(format!("[{} more items]", remaining)));
        items.push(serde_json::Value::Object(summary));
    }

    items
}

fn extract_table_caption(node: &Handle) -> Option<String> {
    for child in node.children.borrow().iter() {
        if let NodeData::Element { name, .. } = &child.data {
            if name.local.as_ref() == "caption" {
                let text = heuristics::normalize_text(&get_text_content(child));
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn extract_table_data(node: &Handle, max_cell_chars: usize) -> (Vec<String>, Vec<Vec<String>>) {
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let max_columns = 12;
    let max_rows = 30;

    fn visit_table(
        node: &Handle,
        headers: &mut Vec<String>,
        rows: &mut Vec<Vec<String>>,
        max_cell_chars: usize,
        max_columns: usize,
        max_rows: usize,
    ) {
        let children = node.children.borrow();
        for child in children.iter() {
            if let NodeData::Element { name, .. } = &child.data {
                let tag = name.local.as_ref();
                match tag {
                    "thead" | "tbody" | "tfoot" => {
                        visit_table(child, headers, rows, max_cell_chars, max_columns, max_rows);
                    }
                    "tr" => {
                        let cells = child.children.borrow();
                        let mut row = Vec::new();
                        let mut is_header_row = true;
                        for cell in cells.iter() {
                            if let NodeData::Element { name, attrs, .. } = &cell.data {
                                let cell_tag = name.local.as_ref();
                                if row.len() >= max_columns {
                                    break;
                                }
                                let cell_text = heuristics::truncate_text(
                                    &get_text_content(cell),
                                    max_cell_chars,
                                );
                                // Handle colspan: repeat the cell text for spanned columns
                                let colspan: usize = attrs
                                    .borrow()
                                    .iter()
                                    .find(|a| a.name.local.as_ref() == "colspan")
                                    .and_then(|a| a.value.parse().ok())
                                    .unwrap_or(1)
                                    .min(max_columns - row.len());
                                if cell_tag == "th" {
                                    for _ in 0..colspan {
                                        if row.len() < max_columns {
                                            row.push(cell_text.clone());
                                        }
                                    }
                                } else if cell_tag == "td" {
                                    is_header_row = false;
                                    for _ in 0..colspan {
                                        if row.len() < max_columns {
                                            row.push(cell_text.clone());
                                        }
                                    }
                                }
                            }
                        }
                        if !row.is_empty() {
                            if is_header_row && headers.is_empty() {
                                headers.extend(row);
                            } else if rows.len() < max_rows {
                                rows.push(row);
                            }
                        }
                    }
                    _ => {
                        visit_table(child, headers, rows, max_cell_chars, max_columns, max_rows);
                    }
                }
            }
        }
    }

    visit_table(
        node,
        &mut headers,
        &mut rows,
        max_cell_chars,
        max_columns,
        max_rows,
    );
    (headers, rows)
}

/// Get text content from a node, recursively collecting all text nodes.
pub fn get_text_content(node: &Handle) -> String {
    let mut text = String::new();
    collect_text(node, &mut text);
    text
}

fn collect_text(node: &Handle, buf: &mut String) {
    match &node.data {
        NodeData::Text { contents } => {
            buf.push_str(&contents.borrow());
        }
        NodeData::Element { name, .. } => {
            let tag = name.local.as_ref();
            if matches!(tag, "script" | "style" | "noscript") {
                return;
            }
            for child in node.children.borrow().iter() {
                collect_text(child, buf);
            }
        }
        _ => {
            for child in node.children.borrow().iter() {
                collect_text(child, buf);
            }
        }
    }
}

fn get_attr_pairs(node: &Handle) -> Vec<(String, String)> {
    if let NodeData::Element { attrs, .. } = &node.data {
        attrs
            .borrow()
            .iter()
            .map(|a| (a.name.local.to_string(), a.value.to_string()))
            .collect()
    } else {
        vec![]
    }
}

/// Extract the HTML `id` attribute from attr pairs, if present and non-empty.
fn extract_html_id(attrs: &[(String, String)]) -> Option<String> {
    attrs
        .iter()
        .find(|(k, _)| k == "id")
        .map(|(_, v)| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn count_links(node: &Handle) -> usize {
    let mut count = 0;
    if let NodeData::Element { name, attrs, .. } = &node.data {
        if name.local.as_ref() == "a"
            && attrs
                .borrow()
                .iter()
                .any(|a| a.name.local.as_ref() == "href")
        {
            count += 1;
        }
    }
    for child in node.children.borrow().iter() {
        count += count_links(child);
    }
    count
}

fn contains_descendant_tag(node: &Handle, tags: &[&str], max_depth: usize) -> bool {
    fn visit(node: &Handle, tags: &[&str], depth: usize, max_depth: usize) -> bool {
        if depth > max_depth {
            return false;
        }
        if let NodeData::Element { name, .. } = &node.data {
            let tag = name.local.as_ref();
            if tags.iter().any(|t| *t == tag) {
                return true;
            }
        }
        for child in node.children.borrow().iter() {
            if visit(child, tags, depth + 1, max_depth) {
                return true;
            }
        }
        false
    }

    visit(node, tags, 0, max_depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_page() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Test Page</title></head>
<body>
<nav>
  <a href="/">Home</a>
  <a href="/about">About</a>
  <a href="/contact">Contact</a>
</nav>
<main>
  <h1>Welcome</h1>
  <p>Hello world</p>
  <a href="/more">Learn more</a>
</main>
</body>
</html>"#;

        let som = compile(html, "https://example.com/page").unwrap();
        assert_eq!(som.title, "Test Page");
        assert_eq!(som.lang, "en");
        assert!(som.regions.len() >= 2); // nav + main at minimum
        assert!(som.meta.som_bytes > 0);
        assert!(som.meta.element_count > 0);
        assert!(som.meta.interactive_count > 0);
    }

    #[test]
    fn test_form_region() {
        let html = r#"<!DOCTYPE html>
<html><head><title>Login</title></head>
<body>
<form action="/login" method="POST" aria-label="Login form">
  <input type="email" placeholder="Email" required>
  <input type="password" placeholder="Password" required>
  <button type="submit">Sign In</button>
</form>
</body>
</html>"#;

        let som = compile(html, "https://example.com").unwrap();
        let form_region = som.regions.iter().find(|r| r.role == RegionRole::Form);
        assert!(form_region.is_some());
        let form = form_region.unwrap();
        assert_eq!(form.label.as_deref(), Some("Login form"));
        assert_eq!(form.action.as_deref(), Some("/login"));
        assert_eq!(form.method.as_deref(), Some("POST"));
        assert!(form.elements.len() >= 3);
    }

    #[test]
    fn test_strips_scripts_and_styles() {
        let html = r#"<!DOCTYPE html>
<html><head><title>Test</title><style>body{color:red}</style></head>
<body>
<script>alert('hi')</script>
<p>Visible content</p>
<noscript>No JS</noscript>
</body>
</html>"#;

        let som = compile(html, "https://example.com").unwrap();
        let json = serde_json::to_string(&som).unwrap();
        assert!(!json.contains("alert"));
        assert!(!json.contains("color:red"));
        assert!(json.contains("Visible content"));
    }

    #[test]
    fn test_token_reduction() {
        // Real-world pages have lots of CSS classes, data attributes, scripts, etc.
        // that SOM strips out. Build a more realistic test page.
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <title>Big Page</title>
  <style>.nav{display:flex;justify-content:space-between;align-items:center;padding:0 20px;background:#fff;border-bottom:1px solid #eee}.btn{background:blue;color:white;padding:8px 16px;border:none;border-radius:4px;cursor:pointer}.btn:hover{opacity:0.9}.card{border:1px solid #ddd;border-radius:8px;padding:16px;margin:8px 0}.footer{background:#f5f5f5;padding:40px 20px;text-align:center}</style>
  <script>document.addEventListener('DOMContentLoaded', function() { console.log('loaded'); var analytics = window.analytics || []; analytics.track('pageview'); });</script>
  <script src="https://cdn.example.com/analytics.js" async defer></script>
  <script>window.dataLayer = window.dataLayer || []; function gtag(){dataLayer.push(arguments)} gtag('js', new Date()); gtag('config', 'GA-123456');</script>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="This is a test page for SOM compression">
  <link rel="stylesheet" href="/styles/main.css">
  <link rel="stylesheet" href="/styles/components.css">
  <link rel="icon" href="/favicon.ico">
</head>
<body>
"#,
        );
        // Add a realistic nav with wrapper divs and classes
        html.push_str(r#"<nav class="navbar navbar-expand-lg navbar-light bg-light" role="navigation" aria-label="Main navigation">
  <div class="container-fluid">
    <a class="navbar-brand" href="/" data-testid="logo">Home</a>
    <div class="collapse navbar-collapse" id="navbarNav">
      <ul class="navbar-nav me-auto mb-2 mb-lg-0">
"#);
        for i in 0..8 {
            html.push_str(&format!(
                r#"        <li class="nav-item"><a class="nav-link active" aria-current="page" href="/page-{}" data-analytics="nav-click-{}">Page {}</a></li>
"#,
                i, i, i
            ));
        }
        html.push_str("      </ul>\n    </div>\n  </div>\n</nav>\n");

        // Main content with lots of divs and classes
        html.push_str(
            r#"<main class="container mt-4" role="main">
  <div class="row">
    <div class="col-lg-8 col-md-12">
      <div class="content-wrapper" data-component="article-list">
"#,
        );
        for i in 0..15 {
            html.push_str(&format!(
                r#"        <div class="card mb-3 shadow-sm" data-article-id="{}">
          <div class="card-body">
            <h2 class="card-title h5"><a href="/article/{}" class="text-decoration-none stretched-link" data-track="article-click">Article title number {}</a></h2>
            <p class="card-text text-muted">This is some descriptive text for article {}. It contains enough words to be realistic.</p>
            <div class="d-flex justify-content-between align-items-center">
              <small class="text-muted">5 min read</small>
              <span class="badge bg-secondary">Technology</span>
            </div>
          </div>
        </div>
"#,
                i, i, i, i
            ));
        }
        html.push_str("      </div>\n    </div>\n  </div>\n</main>\n");

        // Footer
        html.push_str(r#"<footer class="footer mt-auto py-3 bg-light">
  <div class="container text-center">
    <span class="text-muted">Copyright 2026 Example Corp</span>
    <div class="mt-2">
      <a href="/privacy" class="text-muted me-3">Privacy</a>
      <a href="/terms" class="text-muted me-3">Terms</a>
      <a href="/contact" class="text-muted">Contact</a>
    </div>
  </div>
</footer>
<script>var observer=new IntersectionObserver(function(e){e.forEach(function(e){e.isIntersecting&&analytics.track('viewed')})});document.querySelectorAll('.card').forEach(function(e){observer.observe(e)});</script>
</body>
</html>"#);

        let som = compile(&html, "https://example.com").unwrap();
        let ratio = som.meta.html_bytes as f64 / som.meta.som_bytes as f64;
        assert!(
            ratio >= 1.2,
            "SOM should be smaller than HTML even for small synthetic pages. HTML: {} bytes, SOM: {} bytes, Ratio: {:.1}x",
            som.meta.html_bytes,
            som.meta.som_bytes,
            ratio
        );
    }

    #[test]
    fn test_deterministic_ids() {
        let html = r#"<!DOCTYPE html>
<html><head><title>Test</title></head>
<body><main><a href="/link">Click me</a></main></body>
</html>"#;

        let som1 = compile(html, "https://example.com").unwrap();
        let som2 = compile(html, "https://example.com").unwrap();

        let json1 = serde_json::to_string(&som1).unwrap();
        let json2 = serde_json::to_string(&som2).unwrap();
        assert_eq!(json1, json2, "Same input should produce identical SOM");
    }
}
