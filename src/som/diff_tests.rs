use super::*;
use crate::som::types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_meta() -> SomMeta {
    SomMeta {
        html_bytes: 0,
        som_bytes: 0,
        element_count: 0,
        interactive_count: 0,
    }
}

fn empty_som() -> Som {
    Som {
        som_version: "1".into(),
        url: "https://example.com".into(),
        title: "Test".into(),
        lang: "en".into(),
        regions: vec![],
        meta: empty_meta(),
        structured_data: None,
    }
}

fn make_element(id: &str, role: ElementRole, text: Option<&str>) -> Element {
    Element {
        id: id.into(),
        role,
        html_id: None,
        text: text.map(String::from),
        label: None,
        actions: None,
        attrs: None,
        children: None,
        hints: None,
    }
}

fn make_region(id: &str, role: RegionRole, elements: Vec<Element>) -> Region {
    Region {
        id: id.into(),
        role,
        label: None,
        action: None,
        method: None,
        elements,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_empty_vs_empty() {
    let old = empty_som();
    let new = empty_som();
    let diff = diff_soms(&old, &new, false);

    assert_eq!(diff.summary.total_changes, 0);
    assert!(diff.regions.is_empty());
    assert!(diff.page.title_change.is_none());
    assert!(diff.page.url_change.is_none());
}

#[test]
fn test_identical_soms() {
    let elem = make_element("e1", ElementRole::Paragraph, Some("Hello"));
    let region = make_region("r1", RegionRole::Main, vec![elem]);
    let som = Som {
        regions: vec![region],
        meta: SomMeta {
            element_count: 1,
            ..empty_meta()
        },
        ..empty_som()
    };

    let diff = diff_soms(&som, &som, false);
    assert_eq!(diff.summary.total_changes, 0);
    assert!(diff.regions.is_empty());
}

#[test]
fn test_added_element() {
    let old = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![])],
        meta: SomMeta {
            element_count: 0,
            ..empty_meta()
        },
        ..empty_som()
    };

    let new = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("New"))],
        )],
        meta: SomMeta {
            element_count: 1,
            ..empty_meta()
        },
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert_eq!(diff.summary.elements_added, 1);
    assert_eq!(diff.summary.elements_removed, 0);
    assert_eq!(diff.summary.elements_modified, 0);
    assert!(diff.summary.has_structural_changes);
}

#[test]
fn test_removed_element() {
    let old = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("Old"))],
        )],
        meta: SomMeta {
            element_count: 1,
            ..empty_meta()
        },
        ..empty_som()
    };

    let new = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![])],
        meta: SomMeta {
            element_count: 0,
            ..empty_meta()
        },
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert_eq!(diff.summary.elements_removed, 1);
    assert_eq!(diff.summary.elements_added, 0);
    assert!(diff.summary.has_structural_changes);
}

#[test]
fn test_modified_element_text() {
    let old = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("Old text"))],
        )],
        ..empty_som()
    };

    let new = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("New text"))],
        )],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert_eq!(diff.summary.elements_modified, 1);
    assert!(diff.summary.has_content_changes);

    let region_diff = &diff.regions[0];
    let elem_diffs = region_diff.element_changes.as_ref().unwrap();
    assert_eq!(elem_diffs[0].change_type, ChangeType::Modified);
    let tc = elem_diffs[0].text_change.as_ref().unwrap();
    assert_eq!(tc.old, "Old text");
    assert_eq!(tc.new, "New text");
}

