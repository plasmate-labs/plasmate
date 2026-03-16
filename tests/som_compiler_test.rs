use plasmate::som::compiler;
use plasmate::som::types::*;

fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{}", name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", path, e))
}

#[test]
fn test_simple_page_regions() {
    let html = load_fixture("simple_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    assert_eq!(som.title, "Simple Test Page");
    assert_eq!(som.lang, "en");

    // Should have navigation, main, aside, header, footer regions
    let region_roles: Vec<&RegionRole> = som.regions.iter().map(|r| &r.role).collect();
    assert!(
        region_roles.contains(&&RegionRole::Navigation),
        "Missing navigation region. Found: {:?}",
        region_roles
    );
    assert!(
        region_roles.contains(&&RegionRole::Main),
        "Missing main region"
    );
}

#[test]
fn test_simple_page_interactive_elements() {
    let html = load_fixture("simple_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    assert!(
        som.meta.interactive_count >= 5,
        "Should have at least 5 interactive elements (6 links), found {}",
        som.meta.interactive_count
    );
}

#[test]
fn test_login_form() {
    let html = load_fixture("login_form.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    // Should have a form region
    let form = som
        .regions
        .iter()
        .find(|r| r.role == RegionRole::Form)
        .expect("Should have form region");

    assert_eq!(form.label.as_deref(), Some("Login"));
    assert_eq!(form.action.as_deref(), Some("/api/login"));
    assert_eq!(form.method.as_deref(), Some("POST"));

    // Form should contain email input, password input, checkbox, button, link
    let roles: Vec<&ElementRole> = form.elements.iter().map(|e| &e.role).collect();
    assert!(
        roles.contains(&&ElementRole::TextInput),
        "Should have text inputs"
    );
    assert!(
        roles.contains(&&ElementRole::Button),
        "Should have submit button"
    );
    assert!(
        roles.contains(&&ElementRole::Checkbox),
        "Should have checkbox"
    );
}

#[test]
fn test_ecommerce_page() {
    let html = load_fixture("ecommerce.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    // Should find navigation links, product links, buttons, selects, table
    assert!(
        som.meta.interactive_count >= 8,
        "Should have many interactive elements, found {}",
        som.meta.interactive_count
    );

    // Check for select elements with options
    let all_elements: Vec<&Element> = som
        .regions
        .iter()
        .flat_map(|r| r.elements.iter())
        .collect();

    let selects: Vec<&&Element> = all_elements
        .iter()
        .filter(|e| e.role == ElementRole::Select)
        .collect();

    assert!(
        !selects.is_empty(),
        "Should have select elements for quantity"
    );

    // Check table
    let tables: Vec<&&Element> = all_elements
        .iter()
        .filter(|e| e.role == ElementRole::Table)
        .collect();

    assert!(!tables.is_empty(), "Should have product comparison table");
    if let Some(table) = tables.first() {
        if let Some(attrs) = &table.attrs {
            assert!(attrs.get("headers").is_some(), "Table should have headers");
            assert!(attrs.get("rows").is_some(), "Table should have rows");
        }
    }
}

#[test]
fn test_hidden_elements_stripped() {
    let html = load_fixture("hidden_elements.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    let json = serde_json::to_string(&som).unwrap();

    // Hidden content should not appear
    assert!(
        !json.contains("This should be hidden"),
        "display:none content should be stripped"
    );
    assert!(
        !json.contains("This is also hidden"),
        "visibility:hidden content should be stripped"
    );
    assert!(
        !json.contains("Hidden attribute"),
        "hidden attribute content should be stripped"
    );
    assert!(
        !json.contains("Aria hidden content"),
        "aria-hidden content should be stripped"
    );

    // Visible content should appear
    assert!(json.contains("Visible Heading"));
    assert!(json.contains("Visible paragraph"));
    assert!(json.contains("Important image"));

    // Decorative images should be stripped
    assert!(
        !json.contains("decorative.png"),
        "Decorative images should be stripped"
    );
}

#[test]
fn test_news_page_structure() {
    let html = load_fixture("news_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    assert_eq!(som.title, "Tech News Today");

    // Should detect navigation
    let has_nav = som.regions.iter().any(|r| r.role == RegionRole::Navigation);
    assert!(has_nav, "Should detect navigation region");

    // Should have story links
    let all_elements: Vec<&Element> = som
        .regions
        .iter()
        .flat_map(|r| r.elements.iter())
        .collect();

    let links: Vec<&&Element> = all_elements
        .iter()
        .filter(|e| e.role == ElementRole::Link)
        .collect();

    assert!(
        links.len() >= 5,
        "Should have at least 5 story links, found {}",
        links.len()
    );
}

#[test]
fn test_deterministic_ids_across_compiles() {
    let html = load_fixture("simple_page.html");
    let som1 = compiler::compile(&html, "https://example.com").unwrap();
    let som2 = compiler::compile(&html, "https://example.com").unwrap();

    let json1 = serde_json::to_string(&som1).unwrap();
    let json2 = serde_json::to_string(&som2).unwrap();

    assert_eq!(json1, json2, "Same input must produce identical SOM output");
}

#[test]
fn test_scripts_and_styles_never_leak() {
    let html = load_fixture("simple_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let json = serde_json::to_string(&som).unwrap();

    assert!(!json.contains("console.log"), "Script content should not appear in SOM");
    assert!(!json.contains("font-family"), "Style content should not appear in SOM");
}

#[test]
fn test_som_captures_key_info() {
    // For small fixture files, SOM JSON overhead may exceed HTML size.
    // The real compression test is against large real-world pages (8x+).
    // Here we verify that SOM captures the key semantic content.
    let html = load_fixture("ecommerce.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    // SOM should have meaningful content
    assert!(som.meta.element_count > 0);
    assert!(som.meta.interactive_count > 0);

    // Verify the SOM is valid JSON and can round-trip
    let json = serde_json::to_string(&som).unwrap();
    let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(
        som.meta.element_count >= 5,
        "Should capture at least 5 elements from ecommerce page, found {}", som.meta.element_count
    );
}
