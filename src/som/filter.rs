//! Region/element filtering for SOM snapshots.
//!
//! Provides `apply_selector` to narrow a SOM down to specific regions or
//! elements by semantic role or HTML id. Used by both the CLI (`--selector`)
//! and MCP tools (`selector` parameter).

use super::types::{Element, ElementRole, RegionRole, ShadowRoot, Som};

/// Filter a SOM to a specific region or element by semantic selector.
///
/// Supported selectors:
/// - Region roles: `main`, `nav`/`navigation`, `aside`, `header`, `footer`,
///   `form`, `dialog`, `content`
/// - Element roles: `link`, `button`, `text_input`, `textarea`, `select`,
///   `checkbox`, `radio`, `heading`, `image`, `list`, `table`, `paragraph`,
///   `section`, `group`, `separator`, `details`, `iframe`
/// - Action surfaces: `interactive` or `action:click` / `action:type` /
///   `action:clear` / `action:select` / `action:toggle`
/// - Text affordances: `text:query`, `label:query`, `placeholder:query`,
///   `description:query`
/// - Locator attributes: `test_id:value`, `[data-testid=value]`,
///   `[name=q]`, `[aria-label="Save"]`, `[required]`, `input[type=search]`
/// - HTML id: `#some-id` - keeps only elements whose `html_id` matches
///
/// Unrecognised selectors return the full SOM unchanged (with a warning to stderr).
/// If a recognised selector matches nothing, the full SOM is returned (with a
/// warning) so callers always get usable output.
pub fn apply_selector(som: &Som, selector: &str) -> Som {
    let selector = selector.trim();

    // Try to match a region role
    let role_opt: Option<RegionRole> = match selector.to_lowercase().as_str() {
        "main" => Some(RegionRole::Main),
        "nav" | "navigation" => Some(RegionRole::Navigation),
        "aside" => Some(RegionRole::Aside),
        "header" => Some(RegionRole::Header),
        "footer" => Some(RegionRole::Footer),
        "form" => Some(RegionRole::Form),
        "dialog" => Some(RegionRole::Dialog),
        "content" => Some(RegionRole::Content),
        _ => None,
    };

    if let Some(role) = role_opt {
        let filtered: Vec<_> = som
            .regions
            .iter()
            .filter(|r| r.role == role)
            .cloned()
            .collect();
        if filtered.is_empty() {
            eprintln!(
                "Warning: selector '{}' matched no regions - returning full SOM",
                selector
            );
            return som.clone();
        }
        let mut result = som.clone();
        result.regions = filtered;
        return result;
    }

    // Match element roles, preserving parent containers and shadow roots that
    // contain matching descendants.
    if let Some(role) = parse_element_role(selector) {
        return filter_som_elements(som, selector, |element| element.role == role);
    }

    // Match only actionable elements. This is useful when an agent needs a
    // compact menu of possible targets without full body text.
    if selector.eq_ignore_ascii_case("interactive") {
        return filter_som_elements(som, selector, |element| element.role.is_interactive());
    }

    // Match elements that expose a specific action in their compact action list.
    let selector_lower = selector.to_ascii_lowercase();
    if let Some(action) = selector_lower.strip_prefix("action:") {
        let action = action.trim().to_ascii_lowercase();
        if !action.is_empty() {
            return filter_som_elements(som, selector, |element| {
                element
                    .actions
                    .as_ref()
                    .map(|actions| actions.iter().any(|a| a.eq_ignore_ascii_case(&action)))
                    .unwrap_or(false)
            });
        }
    }

    for (prefix, matcher) in [
        (
            "text:",
            element_matches_text_query as fn(&Element, &str) -> bool,
        ),
        ("label:", element_matches_label_query),
        ("placeholder:", element_matches_placeholder_query),
        ("description:", element_matches_description_query),
    ] {
        if let Some(query) = strip_case_insensitive_prefix(selector, prefix) {
            let query = query.trim();
            if !query.is_empty() {
                return filter_som_elements(som, selector, |element| matcher(element, query));
            }
        }
    }

    for prefix in [
        "test_id:",
        "testid:",
        "data-testid:",
        "data-test-id:",
        "data-test:",
        "data-qa:",
    ] {
        if let Some(value) = strip_case_insensitive_prefix(selector, prefix) {
            let value = value.trim();
            if !value.is_empty() {
                return filter_som_elements(som, selector, |element| {
                    attr_string(element.attrs.as_ref(), "test_id") == Some(value)
                });
            }
        }
    }

    if let Some((tag, attr)) = selector_attribute(selector) {
        return filter_som_elements(som, selector, |element| {
            tag.map(|tag| element_matches_tag(element, tag))
                .unwrap_or(true)
                && element_matches_attribute(element, attr.name, attr.value)
        });
    }

    // Try id selector: #my-id. Prefer documented region ids, then HTML ids
    // on elements. If neither matches, return the full SOM as a graceful fallback.
    if let Some(id) = selector.strip_prefix('#') {
        let region_matches: Vec<_> = som.regions.iter().filter(|r| r.id == id).cloned().collect();
        if !region_matches.is_empty() {
            let mut result = som.clone();
            result.regions = region_matches;
            return result;
        }

        let filtered_regions: Vec<_> = som
            .regions
            .iter()
            .filter_map(|r| {
                let els = filter_elements_by_html_id(&r.elements, id);
                if els.is_empty() {
                    None
                } else {
                    let mut region = r.clone();
                    region.elements = els;
                    Some(region)
                }
            })
            .collect();
        if filtered_regions.is_empty() {
            eprintln!("Warning: selector '#{id}' matched no elements - returning full SOM");
            return som.clone();
        }
        let mut result = som.clone();
        result.regions = filtered_regions;
        return result;
    }

    eprintln!(
        "Warning: unrecognised selector '{}' - returning full SOM",
        selector
    );
    som.clone()
}