#[test]
fn test_modified_element_attributes() {
    let mut old_elem = make_element("e1", ElementRole::Link, Some("Click"));
    old_elem.attrs = Some(serde_json::json!({"href": "/old", "class": "btn"}));

    let mut new_elem = make_element("e1", ElementRole::Link, Some("Click"));
    new_elem.attrs = Some(serde_json::json!({"href": "/new", "class": "btn", "target": "_blank"}));

    let old = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![old_elem])],
        ..empty_som()
    };
    let new = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![new_elem])],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert_eq!(diff.summary.elements_modified, 1);

    let elem_diff = &diff.regions[0]
        .element_changes
        .as_ref()
        .unwrap()[0];
    let attr_changes = elem_diff.attr_changes.as_ref().unwrap();

    // href changed and target added — class unchanged.
    let href_change = attr_changes.iter().find(|a| a.key == "href").unwrap();
    assert_eq!(
        href_change.old,
        Some(serde_json::Value::String("/old".into()))
    );
    assert_eq!(
        href_change.new,
        Some(serde_json::Value::String("/new".into()))
    );

    let target_change = attr_changes.iter().find(|a| a.key == "target").unwrap();
    assert!(target_change.old.is_none());
    assert_eq!(
        target_change.new,
        Some(serde_json::Value::String("_blank".into()))
    );

    // class should not appear.
    assert!(attr_changes.iter().all(|a| a.key != "class"));
}

#[test]
fn test_added_region() {
    let old = empty_som();

    let new = Som {
        regions: vec![make_region("r1", RegionRole::Aside, vec![])],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert_eq!(diff.summary.regions_added, 1);
    assert!(diff.summary.has_structural_changes);
    assert_eq!(diff.regions[0].change_type, ChangeType::Added);
    assert_eq!(diff.regions[0].id, "r1");
}

#[test]
fn test_removed_region() {
    let old = Som {
        regions: vec![make_region("r1", RegionRole::Footer, vec![])],
        ..empty_som()
    };
    let new = empty_som();

    let diff = diff_soms(&old, &new, false);
    assert_eq!(diff.summary.regions_removed, 1);
    assert!(diff.summary.has_structural_changes);
    assert_eq!(diff.regions[0].change_type, ChangeType::Removed);
}

#[test]
fn test_price_change_detection() {
    let old = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("Price: $49.99"))],
        )],
        ..empty_som()
    };

    let new = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("Price: $59.99"))],
        )],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert!(diff.summary.has_price_changes);
}

#[test]
fn test_multiple_simultaneous_changes() {
    let old = Som {
        title: "Old Title".into(),
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![
                make_element("e1", ElementRole::Paragraph, Some("Hello")),
                make_element("e2", ElementRole::Link, Some("Click me")),
            ],
        )],
        meta: SomMeta {
            element_count: 2,
            ..empty_meta()
        },
        ..empty_som()
    };

    let new = Som {
        title: "New Title".into(),
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![
                make_element("e1", ElementRole::Paragraph, Some("World")),
                // e2 removed
                make_element("e3", ElementRole::Button, Some("Submit")),
            ],
        )],
        meta: SomMeta {
            element_count: 2,
            ..empty_meta()
        },
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert!(diff.page.title_change.is_some());
    assert_eq!(diff.summary.elements_modified, 1); // e1 text changed
    assert_eq!(diff.summary.elements_removed, 1); // e2 removed
    assert_eq!(diff.summary.elements_added, 1); // e3 added
    assert_eq!(diff.summary.total_changes, 3);
}

#[test]
fn test_nested_children_changes() {
    let mut old_elem = make_element("e1", ElementRole::List, Some("List"));
    old_elem.children = Some(vec![
        make_element("c1", ElementRole::Paragraph, Some("Item 1")),
        make_element("c2", ElementRole::Paragraph, Some("Item 2")),
    ]);

    let mut new_elem = make_element("e1", ElementRole::List, Some("List"));
    new_elem.children = Some(vec![
        make_element("c1", ElementRole::Paragraph, Some("Item 1 updated")),
        make_element("c2", ElementRole::Paragraph, Some("Item 2")),
        make_element("c3", ElementRole::Paragraph, Some("Item 3")),
    ]);

    let old = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![old_elem])],
        ..empty_som()
    };
    let new = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![new_elem])],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    // e1 is modified (children differ).
    assert_eq!(diff.summary.elements_modified, 1);

    let elem_diff = &diff.regions[0]
        .element_changes
        .as_ref()
        .unwrap()[0];
    let children = elem_diff.children_changes.as_ref().unwrap();
    // c1 modified, c3 added.
    assert!(children.iter().any(|c| c.id == "c1" && c.change_type == ChangeType::Modified));
    assert!(children.iter().any(|c| c.id == "c3" && c.change_type == ChangeType::Added));
}

