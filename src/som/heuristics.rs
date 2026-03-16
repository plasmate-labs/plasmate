use markup5ever_rcdom::{Handle, NodeData};

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
        let truncated: String = trimmed.chars().take(197).collect();
        format!("{}...", truncated)
    } else {
        trimmed
    }
}

/// Check if a div is a "wrapper" (single child element, no semantic meaning).
pub fn is_wrapper_div(tag: &str, child_element_count: usize) -> bool {
    tag == "div" && child_element_count == 1
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
