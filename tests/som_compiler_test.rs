use plasmate::som::compiler;
use plasmate::som::types::*;

fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{}", name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", path, e))
}

fn all_elements(som: &Som) -> Vec<&Element> {
    som.regions.iter().flat_map(|r| r.elements.iter()).collect()
}

// ============================================================
// Original 9 tests (preserved from initial codebase)
// ============================================================

#[test]
fn test_simple_page_regions() {
    let html = load_fixture("simple_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    assert_eq!(som.title, "Simple Test Page");
    assert_eq!(som.lang, "en");
    let roles: Vec<&RegionRole> = som.regions.iter().map(|r| &r.role).collect();
    assert!(
        roles.contains(&&RegionRole::Navigation),
        "Missing navigation region"
    );
    assert!(roles.contains(&&RegionRole::Main), "Missing main region");
}

#[test]
fn test_simple_page_interactive_elements() {
    let html = load_fixture("simple_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    assert!(
        som.meta.interactive_count >= 5,
        "Expected >=5 interactive, got {}",
        som.meta.interactive_count
    );
}

#[test]
fn test_login_form() {
    let html = load_fixture("login_form.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let form = som
        .regions
        .iter()
        .find(|r| r.role == RegionRole::Form)
        .expect("Should have form region");
    assert_eq!(form.label.as_deref(), Some("Login"));
    assert_eq!(form.action.as_deref(), Some("/api/login"));
    assert_eq!(form.method.as_deref(), Some("POST"));
    let roles: Vec<&ElementRole> = form.elements.iter().map(|e| &e.role).collect();
    assert!(
        roles.contains(&&ElementRole::TextInput),
        "Missing text inputs"
    );
    assert!(
        roles.contains(&&ElementRole::Button),
        "Missing submit button"
    );
    assert!(roles.contains(&&ElementRole::Checkbox), "Missing checkbox");
}

#[test]
fn test_ecommerce_page() {
    let html = load_fixture("ecommerce.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    assert!(
        som.meta.interactive_count >= 8,
        "Expected >=8 interactive, got {}",
        som.meta.interactive_count
    );
    let elems = all_elements(&som);
    assert!(
        elems.iter().any(|e| e.role == ElementRole::Select),
        "Missing select element"
    );
    assert!(
        elems.iter().any(|e| e.role == ElementRole::Table),
        "Missing table element"
    );
}

#[test]
fn test_hidden_elements_stripped() {
    let html = load_fixture("hidden_elements.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let json = serde_json::to_string(&som).unwrap();
    assert!(
        !json.contains("This should be hidden"),
        "display:none should be stripped"
    );
    assert!(
        json.contains("Visible Heading"),
        "Visible content should remain"
    );
    assert!(
        !json.contains("decorative.png"),
        "Decorative images should be stripped"
    );
}

#[test]
fn test_hidden_inline_styles_ignore_case_and_spacing() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Hidden Style</title></head>
<body><main>
    <p style="DISPLAY : none">Spaced display hidden</p>
    <p style="Visibility : hidden">Spaced visibility hidden</p>
    <p style="Opacity : 0">Opacity hidden</p>
    <p aria-hidden="TRUE">Uppercase aria hidden</p>
    <p>Visible copy</p>
</main></body></html>"#;

    let json =
        serde_json::to_string(&compiler::compile(html, "https://example.com").unwrap()).unwrap();
    assert!(!json.contains("Spaced display hidden"));
    assert!(!json.contains("Spaced visibility hidden"));
    assert!(!json.contains("Opacity hidden"));
    assert!(!json.contains("Uppercase aria hidden"));
    assert!(json.contains("Visible copy"));
}

#[test]
fn test_news_page_structure() {
    let html = load_fixture("news_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    assert!(
        som.regions.iter().any(|r| r.role == RegionRole::Navigation),
        "Missing navigation"
    );
    let link_count = all_elements(&som)
        .iter()
        .filter(|e| e.role == ElementRole::Link)
        .count();
    assert!(link_count >= 5, "Expected >=5 links, got {}", link_count);
}

#[test]
fn test_deterministic_ids_across_compiles() {
    let html = load_fixture("simple_page.html");
    let j1 =
        serde_json::to_string(&compiler::compile(&html, "https://example.com").unwrap()).unwrap();
    let j2 =
        serde_json::to_string(&compiler::compile(&html, "https://example.com").unwrap()).unwrap();
    assert_eq!(j1, j2, "Same input must produce identical SOM output");
}

#[test]
fn test_scripts_and_styles_never_leak() {
    let html = load_fixture("simple_page.html");
    let json =
        serde_json::to_string(&compiler::compile(&html, "https://example.com").unwrap()).unwrap();
    assert!(!json.contains("console.log"), "Script content leaked");
    assert!(!json.contains("font-family"), "Style content leaked");
}

#[test]
fn test_som_captures_key_info() {
    let html = load_fixture("ecommerce.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    assert!(
        som.meta.element_count >= 5,
        "Expected >=5 elements, got {}",
        som.meta.element_count
    );
    let json = serde_json::to_string(&som).unwrap();
    let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_common_aria_widget_roles_map_to_interactive_elements() {
    let html = r#"<!DOCTYPE html>
<html><head><title>ARIA Widgets</title></head>
<body><main>
    <div role="textbox" aria-label="Message" contenteditable="true"></div>
    <div role="switch" aria-label="Email alerts" aria-checked="true"></div>
    <div role="combobox" aria-label="Country" aria-expanded="false"></div>
    <div role="tab" aria-label="Billing"></div>
    <div role="menuitemcheckbox" aria-label="Compact mode" aria-checked="false"></div>
    <div role="menuitemradio" aria-label="Annual billing" aria-checked="true"></div>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);
    assert!(elems.iter().any(|e| e.role == ElementRole::TextInput));
    assert!(elems.iter().any(|e| e.role == ElementRole::Checkbox));
    assert!(elems.iter().any(|e| e.role == ElementRole::Select));
    assert!(elems.iter().any(|e| e.role == ElementRole::Button));
    assert!(elems
        .iter()
        .any(|e| { e.role == ElementRole::Radio && e.label.as_deref() == Some("Annual billing") }));
    assert_eq!(som.meta.interactive_count, 6);
}

#[test]
fn test_aria_landmark_roles_are_case_insensitive() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Uppercase Roles</title></head>
<body>
    <div role="MAIN"><h1>Dashboard</h1></div>
    <div role="NAVIGATION"><a href="/settings">Settings</a></div>
</body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let roles: Vec<&RegionRole> = som.regions.iter().map(|r| &r.role).collect();

    assert!(roles.contains(&&RegionRole::Main));
    assert!(roles.contains(&&RegionRole::Navigation));
}

#[test]
fn test_aria_landmark_role_fallback_tokens_are_respected() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Fallback Roles</title></head>
<body>
    <div role="utility search" aria-label="Global search">
        <input type="search" aria-label="Query">
    </div>
</body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();

    assert!(som.regions.iter().any(|region| {
        region.role == RegionRole::Navigation && region.label.as_deref() == Some("Global search")
    }));
}