fn parse_element_role(selector: &str) -> Option<ElementRole> {
    match selector
        .trim()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "link" => Some(ElementRole::Link),
        "button" => Some(ElementRole::Button),
        "text_input" | "textbox" | "input" => Some(ElementRole::TextInput),
        "textarea" => Some(ElementRole::Textarea),
        "select" => Some(ElementRole::Select),
        "checkbox" => Some(ElementRole::Checkbox),
        "radio" => Some(ElementRole::Radio),
        "heading" => Some(ElementRole::Heading),
        "image" | "img" => Some(ElementRole::Image),
        "list" => Some(ElementRole::List),
        "table" => Some(ElementRole::Table),
        "paragraph" => Some(ElementRole::Paragraph),
        "section" => Some(ElementRole::Section),
        "group" => Some(ElementRole::Group),
        "separator" => Some(ElementRole::Separator),
        "details" => Some(ElementRole::Details),
        "iframe" => Some(ElementRole::Iframe),
        _ => None,
    }
}

fn filter_som_elements<F>(som: &Som, selector: &str, matches: F) -> Som
where
    F: Fn(&Element) -> bool,
{
    let filtered_regions: Vec<_> = som
        .regions
        .iter()
        .filter_map(|region| {
            let elements = filter_elements_by(&region.elements, &matches);
            if elements.is_empty() {
                None
            } else {
                let mut region = region.clone();
                region.elements = elements;
                Some(region)
            }
        })
        .collect();

    if filtered_regions.is_empty() {
        eprintln!(
            "Warning: selector '{}' matched no elements - returning full SOM",
            selector
        );
        return som.clone();
    }

    let mut result = som.clone();
    result.regions = filtered_regions;
    result
}

fn filter_elements_by<F>(elements: &[Element], matches: &F) -> Vec<Element>
where
    F: Fn(&Element) -> bool,
{
    elements
        .iter()
        .filter_map(|element| {
            let mut cloned = element.clone();
            if let Some(children) = &element.children {
                let filtered_children = filter_elements_by(children, matches);
                cloned.children = if filtered_children.is_empty() {
                    None
                } else {
                    Some(filtered_children)
                };
            }

            let shadow_match = if let Some(shadow) = &element.shadow {
                let filtered_shadow_elements = filter_elements_by(&shadow.elements, matches);
                if filtered_shadow_elements.is_empty() {
                    false
                } else {
                    cloned.shadow = Some(ShadowRoot {
                        mode: shadow.mode.clone(),
                        elements: filtered_shadow_elements,
                    });
                    true
                }
            } else {
                false
            };

            if matches(element) || cloned.children.is_some() || shadow_match {
                Some(cloned)
            } else {
                None
            }
        })
        .collect()
}

fn strip_case_insensitive_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    value
        .get(..prefix.len())
        .filter(|head| head.eq_ignore_ascii_case(prefix))
        .and_then(|_| value.get(prefix.len()..))
}

fn element_matches_text_query(element: &Element, query: &str) -> bool {
    element
        .text
        .as_deref()
        .is_some_and(|text| contains_case_insensitive(text, query))
}

fn element_matches_label_query(element: &Element, query: &str) -> bool {
    element
        .label
        .as_deref()
        .is_some_and(|label| contains_case_insensitive(label, query))
        || attr_string_from_aria(element.attrs.as_ref(), "label")
            .is_some_and(|label| contains_case_insensitive(label, query))
}

