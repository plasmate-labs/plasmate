//! Structured data extraction from HTML.
//!
//! Extracts JSON-LD, OpenGraph, Twitter Cards, and HTML meta tags
//! in a single pass. Equivalent to Lightpanda's LP.getStructuredData.

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// All structured data extracted from a page.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredData {
    /// JSON-LD blocks (Schema.org data).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub json_ld: Vec<Value>,
    /// OpenGraph metadata (og:*).
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub open_graph: std::collections::HashMap<String, String>,
    /// Twitter/X Card metadata.
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub twitter_card: std::collections::HashMap<String, String>,
    /// Standard HTML meta tags (description, author, keywords, etc.).
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub meta: std::collections::HashMap<String, String>,
    /// Link elements (canonical, icon, manifest, alternate, etc.).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<LinkElement>,
}

/// A <link> element with rel and href.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkElement {
    pub rel: String,
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
}

impl StructuredData {
    pub fn is_empty(&self) -> bool {
        self.json_ld.is_empty()
            && self.open_graph.is_empty()
            && self.twitter_card.is_empty()
            && self.meta.is_empty()
            && self.links.is_empty()
    }
}

/// Extract all structured data from HTML in a single pass.
pub fn extract_structured_data(html: &str) -> StructuredData {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap();

    let mut data = StructuredData::default();
    visit_node(&dom.document, &mut data);
    data
}

fn visit_node(node: &Handle, data: &mut StructuredData) {
    if let NodeData::Element { name, attrs, .. } = &node.data {
        let tag = name.local.as_ref();
        let attrs_borrowed = attrs.borrow();

        match tag {
            // JSON-LD: <script type="application/ld+json">
            "script" => {
                let is_json_ld = attrs_borrowed.iter().any(|a| {
                    a.name.local.as_ref() == "type"
                        && a.value.as_ref().eq_ignore_ascii_case("application/ld+json")
                });
                if is_json_ld {
                    let text = collect_text(node);
                    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
                        // Handle both single objects and arrays
                        match value {
                            Value::Array(arr) => data.json_ld.extend(arr),
                            v => data.json_ld.push(v),
                        }
                    }
                }
            }

            // Meta tags: OpenGraph, Twitter, standard
            "meta" => {
                let property = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "property")
                    .map(|a| a.value.to_string());
                let name_attr = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "name")
                    .map(|a| a.value.to_string());
                let content = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "content")
                    .map(|a| a.value.to_string());
                let charset = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "charset")
                    .map(|a| a.value.to_string());

                // OpenGraph: <meta property="og:*" content="...">
                if let (Some(prop), Some(content)) = (&property, &content) {
                    if prop.starts_with("og:") {
                        data.open_graph.insert(prop.clone(), content.clone());
                    }
                }

                // Twitter Card: <meta name="twitter:*" content="...">
                if let (Some(name), Some(content)) = (&name_attr, &content) {
                    if name.starts_with("twitter:") {
                        data.twitter_card.insert(name.clone(), content.clone());
                    }
                }

                // Standard meta: description, author, keywords, robots, viewport
                if let (Some(name), Some(content)) = (&name_attr, &content) {
                    let n = name.to_lowercase();
                    if matches!(
                        n.as_str(),
                        "description"
                            | "author"
                            | "keywords"
                            | "robots"
                            | "viewport"
                            | "generator"
                            | "theme-color"
                    ) {
                        data.meta.insert(n, content.clone());
                    }
                }

                // Charset
                if let Some(cs) = charset {
                    data.meta.insert("charset".to_string(), cs);
                }
            }

            // Link elements: canonical, icon, manifest, alternate, stylesheet
            "link" => {
                let rel = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "rel")
                    .map(|a| a.value.to_string());
                let href = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "href")
                    .map(|a| a.value.to_string());

                if let (Some(rel), Some(href)) = (rel, href) {
                    let rel_lower = rel.to_lowercase();
                    // Only keep semantically meaningful links
                    if matches!(
                        rel_lower.as_str(),
                        "canonical"
                            | "icon"
                            | "shortcut icon"
                            | "apple-touch-icon"
                            | "manifest"
                            | "alternate"
                            | "amphtml"
                            | "preconnect"
                            | "dns-prefetch"
                            | "author"
                            | "license"
                            | "search"
                    ) {
                        let link_type = attrs_borrowed
                            .iter()
                            .find(|a| a.name.local.as_ref() == "type")
                            .map(|a| a.value.to_string());
                        let hreflang = attrs_borrowed
                            .iter()
                            .find(|a| a.name.local.as_ref() == "hreflang")
                            .map(|a| a.value.to_string());
                        data.links.push(LinkElement {
                            rel: rel_lower,
                            href,
                            r#type: link_type,
                            hreflang,
                        });
                    }
                }
            }

            _ => {}
        }
    }

    for child in node.children.borrow().iter() {
        visit_node(child, data);
    }
}

