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
                    a.name.local.as_ref() == "type" && is_json_ld_script_type(a.value.as_ref())
                });
                if is_json_ld {
                    if let Some(value) = parse_json_ld_block(&collect_text(node)) {
                        push_json_ld(data, value);
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
                let itemprop = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "itemprop")
                    .map(|a| a.value.to_string());
                let content = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "content")
                    .map(|a| a.value.to_string());
                let charset = attrs_borrowed
                    .iter()
                    .find(|a| a.name.local.as_ref() == "charset")
                    .map(|a| a.value.to_string());

                // OpenGraph: <meta property="og:*" / property="article:*" / property="book:*" / property="profile:*" / property="al:*" content="...">
                if let (Some(prop), Some(content)) = (&property, &content) {
                    if prop.starts_with("og:") {
                        data.open_graph.insert(prop.clone(), content.clone());
                    } else if is_open_graph_article_property(prop) {
                        data.open_graph
                            .insert(prop.to_ascii_lowercase(), content.clone());
                    } else if is_open_graph_book_property(prop) {
                        data.open_graph
                            .insert(prop.to_ascii_lowercase(), content.clone());
                    } else if is_open_graph_profile_property(prop) {
                        data.open_graph
                            .insert(prop.to_ascii_lowercase(), content.clone());
                    } else if is_app_links_property(prop) {
                        data.open_graph
                            .insert(prop.to_ascii_lowercase(), content.clone());
                    }
                }

                // Twitter Card: name="twitter:*" or property="twitter:*"
                if let Some(content) = &content {
                    if let Some(key) = twitter_card_key(name_attr.as_deref(), property.as_deref()) {
                        data.twitter_card.insert(key.to_string(), content.clone());
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
                    ) || n
                        .strip_prefix("citation_")
                        .is_some_and(|rest| !rest.is_empty())
                        || is_dublin_core_meta_name(&n)
                        || is_prism_meta_name(&n)
                        || is_eprints_meta_name(&n)
                        || is_bepress_meta_name(&n)
                        || is_fediverse_meta_name(&n)
                    {
                        data.meta.insert(n, content.clone());
                    }
                }

                if let (Some(itemprop), Some(content)) = (&itemprop, &content) {
                    let n = itemprop.trim().to_lowercase();
                    if is_schema_itemprop_name(&n) {
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
                            | "prev"
                            | "previous"
                            | "next"
                            | "privacy-policy"
                            | "terms-of-service"
                            | "help"
                            | "me"
                            | "shortlink"
                            | "webmention"
                            | "pingback"
                            | "hub"
                    ) {
                        let link_type = attrs_borrowed
                            .iter()
                            .find(|a| a.name.local.as_ref() == "type")
                            .map(|a| a.value.to_string());
                        let hreflang = attrs_borrowed
                            .iter()
                            .find(|a| a.name.local.as_ref() == "hreflang")
                            .map(|a| a.value.to_string());
                        let rel = if rel_lower == "previous" {
                            "prev".to_string()
                        } else {
                            rel_lower
                        };
                        data.links.push(LinkElement {
                            rel,
                            href,
                            r#type: link_type,
                            hreflang,
                        });
                    }
                }
            }

            "base" => {
                if !data.links.iter().any(|link| link.rel == "base") {
                    if let Some(href) = attrs_borrowed
                        .iter()
                        .find(|a| a.name.local.as_ref() == "href")
                        .map(|a| a.value.to_string())
                    {
                        let href = href.trim();
                        if is_document_base_href(href) {
                            data.links.push(LinkElement {
                                rel: "base".to_string(),
                                href: href.to_string(),
                                r#type: None,
                                hreflang: None,
                            });
                        }
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
        match &child.data {
            NodeData::Text { contents } => buf.push_str(&contents.borrow()),
            NodeData::Comment { contents } => buf.push_str(contents),
            _ => {}
        }
    }
    buf
}

fn push_json_ld(data: &mut StructuredData, value: Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                push_json_ld(data, item);
            }
        }
        Value::Object(map) => match map.get("@graph") {
            Some(Value::Array(graph)) if !graph.is_empty() => {
                for item in graph.clone() {
                    push_json_ld(data, item);
                }
            }
            Some(Value::Array(_)) => {}
            _ => data.json_ld.push(Value::Object(map)),
        },
        other => data.json_ld.push(other),
    }
}

fn is_open_graph_article_property(prop: &str) -> bool {
    prop.to_ascii_lowercase()
        .strip_prefix("article:")
        .is_some_and(|rest| !rest.is_empty())
}

fn is_open_graph_book_property(prop: &str) -> bool {
    prop.to_ascii_lowercase()
        .strip_prefix("book:")
        .is_some_and(|rest| !rest.is_empty())
}

fn is_open_graph_profile_property(prop: &str) -> bool {
    prop.to_ascii_lowercase()
        .strip_prefix("profile:")
        .is_some_and(|rest| !rest.is_empty())
}

fn is_app_links_property(prop: &str) -> bool {
    prop.to_ascii_lowercase()
        .strip_prefix("al:")
        .is_some_and(|rest| !rest.is_empty())
}

fn is_dublin_core_meta_name(n: &str) -> bool {
    n.strip_prefix("dcterms.")
        .or_else(|| n.strip_prefix("dc."))
        .is_some_and(|rest| !rest.is_empty())
}

fn is_prism_meta_name(n: &str) -> bool {
    n.strip_prefix("prism.")
        .is_some_and(|rest| !rest.is_empty())
}

fn is_eprints_meta_name(n: &str) -> bool {
    n.strip_prefix("eprints.")
        .is_some_and(|rest| !rest.is_empty())
}

fn is_bepress_meta_name(n: &str) -> bool {
    n.strip_prefix("bepress_citation_")
        .is_some_and(|rest| !rest.is_empty())
}

fn is_schema_itemprop_name(n: &str) -> bool {
    !n.is_empty() && !n.chars().any(char::is_whitespace)
}

fn is_fediverse_meta_name(n: &str) -> bool {
    n.strip_prefix("fediverse:")
        .is_some_and(|rest| !rest.is_empty())
}

