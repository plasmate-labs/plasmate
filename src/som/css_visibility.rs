//! Lightweight CSS visibility cascade.
//!
//! We don't render - we just need to know which elements are hidden by CSS rules.
//! Parses <style> blocks and inline styles to compute visibility state.
//! This is NOT a full CSS engine; it handles the patterns that actually hide content.

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::{HashMap, HashSet};

/// Selectors that hide elements (from <style> blocks).
#[derive(Debug, Default)]
pub struct VisibilityRules {
    /// CSS selectors mapped to their computed display/visibility.
    /// true = hidden, false = visible.
    hidden_classes: HashSet<String>,
    hidden_ids: HashSet<String>,
    hidden_tags: HashSet<String>,
    /// Selectors that explicitly show (for overrides).
    visible_classes: HashSet<String>,
}

impl VisibilityRules {
    /// Parse all <style> blocks from HTML and build visibility rules.
    pub fn from_html(html: &str) -> Self {
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap();

        let mut rules = VisibilityRules::default();
        let mut styles = Vec::new();
        collect_style_blocks(&dom.document, &mut styles);

        for css in &styles {
            rules.parse_css(css);
        }

        rules
    }

    /// Parse a CSS string to extract visibility-related rules.
    fn parse_css(&mut self, css: &str) {
        // Simple CSS parser: find rules with display:none or visibility:hidden
        // We don't need a full parser, just pattern matching on common hiding patterns.
        let mut chars = css.chars().peekable();
        let mut selector = String::new();
        let mut in_block = false;
        let mut block_content = String::new();
        let mut brace_depth = 0;

        while let Some(ch) = chars.next() {
            if ch == '{' {
                brace_depth += 1;
                if brace_depth == 1 {
                    in_block = true;
                    block_content.clear();
                    continue;
                }
            }
            if ch == '}' {
                brace_depth -= 1;
                if brace_depth == 0 && in_block {
                    in_block = false;
                    self.process_rule(selector.trim(), &block_content);
                    selector.clear();
                    continue;
                }
            }

            if in_block {
                block_content.push(ch);
            } else {
                selector.push(ch);
            }
        }
    }

    fn process_rule(&mut self, selector: &str, declarations: &str) {
        let decl_lower = declarations.to_lowercase();
        let decl_lower = decl_lower.replace(' ', "");

        let is_hidden = decl_lower.contains("display:none")
            || decl_lower.contains("visibility:hidden")
            || decl_lower.contains("opacity:0")
            || (decl_lower.contains("clip:rect(0") && decl_lower.contains("position:absolute"))
            || (decl_lower.contains("height:0") && decl_lower.contains("overflow:hidden"))
            || (decl_lower.contains("width:0") && decl_lower.contains("overflow:hidden"))
            || (decl_lower.contains("max-height:0") && decl_lower.contains("overflow:hidden"))
            || decl_lower.contains("font-size:0");

        let is_visible = decl_lower.contains("display:block")
            || decl_lower.contains("display:flex")
            || decl_lower.contains("display:grid")
            || decl_lower.contains("display:inline")
            || decl_lower.contains("visibility:visible");

        if !is_hidden && !is_visible {
            return;
        }

        // Parse the selector to extract classes, IDs, tags
        // Handle comma-separated selectors
        for sel in selector.split(',') {
            let sel = sel.trim();
            if sel.is_empty() || sel.starts_with('@') {
                continue;
            }

            // Extract the last segment (most specific)
            let parts: Vec<&str> = sel.split_whitespace().collect();
            let last = parts.last().copied().unwrap_or(sel);

            for segment in last.split(|c: char| c == '>' || c == '+' || c == '~') {
                let segment = segment.trim();
                if segment.is_empty() {
                    continue;
                }

                // Extract classes and IDs
                let mut i = 0;
                let bytes = segment.as_bytes();
                while i < bytes.len() {
                    if bytes[i] == b'.' {
                        let start = i + 1;
                        i += 1;
                        while i < bytes.len()
                            && (bytes[i].is_ascii_alphanumeric()
                                || bytes[i] == b'-'
                                || bytes[i] == b'_')
                        {
                            i += 1;
                        }
                        let class = &segment[start..i];
                        if !class.is_empty() {
                            if is_hidden {
                                self.hidden_classes.insert(class.to_string());
                            }
                            if is_visible {
                                self.visible_classes.insert(class.to_string());
                            }
                        }
                    } else if bytes[i] == b'#' {
                        let start = i + 1;
                        i += 1;
                        while i < bytes.len()
                            && (bytes[i].is_ascii_alphanumeric()
                                || bytes[i] == b'-'
                                || bytes[i] == b'_')
                        {
                            i += 1;
                        }
                        let id = &segment[start..i];
                        if !id.is_empty() && is_hidden {
                            self.hidden_ids.insert(id.to_string());
                        }
                    } else if bytes[i] == b':' || bytes[i] == b'[' {
                        // Skip pseudo-classes and attribute selectors
                        break;
                    } else {
                        i += 1;
                    }
                }

                // Check for tag-level hiding (rare but happens)
                let tag_part = segment.split(|c: char| c == '.' || c == '#' || c == ':' || c == '[')
                    .next()
                    .unwrap_or("");
                if !tag_part.is_empty() && tag_part.chars().all(|c| c.is_ascii_alphabetic()) && is_hidden {
                    // Only add tag hiding for specific patterns, not broad tags
                    if matches!(tag_part, "noscript" | "template") {
                        self.hidden_tags.insert(tag_part.to_string());
                    }
                }
            }
        }
    }

