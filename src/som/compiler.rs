use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use serde_json::json;
use std::cell::Cell;
use std::collections::HashMap;

use super::element_id::{generate_element_id, generate_region_id, ElementIdTracker};
use super::heuristics;
use super::types::*;

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

    // Extract page title and lang
    let title = extract_title(&dom.document);
    let lang = extract_lang(&dom.document);

    // Find the body node
    let body = find_body(&dom.document);

    let regions = match body {
        Some(body_handle) => {
            extract_regions(&body_handle, &origin, &mut id_tracker, &mut region_counts)
        }
        None => vec![],
    };

    // Count elements
    let element_count = Cell::new(0usize);
    let interactive_count = Cell::new(0usize);
    count_elements(&regions, &element_count, &interactive_count);

    let som = Som {
        som_version: "0.1".to_string(),
        url: page_url.to_string(),
        title,
        lang,
        regions,
        meta: SomMeta {
            html_bytes,
            som_bytes: 0, // Will be updated after serialization
            element_count: element_count.get(),
            interactive_count: interactive_count.get(),
        },
    };

    // Calculate actual SOM bytes
    let som_json = serde_json::to_string(&som).unwrap_or_default();
    let som_bytes = som_json.len();

    Ok(Som {
        meta: SomMeta {
            som_bytes,
            ..som.meta
        },
        ..som
    })
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
        &"0".to_string(),
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
            elements: unassigned_elements,
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
            elements: unassigned_elements,
        });
    }

    // Filter out empty regions
    regions.retain(|r| !r.elements.is_empty());
    regions
}

fn collect_regions(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    region_counts: &mut HashMap<String, usize>,
    regions: &mut Vec<Region>,
    unassigned: &mut Vec<Element>,
    dom_path: &str,
) {
    if heuristics::should_strip(node) {
        return;
    }

    if let NodeData::Element { name, .. } = &node.data {
        let tag = name.local.as_ref();
        let attr_pairs = get_attr_pairs(node);

        // Check for landmark / form region
        if let Some(role_str) = heuristics::landmark_role(tag, &attr_pairs) {
            let region_role = match role_str {
                "navigation" => RegionRole::Navigation,
                "main" => RegionRole::Main,
                "aside" => RegionRole::Aside,
                "header" => RegionRole::Header,
                "footer" => RegionRole::Footer,
                "dialog" => RegionRole::Dialog,
                _ => RegionRole::Content,
            };
            let count = region_counts.entry(role_str.to_string()).or_insert(0);
            let rid = generate_region_id(role_str, *count);
            *count += 1;
            let label = heuristics::get_accessible_label(&attr_pairs);

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
                );
            }

            if !sub_elements.is_empty() {
                regions.push(Region {
                    id: rid,
                    role: region_role,
                    label,
                    action: None,
                    method: None,
                    elements: sub_elements,
                });
            }
            return;
        }

        // Check for form regions
        if heuristics::is_form_region(tag) {
            let count = region_counts.entry("form".to_string()).or_insert(0);
            let rid = generate_region_id("form", *count);
            *count += 1;
            let label = heuristics::get_accessible_label(&attr_pairs)
                .or_else(|| {
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
            extract_elements(node, origin, id_tracker, &mut elements, dom_path);
            if !elements.is_empty() {
                regions.push(Region {
                    id: rid,
                    role: RegionRole::Form,
                    label,
                    action: form_action,
                    method: form_method,
                    elements,
                });
            }
            return;
        }

        // Check for wrapper divs - collapse them
        let children = node.children.borrow();
        let element_children: Vec<_> = children
            .iter()
            .filter(|c| matches!(&c.data, NodeData::Element { .. }))
            .collect();

        if heuristics::is_wrapper_div(tag, element_children.len()) {
            // Just recurse into the single child
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
                );
            }
            return;
        }

        // Check if this subtree looks like navigation (heuristic)
        if matches!(tag, "div" | "ul" | "ol") {
            let link_count = count_links(node);
            let direct_children = children.len();
            if heuristics::looks_like_navigation(link_count, direct_children) {
                let count = region_counts.entry("navigation".to_string()).or_insert(0);
                let rid = generate_region_id("navigation", *count);
                *count += 1;
                let mut elements = Vec::new();
                extract_elements(node, origin, id_tracker, &mut elements, dom_path);
                if !elements.is_empty() {
                    regions.push(Region {
                        id: rid,
                        role: RegionRole::Navigation,
                        label: None,
                        action: None,
                        method: None,
                        elements,
                    });
                }
                return;
            }
        }

        // Not a region - try to convert this element to a SOM element
        // (e.g., <a>, <button>, <input>, <h1>, <p>, <img>, etc.)
        drop(children);
        if let Some(el) = node_to_element(node, origin, id_tracker, dom_path) {
            // For non-interactive elements, also extract interactive children
            if !el.role.is_interactive() {
                let mut child_interactive = Vec::new();
                let children = node.children.borrow();
                for (i, child) in children.iter().enumerate() {
                    let child_path = format!("{}/{}", dom_path, i);
                    extract_interactive_children(child, origin, id_tracker, &mut child_interactive, &child_path);
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
            );
        }
        return;
    }

    // For non-element nodes (text, etc.), try to extract
    if let Some(el) = node_to_element(node, origin, id_tracker, dom_path) {
        unassigned.push(el);
    }
}