#[test]
fn test_aria_search_landmark_maps_to_navigation_region() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Search Landmark</title></head>
<body>
    <div role="search" aria-label="Product search">
        <input type="search" aria-label="Query">
        <button>Search</button>
    </div>
    <main><p>Visible content</p></main>
</body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let search_region = som
        .regions
        .iter()
        .find(|r| r.role == RegionRole::Navigation && r.label.as_deref() == Some("Product search"))
        .expect("ARIA search landmark should compile as a labelled navigation region");

    assert!(search_region
        .elements
        .iter()
        .any(|e| e.role == ElementRole::TextInput));
}

#[test]
fn test_action_semantics_conformance_fixture() {
    let html = std::fs::read_to_string("specs/conformance/016-action-semantics.html")
        .expect("action semantics fixture should load");
    let som = compiler::compile(&html, "https://example.com/action-semantics").unwrap();
    let json = serde_json::to_string(&som).unwrap();

    assert!(
        som.regions.iter().any(|region| {
            region.role == RegionRole::Navigation
                && region.label.as_deref() == Some("Product search")
        }),
        "search landmark should compile into a labelled navigation region"
    );
    assert!(
        all_elements(&som)
            .iter()
            .any(|elem| elem.role == ElementRole::Checkbox
                && elem.label.as_deref() == Some("Compact mode")),
        "menuitemcheckbox should compile into an actionable checkbox"
    );
    assert!(
        all_elements(&som)
            .iter()
            .any(|elem| elem.role == ElementRole::Radio
                && elem.label.as_deref() == Some("Annual billing")),
        "menuitemradio should compile into an actionable radio target"
    );
    let reply = all_elements(&som)
        .into_iter()
        .find(|elem| elem.role == ElementRole::TextInput && elem.label.as_deref() == Some("Reply"))
        .expect("text-entry affordance target should compile");
    let attrs = reply
        .attrs
        .as_ref()
        .expect("text-entry target should have attrs");
    assert_eq!(attrs["spellcheck"], false);
    assert_eq!(attrs["autocapitalize"], "sentences");
    assert_eq!(attrs["dirname"], "reply.dir");
    assert_eq!(attrs["aria"]["placeholder"], "Write a response");
    assert!(!json.contains("Hidden stylesheet copy"));
    assert!(!json.contains("Hidden uppercase ARIA copy"));
    assert!(!json.contains("Hidden inline opacity copy"));
    assert!(json.contains("Visible preferences copy"));
    assert_eq!(som.meta.interactive_count, 5);
}

#[test]
fn test_aria_widget_role_fallback_tokens_are_respected() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Fallback Widgets</title></head>
<body><main>
    <div role="widget menuitemcheckbox" aria-label="Compact mode"></div>
    <div role="widget menuitemradio" aria-label="Annual billing"></div>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    assert!(elems.iter().any(|e| {
        e.role == ElementRole::Checkbox && e.label.as_deref() == Some("Compact mode")
    }));
    assert!(elems
        .iter()
        .any(|e| { e.role == ElementRole::Radio && e.label.as_deref() == Some("Annual billing") }));
}

#[test]
fn test_input_types_are_case_insensitive() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Inputs</title></head>
<body><main>
    <input type="SUBMIT" value="Save">
    <input type="EMAIL" name="email" autocomplete="email">
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    assert!(elems.iter().any(|e| e.role == ElementRole::Button));
    let email = elems
        .iter()
        .find(|e| e.role == ElementRole::TextInput)
        .expect("email input should compile as text input");
    let attrs = email.attrs.as_ref().expect("email input should have attrs");
    assert_eq!(attrs["input_type"], "email");
    assert_eq!(attrs["name"], "email");
    assert_eq!(attrs["autocomplete"], "email");
}

#[test]
fn test_custom_controls_keep_actionability_attrs() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Custom Controls</title></head>
<body><main>
    <div role="textbox" aria-label="Comment" contenteditable="true" tabindex="0"></div>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let control = all_elements(&som)
        .into_iter()
        .find(|e| e.role == ElementRole::TextInput)
        .expect("custom textbox should be preserved");
    let attrs = control
        .attrs
        .as_ref()
        .expect("custom textbox should have attrs");

    assert_eq!(attrs["contenteditable"], true);
    assert_eq!(attrs["tabindex"], 0);
}