fn element_matches_placeholder_query(element: &Element, query: &str) -> bool {
    attr_string(element.attrs.as_ref(), "placeholder")
        .or_else(|| attr_string_from_aria(element.attrs.as_ref(), "placeholder"))
        .is_some_and(|placeholder| contains_case_insensitive(placeholder, query))
}

fn element_matches_description_query(element: &Element, query: &str) -> bool {
    attr_string(element.attrs.as_ref(), "description")
        .is_some_and(|description| contains_case_insensitive(description, query))
}

fn contains_case_insensitive(value: &str, needle: &str) -> bool {
    value
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

struct SelectorAttribute<'a> {
    name: &'a str,
    value: Option<&'a str>,
}

fn selector_attribute(selector: &str) -> Option<(Option<&str>, SelectorAttribute<'_>)> {
    let selector = selector.trim();
    let open = selector.find('[')?;
    let close = selector.rfind(']')?;
    if close <= open || close != selector.len() - 1 {
        return None;
    }

    let tag = selector[..open].trim();
    let tag = if tag.is_empty() { None } else { Some(tag) };
    let body = selector[open + 1..close].trim();
    if body.is_empty() {
        return None;
    }

    let (name, value) = match body.split_once('=') {
        Some((name, value)) => {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            (name.trim(), Some(value))
        }
        None => (body, None),
    };

    if name.is_empty() {
        None
    } else {
        Some((tag, SelectorAttribute { name, value }))
    }
}

fn element_matches_tag(element: &Element, tag: &str) -> bool {
    let tag = tag.trim().to_ascii_lowercase();
    if attr_string(element.attrs.as_ref(), "source_role").is_some_and(|role| {
        role.split_whitespace()
            .any(|token| token.eq_ignore_ascii_case(&tag))
    }) {
        return true;
    }

    match tag.as_str() {
        "a" => element.role == ElementRole::Link,
        "button" => element.role == ElementRole::Button,
        "input" => matches!(
            element.role,
            ElementRole::TextInput
                | ElementRole::Checkbox
                | ElementRole::Radio
                | ElementRole::Button
        ),
        "textarea" => element.role == ElementRole::Textarea,
        "select" => element.role == ElementRole::Select,
        "img" | "image" => element.role == ElementRole::Image,
        "iframe" => element.role == ElementRole::Iframe,
        "details" => element.role == ElementRole::Details,
        "table" => element.role == ElementRole::Table,
        "section" => element.role == ElementRole::Section,
        "p" | "paragraph" => element.role == ElementRole::Paragraph,
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => element.role == ElementRole::Heading,
        other => element.role.as_str().eq_ignore_ascii_case(other),
    }
}

fn element_matches_attribute(element: &Element, name: &str, expected: Option<&str>) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    let actual = match normalized.as_str() {
        "id" => element.html_id.as_deref(),
        "data-testid" | "data-test-id" | "data-test" | "data-qa" | "test_id" => {
            attr_string(element.attrs.as_ref(), "test_id")
        }
        "role" => {
            attr_string(element.attrs.as_ref(), "source_role").or(Some(element.role.as_str()))
        }
        "aria-label" | "label" => element
            .label
            .as_deref()
            .or_else(|| attr_string_from_aria(element.attrs.as_ref(), "label")),
        "aria-labelledby" => attr_string_from_aria(element.attrs.as_ref(), "labelledby"),
        "aria-describedby" => attr_string_from_aria(element.attrs.as_ref(), "describedby"),
        "type" => attr_string(element.attrs.as_ref(), "type")
            .or_else(|| attr_string(element.attrs.as_ref(), "input_type"))
            .or_else(|| attr_string(element.attrs.as_ref(), "button_type")),
        other => attr_string(element.attrs.as_ref(), &other.replace('-', "_"))
            .or_else(|| attr_string(element.attrs.as_ref(), other))
            .or_else(|| {
                other
                    .strip_prefix("aria-")
                    .and_then(|key| attr_string_from_aria(element.attrs.as_ref(), key))
            }),
    };

    match (actual, expected) {
        (Some(_), None) => true,
        (Some(actual), Some(expected)) => actual.eq_ignore_ascii_case(expected),
        (None, Some(expected)) => attr_bool_matches(element.attrs.as_ref(), &normalized, expected),
        (None, None) => attr_bool_present(element.attrs.as_ref(), &normalized),
    }
}