#[test]
fn test_role_change() {
    let old = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("Text"))],
        )],
        ..empty_som()
    };

    let new = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Heading, Some("Text"))],
        )],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert_eq!(diff.summary.elements_modified, 1);

    let elem_diff = &diff.regions[0]
        .element_changes
        .as_ref()
        .unwrap()[0];
    let rc = elem_diff.role_change.as_ref().unwrap();
    assert_eq!(rc.old, "paragraph");
    assert_eq!(rc.new, "heading");
}

#[test]
fn test_actions_change() {
    let mut old_elem = make_element("e1", ElementRole::Button, Some("Go"));
    old_elem.actions = Some(vec!["click".into()]);

    let mut new_elem = make_element("e1", ElementRole::Button, Some("Go"));
    new_elem.actions = Some(vec!["click".into(), "submit".into()]);

    let old = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![old_elem])],
        ..empty_som()
    };
    let new = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![new_elem])],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    assert_eq!(diff.summary.elements_modified, 1);

    let elem_diff = &diff.regions[0]
        .element_changes
        .as_ref()
        .unwrap()[0];
    let ac = elem_diff.actions_change.as_ref().unwrap();
    assert_eq!(ac.old, "click");
    assert_eq!(ac.new, "click,submit");
}

#[test]
fn test_hints_change() {
    let mut old_elem = make_element("e1", ElementRole::Button, Some("Save"));
    old_elem.hints = Some(vec!["primary".into()]);

    let mut new_elem = make_element("e1", ElementRole::Button, Some("Save"));
    new_elem.hints = Some(vec!["primary".into(), "disabled".into()]);

    let old = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![old_elem])],
        ..empty_som()
    };
    let new = Som {
        regions: vec![make_region("r1", RegionRole::Main, vec![new_elem])],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    let elem_diff = &diff.regions[0]
        .element_changes
        .as_ref()
        .unwrap()[0];
    let hc = elem_diff.hints_change.as_ref().unwrap();
    assert_eq!(hc.old, "primary");
    assert_eq!(hc.new, "primary,disabled");
}

