use markup5ever_rcdom::{Handle, NodeData};

/// Configuration for content summarization thresholds.
///
/// These defaults are tuned for token efficiency. The goal of the v0.1 PoC is to
/// preserve intent-relevant structure, not to reproduce full article text.
pub struct ContentConfig {
    /// Max characters for the first paragraph in main content.
    pub first_para_max: usize,
    /// Max characters for subsequent paragraphs.
    pub subsequent_para_max: usize,
    /// Max number of paragraph elements to keep per region.
    pub max_paragraphs: usize,
    /// Max list items to show before collapsing.
    pub max_list_items: usize,
    /// Max link elements to keep in a non-navigation region.
    pub max_links: usize,
    /// Max link elements to keep in a navigation region.
    pub max_navigation_links: usize,
    /// Max total elements to keep per region.
    pub max_elements: usize,
    /// Max characters per table cell string.
    pub max_table_cell_chars: usize,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            first_para_max: 200,
            subsequent_para_max: 80,
            max_paragraphs: 10,
            max_list_items: 5,
            max_links: 200,
            max_navigation_links: 80,
            max_elements: 400,
            max_table_cell_chars: 80,
        }
    }
}

/// Check if a node should be stripped from SOM output.
pub fn should_strip(node: &Handle) -> bool {
    match &node.data {
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.as_ref();
            // Strip script, style, noscript, template, comments
            if matches!(
                tag,
                "script" | "style" | "noscript" | "template" | "meta" | "link"
            ) {
                return true;
            }
            // Strip SVGs unless role=img with accessible name
            if tag == "svg" {
                let attrs = attrs.borrow();
                let has_role_img = attrs
                    .iter()
                    .any(|a| a.name.local.as_ref() == "role" && a.value.as_ref() == "img");
                let has_name = attrs.iter().any(|a| {
                    a.name.local.as_ref() == "aria-label" || a.name.local.as_ref() == "title"
                });
                return !(has_role_img && has_name);
            }
            // Check for hidden elements
            let attrs = attrs.borrow();
            if attrs
                .iter()
                .any(|a| a.name.local.as_ref() == "hidden")
            {
                return true;
            }
            if attrs
                .iter()
                .any(|a| a.name.local.as_ref() == "aria-hidden" && a.value.as_ref() == "true")
            {
                return true;
            }
            // Check inline style for display:none or visibility:hidden
            if let Some(style) = attrs.iter().find(|a| a.name.local.as_ref() == "style") {
                let style_val = style.value.to_lowercase();
                if style_val.contains("display:none")
                    || style_val.contains("display: none")
                    || style_val.contains("visibility:hidden")
                    || style_val.contains("visibility: hidden")
                {
                    return true;
                }
            }
            // Decorative images
            if tag == "img" {
                let is_decorative = attrs.iter().any(|a| {
                    (a.name.local.as_ref() == "alt" && a.value.as_ref().is_empty())
                        || (a.name.local.as_ref() == "role"
                            && a.value.as_ref() == "presentation")
                });
                if is_decorative {
                    return true;
                }
            }
            false
        }
        NodeData::Comment { .. } => true,
        _ => false,
    }
}

/// Determine if a tag is a landmark that defines a region.
pub fn landmark_role(tag: &str, attrs: &[(String, String)]) -> Option<&'static str> {
    // Check ARIA role first
    for (name, value) in attrs {
        if name == "role" {
            return match value.as_str() {
                "navigation" => Some("navigation"),
                "main" => Some("main"),
                "complementary" => Some("aside"),
                "banner" => Some("header"),
                "contentinfo" => Some("footer"),
                "dialog" | "alertdialog" => Some("dialog"),
                _ => None,
            };
        }
    }
    // Then HTML5 landmarks
    match tag {
        "nav" => Some("navigation"),
        "main" => Some("main"),
        "aside" => Some("aside"),
        "header" => Some("header"),
        "footer" => Some("footer"),
        "dialog" => Some("dialog"),
        _ => None,
    }
}

/// Check if an element is a form with content worth promoting to a region.
pub fn is_form_region(tag: &str) -> bool {
    tag == "form"
}

/// Heuristic: check if a node looks like a navigation region
/// (e.g., a list of links near the top of the page).
pub fn looks_like_navigation(link_count: usize, total_children: usize) -> bool {
    total_children > 0 && link_count >= 3 && (link_count as f64 / total_children as f64) > 0.5
}

