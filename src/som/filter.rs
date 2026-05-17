//! Region/element filtering for SOM snapshots.
//!
//! Provides `apply_selector` to narrow a SOM down to specific regions or
//! elements by semantic role, accessible text, or stable DOM/replay attrs. Used
//! by both the CLI (`--selector`) and MCP tools (`selector` parameter).

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
/// - Accessible text: `text:Submit` or `label:Search`, case-insensitive
/// - HTML id: `#some-id` - keeps only elements whose `html_id` matches
/// - Replay attrs: `test_id:save`, `[data-testid=save]`, `[name=q]`,
///   `[aria-label="Save"]`, `[required]`, or tag-qualified `input[type=search]`
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

    if let Some(query) = selector_lower
        .strip_prefix("text:")
        .map(|_| &selector[5..])
        .map(str::trim)
        .filter(|query| !query.is_empty())
    {
        return filter_som_elements(som, selector, |element| {
            element
                .text
                .as_deref()
                .map(|text| contains_case_insensitive(text, query))
                .unwrap_or(false)
        });
    }

    if let Some(query) = selector_lower
        .strip_prefix("label:")
        .map(|_| &selector[6..])
        .map(str::trim)
        .filter(|query| !query.is_empty())
    {
        return filter_som_elements(som, selector, |element| {
            element_accessible_label(element)
                .map(|label| contains_case_insensitive(label, query))
                .unwrap_or(false)
        });
    }

    if let Some(query) = parse_prefixed_test_id(selector) {
        return filter_som_elements(som, selector, |element| {
            element_test_id(element) == Some(query)
        });
    }

    if let Some(attr_selector) = parse_attribute_selector(selector) {
        return filter_som_elements(som, selector, |element| {
            element_matches_attribute_selector(element, &attr_selector)
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttributeSelector {
    tag: Option<String>,
    attr: String,
    value: Option<String>,
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn element_accessible_label(element: &Element) -> Option<&str> {
    element.label.as_deref().or(element.text.as_deref())
}

fn parse_prefixed_test_id(selector: &str) -> Option<&str> {
    let (prefix, value) = selector.split_once(':')?;
    match prefix
        .trim()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "test_id" | "testid" | "data_testid" | "data_test_id" | "data_test" | "data_qa" => {
            let value = value.trim();
            (!value.is_empty()).then_some(value)
        }
        _ => None,
    }
}

fn parse_attribute_selector(selector: &str) -> Option<AttributeSelector> {
    let selector = selector.trim();
    let open = selector.find('[')?;
    let close = selector.rfind(']')?;
    if close <= open || !selector[close + 1..].trim().is_empty() {
        return None;
    }

    let tag = selector[..open].trim();
    let tag = if tag.is_empty() {
        None
    } else {
        Some(tag.to_ascii_lowercase())
    };
    let body = selector[open + 1..close].trim();
    if body.is_empty() {
        return None;
    }

    let (attr, value) = if let Some((attr, value)) = body.split_once('=') {
        let attr = attr.trim();
        let value = strip_selector_quotes(value.trim());
        (attr, Some(value.to_string()))
    } else {
        (body, None)
    };

    if attr.is_empty() {
        return None;
    }

    Some(AttributeSelector {
        tag,
        attr: attr.to_ascii_lowercase(),
        value,
    })
}

fn strip_selector_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(value)
}

fn element_matches_attribute_selector(element: &Element, selector: &AttributeSelector) -> bool {
    if let Some(tag) = &selector.tag {
        if !element_matches_tag_hint(element, tag) {
            return false;
        }
    }

    let Some(actual) = element_attribute_value(element, &selector.attr) else {
        return false;
    };

    selector
        .value
        .as_deref()
        .map(|expected| actual == expected)
        .unwrap_or(true)
}

fn element_matches_tag_hint(element: &Element, tag: &str) -> bool {
    match tag {
        "a" => element.role == ElementRole::Link,
        "button" => element.role == ElementRole::Button,
        "input" => matches!(
            element.role,
            ElementRole::TextInput | ElementRole::Checkbox | ElementRole::Radio
        ),
        "textarea" => element.role == ElementRole::Textarea,
        "select" => element.role == ElementRole::Select,
        "img" => element.role == ElementRole::Image,
        "iframe" => element.role == ElementRole::Iframe,
        "details" => element.role == ElementRole::Details,
        _ => element.role.as_str() == tag.replace('-', "_"),
    }
}

fn element_attribute_value<'a>(element: &'a Element, attr: &str) -> Option<&'a str> {
    match attr {
        "id" => element.html_id.as_deref(),
        "data-plasmate-id" => Some(element.id.as_str()),
        "data-som-role" | "role" => Some(element.role.as_str()),
        "aria-label" => element.label.as_deref(),
        "data-testid" | "data-test-id" | "data-test" | "data-qa" | "test_id" | "testid" => {
            element_test_id(element)
        }
        "type" => attr_string(element.attrs.as_ref(), "type")
            .or_else(|| attr_string(element.attrs.as_ref(), "input_type"))
            .or_else(|| attr_string(element.attrs.as_ref(), "button_type")),
        "aria-labelledby" => attr_string(element.attrs.as_ref(), "labelledby")
            .or_else(|| nested_attr_string(element.attrs.as_ref(), "aria", "labelledby")),
        "aria-describedby" => attr_string(element.attrs.as_ref(), "describedby")
            .or_else(|| nested_attr_string(element.attrs.as_ref(), "aria", "describedby")),
        _ => attr_string(element.attrs.as_ref(), attr)
            .or_else(|| bool_attr_string(element.attrs.as_ref(), attr)),
    }
}