    /// Check if an element is hidden by CSS rules.
    pub fn is_hidden(&self, tag: &str, class: &str, id: &str) -> bool {
        // Check tag-level hiding
        if self.hidden_tags.contains(tag) {
            return true;
        }

        // Check ID hiding
        if !id.is_empty() && self.hidden_ids.contains(id) {
            return true;
        }

        // Check class hiding (any matching class triggers)
        for cls in class.split_whitespace() {
            if self.hidden_classes.contains(cls) && !self.visible_classes.contains(cls) {
                return true;
            }
        }

        false
    }

    /// Number of hiding rules found.
    pub fn rule_count(&self) -> usize {
        self.hidden_classes.len() + self.hidden_ids.len() + self.hidden_tags.len()
    }
}

fn collect_style_blocks(node: &Handle, styles: &mut Vec<String>) {
    if let NodeData::Element { name, .. } = &node.data {
        if name.local.as_ref() == "style" {
            let mut text = String::new();
            for child in node.children.borrow().iter() {
                if let NodeData::Text { contents } = &child.data {
                    text.push_str(&contents.borrow());
                }
            }
            if !text.is_empty() {
                styles.push(text);
            }
        }
    }
    for child in node.children.borrow().iter() {
        collect_style_blocks(child, styles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_none_class() {
        let html = r#"<html><head><style>
            .hidden { display: none; }
            .sr-only { position: absolute; clip: rect(0,0,0,0); height: 1px; overflow: hidden; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        assert!(rules.is_hidden("div", "hidden", ""));
        assert!(rules.is_hidden("span", "sr-only", ""));
        assert!(!rules.is_hidden("div", "visible", ""));
    }

    #[test]
    fn test_hidden_by_id() {
        let html = r#"<html><head><style>
            #modal-backdrop { display: none; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        assert!(rules.is_hidden("div", "", "modal-backdrop"));
    }

    #[test]
    fn test_visibility_hidden() {
        let html = r#"<html><head><style>
            .invisible { visibility: hidden; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        assert!(rules.is_hidden("span", "invisible", ""));
    }

    #[test]
    fn test_multiple_selectors() {
        let html = r#"<html><head><style>
            .hide, .d-none, .hidden { display: none; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        assert!(rules.is_hidden("div", "hide", ""));
        assert!(rules.is_hidden("div", "d-none", ""));
        assert!(rules.is_hidden("div", "hidden", ""));
    }

    #[test]
    fn test_zero_size_overflow_hidden() {
        let html = r#"<html><head><style>
            .clip-text { height: 0; overflow: hidden; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        assert!(rules.is_hidden("div", "clip-text", ""));
    }

    #[test]
    fn test_no_false_positives() {
        let html = r#"<html><head><style>
            .card { background: white; border: 1px solid #ddd; padding: 16px; }
            .btn { display: inline-block; padding: 8px 16px; }
            .container { max-width: 1200px; margin: 0 auto; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        assert!(!rules.is_hidden("div", "card", ""));
        assert!(!rules.is_hidden("button", "btn", ""));
        assert!(!rules.is_hidden("div", "container", ""));
    }

    #[test]
    fn test_bootstrap_common_patterns() {
        let html = r#"<html><head><style>
            .d-none { display: none !important; }
            .visually-hidden { position: absolute !important; width: 1px; height: 1px; overflow: hidden; clip: rect(0,0,0,0); }
            .collapse:not(.show) { display: none; }
            .modal { display: none; }
            .modal.show { display: block; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        assert!(rules.is_hidden("div", "d-none", ""));
        assert!(rules.is_hidden("div", "visually-hidden", ""));
    }

    #[test]
    fn test_media_query_at_rules_ignored() {
        let html = r#"<html><head><style>
            @media (max-width: 768px) { .desktop-only { display: none; } }
            .always-hidden { display: none; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        // We should still pick up rules inside media queries
        assert!(rules.is_hidden("div", "always-hidden", ""));
    }

    #[test]
    fn test_class_with_multiple_classes() {
        let html = r#"<html><head><style>
            .hidden { display: none; }
        </style></head><body></body></html>"#;
        let rules = VisibilityRules::from_html(html);
        // Element has multiple classes, one of which is hidden
        assert!(rules.is_hidden("div", "card mb-3 hidden", ""));
        assert!(!rules.is_hidden("div", "card mb-3 visible", ""));
    }
}
