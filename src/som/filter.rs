//! Region/element filtering for SOM snapshots.
//!
//! Provides `apply_selector` to narrow a SOM down to specific regions or
//! elements by semantic role or HTML id. Used by both the CLI (`--selector`)
//! and MCP tools (`selector` parameter).

use super::types::{Element, RegionRole, ShadowRoot, Som};

/// Filter a SOM to a specific region or element by semantic selector.
///
/// Supported selectors:
/// - Region roles: `main`, `nav`/`navigation`, `aside`, `header`, `footer`,
///   `form`, `dialog`, `content`
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
                    elements: vec![Element {
                        id: "e1".to_string(),
                        role: ElementRole::Link,
                        html_id: None,
                        text: Some("Home".to_string()),
                        label: None,
                        actions: None,
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
                    elements: vec![Element {
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
                    }],
                },
            ],
            meta: SomMeta {
                html_bytes: 100,
                som_bytes: 50,
                element_count: 2,
                interactive_count: 1,
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