fn element_test_id(element: &Element) -> Option<&str> {
    attr_string(element.attrs.as_ref(), "test_id")
}

fn attr_string<'a>(attrs: Option<&'a serde_json::Value>, key: &str) -> Option<&'a str> {
    attrs
        .and_then(|attrs| attrs.get(key))
        .and_then(|v| v.as_str())
}

fn nested_attr_string<'a>(
    attrs: Option<&'a serde_json::Value>,
    parent: &str,
    key: &str,
) -> Option<&'a str> {
    attrs
        .and_then(|attrs| attrs.get(parent))
        .and_then(|parent| parent.get(key))
        .and_then(|v| v.as_str())
}

fn bool_attr_string<'a>(attrs: Option<&'a serde_json::Value>, key: &str) -> Option<&'a str> {
    attrs
        .and_then(|attrs| attrs.get(key))
        .and_then(|value| value.as_bool())
        .and_then(|value| value.then_some("true"))
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
                            label: Some("Save changes".to_string()),
                            actions: Some(vec!["click".to_string()]),
                            attrs: Some(json!({
                                "test_id": "save-action",
                                "name": "save",
                                "button_type": "submit"
                            })),
                            children: None,
                            hints: None,
                            shadow: None,
                        },
                        Element {
                            id: "e4".to_string(),
                            role: ElementRole::TextInput,
                            html_id: Some("q".to_string()),
                            text: None,
                            label: Some("Search query".to_string()),
                            actions: Some(vec!["type".to_string(), "clear".to_string()]),
                            attrs: Some(json!({
                                "test_id": "search-field",
                                "name": "q",
                                "input_type": "search",
                                "required": true
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
                element_count: 3,
                interactive_count: 2,
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
    fn test_selector_text_prefix_matches_case_insensitive() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "text:home");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(
            filtered.regions[0].elements[0].text.as_deref(),
            Some("Home")
        );
    }

    #[test]
    fn test_selector_label_prefix_matches_accessible_name() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "label:search");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(
            filtered.regions[0].elements[0].label.as_deref(),
            Some("Search query")
        );
    }

    #[test]
    fn test_selector_test_id_prefix() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "test_id:save-action");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e3");
    }

    #[test]
    fn test_selector_attribute_value() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "[data-testid=search-field]");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e4");
    }

    #[test]
    fn test_selector_attribute_existence() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "[required]");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e4");
    }

    #[test]
    fn test_selector_tag_qualified_attribute() {
        let som = make_test_som();
        let filtered = apply_selector(&som, "input[type=search]");
        assert_eq!(filtered.regions.len(), 1);
        assert_eq!(filtered.regions[0].elements[0].id, "e4");
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