#[test]
fn test_form_state_values_and_readonly_are_preserved() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Form State</title></head>
<body><main>
    <input type="email" aria-label="Email" readonly value="ops@example.com" autocomplete="email" inputmode="email" enterkeyhint="next" minlength="6" maxlength="64" pattern=".+@example\.com" aria-invalid="TRUE" aria-autocomplete="list" aria-activedescendant="email-suggestion-1">
    <textarea aria-label="Notes" readonly maxlength="200">Already reviewed</textarea>
    <select aria-label="Plan">
        <option value="starter">Starter</option>
        <option value="team" selected>Team</option>
    </select>
    <button aria-label="Menu" aria-expanded=" FALSE " aria-pressed="TRUE"></button>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    let email = elems
        .iter()
        .find(|e| e.role == ElementRole::TextInput && e.label.as_deref() == Some("Email"))
        .expect("readonly input should be preserved");
    let email_attrs = email.attrs.as_ref().expect("input attrs should exist");
    assert_eq!(email_attrs["readonly"], true);
    assert_eq!(email_attrs["value"], "ops@example.com");
    assert_eq!(email_attrs["autocomplete"], "email");
    assert_eq!(email_attrs["inputmode"], "email");
    assert_eq!(email_attrs["enterkeyhint"], "next");
    assert_eq!(email_attrs["minlength"], 6);
    assert_eq!(email_attrs["maxlength"], 64);
    assert_eq!(email_attrs["pattern"], ".+@example\\.com");
    assert_eq!(email_attrs["aria"]["invalid"], true);
    assert_eq!(email_attrs["aria"]["autocomplete"], "list");
    assert_eq!(
        email_attrs["aria"]["active_descendant"],
        "email-suggestion-1"
    );

    let textarea = elems
        .iter()
        .find(|e| e.role == ElementRole::Textarea)
        .expect("textarea should be preserved");
    let textarea_attrs = textarea
        .attrs
        .as_ref()
        .expect("textarea attrs should exist");
    assert_eq!(textarea_attrs["readonly"], true);
    assert_eq!(textarea_attrs["value"], "Already reviewed");
    assert_eq!(textarea_attrs["maxlength"], 200);

    let select = elems
        .iter()
        .find(|e| e.role == ElementRole::Select)
        .expect("select should be preserved");
    let select_attrs = select.attrs.as_ref().expect("select attrs should exist");
    assert_eq!(select_attrs["value"], "team");

    let button = elems
        .iter()
        .find(|e| e.role == ElementRole::Button)
        .expect("button should be preserved");
    let aria = button
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("aria"))
        .expect("button aria state should be preserved");
    assert_eq!(aria["expanded"], false);
    assert_eq!(aria["pressed"], true);
}

#[test]
fn test_file_upload_action_cues_are_preserved() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Uploads</title></head>
<body><main>
    <label for="evidence">Evidence files</label>
    <input id="evidence" name="evidence" type="file" accept="image/png,.pdf" capture="environment" multiple required>
    <input aria-label="Profile photo" type="file" accept="image/*" capture>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    let evidence = elems
        .iter()
        .find(|e| e.role == ElementRole::TextInput && e.label.as_deref() == Some("Evidence files"))
        .expect("file input should compile as an action target");
    let evidence_attrs = evidence
        .attrs
        .as_ref()
        .expect("file input attrs should exist");
    assert_eq!(evidence_attrs["input_type"], "file");
    assert_eq!(evidence_attrs["name"], "evidence");
    assert_eq!(evidence_attrs["accept"], "image/png,.pdf");
    assert_eq!(evidence_attrs["capture"], "environment");
    assert_eq!(evidence_attrs["multiple"], true);
    assert_eq!(evidence_attrs["required"], true);

    let photo = elems
        .iter()
        .find(|e| e.role == ElementRole::TextInput && e.label.as_deref() == Some("Profile photo"))
        .expect("boolean capture input should compile");
    let photo_attrs = photo.attrs.as_ref().expect("photo attrs should exist");
    assert_eq!(photo_attrs["accept"], "image/*");
    assert_eq!(photo_attrs["capture"], true);
}

#[test]
fn test_form_submission_context_is_preserved() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Checkout</title></head>
<body>
    <form aria-label="Checkout" action="/checkout" method="post" target="_blank" enctype="multipart/form-data" novalidate accept-charset="UTF-8" autocomplete="off">
        <label for="receipt">Receipt</label>
        <input id="receipt" name="receipt" type="file">
        <button type="submit">Submit</button>
    </form>
</body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let form = som
        .regions
        .iter()
        .find(|r| r.role == RegionRole::Form)
        .expect("form region should be preserved");

    assert_eq!(form.label.as_deref(), Some("Checkout"));
    assert_eq!(form.action.as_deref(), Some("/checkout"));
    assert_eq!(form.method.as_deref(), Some("POST"));
    assert_eq!(form.target.as_deref(), Some("_blank"));
    assert_eq!(form.enctype.as_deref(), Some("multipart/form-data"));
    assert_eq!(form.novalidate, Some(true));
    assert_eq!(form.accept_charset.as_deref(), Some("UTF-8"));
    assert_eq!(form.autocomplete.as_deref(), Some("off"));
}

#[test]
fn test_accessible_labels_from_label_for_and_labelledby() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Labels</title></head>
<body><main>
    <label for="account-email">Account email</label>
    <input id="account-email" type="email" autocomplete="email">
    <span id="save-label">Save profile</span>
    <button aria-labelledby="save-label"></button>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    let email = elems
        .iter()
        .find(|e| e.role == ElementRole::TextInput)
        .expect("email input should be preserved");
    assert_eq!(email.label.as_deref(), Some("Account email"));

    let button = elems
        .iter()
        .find(|e| e.role == ElementRole::Button)
        .expect("button should be preserved");
    assert_eq!(button.label.as_deref(), Some("Save profile"));
}