fn is_document_base_href(href: &str) -> bool {
    let href = href.trim();
    if href.is_empty() {
        return false;
    }
    let lower = href.to_ascii_lowercase();
    !(lower.starts_with("javascript:")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:"))
}

fn is_json_ld_script_type(value: &str) -> bool {
    value
        .split(';')
        .next()
        .map(str::trim)
        .is_some_and(|essence| essence.eq_ignore_ascii_case("application/ld+json"))
}

fn unwrap_json_ld_script(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(inner) = trimmed
        .strip_prefix("<!--")
        .and_then(|rest| rest.strip_suffix("-->"))
    {
        return inner.trim();
    }
    let without_open = if let Some(rest) = trimmed.strip_prefix("//<![CDATA[") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("<![CDATA[") {
        rest
    } else {
        return trimmed;
    };
    let without_open = without_open.trim();
    without_open
        .strip_suffix("//]]>")
        .or_else(|| without_open.strip_suffix("]]>"))
        .unwrap_or(without_open)
        .trim()
}

fn parse_json_ld_block(text: &str) -> Option<Value> {
    serde_json::from_str(unwrap_json_ld_script(text)).ok()
}

fn twitter_card_key<'a>(name: Option<&'a str>, property: Option<&'a str>) -> Option<&'a str> {
    [name, property].into_iter().flatten().find(|key| {
        key.strip_prefix("twitter:")
            .is_some_and(|rest| !rest.is_empty())
    })
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
    fn json_ld_graph_nodes_are_extracted() {
        let html = r#"<html><head>
            <script type="application/ld+json">
            {"@context":"https://schema.org","@graph":[
                {"@type":"Organization","name":"Plasmate"},
                {"@type":"WebPage","name":"Docs"}
            ]}
            </script>
            <script type="application/ld+json">
            {"@context":"https://schema.org","@type":"SoftwareApplication","name":"Direct"}
            </script>
            <script type="application/ld+json">
            {"@graph":[]}
            </script>
            <script type="application/ld+json">
            {"@graph":{"@type":"NotFlattened"}}
            </script>
            <script type="application/json">
            {"@graph":[{"@type":"NotJsonLd"}]}
            </script>
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(
            data.json_ld.len(),
            4,
            "JSON-LD @graph nodes must surface as entities: {data:?}"
        );
        assert_eq!(data.json_ld[0]["@type"], "Organization");
        assert_eq!(data.json_ld[0]["name"], "Plasmate");
        assert_eq!(data.json_ld[1]["@type"], "WebPage");
        assert_eq!(data.json_ld[1]["name"], "Docs");
        assert_eq!(data.json_ld[2]["@type"], "SoftwareApplication");
        assert_eq!(data.json_ld[2]["name"], "Direct");
        assert_eq!(data.json_ld[3]["@graph"]["@type"], "NotFlattened");
        assert!(
            data.json_ld
                .iter()
                .all(|block| block["@type"] != "NotJsonLd"),
            "application/json must not copy JSON-LD @graph mapping: {data:?}"
        );
        assert!(
            data.json_ld
                .iter()
                .all(|block| block.get("@graph").is_none()
                    || block["@graph"]["@type"] == "NotFlattened"),
            "empty @graph must not invent entities: {data:?}"
        );
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
    fn twitter_card_property_attr_is_extracted() {
        let html = r#"<html><head>
            <meta property="twitter:card" content="summary">
            <meta property="twitter:title" content="Docs">
            <meta name="twitter:site" content="@plasmate">
            <meta property="twitter:" content="empty-suffix">
            <meta property="twitter" content="too-short">
            <meta property="twitter-card" content="hyphenated">
            <meta name="twitter:creator" property="og:title" content="Name wins">
            <meta property="og:image" content="https://example.test/img.png">
            <meta name="description" content="A page">
            <meta property="article:author" content="Not Twitter">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.twitter_card["twitter:card"], "summary");
        assert_eq!(data.twitter_card["twitter:title"], "Docs");
        assert_eq!(data.twitter_card["twitter:site"], "@plasmate");
        assert_eq!(data.twitter_card["twitter:creator"], "Name wins");
        assert_eq!(data.open_graph["og:title"], "Name wins");
        assert_eq!(data.open_graph["og:image"], "https://example.test/img.png");
        assert_eq!(data.meta["description"], "A page");
        assert!(
            !data.twitter_card.contains_key("twitter:")
                && !data.twitter_card.contains_key("twitter")
                && !data.twitter_card.contains_key("twitter-card"),
            "bare/hyphenated twitter keys must not copy twitter: mapping: {data:?}"
        );
        assert!(
            !data.open_graph.contains_key("twitter:card")
                && !data.open_graph.contains_key("twitter:title"),
            "property=twitter:* must not copy onto OpenGraph: {data:?}"
        );
        assert!(
            !data.meta.contains_key("twitter:card") && !data.meta.contains_key("article:author"),
            "twitter property cards must not copy onto standard meta: {data:?}"
        );
        assert!(
            !data.twitter_card.contains_key("article:author"),
            "article: properties must not copy onto Twitter cards: {data:?}"
        );
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
    fn pagination_link_rels_are_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/docs/page-2">
            <link rel="prev" href="/docs/page-1">
            <link rel="Next" href="/docs/page-3">
            <link rel="first" href="/docs/page-1">
            <link rel="last" href="/docs/page-9">
            <link rel="prefetch" href="/docs/page-3">
            <link rel="stylesheet" href="/style.css">
            <link rel="next prefetch" href="/docs/mixed">
            <a rel="next" href="/body-next">Body next</a>
            <meta name="citation_title" content="Not a link">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "prev" && link.href == "/docs/page-1"),
            "rel=prev must stay in structured data: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "next" && link.href == "/docs/page-3"),
            "rel=next must stay in structured data: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| link.rel == "canonical"),
            "canonical must remain: {data:?}"
        );
        assert!(
            !rels.iter().any(|rel| *rel == "first" || *rel == "last"),
            "first/last must not copy pagination mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"prefetch"),
            "prefetch must not copy pagination mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"stylesheet"),
            "stylesheet must stay excluded: {data:?}"
        );
        assert!(
            !data
                .links
                .iter()
                .any(|link| link.href == "/docs/mixed" || link.href == "/body-next"),
            "multi-token rel and body anchors must not copy head pagination mapping: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a link");
        assert!(
            !data.open_graph.contains_key("prev") && data.twitter_card.is_empty(),
            "pagination links must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
    }

    #[test]
    fn pagination_previous_synonym_canonicalizes_to_prev() {
        let html = r#"<html><head>
            <link rel="Previous" href="/docs/page-1">
            <link rel="prev" href="/docs/also-prev">
            <link rel="next" href="/docs/page-3">
            <link rel="previous prefetch" href="/docs/mixed">
            <link rel="first" href="/docs/page-1">
            <a rel="previous" href="/body-previous">Body previous</a>
            <meta property="og:title" content="Docs">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "prev" && link.href == "/docs/page-1"),
            "rel=previous must canonicalize to prev: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "prev" && link.href == "/docs/also-prev"),
            "rel=prev must remain: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "next" && link.href == "/docs/page-3"),
            "rel=next must remain: {data:?}"
        );
        assert!(
            !rels.iter().any(|rel| *rel == "previous"),
            "previous must not be stored as a distinct rel: {data:?}"
        );
        assert!(
            !rels.iter().any(|rel| *rel == "first"),
            "first must not copy previous synonym mapping: {data:?}"
        );
        assert!(
            !data
                .links
                .iter()
                .any(|link| link.href == "/docs/mixed" || link.href == "/body-previous"),
            "multi-token rel and body anchors must not copy previous mapping: {data:?}"
        );
        assert_eq!(data.open_graph["og:title"], "Docs");
        assert!(
            data.twitter_card.is_empty(),
            "previous synonym must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn legal_document_link_rels_are_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/app">
            <link rel="privacy-policy" href="/legal/privacy">
            <link rel="Terms-of-Service" href="/legal/terms">
            <link rel="tag" href="/tags/som">
            <link rel="license" href="/license">
            <link rel="prefetch" href="/legal/privacy">
            <link rel="stylesheet" href="/style.css">
            <link rel="privacy-policy prefetch" href="/legal/mixed">
            <a rel="privacy-policy" href="/body-privacy">Body privacy</a>
            <meta name="citation_title" content="Not a link">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "privacy-policy" && link.href == "/legal/privacy"),
            "rel=privacy-policy must stay in structured data: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "terms-of-service" && link.href == "/legal/terms"),
            "rel=terms-of-service must stay in structured data: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "canonical" && link.href == "https://example.test/app"),
            "canonical must remain: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "license" && link.href == "/license"),
            "license must remain: {data:?}"
        );
        assert!(
            !rels.contains(&"tag"),
            "tag must not copy legal-document mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"prefetch"),
            "prefetch must not copy legal-document mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"stylesheet"),
            "stylesheet must stay excluded: {data:?}"
        );
        assert!(
            !data
                .links
                .iter()
                .any(|link| link.href == "/legal/mixed" || link.href == "/body-privacy"),
            "multi-token rel and body anchors must not copy head legal-document mapping: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a link");
        assert!(
            !data.open_graph.contains_key("privacy-policy")
                && !data.open_graph.contains_key("terms-of-service")
                && data.twitter_card.is_empty(),
            "legal-document links must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
    }

    #[test]
    fn help_link_rels_are_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/app">
            <link rel="help" href="/docs/help">
            <link rel="Help" href="/docs/help-alias">
            <link rel="license" href="/license">
            <link rel="privacy-policy" href="/legal/privacy">
            <link rel="tag" href="/tags/som">
            <link rel="prefetch" href="/docs/help">
            <link rel="stylesheet" href="/style.css">
            <link rel="help prefetch" href="/docs/mixed">
            <a rel="help" href="/body-help">Body help</a>
            <meta name="citation_title" content="Not a link">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "help" && link.href == "/docs/help"),
            "rel=help must stay in structured data: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "help" && link.href == "/docs/help-alias"),
            "rel=Help must canonicalize to help: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "canonical" && link.href == "https://example.test/app"),
            "canonical must remain: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "license" && link.href == "/license"),
            "license must remain: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "privacy-policy" && link.href == "/legal/privacy"),
            "privacy-policy must remain: {data:?}"
        );
        assert!(
            !rels.contains(&"tag"),
            "tag must not copy help mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"prefetch"),
            "prefetch must not copy help mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"stylesheet"),
            "stylesheet must stay excluded: {data:?}"
        );
        assert!(
            !data
                .links
                .iter()
                .any(|link| link.href == "/docs/mixed" || link.href == "/body-help"),
            "multi-token rel and body anchors must not copy head help mapping: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a link");
        assert!(
            !data.open_graph.contains_key("help") && data.twitter_card.is_empty(),
            "help links must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
    }

    #[test]
    fn document_base_href_is_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/app">
            <!-- <base href="/ignored/"> -->
            <base target="_blank">
            <base href="javascript:void(0)">
            <base href="/app/">
            <base href="/later/">
            <link rel="base" href="/not-a-base">
            <link rel="help" href="/docs/help">
            <a href="/body">Body</a>
            <meta name="citation_title" content="Not a base">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "base" && link.href == "/app/"),
            "first usable <base href> must stay in structured data: {data:?}"
        );
        assert_eq!(
            data.links.iter().filter(|link| link.rel == "base").count(),
            1,
            "later <base> elements must not replace the first usable href: {data:?}"
        );
        assert!(
            !data.links.iter().any(|link| link.href == "/ignored/"
                || link.href == "/later/"
                || link.href == "javascript:void(0)"
                || link.href == "/not-a-base"),
            "comment, javascript, later, and link rel=base must not copy <base>: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "canonical" && link.href == "https://example.test/app"),
            "canonical must remain: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "help" && link.href == "/docs/help"),
            "help must remain: {data:?}"
        );
        assert!(
            !rels.iter().any(|rel| *rel == "tag"),
            "unrelated rels must stay excluded: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a base");
        assert!(
            !data.open_graph.contains_key("base") && data.twitter_card.is_empty(),
            "document base must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
    }

    #[test]
    fn identity_link_rels_are_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/app">
            <link rel="me" href="https://github.com/plasmate-labs">
            <link rel="Me" href="https://mastodon.social/@plasmate">
            <link rel="help" href="/docs/help">
            <link rel="author" href="https://example.test/authors/ada">
            <link rel="tag" href="/tags/som">
            <link rel="prefetch" href="https://github.com/plasmate-labs">
            <link rel="stylesheet" href="/style.css">
            <link rel="me prefetch" href="https://example.test/mixed">
            <a rel="me" href="https://example.test/body-me">Body identity</a>
            <meta name="citation_title" content="Not a link">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links.iter().any(|link| {
                link.rel == "me" && link.href == "https://github.com/plasmate-labs"
            }),
            "rel=me must stay in structured data: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "me" && link.href == "https://mastodon.social/@plasmate"
            }),
            "rel=Me must canonicalize to me: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "canonical" && link.href == "https://example.test/app"),
            "canonical must remain: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "help" && link.href == "/docs/help"),
            "help must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "author" && link.href == "https://example.test/authors/ada"
            }),
            "author must remain: {data:?}"
        );
        assert!(
            !rels.contains(&"tag"),
            "tag must not copy identity mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"prefetch"),
            "prefetch must not copy identity mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"stylesheet"),
            "stylesheet must stay excluded: {data:?}"
        );
        assert!(
            !data.links.iter().any(|link| {
                link.href == "https://example.test/mixed"
                    || link.href == "https://example.test/body-me"
            }),
            "multi-token rel and body anchors must not copy head identity mapping: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a link");
        assert!(
            !data.open_graph.contains_key("me") && data.twitter_card.is_empty(),
            "identity links must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
    }

    #[test]
    fn shortlink_link_rels_are_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/wiki/Semantic_Object_Model">
            <link rel="shortlink" href="https://example.test/?curid=42">
            <link rel="ShortLink" href="https://example.test/?p=42">
            <link rel="me" href="https://github.com/plasmate-labs">
            <link rel="help" href="/docs/help">
            <link rel="tag" href="/tags/som">
            <link rel="prefetch" href="https://example.test/?curid=42">
            <link rel="stylesheet" href="/style.css">
            <link rel="shortlink prefetch" href="https://example.test/mixed">
            <a rel="shortlink" href="https://example.test/body-short">Body shortlink</a>
            <meta name="citation_title" content="Not a link">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links.iter().any(|link| {
                link.rel == "shortlink" && link.href == "https://example.test/?curid=42"
            }),
            "rel=shortlink must stay in structured data: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "shortlink" && link.href == "https://example.test/?p=42"
            }),
            "rel=ShortLink must canonicalize to shortlink: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "canonical"
                    && link.href == "https://example.test/wiki/Semantic_Object_Model"
            }),
            "canonical must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "me" && link.href == "https://github.com/plasmate-labs"
            }),
            "identity me must remain: {data:?}"
        );
        assert!(
            data.links
                .iter()
                .any(|link| link.rel == "help" && link.href == "/docs/help"),
            "help must remain: {data:?}"
        );
        assert!(
            !rels.contains(&"tag"),
            "tag must not copy shortlink mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"prefetch"),
            "prefetch must not copy shortlink mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"stylesheet"),
            "stylesheet must stay excluded: {data:?}"
        );
        assert!(
            !data.links.iter().any(|link| {
                link.href == "https://example.test/mixed"
                    || link.href == "https://example.test/body-short"
            }),
            "multi-token rel and body anchors must not copy head shortlink mapping: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a link");
        assert!(
            !data.open_graph.contains_key("shortlink") && data.twitter_card.is_empty(),
            "shortlink must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
    }

    #[test]
    fn webmention_link_rels_are_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/notes/som">
            <link rel="webmention" href="https://example.test/webmention">
            <link rel="WebMention" href="https://webmention.io/example.test/webmention">
            <link rel="me" href="https://github.com/plasmate-labs">
            <link rel="shortlink" href="https://example.test/?p=42">
            <link rel="pingback" href="https://example.test/xmlrpc.php">
            <link rel="micropub" href="https://example.test/micropub">
            <link rel="tag" href="/tags/som">
            <link rel="prefetch" href="https://example.test/webmention">
            <link rel="stylesheet" href="/style.css">
            <link rel="webmention prefetch" href="https://example.test/mixed">
            <a rel="webmention" href="https://example.test/body-webmention">Body webmention</a>
            <meta name="citation_title" content="Not a link">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links.iter().any(|link| {
                link.rel == "webmention" && link.href == "https://example.test/webmention"
            }),
            "rel=webmention must stay in structured data: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "webmention"
                    && link.href == "https://webmention.io/example.test/webmention"
            }),
            "rel=WebMention must canonicalize to webmention: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "canonical" && link.href == "https://example.test/notes/som"
            }),
            "canonical must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "me" && link.href == "https://github.com/plasmate-labs"
            }),
            "identity me must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "shortlink" && link.href == "https://example.test/?p=42"
            }),
            "shortlink must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "pingback" && link.href == "https://example.test/xmlrpc.php"
            }),
            "pingback must remain: {data:?}"
        );
        assert!(
            !rels.contains(&"micropub"),
            "micropub must not copy webmention mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"tag"),
            "tag must not copy webmention mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"prefetch"),
            "prefetch must not copy webmention mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"stylesheet"),
            "stylesheet must stay excluded: {data:?}"
        );
        assert!(
            !data.links.iter().any(|link| {
                link.href == "https://example.test/mixed"
                    || link.href == "https://example.test/body-webmention"
            }),
            "multi-token rel and body anchors must not copy head webmention mapping: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a link");
        assert!(
            !data.open_graph.contains_key("webmention") && data.twitter_card.is_empty(),
            "webmention must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
    }

    #[test]
    fn pingback_link_rels_are_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/notes/som">
            <link rel="pingback" href="https://example.test/xmlrpc.php">
            <link rel="PingBack" href="https://pingback.example.test/xmlrpc">
            <link rel="webmention" href="https://example.test/webmention">
            <link rel="me" href="https://github.com/plasmate-labs">
            <link rel="shortlink" href="https://example.test/?p=42">
            <link rel="micropub" href="https://example.test/micropub">
            <link rel="tag" href="/tags/som">
            <link rel="prefetch" href="https://example.test/xmlrpc.php">
            <link rel="stylesheet" href="/style.css">
            <link rel="pingback prefetch" href="https://example.test/mixed">
            <a rel="pingback" href="https://example.test/body-pingback">Body pingback</a>
            <meta name="citation_title" content="Not a link">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links.iter().any(|link| {
                link.rel == "pingback" && link.href == "https://example.test/xmlrpc.php"
            }),
            "rel=pingback must stay in structured data: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "pingback" && link.href == "https://pingback.example.test/xmlrpc"
            }),
            "rel=PingBack must canonicalize to pingback: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "canonical" && link.href == "https://example.test/notes/som"
            }),
            "canonical must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "webmention" && link.href == "https://example.test/webmention"
            }),
            "webmention must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "me" && link.href == "https://github.com/plasmate-labs"
            }),
            "identity me must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "shortlink" && link.href == "https://example.test/?p=42"
            }),
            "shortlink must remain: {data:?}"
        );
        assert!(
            !rels.contains(&"micropub"),
            "micropub must not copy pingback mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"tag"),
            "tag must not copy pingback mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"prefetch"),
            "prefetch must not copy pingback mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"stylesheet"),
            "stylesheet must stay excluded: {data:?}"
        );
        assert!(
            !data.links.iter().any(|link| {
                link.href == "https://example.test/mixed"
                    || link.href == "https://example.test/body-pingback"
            }),
            "multi-token rel and body anchors must not copy head pingback mapping: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a link");
        assert!(
            !data.open_graph.contains_key("pingback") && data.twitter_card.is_empty(),
            "pingback must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
    }

    #[test]
    fn hub_link_rels_are_extracted() {
        let html = r#"<html><head>
            <link rel="canonical" href="https://example.test/feed">
            <link rel="hub" href="https://example.test/hub">
            <link rel="Hub" href="https://pubsubhubbub.example.test/">
            <link rel="alternate" type="application/atom+xml" href="https://example.test/feed.atom">
            <link rel="webmention" href="https://example.test/webmention">
            <link rel="pingback" href="https://example.test/xmlrpc.php">
            <link rel="micropub" href="https://example.test/micropub">
            <link rel="tag" href="/tags/som">
            <link rel="prefetch" href="https://example.test/hub">
            <link rel="stylesheet" href="/style.css">
            <link rel="hub prefetch" href="https://example.test/mixed">
            <a rel="hub" href="https://example.test/body-hub">Body hub</a>
            <meta name="citation_title" content="Not a link">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        let rels: Vec<_> = data.links.iter().map(|link| link.rel.as_str()).collect();
        assert!(
            data.links
                .iter()
                .any(|link| { link.rel == "hub" && link.href == "https://example.test/hub" }),
            "rel=hub must stay in structured data: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "hub" && link.href == "https://pubsubhubbub.example.test/"
            }),
            "rel=Hub must canonicalize to hub: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "canonical" && link.href == "https://example.test/feed"
            }),
            "canonical must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "alternate" && link.href == "https://example.test/feed.atom"
            }),
            "alternate feed must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "webmention" && link.href == "https://example.test/webmention"
            }),
            "webmention must remain: {data:?}"
        );
        assert!(
            data.links.iter().any(|link| {
                link.rel == "pingback" && link.href == "https://example.test/xmlrpc.php"
            }),
            "pingback must remain: {data:?}"
        );
        assert!(
            !rels.contains(&"micropub"),
            "micropub must not copy hub mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"tag"),
            "tag must not copy hub mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"prefetch"),
            "prefetch must not copy hub mapping: {data:?}"
        );
        assert!(
            !rels.contains(&"stylesheet"),
            "stylesheet must stay excluded: {data:?}"
        );
        assert!(
            !data.links.iter().any(|link| {
                link.href == "https://example.test/mixed"
                    || link.href == "https://example.test/body-hub"
            }),
            "multi-token rel and body anchors must not copy head hub mapping: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Not a link");
        assert!(
            !data.open_graph.contains_key("hub") && data.twitter_card.is_empty(),
            "hub must not copy onto OpenGraph or Twitter cards: {data:?}"
        );
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

    #[test]
    fn json_ld_html_comment_wrapper_is_extracted() {
        let html = r#"<html><head>
            <script type="application/ld+json">
            <!--
            {"@context":"https://schema.org","@type":"Organization","name":"Plasmate"}
            -->
            </script>
            <script type="application/ld+json">
            //<![CDATA[
            {"@type":"WebPage","name":"Docs"}
            //]]>
            </script>
            <script type="application/json">
            <!--{"@type":"NotJsonLd"}-->
            </script>
            <script type="application/ld+json"><!-- not json --></script>
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(
            data.json_ld.len(),
            2,
            "comment/CDATA wrappers must not drop JSON-LD: {data:?}"
        );
        assert_eq!(data.json_ld[0]["@type"], "Organization");
        assert_eq!(data.json_ld[0]["name"], "Plasmate");
        assert_eq!(data.json_ld[1]["@type"], "WebPage");
        assert_eq!(data.json_ld[1]["name"], "Docs");
        assert!(
            data.json_ld
                .iter()
                .all(|block| block["@type"] != "NotJsonLd"),
            "non-ld+json scripts must stay unparsed: {data:?}"
        );
    }

    #[test]
    fn json_ld_mime_parameters_are_extracted() {
        let html = r#"<html><head>
            <script type="application/ld+json; charset=UTF-8">
            {"@type":"Organization","name":"Plasmate"}
            </script>
            <script type=" Application/LD+JSON ;profile=https://www.w3.org/ns/activitystreams">
            {"@type":"WebPage","name":"Docs"}
            </script>
            <script type="application/json; charset=utf-8">
            {"@type":"NotJsonLd"}
            </script>
            <script type="application/ld+jsonx">
            {"@type":"NotJsonLdEither"}
            </script>
            <script type="application/ld+json">
            {"@type":"SoftwareApplication","name":"Exact"}
            </script>
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(
            data.json_ld.len(),
            3,
            "JSON-LD MIME parameters must not drop Schema.org: {data:?}"
        );
        assert_eq!(data.json_ld[0]["@type"], "Organization");
        assert_eq!(data.json_ld[0]["name"], "Plasmate");
        assert_eq!(data.json_ld[1]["@type"], "WebPage");
        assert_eq!(data.json_ld[1]["name"], "Docs");
        assert_eq!(data.json_ld[2]["@type"], "SoftwareApplication");
        assert_eq!(data.json_ld[2]["name"], "Exact");
        assert!(
            data.json_ld.iter().all(|block| {
                block["@type"] != "NotJsonLd" && block["@type"] != "NotJsonLdEither"
            }),
            "application/json and ld+jsonx must not copy JSON-LD mapping: {data:?}"
        );
    }

    #[test]
    fn highwire_citation_meta_is_extracted() {
        let html = r#"<html><head>
            <meta name="citation_title" content="Semantic Object Model">
            <meta name="Citation_DOI" content="10.1000/plasmate">
            <meta name="citation_pdf_url" content="https://example.test/paper.pdf">
            <meta name="citation_" content="empty-suffix">
            <meta name="citation" content="too-short">
            <meta name="citations" content="not-highwire">
            <meta name="description" content="A paper">
            <meta property="citation_title" content="not-a-name-attr">
            <meta property="og:title" content="OG Title">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.meta["citation_title"], "Semantic Object Model");
        assert_eq!(data.meta["citation_doi"], "10.1000/plasmate");
        assert_eq!(
            data.meta["citation_pdf_url"],
            "https://example.test/paper.pdf"
        );
        assert_eq!(data.meta["description"], "A paper");
        assert!(
            !data.meta.contains_key("citation_"),
            "bare citation_ prefix must not be kept: {data:?}"
        );
        assert!(
            !data.meta.contains_key("citation"),
            "citation without underscore must not copy Highwire mapping: {data:?}"
        );
        assert!(
            !data.meta.contains_key("citations"),
            "citations must not copy Highwire mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("citation_title").map(String::as_str),
            Some("not-a-name-attr"),
            "property= citation_title must not copy name= mapping: {data:?}"
        );
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert!(
            !data.open_graph.contains_key("citation_title"),
            "citation meta must not copy onto OpenGraph: {data:?}"
        );
        assert!(
            data.twitter_card.is_empty(),
            "citation meta must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn article_open_graph_properties_are_extracted() {
        let html = r#"<html><head>
            <meta property="article:published_time" content="2026-09-13T00:00:00Z">
            <meta property="Article:author" content="https://example.test/authors/ada">
            <meta property="article:section" content="Engineering">
            <meta property="article:" content="empty-suffix">
            <meta property="article" content="too-short">
            <meta property="article-published_time" content="hyphen-not-colon">
            <meta property="music:duration" content="not-article">
            <meta property="og:title" content="OG Title">
            <meta name="article:published_time" content="not-a-property-attr">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta name="twitter:card" content="summary">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(
            data.open_graph["article:published_time"],
            "2026-09-13T00:00:00Z"
        );
        assert_eq!(
            data.open_graph["article:author"],
            "https://example.test/authors/ada"
        );
        assert_eq!(data.open_graph["article:section"], "Engineering");
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert!(
            !data.open_graph.contains_key("article:")
                && !data.open_graph.contains_key("article")
                && !data.open_graph.contains_key("article-published_time")
                && !data.open_graph.contains_key("music:duration"),
            "bare article / music:duration must not copy article mapping: {data:?}"
        );
        assert_ne!(
            data.open_graph
                .get("article:published_time")
                .map(String::as_str),
            Some("not-a-property-attr"),
            "name= article:* must not copy property= mapping: {data:?}"
        );
        assert!(
            !data.meta.contains_key("article:published_time")
                && !data.meta.contains_key("article:author"),
            "article properties must not copy onto standard meta: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.twitter_card["twitter:card"], "summary");
        assert!(
            !data.twitter_card.contains_key("article:published_time"),
            "article properties must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn book_open_graph_properties_are_extracted() {
        let html = r#"<html><head>
            <meta property="book:isbn" content="978-0-123456-47-2">
            <meta property="Book:author" content="https://example.test/authors/ada">
            <meta property="book:release_date" content="2026-09-14">
            <meta property="book:" content="empty-suffix">
            <meta property="book" content="too-short">
            <meta property="book-isbn" content="hyphen-not-colon">
            <meta property="music:duration" content="not-book">
            <meta property="video:duration" content="not-book-either">
            <meta property="article:section" content="Engineering">
            <meta property="og:title" content="OG Title">
            <meta name="book:isbn" content="not-a-property-attr">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta name="twitter:card" content="summary">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.open_graph["book:isbn"], "978-0-123456-47-2");
        assert_eq!(
            data.open_graph["book:author"],
            "https://example.test/authors/ada"
        );
        assert_eq!(data.open_graph["book:release_date"], "2026-09-14");
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert_eq!(data.open_graph["article:section"], "Engineering");
        assert!(
            !data.open_graph.contains_key("book:")
                && !data.open_graph.contains_key("book")
                && !data.open_graph.contains_key("book-isbn")
                && !data.open_graph.contains_key("music:duration")
                && !data.open_graph.contains_key("video:duration"),
            "bare book / music: / video: must not copy book mapping: {data:?}",
        );
        assert_ne!(
            data.open_graph.get("book:isbn").map(String::as_str),
            Some("not-a-property-attr"),
            "name= book:* must not copy property= mapping: {data:?}"
        );
        assert!(
            !data.meta.contains_key("book:isbn") && !data.meta.contains_key("book:author"),
            "book properties must not copy onto standard meta: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.twitter_card["twitter:card"], "summary");
        assert!(
            !data.twitter_card.contains_key("book:isbn"),
            "book properties must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn profile_open_graph_properties_are_extracted() {
        let html = r#"<html><head>
            <meta property="profile:first_name" content="Ada">
            <meta property="Profile:username" content="ada">
            <meta property="profile:last_name" content="Lovelace">
            <meta property="profile:" content="empty-suffix">
            <meta property="profile" content="too-short">
            <meta property="profile-username" content="hyphen-not-colon">
            <meta property="music:duration" content="not-profile">
            <meta property="video:duration" content="not-profile-either">
            <meta property="book:isbn" content="978-0-123456-47-2">
            <meta property="article:section" content="Engineering">
            <meta property="og:title" content="OG Title">
            <meta name="profile:username" content="not-a-property-attr">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta name="twitter:card" content="summary">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.open_graph["profile:first_name"], "Ada");
        assert_eq!(data.open_graph["profile:username"], "ada");
        assert_eq!(data.open_graph["profile:last_name"], "Lovelace");
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert_eq!(data.open_graph["book:isbn"], "978-0-123456-47-2");
        assert_eq!(data.open_graph["article:section"], "Engineering");
        assert!(
            !data.open_graph.contains_key("profile:")
                && !data.open_graph.contains_key("profile")
                && !data.open_graph.contains_key("profile-username")
                && !data.open_graph.contains_key("music:duration")
                && !data.open_graph.contains_key("video:duration"),
            "bare profile / music: / video: must not copy profile mapping: {data:?}"
        );
        assert_ne!(
            data.open_graph.get("profile:username").map(String::as_str),
            Some("not-a-property-attr"),
            "name= profile:* must not copy property= mapping: {data:?}"
        );
        assert!(
            !data.meta.contains_key("profile:username")
                && !data.meta.contains_key("profile:first_name"),
            "profile properties must not copy onto standard meta: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.twitter_card["twitter:card"], "summary");
        assert!(
            !data.twitter_card.contains_key("profile:username"),
            "profile properties must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn app_links_properties_are_extracted() {
        let html = r#"<html><head>
            <meta property="al:ios:url" content="plasmate://docs">
            <meta property="AL:android:url" content="plasmate://docs">
            <meta property="al:web:url" content="https://example.test/docs">
            <meta property="al:" content="empty-suffix">
            <meta property="al" content="too-short">
            <meta property="al-ios:url" content="hyphen-not-colon">
            <meta property="music:duration" content="not-app-links">
            <meta property="video:duration" content="not-app-links-either">
            <meta property="profile:username" content="ada">
            <meta property="og:title" content="OG Title">
            <meta name="al:ios:url" content="not-a-property-attr">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta name="twitter:card" content="summary">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.open_graph["al:ios:url"], "plasmate://docs");
        assert_eq!(data.open_graph["al:android:url"], "plasmate://docs");
        assert_eq!(data.open_graph["al:web:url"], "https://example.test/docs");
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert_eq!(data.open_graph["profile:username"], "ada");
        assert!(
            !data.open_graph.contains_key("al:")
                && !data.open_graph.contains_key("al")
                && !data.open_graph.contains_key("al-ios:url")
                && !data.open_graph.contains_key("music:duration")
                && !data.open_graph.contains_key("video:duration"),
            "bare al / music: / video: must not copy app links mapping: {data:?}"
        );
        assert_ne!(
            data.open_graph.get("al:ios:url").map(String::as_str),
            Some("not-a-property-attr"),
            "name= al:* must not copy property= mapping: {data:?}"
        );
        assert!(
            !data.meta.contains_key("al:ios:url") && !data.meta.contains_key("al:android:url"),
            "app links properties must not copy onto standard meta: {data:?}"
        );
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.twitter_card["twitter:card"], "summary");
        assert!(
            !data.twitter_card.contains_key("al:ios:url"),
            "app links properties must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn dublin_core_meta_is_extracted() {
        let html = r#"<html><head>
            <meta name="DC.title" content="Semantic Object Model">
            <meta name="DC.Creator" content="Plasmate Labs">
            <meta name="DCTERMS.abstract" content="Structured page models for agents">
            <meta name="dcterms.identifier" content="https://example.test/paper">
            <meta name="DC" content="no-qualifier">
            <meta name="dc." content="empty-suffix">
            <meta name="dcterms" content="too-short">
            <meta name="dc-title" content="hyphen-not-dotted">
            <meta name="eprints.title" content="not-dublin-core">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta property="DC.title" content="not-a-name-attr">
            <meta property="og:title" content="OG Title">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.meta["dc.title"], "Semantic Object Model");
        assert_eq!(data.meta["dc.creator"], "Plasmate Labs");
        assert_eq!(
            data.meta["dcterms.abstract"],
            "Structured page models for agents"
        );
        assert_eq!(
            data.meta["dcterms.identifier"],
            "https://example.test/paper"
        );
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert_eq!(data.meta["eprints.title"], "not-dublin-core");
        assert!(
            !data.meta.contains_key("dc") && !data.meta.contains_key("dc."),
            "bare DC / dc. must not be kept: {data:?}"
        );
        assert!(
            !data.meta.contains_key("dcterms"),
            "DCTERMS without qualifier must not copy Dublin Core mapping: {data:?}"
        );
        assert!(
            !data.meta.contains_key("dc-title"),
            "hyphenated dc-title must not copy Dublin Core mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("dc.title").map(String::as_str),
            Some("not-dublin-core"),
            "eprints must not copy Dublin Core mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("dc.title").map(String::as_str),
            Some("not-a-name-attr"),
            "property= DC.title must not copy name= mapping: {data:?}"
        );
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert!(
            !data.open_graph.contains_key("dc.title") && !data.open_graph.contains_key("DC.title"),
            "Dublin Core meta must not copy onto OpenGraph: {data:?}"
        );
        assert!(
            data.twitter_card.is_empty(),
            "Dublin Core meta must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn prism_meta_is_extracted() {
        let html = r#"<html><head>
            <meta name="prism.publicationName" content="Nature">
            <meta name="Prism.doi" content="10.1038/example">
            <meta name="prism.issn" content="0028-0836">
            <meta name="prism." content="empty-suffix">
            <meta name="prism" content="too-short">
            <meta name="prism-title" content="hyphen-not-dotted">
            <meta name="prisms.doi" content="not-prism">
            <meta name="eprints.title" content="not-prism-either">
            <meta name="dc.title" content="Dublin Core Title">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta property="prism.doi" content="not-a-name-attr">
            <meta property="og:title" content="OG Title">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.meta["prism.publicationname"], "Nature");
        assert_eq!(data.meta["prism.doi"], "10.1038/example");
        assert_eq!(data.meta["prism.issn"], "0028-0836");
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.meta["dc.title"], "Dublin Core Title");
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert_eq!(data.meta["eprints.title"], "not-prism-either");
        assert!(
            !data.meta.contains_key("prism") && !data.meta.contains_key("prism."),
            "bare prism / prism. must not be kept: {data:?}"
        );
        assert!(
            !data.meta.contains_key("prism-title"),
            "hyphenated prism-title must not copy PRISM mapping: {data:?}"
        );
        assert!(
            !data.meta.contains_key("prisms.doi"),
            "prisms must not copy PRISM mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("prism.doi").map(String::as_str),
            Some("not-prism-either"),
            "eprints must not copy PRISM mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("prism.doi").map(String::as_str),
            Some("not-a-name-attr"),
            "property= prism.doi must not copy name= mapping: {data:?}"
        );
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert!(
            !data.open_graph.contains_key("prism.doi")
                && !data.open_graph.contains_key("prism.publicationname"),
            "PRISM meta must not copy onto OpenGraph: {data:?}"
        );
        assert!(
            data.twitter_card.is_empty(),
            "PRISM meta must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn eprints_meta_is_extracted() {
        let html = r#"<html><head>
            <meta name="eprints.title" content="Semantic Object Model">
            <meta name="EPrints.creators_name" content="Plasmate Labs">
            <meta name="eprints.abstract" content="Structured page models for agents">
            <meta name="eprints.date" content="2026-09-14">
            <meta name="eprints." content="empty-suffix">
            <meta name="eprints" content="too-short">
            <meta name="eprints-title" content="hyphen-not-dotted">
            <meta name="eprints_title" content="underscore-not-dotted">
            <meta name="eprint.title" content="singular-not-eprints">
            <meta name="dc.title" content="Dublin Core Title">
            <meta name="prism.doi" content="10.1038/example">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta property="eprints.title" content="not-a-name-attr">
            <meta property="og:title" content="OG Title">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.meta["eprints.title"], "Semantic Object Model");
        assert_eq!(data.meta["eprints.creators_name"], "Plasmate Labs");
        assert_eq!(
            data.meta["eprints.abstract"],
            "Structured page models for agents"
        );
        assert_eq!(data.meta["eprints.date"], "2026-09-14");
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.meta["dc.title"], "Dublin Core Title");
        assert_eq!(data.meta["prism.doi"], "10.1038/example");
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert!(
            !data.meta.contains_key("eprints") && !data.meta.contains_key("eprints."),
            "bare eprints / eprints. must not be kept: {data:?}"
        );
        assert!(
            !data.meta.contains_key("eprints-title")
                && !data.meta.contains_key("eprints_title")
                && !data.meta.contains_key("eprint.title"),
            "hyphen/underscore/singular eprints keys must not copy EPrints mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("eprints.title").map(String::as_str),
            Some("not-a-name-attr"),
            "property= eprints.title must not copy name= mapping: {data:?}"
        );
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert!(
            !data.open_graph.contains_key("eprints.title")
                && !data.open_graph.contains_key("eprints.creators_name"),
            "EPrints meta must not copy onto OpenGraph: {data:?}"
        );
        assert!(
            data.twitter_card.is_empty(),
            "EPrints meta must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn bepress_citation_meta_is_extracted() {
        let html = r#"<html><head>
            <meta name="bepress_citation_title" content="Semantic Object Model">
            <meta name="Bepress_Citation_Author" content="Plasmate Labs">
            <meta name="bepress_citation_pdf_url" content="https://example.test/paper.pdf">
            <meta name="bepress_citation_date" content="2026-09-14">
            <meta name="bepress_citation_" content="empty-suffix">
            <meta name="bepress_citation" content="too-short">
            <meta name="bepress" content="missing-citation">
            <meta name="bepress-citation_title" content="hyphen-not-underscore">
            <meta name="bepress.citation_title" content="dot-not-underscore">
            <meta name="bepresses_citation_title" content="plural-not-bepress">
            <meta name="eprints.title" content="not-bepress">
            <meta name="dc.title" content="Dublin Core Title">
            <meta name="prism.doi" content="10.1038/example">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta property="bepress_citation_title" content="not-a-name-attr">
            <meta property="og:title" content="OG Title">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.meta["bepress_citation_title"], "Semantic Object Model");
        assert_eq!(data.meta["bepress_citation_author"], "Plasmate Labs");
        assert_eq!(
            data.meta["bepress_citation_pdf_url"],
            "https://example.test/paper.pdf"
        );
        assert_eq!(data.meta["bepress_citation_date"], "2026-09-14");
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.meta["dc.title"], "Dublin Core Title");
        assert_eq!(data.meta["prism.doi"], "10.1038/example");
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert_eq!(data.meta["eprints.title"], "not-bepress");
        assert!(
            !data.meta.contains_key("bepress")
                && !data.meta.contains_key("bepress_citation")
                && !data.meta.contains_key("bepress_citation_"),
            "bare bepress / bepress_citation / bepress_citation_ must not be kept: {data:?}"
        );
        assert!(
            !data.meta.contains_key("bepress-citation_title")
                && !data.meta.contains_key("bepress.citation_title")
                && !data.meta.contains_key("bepresses_citation_title"),
            "hyphen/dot/plural bepress keys must not copy Bepress mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("bepress_citation_title").map(String::as_str),
            Some("not-a-name-attr"),
            "property= bepress_citation_title must not copy name= mapping: {data:?}"
        );
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert!(
            !data.open_graph.contains_key("bepress_citation_title")
                && !data.open_graph.contains_key("bepress_citation_author"),
            "Bepress meta must not copy onto OpenGraph: {data:?}"
        );
        assert!(
            data.twitter_card.is_empty(),
            "Bepress meta must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn schema_itemprop_meta_is_extracted() {
        let html = r#"<html><head>
            <meta itemprop="name" content="Semantic Object Model">
            <meta itemprop="DatePublished" content="2026-09-14">
            <meta itemprop="image" content="https://example.test/paper.png">
            <meta itemprop="" content="empty-itemprop">
            <meta itemprop="   " content="whitespace-itemprop">
            <meta itemprop="name url" content="multi-token">
            <meta name="datePublished" content="not-an-itemprop-name">
            <meta name="eprints.title" content="not-itemprop">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta property="name" content="not-an-itemprop-attr">
            <meta property="og:title" content="OG Title">
        </head><body>
            <span itemprop="sku">ABC123</span>
            <p>Body</p>
        </body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.meta["name"], "Semantic Object Model");
        assert_eq!(data.meta["datepublished"], "2026-09-14");
        assert_eq!(data.meta["image"], "https://example.test/paper.png");
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.meta["eprints.title"], "not-itemprop");
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert!(
            !data.meta.contains_key("sku"),
            "non-meta itemprop must not copy meta mapping: {data:?}"
        );
        assert!(
            data.meta.values().all(|value| {
                value != "empty-itemprop"
                    && value != "whitespace-itemprop"
                    && value != "multi-token"
            }),
            "empty or multi-token itemprop must not be kept: {data:?}"
        );
        assert!(
            !data.meta.contains_key("name url")
                && data.meta.get("name").map(String::as_str) != Some("multi-token"),
            "multi-token itemprop must not be kept: {data:?}"
        );
        assert_ne!(
            data.meta.get("datepublished").map(String::as_str),
            Some("not-an-itemprop-name"),
            "name= datePublished must not copy itemprop mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("name").map(String::as_str),
            Some("not-an-itemprop-attr"),
            "property= name must not copy itemprop mapping: {data:?}"
        );
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert!(
            !data.open_graph.contains_key("name") && !data.open_graph.contains_key("datepublished"),
            "itemprop meta must not copy onto OpenGraph: {data:?}"
        );
        assert!(
            data.twitter_card.is_empty(),
            "itemprop meta must not copy onto Twitter cards: {data:?}"
        );
    }

    #[test]
    fn fediverse_creator_meta_is_extracted() {
        let html = r#"<html><head>
            <meta name="fediverse:creator" content="@plasmate@example.test">
            <meta name="Fediverse:Creator" content="@alias@example.test">
            <meta name="fediverse:creator:id" content="https://example.test/users/plasmate">
            <meta name="fediverse:" content="empty-suffix">
            <meta name="fediverse" content="too-short">
            <meta name="fediverse-creator" content="hyphen-not-colon">
            <meta name="fediverse_creator" content="underscore-not-colon">
            <meta name="twitter:creator" content="@twitter">
            <meta name="eprints.title" content="not-fediverse">
            <meta name="dc.title" content="Dublin Core Title">
            <meta name="citation_title" content="Highwire Title">
            <meta name="description" content="A paper">
            <meta property="fediverse:creator" content="not-a-name-attr">
            <meta property="og:title" content="OG Title">
        </head><body><p>Body</p></body></html>"#;
        let data = extract_structured_data(html);
        assert_eq!(data.meta["fediverse:creator"], "@alias@example.test");
        assert_eq!(
            data.meta["fediverse:creator:id"],
            "https://example.test/users/plasmate"
        );
        assert_eq!(data.meta["description"], "A paper");
        assert_eq!(data.meta["dc.title"], "Dublin Core Title");
        assert_eq!(data.meta["citation_title"], "Highwire Title");
        assert_eq!(data.meta["eprints.title"], "not-fediverse");
        assert!(
            !data.meta.contains_key("fediverse") && !data.meta.contains_key("fediverse:"),
            "bare fediverse / fediverse: must not be kept: {data:?}"
        );
        assert!(
            !data.meta.contains_key("fediverse-creator")
                && !data.meta.contains_key("fediverse_creator"),
            "hyphen/underscore fediverse keys must not copy fediverse mapping: {data:?}"
        );
        assert_ne!(
            data.meta.get("fediverse:creator").map(String::as_str),
            Some("not-a-name-attr"),
            "property= fediverse:creator must not copy name= mapping: {data:?}"
        );
        assert_eq!(data.open_graph["og:title"], "OG Title");
        assert!(
            !data.open_graph.contains_key("fediverse:creator")
                && !data.open_graph.contains_key("fediverse:creator:id"),
            "fediverse meta must not copy onto OpenGraph: {data:?}"
        );
        assert_eq!(data.twitter_card["twitter:creator"], "@twitter");
        assert!(
            !data.twitter_card.contains_key("fediverse:creator"),
            "fediverse meta must not copy onto Twitter cards: {data:?}"
        );
    }
}