/// Normalize and clean text content.
pub fn normalize_text(text: &str) -> String {
    let trimmed: String = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if trimmed.len() > 200 {
        // Smart truncation: prefer breaking at sentence boundaries
        smart_truncate(&trimmed, 200)
    } else {
        trimmed
    }
}

/// Smart truncation that prefers sentence boundaries over mid-word cuts.
/// Looks for the last sentence-ending punctuation before max_chars,
/// falls back to word boundary if no sentence end found.
pub fn smart_truncate(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let window = &text[..max_chars];

    // Try to find last sentence boundary (. ! ?)
    let sentence_end = window.rfind(|c: char| c == '.' || c == '!' || c == '?');
    if let Some(pos) = sentence_end {
        // Only use sentence boundary if it captures at least 40% of the budget
        if pos >= max_chars * 2 / 5 {
            return text[..=pos].to_string();
        }
    }

    // Fall back to word boundary
    let word_end = window.rfind(char::is_whitespace);
    if let Some(pos) = word_end {
        format!("{}...", &text[..pos])
    } else {
        format!("{}...", &text[..max_chars.saturating_sub(3)])
    }
}


/// Get the accessible label for an element from its attributes.
pub fn get_accessible_label(attrs: &[(String, String)]) -> Option<String> {
    // Priority: aria-label > title > placeholder
    for (name, value) in attrs {
        if name == "aria-label" && !value.is_empty() {
            return Some(normalize_text(value));
        }
    }
    for (name, value) in attrs {
        if name == "title" && !value.is_empty() {
            return Some(normalize_text(value));
        }
    }
    for (name, value) in attrs {
        if name == "placeholder" && !value.is_empty() {
            return Some(normalize_text(value));
        }
    }
    None
}

/// Detect if a table is used for layout purposes rather than data display.
/// Layout tables typically have no semantic structure (no <th>, no <caption>)
/// and contain layout elements like <nav>, <form>, <div>, etc.
pub fn is_layout_table(node: &Handle) -> bool {
    // Many legacy sites (HN, old forums) use tables for layout.
    // We want to treat those as containers, not as data tables.

    let mut has_th = false;
    let mut has_caption = false;
    let mut has_layout_children = false;
    let mut nested_table_count = 0;
    let mut cell_count = 0;

    check_table_structure(
        node,
        &mut has_th,
        &mut has_caption,
        &mut has_layout_children,
        &mut nested_table_count,
        &mut cell_count,
        0,
    );

    if has_th || has_caption {
        // Real data tables often have headers or captions.
        return false;
    }

    // Strong signals of layout.
    if nested_table_count > 0 {
        return true;
    }
    if has_layout_children {
        return true;
    }

    // Layout tables commonly use cellpadding/cellspacing/border/width/bgcolor.
    if has_layout_table_attributes(node) {
        return true;
    }

    let link_count = count_descendant_links(node);

    // High link density with no headers is almost always layout.
    if link_count >= 20 {
        return true;
    }

    // Very few cells suggests a wrapper.
    if cell_count <= 2 {
        return true;
    }

    false
}

fn has_layout_table_attributes(node: &Handle) -> bool {
    if let NodeData::Element { attrs, .. } = &node.data {
        let attrs = attrs.borrow();
        for a in attrs.iter() {
            let n = a.name.local.as_ref();
            if matches!(n, "cellpadding" | "cellspacing" | "border" | "width" | "bgcolor") {
                return true;
            }
        }
    }
    false
}

fn check_table_structure(
    node: &Handle,
    has_th: &mut bool,
    has_caption: &mut bool,
    has_layout_children: &mut bool,
    nested_table_count: &mut usize,
    cell_count: &mut usize,
    depth: usize,
) {
    if let NodeData::Element { name, .. } = &node.data {
        let tag = name.local.as_ref();

        match tag {
            "th" => *has_th = true,
            "caption" => *has_caption = true,
            "td" => *cell_count += 1,
            "table" if depth > 0 => *nested_table_count += 1,
            "nav" | "form" | "header" | "footer" | "aside" | "main" | "article" | "section" => {
                *has_layout_children = true;
            }
            "div" => {
                // Check if div has significant content (links, forms, etc.)
                let link_count = count_descendant_links(node);
                if link_count >= 2 {
                    *has_layout_children = true;
                }
            }
            _ => {}
        }

        // Don't recurse into nested tables for structure check
        if tag == "table" && depth > 0 {
            return;
        }
    }

    let children = node.children.borrow();
    for child in children.iter() {
        check_table_structure(
            child,
            has_th,
            has_caption,
            has_layout_children,
            nested_table_count,
            cell_count,
            depth + 1
        );
    }
}