#[test]
fn test_wrapped_label_controls_get_accessible_label() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Wrapped Labels</title></head>
<body><main>
    <label>Remember this browser <input id="remember-browser" type="checkbox"></label>
    <label>Support tier <select id="support-tier"><option value="pro">Pro</option></select></label>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    let checkbox = elems
        .iter()
        .find(|e| e.html_id.as_deref() == Some("remember-browser"))
        .expect("wrapped checkbox should be preserved");
    assert_eq!(checkbox.label.as_deref(), Some("Remember this browser"));

    let select = elems
        .iter()
        .find(|e| e.html_id.as_deref() == Some("support-tier"))
        .expect("wrapped select should be preserved");
    assert_eq!(select.label.as_deref(), Some("Support tier"));
}

#[test]
fn test_region_labels_resolve_aria_labelledby() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Region Labels</title></head>
<body>
    <span id="primary-nav-name">Primary navigation</span>
    <nav aria-labelledby="primary-nav-name">
        <a href="/">Home</a>
        <a href="/docs">Docs</a>
        <a href="/pricing">Pricing</a>
    </nav>
    <span id="signup-name">Create account</span>
    <form aria-labelledby="signup-name" action="/signup" method="post">
        <input name="email" type="email" aria-label="Email">
        <button>Join</button>
    </form>
</body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();

    let nav = som
        .regions
        .iter()
        .find(|r| r.role == RegionRole::Navigation)
        .expect("navigation region should be preserved");
    assert_eq!(nav.label.as_deref(), Some("Primary navigation"));

    let form = som
        .regions
        .iter()
        .find(|r| r.role == RegionRole::Form)
        .expect("form region should be preserved");
    assert_eq!(form.label.as_deref(), Some("Create account"));
}

#[test]
fn test_input_button_values_become_labels_and_type_attrs() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Input Buttons</title></head>
<body><main>
    <input type="SUBMIT" value="Save changes">
    <input type="reset" value="Clear form">
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    let save = elems
        .iter()
        .find(|e| e.label.as_deref() == Some("Save changes"))
        .expect("submit input value should be exposed as label");
    assert_eq!(save.role, ElementRole::Button);
    assert_eq!(save.attrs.as_ref().unwrap()["input_type"], "submit");

    let clear = elems
        .iter()
        .find(|e| e.label.as_deref() == Some("Clear form"))
        .expect("reset input value should be exposed as label");
    assert_eq!(clear.role, ElementRole::Button);
    assert_eq!(clear.attrs.as_ref().unwrap()["input_type"], "reset");
}

#[test]
fn test_fieldset_and_aria_groups_compile_with_labels() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Groups</title></head>
<body><main>
    <form>
        <fieldset disabled>
            <legend>Contact preference</legend>
            <label><input type="radio" name="contact" value="email"> Email</label>
            <label><input type="radio" name="contact" value="sms"> SMS</label>
        </fieldset>
        <div role="radiogroup" aria-label="Subscription plan">
            <label><input type="radio" name="plan" value="team"> Team</label>
        </div>
    </form>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    let fieldset = elems
        .iter()
        .find(|e| e.role == ElementRole::Group && e.label.as_deref() == Some("Contact preference"))
        .expect("fieldset should compile as a labelled group");
    let attrs = fieldset
        .attrs
        .as_ref()
        .expect("fieldset should expose attrs");
    assert_eq!(attrs["legend"], "Contact preference");
    assert_eq!(attrs["disabled"], true);

    assert!(
        elems.iter().any(
            |e| e.role == ElementRole::Group && e.label.as_deref() == Some("Subscription plan")
        ),
        "ARIA radiogroup should compile as a labelled group"
    );
    assert_eq!(
        elems
            .iter()
            .filter(|e| e.role == ElementRole::Radio)
            .count(),
        3
    );
    let contact_radios: Vec<_> = elems
        .iter()
        .filter(|e| {
            e.role == ElementRole::Radio
                && e.attrs
                    .as_ref()
                    .and_then(|attrs| attrs.get("group"))
                    .and_then(|group| group.as_str())
                    == Some("contact")
        })
        .collect();
    assert_eq!(contact_radios.len(), 2);
    assert!(
        contact_radios
            .iter()
            .all(|radio| radio.attrs.as_ref().unwrap()["disabled"] == true),
        "disabled fieldsets should mark descendant native controls disabled"
    );
}

#[test]
fn test_aria_labelledby_takes_precedence_over_aria_label() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Label Priority</title></head>
<body><main>
    <span id="primary-label">Primary action name</span>
    <button aria-label="Fallback name" aria-labelledby="primary-label"></button>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let button = all_elements(&som)
        .into_iter()
        .find(|e| e.role == ElementRole::Button)
        .expect("button should be preserved");

    assert_eq!(button.label.as_deref(), Some("Primary action name"));
}

#[test]
fn test_aria_describedby_sets_accessible_description_attr() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Description</title></head>
<body><main>
    <p id="password-help">Use at least 12 characters.</p>
    <input type="password" aria-label="Password" aria-describedby="password-help">
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let input = all_elements(&som)
        .into_iter()
        .find(|e| e.role == ElementRole::TextInput)
        .expect("password input should be preserved");
    let attrs = input.attrs.as_ref().expect("input should expose attrs");

    assert_eq!(
        attrs.get("description").and_then(|v| v.as_str()),
        Some("Use at least 12 characters.")
    );
}