fn collect_text(node: &Handle) -> String {
    let mut buf = String::new();
    for child in node.children.borrow().iter() {
        if let NodeData::Text { contents } = &child.data {
            buf.push_str(&contents.borrow());
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_ld_extraction() {
        let html = r#"<html><head>
            <script type="application/ld+json">
            {"@context":"https://schema.org","@type":"WebPage","name":"Test Page","description":"A test"}
            </script>
        </head><body></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.json_ld.len(), 1);
        assert_eq!(data.json_ld[0]["@type"], "WebPage");
        assert_eq!(data.json_ld[0]["name"], "Test Page");
    }

    #[test]
    fn test_json_ld_array() {
        let html = r#"<html><head>
            <script type="application/ld+json">
            [{"@type":"Article","name":"A1"},{"@type":"Article","name":"A2"}]
            </script>
        </head><body></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.json_ld.len(), 2);
    }

    #[test]
    fn test_open_graph() {
        let html = r#"<html><head>
            <meta property="og:title" content="My Page">
            <meta property="og:description" content="Page description">
            <meta property="og:image" content="https://example.com/img.jpg">
            <meta property="og:url" content="https://example.com">
            <meta property="og:type" content="website">
        </head><body></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.open_graph.len(), 5);
        assert_eq!(data.open_graph["og:title"], "My Page");
        assert_eq!(data.open_graph["og:type"], "website");
    }

    #[test]
    fn test_twitter_card() {
        let html = r#"<html><head>
            <meta name="twitter:card" content="summary_large_image">
            <meta name="twitter:site" content="@example">
            <meta name="twitter:title" content="Title">
        </head><body></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.twitter_card.len(), 3);
        assert_eq!(data.twitter_card["twitter:card"], "summary_large_image");
    }

    #[test]
    fn test_standard_meta() {
        let html = r#"<html><head>
            <meta charset="utf-8">
            <meta name="description" content="A great page">
            <meta name="author" content="John Doe">
            <meta name="robots" content="index,follow">
            <meta name="viewport" content="width=device-width">
        </head><body></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.meta["charset"], "utf-8");
        assert_eq!(data.meta["description"], "A great page");
        assert_eq!(data.meta["author"], "John Doe");
    }

    #[test]
    fn test_link_elements() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.com/page">
            <link rel="icon" href="/favicon.ico" type="image/x-icon">
            <link rel="alternate" href="/es" hreflang="es">
            <link rel="stylesheet" href="/style.css">
        </head><body></body></html>"#;
        let data = extract_structured_data(html);
        // stylesheet should be excluded
        assert_eq!(data.links.len(), 3);
        assert_eq!(data.links[0].rel, "canonical");
        assert_eq!(data.links[2].hreflang.as_deref(), Some("es"));
    }

    #[test]
    fn test_full_page_structured_data() {
        let html = r#"<!DOCTYPE html>
<html><head>
    <meta charset="utf-8">
    <title>Product Page</title>
    <meta name="description" content="Buy our product">
    <meta property="og:title" content="Product">
    <meta property="og:price:amount" content="29.99">
    <meta name="twitter:card" content="product">
    <link rel="canonical" href="https://shop.example.com/product">
    <script type="application/ld+json">
    {"@context":"https://schema.org","@type":"Product","name":"Widget","offers":{"@type":"Offer","price":"29.99","priceCurrency":"USD"}}
    </script>
</head><body><h1>Widget</h1></body></html>"#;
        let data = extract_structured_data(html);
        assert!(!data.is_empty());
        assert_eq!(data.json_ld.len(), 1);
        assert_eq!(data.json_ld[0]["@type"], "Product");
        assert!(data.open_graph.contains_key("og:title"));
        assert!(data.twitter_card.contains_key("twitter:card"));
        assert!(data.meta.contains_key("description"));
        assert_eq!(data.links.len(), 1);
    }

    #[test]
    fn test_invalid_json_ld_skipped() {
        let html = r#"<html><head>
            <script type="application/ld+json">not valid json{{{</script>
            <script type="application/ld+json">{"@type":"Valid"}</script>
        </head><body></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.json_ld.len(), 1, "Invalid JSON-LD should be skipped");
    }
}