fn attr_string<'a>(attrs: Option<&'a serde_json::Value>, key: &str) -> Option<&'a str> {
    attrs
        .and_then(|attrs| attrs.get(key))
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
}

fn attr_string_from_aria<'a>(attrs: Option<&'a serde_json::Value>, key: &str) -> Option<&'a str> {
    attrs
        .and_then(|attrs| attrs.get("aria"))
        .and_then(|aria| aria.get(key))
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
}

fn attr_bool_matches(attrs: Option<&serde_json::Value>, key: &str, expected: &str) -> bool {
    let expected = expected.trim().to_ascii_lowercase();
    attr_bool_present(attrs, key) && (expected.is_empty() || expected == "true" || expected == key)
}

fn attr_bool_present(attrs: Option<&serde_json::Value>, key: &str) -> bool {
    let key = key.replace('-', "_");
    attrs
        .and_then(|attrs| attrs.get(&key))
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn filter_elements_by_html_id(elements: &[Element], id: &str) -> Vec<Element> {
    elements
        .iter()
        .filter_map(|element| {
            let mut cloned = element.clone();
            if let Some(children) = &element.children {
                let filtered_children = filter_elements_by_html_id(children, id);
                cloned.children = if filtered_children.is_empty() {
                    None
                } else {
                    Some(filtered_children)
                };
            }

            let shadow_match = if let Some(shadow) = &element.shadow {
                let filtered_shadow_elements = filter_elements_by_html_id(&shadow.elements, id);
                if filtered_shadow_elements.is_empty() {
                    false
                } else {
                    cloned.shadow = Some(ShadowRoot {
                        mode: shadow.mode.clone(),
                        elements: filtered_shadow_elements,
                    });
                    true
                }
            } else {
                false
            };

            if element.html_id.as_deref() == Some(id) || cloned.children.is_some() || shadow_match {
                Some(cloned)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::som::types::*;
    use serde_json::json;

    fn make_test_som() -> Som {
        Som {
            som_version: "1".to_string(),
            url: "https://example.com".to_string(),
            title: "Test".to_string(),
            lang: "en".to_string(),
            regions: vec![
                Region {
                    id: "r1".to_string(),
                    role: RegionRole::Navigation,
                    label: None,
                    action: None,
                    method: None,
                    target: None,
                    enctype: None,
                    novalidate: None,
                    accept_charset: None,
                    autocomplete: None,
                    elements: vec![Element {
                        id: "e1".to_string(),
                        role: ElementRole::Link,
                        html_id: None,
                        text: Some("Home".to_string()),
                        label: None,
                        actions: Some(vec!["click".to_string()]),
                        attrs: None,
                        children: None,
                        hints: None,
                        shadow: None,
                    }],
                },
                Region {
                    id: "r2".to_string(),
                    role: RegionRole::Main,
                    label: None,
                    action: None,
                    method: None,
                    target: None,
                    enctype: None,
                    novalidate: None,
                    accept_charset: None,
                    autocomplete: None,
                    elements: vec![
                        Element {
                            id: "e2".to_string(),
                            role: ElementRole::Paragraph,
                            html_id: Some("intro".to_string()),
                            text: Some("Hello world".to_string()),
                            label: None,
                            actions: None,
                            attrs: None,
                            children: None,
                            hints: None,
                            shadow: None,
                        },
                        Element {
                            id: "e3".to_string(),
                            role: ElementRole::Button,
                            html_id: Some("save".to_string()),
                            text: Some("Save".to_string()),
                            label: None,
                            actions: Some(vec!["click".to_string()]),
                            attrs: None,
                            children: None,
                            hints: None,
                            shadow: None,
                        },
                        Element {
                            id: "e4".to_string(),
                            role: ElementRole::TextInput,
                            html_id: Some("search-box".to_string()),
                            text: None,
                            label: Some("Site Search".to_string()),
                            actions: Some(vec!["type".to_string(), "clear".to_string()]),
                            attrs: Some(json!({
                                "input_type": "search",
                                "name": "q",
                                "placeholder": "Search docs",
                                "required": true,
                                "test_id": "global-search",
                                "description": "Find product documentation",
                                "aria": {
                                    "label": "Site Search",
                                    "describedby": "search-help"
                                }
                            })),
                            children: None,
                            hints: None,
                            shadow: None,
                        },
                    ],
                },
            ],
            meta: SomMeta {
                html_bytes: 100,
                som_bytes: 50,
                element_count: 4,
                interactive_count: 3,
            },
            structured_data: None,
        }
    }

    #[test]
    fn test_selector_main() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "main");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].role, RegionRole::Main);
    }

    #[test]
    fn test_selector_nav() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "nav");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].role, RegionRole::Navigation);
    }

    #[test]
    fn test_selector_html_id() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "#intro");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(
            filtered.regions[0].elements[0].html_id.as_deref(),
            Some("intro")
        );
    }

    #[test]
    fn test_selector_element_role() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "button");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].role, ElementRole::Button);
    }

    #[test]
    fn test_selector_interactive_elements() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "interactive");
        assert_eq!(filtered.regions.len(), 2);
        assert!(filtered
            .regions
            .iter()
            .flat_map(|region| region.elements.iter())
            .all(|element| element.role.is_interactive()));
    }

    #[test]
    fn test_selector_action() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "action:click");
        assert_eq!(filtered.regions.len(), 2);
        assert!(filtered
            .regions
            .iter()
            .flat_map(|region| region.elements.iter())
            .all(|element| element
                .actions
                .as_ref()
                .is_some_and(|actions| actions.contains(&"click".to_string()))));
    }

    #[test]
    fn test_selector_text_and_label_queries_are_case_insensitive() {
        let som = make_test_som();

        let text_filtered = apply_selector(&som, "text:hello");
        assert_eq!(text_filtered.regions.len(), 1);
        assert_eq!(
            text_filtered.regions[0].elements[0].html_id.as_deref(),
            Some("intro")
        );

        let label_filtered = apply_selector(&som, "LABEL:site search");
        assert_eq!(label_filtered.regions.len(), 1);
        assert_eq!(
            label_filtered.regions[0].elements[0].html_id.as_deref(),
            Some("search-box")
        );
    }

    #[test]
    fn test_selector_placeholder_and_description_queries() {
        let som = make_test_som();

        let placeholder_filtered = apply_selector(&som, "placeholder:docs");
        assert_eq!(placeholder_filtered.regions.len(), 1);
        assert_eq!(
            placeholder_filtered.regions[0].elements[0]
                .html_id
                .as_deref(),
            Some("search-box")
        );

        let description_filtered = apply_selector(&som, "description:product");
        assert_eq!(description_filtered.regions.len(), 1);
        assert_eq!(
            description_filtered.regions[0].elements[0]
                .html_id
                .as_deref(),
            Some("search-box")
        );
    }

    #[test]
    fn test_selector_locator_and_attribute_queries() {
        let som = make_test_som();

        let test_id_filtered = apply_selector(&som, "test_id:global-search");
        assert_eq!(test_id_filtered.regions.len(), 1);
        assert_eq!(
            test_id_filtered.regions[0].elements[0].html_id.as_deref(),
            Some("search-box")
        );

        let name_filtered = apply_selector(&som, "[name=q]");
        assert_eq!(name_filtered.regions.len(), 1);
        assert_eq!(
            name_filtered.regions[0].elements[0].html_id.as_deref(),
            Some("search-box")
        );

        let tag_attr_filtered = apply_selector(&som, "input[type=search]");
        assert_eq!(tag_attr_filtered.regions.len(), 1);
        assert_eq!(
            tag_attr_filtered.regions[0].elements[0].html_id.as_deref(),
            Some("search-box")
        );

        let required_filtered = apply_selector(&som, "[required]");
        assert_eq!(required_filtered.regions.len(), 1);
        assert_eq!(
            required_filtered.regions[0].elements[0].html_id.as_deref(),
            Some("search-box")
        );
    }

    #[test]
    fn test_selector_region_id() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "#r2");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].id, "r2");
    }

    #[test]
    fn test_selector_trims_whitespace() {
        let som = make_test_som();
        let filtered = apply_selector(&som, " main ");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].role, RegionRole::Main);
    }

    #[test]
    fn test_selector_nested_html_id() {
        let mut som = make_test_som();
        som.regions[0].elements[0].children = Some(vec![Element {
            id: "e-child".to_string(),
            role: ElementRole::Button,
            html_id: Some("nested-action".to_string()),
            text: Some("Act".to_string()),
            label: None,
            actions: None,
            attrs: None,
            children: None,
            hints: None,
            shadow: None,
        }]);

        let filtered = apply_selector(&som, "#nested-action");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements.len(), 1);
        let children = filtered.regions[0].elements[0].children.as_ref().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].html_id.as_deref(), Some("nested-action"));
    }

    #[test]
    fn test_selector_no_match_returns_full() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "dialog");
        assert_eq!(filtered.regions.len(), 2); // Full SOM returned
    }

    #[test]
    fn test_selector_unrecognised_returns_full() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "foobar");
        assert_eq!(filtered.regions.len(), 2);
    }
}