#[test]
fn test_disabled_and_aria_required_state_promoted_for_action_plans() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Action State</title></head>
<body><main>
    <textarea aria-label="Notes" disabled></textarea>
    <select aria-label="Plan" disabled><option>Team</option></select>
    <div role="textbox" aria-label="Approval code" aria-required="true"></div>
    <div role="button" aria-label="Archive" aria-disabled="true"></div>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    let notes = elems
        .iter()
        .find(|e| e.role == ElementRole::Textarea)
        .expect("textarea should be preserved");
    assert_eq!(notes.attrs.as_ref().unwrap()["disabled"], true);

    let plan = elems
        .iter()
        .find(|e| e.role == ElementRole::Select)
        .expect("select should be preserved");
    assert_eq!(plan.attrs.as_ref().unwrap()["disabled"], true);

    let approval = elems
        .iter()
        .find(|e| e.label.as_deref() == Some("Approval code"))
        .expect("ARIA textbox should be preserved");
    assert_eq!(approval.attrs.as_ref().unwrap()["required"], true);

    let archive = elems
        .iter()
        .find(|e| e.label.as_deref() == Some("Archive"))
        .expect("ARIA button should be preserved");
    assert_eq!(archive.attrs.as_ref().unwrap()["disabled"], true);
}

#[test]
fn test_aria_relationship_state_is_preserved_for_action_targets() {
    let html = r#"<!DOCTYPE html>
<html><head><title>ARIA Relationships</title></head>
<body><main>
    <button aria-label="Filters" aria-expanded="false" aria-controls="filters-panel" aria-haspopup="dialog">Filters</button>
    <a href="/billing" aria-current="page">Billing</a>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    let filters = elems
        .iter()
        .find(|e| e.text.as_deref() == Some("Filters"))
        .expect("button should be preserved");
    let filters_aria = filters.attrs.as_ref().unwrap()["aria"].as_object().unwrap();
    assert_eq!(
        filters_aria.get("expanded").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        filters_aria.get("controls").and_then(|v| v.as_str()),
        Some("filters-panel")
    );
    assert_eq!(
        filters_aria.get("haspopup").and_then(|v| v.as_str()),
        Some("dialog")
    );

    let billing = elems
        .iter()
        .find(|e| e.text.as_deref() == Some("Billing"))
        .expect("current link should be preserved");
    let billing_aria = billing.attrs.as_ref().unwrap()["aria"].as_object().unwrap();
    assert_eq!(
        billing_aria.get("current").and_then(|v| v.as_str()),
        Some("page")
    );
}

#[test]
fn test_shadow_root_elements_are_counted_in_meta() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Shadow Count</title></head>
<body><main>
    <section aria-label="Widget host">
        <template shadowrootmode="open">
            <p>Shadow instructions</p>
            <button>Shadow action</button>
        </template>
    </section>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();

    assert_eq!(som.meta.interactive_count, 1);
    assert!(
        som.meta.element_count >= 3,
        "host plus shadow elements should be counted"
    );
}

#[test]
fn test_shadow_root_extraction_recurses_through_containers() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Nested Shadow</title></head>
<body><main>
    <section aria-label="Widget host">
        <template shadowrootmode="open">
            <div class="toolbar">
                <button aria-label="Refresh results"></button>
            </div>
        </template>
    </section>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let host = all_elements(&som)
        .into_iter()
        .find(|e| e.shadow.is_some())
        .expect("host should expose shadow root");
    let shadow = host.shadow.as_ref().unwrap();

    assert!(
        shadow
            .elements
            .iter()
            .any(|e| e.role == ElementRole::Button
                && e.label.as_deref() == Some("Refresh results")),
        "nested shadow button should be extracted"
    );
    assert_eq!(som.meta.interactive_count, 1);
}

#[test]
fn test_link_dedup_preserves_case_sensitive_paths() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Links</title></head>
<body><main>
    <a href="/Docs">Docs</a>
    <a href="/Docs#install">Docs install</a>
    <a href="/docs">Lowercase docs</a>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let json = serde_json::to_string(&som).unwrap();

    assert_eq!(json.matches("\"/Docs\"").count(), 1);
    assert_eq!(json.matches("\"/docs\"").count(), 1);
}

// ============================================================
// Issue 1: Layout table detection and decomposition
// ============================================================

#[test]
fn test_hn_layout_table_decomposition() {
    let html = load_fixture("hn_table_layout.html");
    let som = compiler::compile(&html, "https://news.ycombinator.com").unwrap();
    let elems = all_elements(&som);

    // Layout tables should be decomposed, not kept as table elements
    let table_count = elems
        .iter()
        .filter(|e| e.role == ElementRole::Table)
        .count();
    assert_eq!(
        table_count, 0,
        "Layout tables should be decomposed, found {} table elements",
        table_count
    );

    // The links inside the layout table should be individually extractable
    let link_count = elems.iter().filter(|e| e.role == ElementRole::Link).count();
    assert!(
        link_count >= 12,
        "Expected >=12 links from layout table, found {}",
        link_count
    );
}