#[test]
fn test_title_change() {
    let old = Som {
        title: "Old Title".into(),
        ..empty_som()
    };
    let new = Som {
        title: "New Title".into(),
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    let tc = diff.page.title_change.as_ref().unwrap();
    assert_eq!(tc.old, "Old Title");
    assert_eq!(tc.new, "New Title");
}

#[test]
fn test_url_change() {
    let old = Som {
        url: "https://example.com/old".into(),
        ..empty_som()
    };
    let new = Som {
        url: "https://example.com/new".into(),
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    let uc = diff.page.url_change.as_ref().unwrap();
    assert_eq!(uc.old, "https://example.com/old");
    assert_eq!(uc.new, "https://example.com/new");
}

#[test]
fn test_ignore_meta_flag() {
    let old = Som {
        meta: SomMeta {
            html_bytes: 1000,
            som_bytes: 500,
            element_count: 10,
            interactive_count: 3,
        },
        ..empty_som()
    };
    let new = Som {
        meta: SomMeta {
            html_bytes: 2000,
            som_bytes: 800,
            element_count: 10,
            interactive_count: 3,
        },
        ..empty_som()
    };

    let with_meta = diff_soms(&old, &new, false);
    assert!(with_meta.meta.is_some());

    let without_meta = diff_soms(&old, &new, true);
    assert!(without_meta.meta.is_none());
}

#[test]
fn test_render_summary_no_changes() {
    let s = DiffSummary {
        total_changes: 0,
        elements_added: 0,
        elements_removed: 0,
        elements_modified: 0,
        regions_added: 0,
        regions_removed: 0,
        has_price_changes: false,
        has_content_changes: false,
        has_structural_changes: false,
    };
    assert_eq!(render_summary(&s), "no changes");
}

#[test]
fn test_render_summary_with_changes() {
    let s = DiffSummary {
        total_changes: 10,
        elements_added: 3,
        elements_removed: 2,
        elements_modified: 5,
        regions_added: 0,
        regions_removed: 0,
        has_price_changes: true,
        has_content_changes: true,
        has_structural_changes: true,
    };
    let summary = render_summary(&s);
    assert!(summary.contains("3 added"));
    assert!(summary.contains("2 removed"));
    assert!(summary.contains("5 modified"));
    assert!(summary.contains("price change detected"));
}

#[test]
fn test_json_roundtrip() {
    let old = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("Hello"))],
        )],
        ..empty_som()
    };
    let new = Som {
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("World"))],
        )],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    let json = serde_json::to_string_pretty(&diff).unwrap();
    let parsed: SomDiff = serde_json::from_str(&json).unwrap();
    assert_eq!(diff, parsed);
}

#[test]
fn test_realistic_som_diff() {
    // Simulate a product page that changed between two snapshots.
    let old = Som {
        som_version: "1".into(),
        url: "https://shop.example.com/product/42".into(),
        title: "Widget Pro - $49.99".into(),
        lang: "en".into(),
        meta: SomMeta {
            html_bytes: 45000,
            som_bytes: 2200,
            element_count: 18,
            interactive_count: 5,
        },
        structured_data: None,
        regions: vec![
            make_region(
                "nav-1",
                RegionRole::Navigation,
                vec![
                    make_element("n1", ElementRole::Link, Some("Home")),
                    make_element("n2", ElementRole::Link, Some("Products")),
                    make_element("n3", ElementRole::Link, Some("Cart (2)")),
                ],
            ),
            {
                let mut price_elem =
                    make_element("m3", ElementRole::Paragraph, Some("$49.99"));
                price_elem.attrs =
                    Some(serde_json::json!({"data-price": "49.99", "class": "price"}));

                let mut buy_btn = make_element("m4", ElementRole::Button, Some("Add to Cart"));
                buy_btn.actions = Some(vec!["click".into()]);
                buy_btn.hints = Some(vec!["primary".into()]);

                make_region(
                    "main-1",
                    RegionRole::Main,
                    vec![
                        make_element("m1", ElementRole::Heading, Some("Widget Pro")),
                        make_element(
                            "m2",
                            ElementRole::Paragraph,
                            Some("The best widget for professionals."),
                        ),
                        price_elem,
                        buy_btn,
                        make_element("m5", ElementRole::Image, None),
                    ],
                )
            },
            make_region(
                "footer-1",
                RegionRole::Footer,
                vec![make_element(
                    "f1",
                    ElementRole::Paragraph,
                    Some("© 2025 Shop Inc."),
                )],
            ),
        ],
    };

    let new = Som {
        som_version: "1".into(),
        url: "https://shop.example.com/product/42".into(),
        title: "Widget Pro - $59.99 (Sale!)".into(),
        lang: "en".into(),
        meta: SomMeta {
            html_bytes: 47000,
            som_bytes: 2400,
            element_count: 20,
            interactive_count: 6,
        },
        structured_data: None,
        regions: vec![
            make_region(
                "nav-1",
                RegionRole::Navigation,
                vec![
                    make_element("n1", ElementRole::Link, Some("Home")),
                    make_element("n2", ElementRole::Link, Some("Products")),
                    make_element("n3", ElementRole::Link, Some("Cart (3)")),
                ],
            ),
            {
                let mut price_elem =
                    make_element("m3", ElementRole::Paragraph, Some("$59.99"));
                price_elem.attrs =
                    Some(serde_json::json!({"data-price": "59.99", "class": "price sale"}));

                let mut buy_btn = make_element("m4", ElementRole::Button, Some("Add to Cart"));
                buy_btn.actions = Some(vec!["click".into()]);
                buy_btn.hints = Some(vec!["primary".into(), "sale".into()]);

                make_region(
                    "main-1",
                    RegionRole::Main,
                    vec![
                        make_element("m1", ElementRole::Heading, Some("Widget Pro")),
                        make_element(
                            "m2",
                            ElementRole::Paragraph,
                            Some("The best widget for professionals. Now on sale!"),
                        ),
                        price_elem,
                        buy_btn,
                        make_element("m5", ElementRole::Image, None),
                        // New element: sale badge.
                        make_element("m6", ElementRole::Paragraph, Some("SALE")),
                    ],
                )
            },
            make_region(
                "footer-1",
                RegionRole::Footer,
                vec![make_element(
                    "f1",
                    ElementRole::Paragraph,
                    Some("© 2025 Shop Inc."),
                )],
            ),
            // New aside region for related products.
            make_region(
                "aside-1",
                RegionRole::Aside,
                vec![make_element(
                    "a1",
                    ElementRole::Link,
                    Some("Widget Basic - $29.99"),
                )],
            ),
        ],
    };

    let diff = diff_soms(&old, &new, false);

    // Page-level.
    assert!(diff.page.title_change.is_some());
    assert_eq!(diff.page.element_count_delta, Some(2));
    assert_eq!(diff.page.interactive_count_delta, Some(1));

    // Price change detected.
    assert!(diff.summary.has_price_changes);

    // Content changes in main.
    assert!(diff.summary.has_content_changes);

    // Structural changes (new region, new element).
    assert!(diff.summary.has_structural_changes);
    assert_eq!(diff.summary.regions_added, 1);

    // Element counts: n3 modified, m2 modified, m3 modified, m4 modified
    // (hints), m6 added.
    assert!(diff.summary.elements_modified >= 3);
    assert!(diff.summary.elements_added >= 1);

    // Verify text renders without panic.
    let text = render_text(&diff);
    assert!(!text.is_empty());
    assert!(text.contains("price change detected"));

    // Verify JSON serialization.
    let json = serde_json::to_string(&diff).unwrap();
    assert!(json.contains("price_changes"));
}