fn count_descendant_links(node: &Handle) -> usize {
    let mut count = 0;
    if let NodeData::Element { name, attrs, .. } = &node.data {
        if name.local.as_ref() == "a" {
            let attrs = attrs.borrow();
            if attrs.iter().any(|a| a.name.local.as_ref() == "href") {
                count += 1;
            }
        }
    }
    for child in node.children.borrow().iter() {
        count += count_descendant_links(child);
    }
    count
}

/// Check class/id attributes for semantic hints.
/// Returns a region role string if the class/id suggests a semantic region.
pub fn class_id_region_hint(attrs: &[(String, String)]) -> Option<&'static str> {
    let class_val = attrs.iter()
        .find(|(n, _)| n == "class")
        .map(|(_, v)| v.to_lowercase());

    let id_val = attrs.iter()
        .find(|(n, _)| n == "id")
        .map(|(_, v)| v.to_lowercase());

    // Check patterns in class and id
    for val in [&class_val, &id_val].iter().filter_map(|v| v.as_ref()) {
        // Navigation patterns
        if val.contains("nav") || val.contains("menu") || val.contains("navigation") {
            return Some("navigation");
        }
        // Main content patterns
        if val.contains("main-content") || val.contains("maincontent") ||
           val.contains("primary-content") || val.contains("article-body") ||
           (val.contains("main") && !val.contains("nav")) {
            return Some("main");
        }
        // Sidebar patterns
        if val.contains("sidebar") || val.contains("side-bar") ||
           val.contains("aside") || val.contains("rail") {
            return Some("aside");
        }
        // Footer patterns
        if val.contains("footer") || val.contains("copyright") ||
           val.contains("site-footer") || val.contains("page-footer") {
            return Some("footer");
        }
        // Header patterns
        if val.contains("header") || val.contains("site-header") ||
           val.contains("page-header") || val.contains("masthead") {
            return Some("header");
        }
        // Search patterns
        if val.contains("search") {
            return Some("search");
        }
    }

    None
}

/// Truncate text for summarization with a given max length.
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    let trimmed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.len() > max_chars {
        let truncated: String = trimmed.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        trimmed
    }
}

/// Check if a div is a wrapper that should be collapsed.
/// Now handles: single element child, text-only content, or single interactive element.
pub fn is_collapsible_wrapper(tag: &str, node: &Handle) -> bool {
    if tag != "div" && tag != "span" {
        return false;
    }

    let children = node.children.borrow();
    let mut element_count = 0;
    let mut text_only = true;
    let mut interactive_count = 0;

    for child in children.iter() {
        match &child.data {
            NodeData::Element { name, attrs, .. } => {
                element_count += 1;
                text_only = false;
                let child_tag = name.local.as_ref();
                // Check if interactive
                if matches!(child_tag, "a" | "button" | "input" | "select" | "textarea") {
                    let attrs = attrs.borrow();
                    // For <a>, only count if it has href
                    if child_tag == "a" {
                        if attrs.iter().any(|a| a.name.local.as_ref() == "href") {
                            interactive_count += 1;
                        }
                    } else {
                        interactive_count += 1;
                    }
                }
            }
            NodeData::Text { contents } => {
                let text = contents.borrow();
                if !text.trim().is_empty() {
                    // Has meaningful text
                }
            }
            _ => {}
        }
    }

    // Collapse if:
    // 1. Single element child (original behavior)
    // 2. Text-only content (no element children)
    // Note: a single interactive element is already covered by the single-element rule.
    element_count == 1 || (text_only && element_count == 0)
}