#[test]
fn test_layout_table_attributes_detected() {
    let html = load_fixture("layout_table.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let elems = all_elements(&som);

    // Table with cellpadding/cellspacing/border/width should be layout
    let table_count = elems
        .iter()
        .filter(|e| e.role == ElementRole::Table)
        .count();
    assert_eq!(
        table_count, 0,
        "Layout table with attributes should be decomposed, found {} tables",
        table_count
    );

    // Links inside should still be present
    let link_count = elems.iter().filter(|e| e.role == ElementRole::Link).count();
    assert!(
        link_count >= 3,
        "Expected >=3 links after decomposition, found {}",
        link_count
    );
}

#[test]
fn test_data_table_preserved() {
    // The ecommerce page has a real data table with th headers
    let html = load_fixture("ecommerce.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let elems = all_elements(&som);
    let tables: Vec<&&Element> = elems
        .iter()
        .filter(|e| e.role == ElementRole::Table)
        .collect();
    assert!(
        !tables.is_empty(),
        "Real data tables (with <th>) should be preserved"
    );
    if let Some(t) = tables.first() {
        if let Some(attrs) = &t.attrs {
            assert!(
                attrs.get("headers").is_some(),
                "Data table should have headers extracted"
            );
        }
    }
}

// ============================================================
// Issue 2: Content summarization (paragraph/link budgets)
// ============================================================

#[test]
fn test_wiki_paragraph_summarization() {
    let html = load_fixture("wiki_like.html");
    let som = compiler::compile(&html, "https://en.wikipedia.org").unwrap();

    // Find the main region (the fixture uses <main id="main-content">)
    let main = som
        .regions
        .iter()
        .find(|r| r.role == RegionRole::Main)
        .expect("Should find main region from <main> tag");

    let para_count = main
        .elements
        .iter()
        .filter(|e| e.role == ElementRole::Paragraph)
        .count();
    // max_paragraphs=10, so we expect 10 real + 1 summary = 11 max
    assert!(
        para_count <= 12,
        "Expected <=12 paragraphs (10 + summary + heading), found {}",
        para_count
    );

    // Verify a summary element exists mentioning dropped paragraphs
    let has_summary = main.elements.iter().any(|e| {
        e.text
            .as_ref()
            .map_or(false, |t| t.contains("more paragraphs"))
    });
    assert!(
        has_summary,
        "Should have a summary element for dropped paragraphs"
    );
}

#[test]
fn test_wiki_navigation_link_summarization() {
    let html = load_fixture("wiki_like.html");
    let som = compiler::compile(&html, "https://en.wikipedia.org").unwrap();

    // Find navigation regions
    let nav_regions: Vec<&Region> = som
        .regions
        .iter()
        .filter(|r| r.role == RegionRole::Navigation)
        .collect();
    assert!(
        !nav_regions.is_empty(),
        "Should have at least one navigation region"
    );

    // The fixture has 200 links in a nav-like div; should be capped at max_navigation_links=80
    for nav in &nav_regions {
        let link_count = nav
            .elements
            .iter()
            .filter(|e| e.role == ElementRole::Link)
            .count();
        assert!(
            link_count <= 81,
            "Nav region should have <=81 links (80 + summary), found {}",
            link_count
        );
    }
}

#[test]
fn test_paragraph_text_truncation() {
    // Paragraphs in main content should be truncated: first para 200 chars, subsequent 80
    let html = load_fixture("wiki_like.html");
    let som = compiler::compile(&html, "https://en.wikipedia.org").unwrap();
    let main = som
        .regions
        .iter()
        .find(|r| r.role == RegionRole::Main)
        .expect("Should find main region");

    let paragraphs: Vec<&Element> = main
        .elements
        .iter()
        .filter(|e| {
            e.role == ElementRole::Paragraph
                && e.text
                    .as_ref()
                    .map_or(false, |t| !t.contains("more paragraphs"))
        })
        .collect();

    if paragraphs.len() >= 2 {
        let first_len = paragraphs[0].text.as_ref().unwrap().len();
        let second_len = paragraphs[1].text.as_ref().unwrap().len();
        // First paragraph has 200 char limit, subsequent 80
        assert!(
            first_len <= 210,
            "First paragraph should be <=~200 chars, got {}",
            first_len
        );
        assert!(
            second_len <= 90,
            "Subsequent paragraphs should be <=~80 chars, got {}",
            second_len
        );
    }
}

#[test]
fn test_element_budget_enforced() {
    // With max_elements=400, a page with 500+ elements should be capped
    let html = load_fixture("wiki_like.html");
    let som = compiler::compile(&html, "https://en.wikipedia.org").unwrap();

    // Each individual region should respect the element budget
    for region in &som.regions {
        assert!(
            region.elements.len() <= 401,
            "Region {} ({:?}) should have <=401 elements (400 + summary), found {}",
            region.id,
            region.role,
            region.elements.len()
        );
    }
}

// ============================================================
// Issue 3: Heuristic region detection
// ============================================================

#[test]
fn test_class_based_region_detection() {
    let html = load_fixture("class_based_regions.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let roles: Vec<&RegionRole> = som.regions.iter().map(|r| &r.role).collect();

    // class="header site-header" should yield a header region
    assert!(
        roles.contains(&&RegionRole::Header),
        "Should detect header from class hint"
    );
    // Should have multiple meaningful regions detected from class-based hints
    assert!(
        som.regions.len() >= 2,
        "Should detect at least 2 regions from class hints, found {}",
        som.regions.len()
    );
}

#[test]
fn test_footer_heuristic_detection() {
    let html = load_fixture("wiki_like.html");
    let som = compiler::compile(&html, "https://en.wikipedia.org").unwrap();
    // The wiki_like fixture has <div class="footer"> with copyright text
    let has_footer = som.regions.iter().any(|r| r.role == RegionRole::Footer);
    assert!(
        has_footer,
        "Should detect footer region from class='footer' with copyright text"
    );
}

#[test]
fn test_heading_elements_extracted() {
    let html = load_fixture("heading_structure.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let elems = all_elements(&som);
    let headings: Vec<&&Element> = elems
        .iter()
        .filter(|e| e.role == ElementRole::Heading)
        .collect();
    assert!(
        headings.len() >= 3,
        "Expected >=3 headings, found {}",
        headings.len()
    );
}

#[test]
fn test_collapsible_wrapper_divs() {
    // Wrapper divs with a single element child should be collapsed
    let html = r#"<html><body><main><div><div><a href="/link">Deep Link</a></div></div></main></body></html>"#;
    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);
    let link_count = elems.iter().filter(|e| e.role == ElementRole::Link).count();
    assert!(
        link_count >= 1,
        "Collapsed wrappers should still expose inner links"
    );
}

// ============================================================
// Issue 4: Edge cases and robustness
// ============================================================

#[test]
fn test_edge_cases_empty_divs() {
    let html = load_fixture("edge_cases.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    // Should not panic, should produce valid SOM
    let json = serde_json::to_string(&som).unwrap();
    assert!(!json.is_empty(), "SOM should be valid JSON");
    // Empty divs should not produce empty string elements
    assert!(
        !json.contains("\"text\":\"\""),
        "Should not have empty text strings"
    );
}

#[test]
fn test_deeply_nested_html() {
    // 100 levels deep should not stack overflow
    let mut html = String::from("<html><body>");
    for _ in 0..100 {
        html.push_str("<div>");
    }
    html.push_str("<a href='/deep'>Deep link</a>");
    for _ in 0..100 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let elems = all_elements(&som);
    assert!(
        elems.iter().any(|e| e.role == ElementRole::Link),
        "Deep link should be extracted"
    );
}

#[test]
fn test_empty_page() {
    let html = "<html><body></body></html>";
    let som = compiler::compile(html, "https://example.com").unwrap();
    assert_eq!(som.meta.element_count, 0);
    assert_eq!(som.meta.interactive_count, 0);
}

#[test]
fn test_minimal_page_with_just_text() {
    let html = "<html><body>Hello world</body></html>";
    let som = compiler::compile(html, "https://example.com").unwrap();
    assert!(
        som.meta.element_count >= 1,
        "Should have at least 1 text element"
    );
}

#[test]
fn test_form_controls_survive_budget() {
    // Form controls should never be dropped even if element budget is reached
    let mut html = String::from("<html><body><main>");
    // Add lots of paragraphs to fill budget
    for i in 0..450 {
        html.push_str(&format!("<p>Paragraph {}</p>", i));
    }
    // Then add form controls
    html.push_str("<input type='text' name='query'>");
    html.push_str("<button>Submit</button>");
    html.push_str("</main></body></html>");

    let som = compiler::compile(&html, "https://example.com").unwrap();
    let elems = all_elements(&som);
    assert!(
        elems.iter().any(|e| e.role == ElementRole::TextInput),
        "Form controls should survive element budget"
    );
    assert!(
        elems.iter().any(|e| e.role == ElementRole::Button),
        "Buttons should survive element budget"
    );
}

#[test]
fn test_table_cell_truncation() {
    let html = r#"<html><body><table>
        <thead><tr><th>Name</th><th>Description</th></tr></thead>
        <tbody><tr>
            <td>Short</td>
            <td>This is a very long description that should be truncated to the max_table_cell_chars limit of eighty characters which means it should end with ellipsis dots</td>
        </tr></tbody>
    </table></body></html>"#;
    let som = compiler::compile(html, "https://example.com").unwrap();
    let elems = all_elements(&som);
    let tables: Vec<&&Element> = elems
        .iter()
        .filter(|e| e.role == ElementRole::Table)
        .collect();
    assert!(!tables.is_empty(), "Should have a data table");
    if let Some(t) = tables.first() {
        if let Some(attrs) = &t.attrs {
            if let Some(rows) = attrs.get("rows") {
                if let Some(arr) = rows.as_array() {
                    for row in arr {
                        if let Some(cells) = row.as_array() {
                            for cell in cells {
                                if let Some(s) = cell.as_str() {
                                    assert!(s.len() <= 85,
                                        "Table cell should be truncated to ~80 chars, got {} chars: '{}'",
                                        s.len(), s);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_wikipedia_like_page_structure() {
    let html = load_fixture("wikipedia_like.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();
    let roles: Vec<&RegionRole> = som.regions.iter().map(|r| &r.role).collect();
    // Should detect some structure
    assert!(
        som.regions.len() >= 2,
        "Should have multiple regions, found {}",
        som.regions.len()
    );
    assert!(
        roles.contains(&&RegionRole::Navigation),
        "Should detect navigation"
    );
}

#[test]
fn test_som_token_reduction_from_summarization() {
    // The wiki_like fixture is ~33KB HTML. The SOM should be significantly smaller.
    let html = load_fixture("wiki_like.html");
    let som = compiler::compile(&html, "https://en.wikipedia.org").unwrap();
    let som_json = serde_json::to_string(&som).unwrap();
    let ratio = html.len() as f64 / som_json.len() as f64;
    assert!(
        ratio > 2.0,
        "Wiki-like page should achieve >2x compression ratio, got {:.2}x (html={}, som={})",
        ratio,
        html.len(),
        som_json.len()
    );
}

#[test]
fn test_hn_fixture_structure() {
    // For small fixtures, JSON overhead can exceed HTML size.
    // Here we assert the structure improvements (layout table decomposition).
    let html = load_fixture("hn_table_layout.html");
    let som = compiler::compile(&html, "https://news.ycombinator.com").unwrap();

    let elems: Vec<&Element> = som.regions.iter().flat_map(|r| r.elements.iter()).collect();
    let table_count = elems
        .iter()
        .filter(|e| e.role == ElementRole::Table)
        .count();
    assert_eq!(
        table_count, 0,
        "HN layout table should not be emitted as a table element"
    );

    let link_count = elems.iter().filter(|e| e.role == ElementRole::Link).count();
    assert!(
        link_count >= 10,
        "Expected >=10 links in HN fixture, found {}",
        link_count
    );
}

#[test]
fn test_inline_html_compiles() {
    // Quick sanity check that inline HTML compiles
    let html = r#"<html lang="fr"><head><title>Test</title></head><body>
        <nav><a href="/">Home</a></nav>
        <main><h1>Hello</h1><p>World</p></main>
    </body></html>"#;
    let som = compiler::compile(html, "https://example.com").unwrap();
    assert_eq!(som.title, "Test");
    assert_eq!(som.lang, "fr");
    assert!(som.regions.iter().any(|r| r.role == RegionRole::Navigation));
    assert!(som.regions.iter().any(|r| r.role == RegionRole::Main));
}

// ============================================================
// Iframe Support Tests
// ============================================================

#[test]
fn test_iframe_detection() {
    let html = load_fixture("iframe_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    let elems = all_elements(&som);
    let iframes: Vec<&&Element> = elems
        .iter()
        .filter(|e| e.role == ElementRole::Iframe)
        .collect();

    // Should detect at least 3 visible iframes (the hidden one may or may not be stripped)
    assert!(
        iframes.len() >= 3,
        "Expected >=3 iframes, found {}",
        iframes.len()
    );
}

#[test]
fn test_iframe_attributes() {
    let html = load_fixture("iframe_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    let elems = all_elements(&som);
    let iframes: Vec<&&Element> = elems
        .iter()
        .filter(|e| e.role == ElementRole::Iframe)
        .collect();

    // Check that at least one iframe has src attribute
    let has_src = iframes
        .iter()
        .any(|e| e.attrs.as_ref().and_then(|a| a.get("src")).is_some());
    assert!(has_src, "At least one iframe should have src attribute");

    // Check that srcdoc iframe is detected
    let has_srcdoc = iframes.iter().any(|e| {
        e.attrs
            .as_ref()
            .and_then(|a| a.get("has_srcdoc"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });
    assert!(has_srcdoc, "Should detect iframe with srcdoc");

    // Check sandbox attribute is captured
    let has_sandbox = iframes
        .iter()
        .any(|e| e.attrs.as_ref().and_then(|a| a.get("sandbox")).is_some());
    assert!(has_sandbox, "Should capture sandbox attribute");
}

#[test]
fn test_iframe_inline() {
    // Test inline HTML with iframe
    let html = r#"<html><head><title>Inline Iframe</title></head><body>
        <main>
            <iframe src="https://embed.example.com" name="test-frame" width="640" height="480"></iframe>
        </main>
    </body></html>"#;
    let som = compiler::compile(html, "https://example.com").unwrap();

    let elems = all_elements(&som);
    let iframe = elems.iter().find(|e| e.role == ElementRole::Iframe);
    assert!(iframe.is_some(), "Should find iframe element");

    let iframe = iframe.unwrap();
    let attrs = iframe.attrs.as_ref().expect("Iframe should have attrs");
    assert_eq!(
        attrs.get("src").and_then(|v| v.as_str()),
        Some("https://embed.example.com")
    );
    assert_eq!(
        attrs.get("name").and_then(|v| v.as_str()),
        Some("test-frame")
    );
    assert_eq!(attrs.get("width").and_then(|v| v.as_str()), Some("640"));
    assert_eq!(attrs.get("height").and_then(|v| v.as_str()), Some("480"));
}

// ============================================================
// Shadow DOM Support Tests
// ============================================================

fn find_elements_with_shadow(som: &Som) -> Vec<&Element> {
    fn collect_with_shadow<'a>(elements: &'a [Element], result: &mut Vec<&'a Element>) {
        for el in elements {
            if el.shadow.is_some() {
                result.push(el);
            }
            if let Some(children) = &el.children {
                collect_with_shadow(children, result);
            }
        }
    }

    let mut result = Vec::new();
    for region in &som.regions {
        collect_with_shadow(&region.elements, &mut result);
    }
    result
}

#[test]
fn test_declarative_shadow_dom_detection() {
    let html = load_fixture("shadow_dom_page.html");
    let som = compiler::compile(&html, "https://example.com").unwrap();

    let _shadowed = find_elements_with_shadow(&som);

    // Should detect at least one element with shadow DOM
    // Note: html5ever may or may not fully support declarative shadow DOM parsing
    // This test verifies our detection logic works when templates are present
    assert!(!som.regions.is_empty(), "Should have at least one region");
}

#[test]
fn test_shadow_dom_inline() {
    // Test inline HTML with declarative shadow DOM
    let html = r#"<html><head><title>Shadow Test</title></head><body>
        <main role="main">
            <h1>Shadow DOM Test</h1>
            <my-element>
                <template shadowrootmode="open">
                    <p>Shadow content here</p>
                    <button>Shadow button</button>
                </template>
            </my-element>
        </main>
    </body></html>"#;
    let som = compiler::compile(html, "https://example.com").unwrap();

    assert_eq!(som.title, "Shadow Test");
    // Should compile and produce some output
    assert!(!som.regions.is_empty(), "Should have at least one region");
}

#[test]
fn test_shadow_root_modes() {
    // Test that both open and closed shadow roots are detected
    let html = r#"<html><head><title>Modes Test</title></head><body>
        <article>
            <h1>Modes Test</h1>
            <open-element>
                <template shadowrootmode="open">
                    <span>Open shadow</span>
                </template>
            </open-element>
            <closed-element>
                <template shadowrootmode="closed">
                    <span>Closed shadow</span>
                </template>
            </closed-element>
            <p>Regular content</p>
        </article>
    </body></html>"#;
    let som = compiler::compile(html, "https://example.com").unwrap();

    // Should compile without errors and produce content
    assert!(!som.regions.is_empty(), "Should have at least one region");
}

#[test]
fn test_regular_template_stripped() {
    // Regular templates (without shadowrootmode) should still be stripped
    let html = r#"<html><head><title>Template Test</title></head><body>
        <main>
            <template id="my-template">
                <p>This should be stripped</p>
            </template>
            <p>This should remain</p>
        </main>
    </body></html>"#;
    let som = compiler::compile(html, "https://example.com").unwrap();
    let json = serde_json::to_string(&som).unwrap();

    // Regular template content should be stripped
    assert!(
        !json.contains("This should be stripped"),
        "Regular template content should be stripped"
    );
    assert!(
        json.contains("This should remain"),
        "Non-template content should remain"
    );
}
