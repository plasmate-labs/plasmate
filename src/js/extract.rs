//! Extract inline and evaluable `<script>` blocks from parsed HTML.

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

/// An extracted script block.
#[derive(Debug, Clone)]
pub struct ScriptBlock {
    /// The script source code.
    pub source: String,
    /// Label for error reporting (e.g. "inline-1", or the src URL).
    pub label: String,
    /// Whether this was an inline script (vs external src).
    pub is_inline: bool,
    /// The script's position in document order.
    pub index: usize,
}

/// Extract all executable script blocks from HTML.
///
/// Skips:
/// - Scripts with type="module" (would need import resolution)
/// - Scripts with type="application/json" or type="application/ld+json"
/// - Scripts with src="" (external; would need fetch, handled separately)
/// - Empty scripts
pub fn extract_scripts(html: &str) -> Vec<ScriptBlock> {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap();

    let mut scripts = Vec::new();
    let mut index = 0;
    visit_scripts(&dom.document, &mut scripts, &mut index);
    scripts
}

fn visit_scripts(node: &Handle, scripts: &mut Vec<ScriptBlock>, index: &mut usize) {
    if let NodeData::Element { name, attrs, .. } = &node.data {
        if name.local.as_ref() == "script" {
            let attrs_borrowed = attrs.borrow();
            let script_type = attrs_borrowed
                .iter()
                .find(|a| a.name.local.as_ref() == "type")
                .map(|a| a.value.to_string().to_lowercase());
            let has_src = attrs_borrowed
                .iter()
                .any(|a| a.name.local.as_ref() == "src");

            // Skip non-executable types
            let skip = match script_type.as_deref() {
                Some("module") => true,
                Some("application/json") | Some("application/ld+json") => true,
                Some("text/html") | Some("text/template") => true,
                Some(t) if t != "text/javascript" && t != "application/javascript" && t != "" => {
                    true
                }
                _ => false,
            };

            if !skip && !has_src {
                // Collect inline text content
                let mut source = String::new();
                collect_script_text(node, &mut source);
                let source = source.trim().to_string();

                if !source.is_empty() {
                    scripts.push(ScriptBlock {
                        source,
                        label: format!("inline-{}", *index),
                        is_inline: true,
                        index: *index,
                    });
                    *index += 1;
                }
            }

            if !skip && has_src {
                let src = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "src")
                    .map(|a| a.value.to_string())
                    .unwrap_or_default();
                scripts.push(ScriptBlock {
                    source: String::new(),
                    label: src,
                    is_inline: false,
                    index: *index,
                });
                *index += 1;
            }

            return; // Don't recurse into script contents
        }
    }

    for child in node.children.borrow().iter() {
        visit_scripts(child, scripts, index);
    }
}

fn collect_script_text(node: &Handle, buf: &mut String) {
    match &node.data {
        NodeData::Text { contents } => {
            buf.push_str(&contents.borrow());
        }
        _ => {
            for child in node.children.borrow().iter() {
                collect_script_text(child, buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_inline_script() {
        let html = r#"<html><head><script>var x = 1;</script></head><body></body></html>"#;
        let scripts = extract_scripts(html);
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].source, "var x = 1;");
        assert!(scripts[0].is_inline);
    }

    #[test]
    fn test_skip_json_ld() {
        let html = r#"<html><head>
            <script type="application/ld+json">{"@type":"WebPage"}</script>
            <script>var x = 1;</script>
        </head><body></body></html>"#;
        let scripts = extract_scripts(html);
        let inline: Vec<_> = scripts.iter().filter(|s| s.is_inline).collect();
        assert_eq!(inline.len(), 1);
        assert_eq!(inline[0].source, "var x = 1;");
    }

    #[test]
    fn test_skip_module() {
        let html =
            r#"<html><head><script type="module">import x from './x';</script></head></html>"#;
        let scripts = extract_scripts(html);
        let inline: Vec<_> = scripts.iter().filter(|s| s.is_inline).collect();
        assert_eq!(inline.len(), 0);
    }

    #[test]
    fn test_external_script_noted() {
        let html = r#"<html><head><script src="/app.js"></script></head></html>"#;
        let scripts = extract_scripts(html);
        assert_eq!(scripts.len(), 1);
        assert!(!scripts[0].is_inline);
        assert_eq!(scripts[0].label, "/app.js");
    }

    #[test]
    fn test_multiple_scripts_ordered() {
        let html = r#"<html><body>
            <script>var a = 1;</script>
            <p>content</p>
            <script>var b = 2;</script>
            <script>var c = 3;</script>
        </body></html>"#;
        let scripts = extract_scripts(html);
        let inline: Vec<_> = scripts.iter().filter(|s| s.is_inline).collect();
        assert_eq!(inline.len(), 3);
        assert_eq!(inline[0].index, 0);
        assert_eq!(inline[1].index, 1);
        assert_eq!(inline[2].index, 2);
    }

    #[test]
    fn test_skip_empty_scripts() {
        let html = r#"<html><head><script></script><script>  </script></head></html>"#;
        let scripts = extract_scripts(html);
        let inline: Vec<_> = scripts.iter().filter(|s| s.is_inline).collect();
        assert_eq!(inline.len(), 0);
    }

    #[test]
    fn test_skip_external_module() {
        let html =
            r#"<html><head><script type="module" src="/app.js"></script></head></html>"#;
        let scripts = extract_scripts(html);
        assert_eq!(scripts.len(), 0, "External module scripts should be skipped");
    }
}