#[test]
fn test_price_detection_various_currencies() {
    assert!(is_price_text("$49.99"));
    assert!(is_price_text("€199.00"));
    assert!(is_price_text("£1,299.99"));
    assert!(is_price_text("¥5000"));
    assert!(is_price_text("Total: $12.50"));
    assert!(!is_price_text("Hello world"));
    assert!(!is_price_text("100 items"));
}

#[test]
fn test_language_change() {
    let old = Som {
        lang: "en".into(),
        ..empty_som()
    };
    let new = Som {
        lang: "fr".into(),
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    let lc = diff.page.lang_change.as_ref().unwrap();
    assert_eq!(lc.old, "en");
    assert_eq!(lc.new, "fr");
}

#[test]
fn test_render_text_output() {
    let old = Som {
        title: "Old".into(),
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("A"))],
        )],
        ..empty_som()
    };
    let new = Som {
        title: "New".into(),
        regions: vec![make_region(
            "r1",
            RegionRole::Main,
            vec![make_element("e1", ElementRole::Paragraph, Some("B"))],
        )],
        ..empty_som()
    };

    let diff = diff_soms(&old, &new, false);
    let text = render_text(&diff);
    assert!(text.contains("Title:"));
    assert!(text.contains("[~] Region r1"));
    assert!(text.contains("[~] Element e1"));
    assert!(text.contains("\"A\" → \"B\""));
}