fn extract_elements(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    elements: &mut Vec<Element>,
    dom_path: &str,
) {
    if heuristics::should_strip(node) {
        return;
    }

    // Try to convert this node into an element
    if let Some(el) = node_to_element(node, origin, id_tracker, dom_path) {
        // For non-interactive container elements (paragraph, section, list),
        // also extract any interactive children they contain
        if !el.role.is_interactive() {
            let mut child_interactive = Vec::new();
            let children = node.children.borrow();
            for (i, child) in children.iter().enumerate() {
                let child_path = format!("{}/{}", dom_path, i);
                extract_interactive_children(child, origin, id_tracker, &mut child_interactive, &child_path);
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
        extract_elements(child, origin, id_tracker, elements, &child_path);
    }
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
                if let Some(el) = node_to_element(node, origin, id_tracker, dom_path) {
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

fn node_to_element(
    node: &Handle,
    origin: &str,
    id_tracker: &mut ElementIdTracker,
    dom_path: &str,
) -> Option<Element> {
    match &node.data {
        NodeData::Element { name, .. } => {
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
            let accessible_name = label
                .as_deref()
                .or(text.as_deref())
                .unwrap_or("");
            let raw_id = generate_element_id(origin, role.as_str(), accessible_name, dom_path);
            let id = id_tracker.register(raw_id);
            let actions = role.default_actions();
            let actions = if actions.is_empty() {
                None
            } else {
                Some(actions)
            };
            let element_attrs = build_element_attrs(tag, &attr_pairs, node);
            let children = build_children(node, origin, id_tracker, dom_path, &role);

            Some(Element {
                id,
                role,
                text,
                label,
                actions,
                attrs: element_attrs,
                children,
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
                text: Some(text),
                label: None,
                actions: None,
                attrs: None,
                children: None,
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
        _ => None,
    }
}

fn resolve_label(
    _tag: &str,
    attrs: &[(String, String)],
    text: &Option<String>,
) -> Option<String> {
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
        "h1" => { map.insert("level".into(), json!(1)); }
        "h2" => { map.insert("level".into(), json!(2)); }
        "h3" => { map.insert("level".into(), json!(3)); }
        "h4" => { map.insert("level".into(), json!(4)); }
        "h5" => { map.insert("level".into(), json!(5)); }
        "h6" => { map.insert("level".into(), json!(6)); }
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
            let items = extract_list_items(node);
            if !items.is_empty() {
                map.insert("items".into(), json!(items));
            }
        }
        "ol" => {
            map.insert("ordered".into(), json!(true));
            let items = extract_list_items(node);
            if !items.is_empty() {
                map.insert("items".into(), json!(items));
            }
        }
        "table" => {
            let (headers, rows) = extract_table_data(node);
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
        _ => {}
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

fn extract_list_items(node: &Handle) -> Vec<serde_json::Value> {
    let mut items = Vec::new();
    let children = node.children.borrow();
    for child in children.iter() {
        if let NodeData::Element { name, .. } = &child.data {
            if name.local.as_ref() == "li" {
                let text = get_text_content(child);
                if !text.trim().is_empty() {
                    let mut item = serde_json::Map::new();
                    item.insert("text".into(), json!(heuristics::normalize_text(&text)));
                    items.push(serde_json::Value::Object(item));
                }
            }
        }
    }
    items
}

fn extract_table_data(node: &Handle) -> (Vec<String>, Vec<Vec<String>>) {
    let mut headers = Vec::new();
    let mut rows = Vec::new();

    fn visit_table(node: &Handle, headers: &mut Vec<String>, rows: &mut Vec<Vec<String>>) {
        let children = node.children.borrow();
        for child in children.iter() {
            if let NodeData::Element { name, .. } = &child.data {
                let tag = name.local.as_ref();
                match tag {
                    "thead" | "tbody" | "tfoot" => {
                        visit_table(child, headers, rows);
                    }
                    "tr" => {
                        let cells = child.children.borrow();
                        let mut row = Vec::new();
                        let mut is_header_row = true;
                        for cell in cells.iter() {
                            if let NodeData::Element { name, .. } = &cell.data {
                                let cell_tag = name.local.as_ref();
                                if cell_tag == "th" {
                                    row.push(heuristics::normalize_text(&get_text_content(cell)));
                                } else if cell_tag == "td" {
                                    is_header_row = false;
                                    row.push(heuristics::normalize_text(&get_text_content(cell)));
                                }
                            }
                        }
                        if !row.is_empty() {
                            if is_header_row && headers.is_empty() {
                                headers.extend(row);
                            } else if rows.len() < 20 {
                                rows.push(row);
                            }
                        }
                    }
                    _ => {
                        visit_table(child, headers, rows);
                    }
                }
            }
        }
    }

    visit_table(node, &mut headers, &mut rows);
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
        let mut html = String::from(r#"<!DOCTYPE html>
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
"#);
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
        html.push_str(r#"<main class="container mt-4" role="main">
  <div class="row">
    <div class="col-lg-8 col-md-12">
      <div class="content-wrapper" data-component="article-list">
"#);
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