/// Infer semantic hints from CSS class names.
/// Returns a list of hints like "primary", "danger", "disabled", "active", etc.
/// These give agents context about element importance without seeing raw CSS.
pub fn infer_class_hints(attrs: &[(String, String)]) -> Option<Vec<String>> {
    let class_val = attrs.iter()
        .find(|(n, _)| n == "class")
        .map(|(_, v)| v.to_lowercase())?;

    let mut hints = Vec::new();

    // Importance / variant
    if class_val.contains("primary") || class_val.contains("cta") {
        hints.push("primary".to_string());
    }
    if class_val.contains("secondary") {
        hints.push("secondary".to_string());
    }
    if class_val.contains("danger") || class_val.contains("destructive") || class_val.contains("delete") {
        hints.push("danger".to_string());
    }
    if class_val.contains("warning") || class_val.contains("caution") {
        hints.push("warning".to_string());
    }
    if class_val.contains("success") {
        hints.push("success".to_string());
    }
    if class_val.contains("error") || class_val.contains("invalid") {
        hints.push("error".to_string());
    }

    // State
    if class_val.contains("disabled") || class_val.contains("is-disabled") {
        hints.push("disabled".to_string());
    }
    if class_val.contains("active") || class_val.contains("is-active") || class_val.contains("current") {
        hints.push("active".to_string());
    }
    if class_val.contains("selected") || class_val.contains("is-selected") || class_val.contains("checked") {
        hints.push("selected".to_string());
    }
    if class_val.contains("hidden") || class_val.contains("sr-only") || class_val.contains("visually-hidden") {
        hints.push("hidden".to_string());
    }
    if class_val.contains("loading") || class_val.contains("spinner") || class_val.contains("skeleton") {
        hints.push("loading".to_string());
    }
    if class_val.contains("collapsed") || class_val.contains("is-closed") {
        hints.push("collapsed".to_string());
    }
    if class_val.contains("expanded") || class_val.contains("is-open") || class_val.contains("show") {
        hints.push("expanded".to_string());
    }

    // Size
    if class_val.contains("lg") || class_val.contains("large") || class_val.contains("xl") {
        hints.push("large".to_string());
    }
    if class_val.contains("sm") || class_val.contains("small") || class_val.contains("xs") || class_val.contains("mini") {
        hints.push("small".to_string());
    }

    // Layout / grouping
    if class_val.contains("card") && !class_val.contains("discard") {
        hints.push("card".to_string());
    }
    if class_val.contains("hero") || class_val.contains("jumbotron") || class_val.contains("banner") {
        hints.push("hero".to_string());
    }
    if class_val.contains("modal") || class_val.contains("dialog") || class_val.contains("popup") || class_val.contains("overlay") {
        hints.push("modal".to_string());
    }
    if class_val.contains("toast") || class_val.contains("snackbar") || class_val.contains("notification") || class_val.contains("alert") {
        hints.push("notification".to_string());
    }
    if class_val.contains("badge") || class_val.contains("chip") || class_val.contains("tag") || class_val.contains("pill") {
        hints.push("badge".to_string());
    }
    if class_val.contains("sticky") || class_val.contains("fixed") || class_val.contains("pinned") {
        hints.push("sticky".to_string());
    }
    if class_val.contains("required") || class_val.contains("mandatory") {
        hints.push("required".to_string());
    }

    if hints.is_empty() {
        None
    } else {
        hints.sort();
        hints.dedup();
        Some(hints)
    }
}

/// Detect footer heuristically: last block element with copyright/privacy/terms content.
pub fn looks_like_footer(node: &Handle) -> bool {
    if let NodeData::Element { .. } = &node.data {
        let text = get_all_text(node).to_lowercase();
        // Check for footer-like content
        if text.contains("copyright") || text.contains("©") ||
           text.contains("privacy") || text.contains("terms of") ||
           text.contains("all rights reserved") {
            return true;
        }
    }
    false
}

fn get_all_text(node: &Handle) -> String {
    let mut buf = String::new();
    collect_all_text(node, &mut buf);
    buf
}

fn collect_all_text(node: &Handle, buf: &mut String) {
    match &node.data {
        NodeData::Text { contents } => {
            buf.push_str(&contents.borrow());
        }
        NodeData::Element { name, .. } => {
            let tag = name.local.as_ref();
            if !matches!(tag, "script" | "style") {
                for child in node.children.borrow().iter() {
                    collect_all_text(child, buf);
                }
            }
        }
        _ => {
            for child in node.children.borrow().iter() {
                collect_all_text(child, buf);
            }
        }
    }
}
