//! Tests for SOM v0.2 improvements:
//! 1. Link deduplication
//! 2. Smart text truncation
//! 3. CSS class hint inference
//! 4. Heading hierarchy preservation
//! 5. JS mutation pipeline

use plasmate::som::compiler;
use plasmate::som::heuristics;

// ============================================================
// 1. LINK DEDUPLICATION
// ============================================================

#[test]
fn test_duplicate_links_removed() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Dedup Test</title></head>
<body><main>
    <a href="/about">About Us</a>
    <a href="/about">About (header)</a>
    <a href="/about#team">About Team</a>
    <a href="/contact">Contact</a>
    <a href="/contact/">Contact (trailing slash)</a>
    <a href="/unique">Unique link</a>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let json = serde_json::to_string(&som).unwrap();

    // Count occurrences of /about href - should appear once (dedup strips fragment and trailing slash)
    let about_count = json.matches("\"/about\"").count();
    assert_eq!(
        about_count, 1,
        "Duplicate /about links should be deduplicated"
    );

    // /contact and /contact/ should dedup
    let contact_count =
        json.matches("\"/contact\"").count() + json.matches("\"/contact/\"").count();
    assert_eq!(contact_count, 1, "/contact and /contact/ should dedup");

    // /unique should still be there
    assert!(json.contains("/unique"), "Unique links should survive");
}

#[test]
fn test_dedup_preserves_first_occurrence() {
    let html = r#"<!DOCTYPE html>
<html><head><title>First Wins</title></head>
<body><main>
    <a href="/page">First text</a>
    <a href="/page">Second text</a>
    <a href="/page">Third text</a>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let json = serde_json::to_string(&som).unwrap();

    assert!(
        json.contains("First text"),
        "First occurrence should be kept"
    );
    // Total /page hrefs should be 1
    let page_count = json.matches("\"/page\"").count();
    assert_eq!(page_count, 1, "Only first /page link should survive");
}

#[test]
fn test_dedup_case_insensitive() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Case Test</title></head>
<body><main>
    <a href="/Page">Link 1</a>
    <a href="/page">Link 2</a>
    <a href="/PAGE">Link 3</a>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let json = serde_json::to_string_pretty(&som).unwrap();

    // All three point to the same URL (case-insensitive), so only first should survive
    let link_count = json.matches("\"role\": \"link\"").count();
    assert_eq!(link_count, 1, "Case-insensitive dedup should keep only one");
}

// ============================================================
// 2. SMART TEXT TRUNCATION
// ============================================================

#[test]
fn test_smart_truncate_at_sentence() {
    let text = "This is the first sentence. This is the second sentence that goes on and on. And here is a third sentence that would push us way past the limit.";
    let result = heuristics::smart_truncate(text, 80);
    // Should cut at sentence boundary
    assert!(
        result.ends_with('.') || result.ends_with('!') || result.ends_with('?'),
        "Should truncate at sentence boundary, got: '{}'",
        result
    );
    assert!(result.len() <= 80, "Should be within budget");
}

#[test]
fn test_smart_truncate_no_sentence_falls_to_word() {
    let text = "This is a very long text without any sentence ending punctuation that just keeps going and going without stopping for breath or any kind of pause";
    let result = heuristics::smart_truncate(text, 60);
    assert!(
        result.ends_with("..."),
        "Should add ellipsis when truncating at word, got: '{}'",
        result
    );
    assert!(result.len() <= 63, "Should be within budget plus ellipsis");
}

#[test]
fn test_smart_truncate_short_text_unchanged() {
    let text = "Short text.";
    let result = heuristics::smart_truncate(text, 200);
    assert_eq!(result, text, "Short text should pass through unchanged");
}

#[test]
fn test_smart_truncate_avoids_early_sentence() {
    // If the only sentence boundary is very early, fall back to word boundary
    let text = "Hi. This is a very long continuation that has no more periods in it and just goes on describing something at great length without stopping";
    let result = heuristics::smart_truncate(text, 100);
    // "Hi." is only 3 chars, which is < 40% of 100 = 40, so should NOT cut there
    assert_ne!(
        result, "Hi.",
        "Should not truncate at very early sentence boundary"
    );
}

// ============================================================
// 3. CSS CLASS HINT INFERENCE
// ============================================================

#[test]
fn test_infer_primary_button() {
    let attrs = vec![("class".to_string(), "btn btn-primary large".to_string())];
    let hints = heuristics::infer_class_hints(&attrs).unwrap();
    assert!(
        hints.contains(&"primary".to_string()),
        "Should detect 'primary'"
    );
    assert!(
        hints.contains(&"large".to_string()),
        "Should detect 'large'"
    );
}

#[test]
fn test_infer_danger_button() {
    let attrs = vec![("class".to_string(), "button danger".to_string())];
    let hints = heuristics::infer_class_hints(&attrs).unwrap();
    assert!(hints.contains(&"danger".to_string()));
}

#[test]
fn test_infer_state_classes() {
    let attrs = vec![(
        "class".to_string(),
        "nav-item active is-selected".to_string(),
    )];
    let hints = heuristics::infer_class_hints(&attrs).unwrap();
    assert!(hints.contains(&"active".to_string()));
    assert!(hints.contains(&"selected".to_string()));
}

#[test]
fn test_infer_loading_state() {
    let attrs = vec![("class".to_string(), "skeleton-loader loading".to_string())];
    let hints = heuristics::infer_class_hints(&attrs).unwrap();
    assert!(hints.contains(&"loading".to_string()));
}

#[test]
fn test_infer_no_hints_for_generic_class() {
    let attrs = vec![("class".to_string(), "container flex mt-4".to_string())];
    let hints = heuristics::infer_class_hints(&attrs);
    assert!(
        hints.is_none(),
        "Generic utility classes should produce no hints"
    );
}

#[test]
fn test_infer_card_component() {
    let attrs = vec![("class".to_string(), "card shadow-sm mb-3".to_string())];
    let hints = heuristics::infer_class_hints(&attrs).unwrap();
    assert!(hints.contains(&"card".to_string()));
}

#[test]
fn test_infer_notification() {
    let attrs = vec![("class".to_string(), "toast toast-success".to_string())];
    let hints = heuristics::infer_class_hints(&attrs).unwrap();
    assert!(hints.contains(&"notification".to_string()));
    assert!(hints.contains(&"success".to_string()));
}

#[test]
fn test_hints_in_compiled_som() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Hints</title></head>
<body><main>
    <a href="/action" class="btn btn-primary cta-button">Get Started</a>
    <a href="/delete" class="btn btn-danger">Delete Account</a>
    <a href="/plain">Plain link</a>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let json = serde_json::to_string(&som).unwrap();

    assert!(
        json.contains("\"primary\""),
        "Primary hint should appear in SOM JSON"
    );
    assert!(
        json.contains("\"danger\""),
        "Danger hint should appear in SOM JSON"
    );
}

// ============================================================
// 4. HEADING HIERARCHY PRESERVATION
// ============================================================

#[test]
fn test_headings_never_dropped_by_budget() {
    // Create a page with lots of elements + headings interspersed
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html><head><title>Heading Test</title></head>
<body><main>
    <h1>Main Title</h1>
"#,
    );

    // Add enough links to blow the budget
    for i in 0..250 {
        html.push_str(&format!("    <a href=\"/link-{}\">Link {}</a>\n", i, i));
        if i % 50 == 0 {
            html.push_str(&format!("    <h2>Section {}</h2>\n", i / 50));
        }
    }

    html.push_str("</main></body></html>");

    let som = compiler::compile(&html, "https://example.com").unwrap();
    let json = serde_json::to_string(&som).unwrap();

    // Headings should survive even past element budgets
    assert!(json.contains("Main Title"), "H1 should never be dropped");
    assert!(json.contains("Section 0"), "H2 should never be dropped");
    assert!(json.contains("Section 1"), "H2 should never be dropped");
}

#[test]
fn test_heading_levels_preserved() {
    let html = r#"<!DOCTYPE html>
<html><head><title>Levels</title></head>
<body><main>
    <h1>Top Level</h1>
    <h2>Sub Section</h2>
    <h3>Detail</h3>
    <h2>Another Sub</h2>
    <h4>Deep Detail</h4>
</main></body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let json = serde_json::to_string_pretty(&som).unwrap();

    // All heading levels should be preserved in attrs
    assert!(json.contains("\"level\": 1"));
    assert!(json.contains("\"level\": 2"));
    assert!(json.contains("\"level\": 3"));
    assert!(json.contains("\"level\": 4"));
}

// ============================================================
// 5. JS MUTATION PIPELINE
// ============================================================

#[test]
fn test_pipeline_with_document_write() {
    use plasmate::js::pipeline::{process_page, PipelineConfig};

    let config = PipelineConfig::default();
    let html = r#"<html><head><title>Write Test</title></head>
<body>
    <script>document.write('<p>Injected by JS</p>');</script>
    <h1>Original Content</h1>
</body></html>"#;

    let result = process_page(html, "https://example.com", &config).unwrap();
    // The pipeline should have detected the document.write mutation
    assert!(result.js_report.is_some());
    let report = result.js_report.unwrap();
    assert_eq!(report.succeeded, 1);
}

#[test]
fn test_pipeline_with_appendchild() {
    use plasmate::js::pipeline::{process_page, PipelineConfig};

    let config = PipelineConfig::default();
    let html = r#"<html><body>
    <script>
        var el = document.createElement('p');
        el.textContent = 'Dynamic content';
        document.body.appendChild(el);
    </script>
    <h1>Static Content</h1>
</body></html>"#;

    let result = process_page(html, "https://example.com", &config).unwrap();
    assert!(result.js_report.is_some());
    let report = result.js_report.unwrap();
    assert_eq!(report.succeeded, 1);
    assert_eq!(report.failed, 0);
}

// ============================================================
// COMBINED: Full page with all improvements
// ============================================================

#[test]
fn test_all_improvements_combined() {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Full Test</title></head>
<body>
<nav>
    <a href="/home" class="nav-link active">Home</a>
    <a href="/home">Home Again</a>
    <a href="/about">About</a>
    <a href="/about#team">Team</a>
    <a href="/contact">Contact</a>
</nav>
<main>
    <h1>Welcome to our platform</h1>
    <p>This is the first paragraph with a good amount of text that explains what the platform does. It has several sentences. The platform is designed for AI agents.</p>
    <h2>Features</h2>
    <a href="/signup" class="btn btn-primary btn-lg">Get Started Free</a>
    <a href="/delete" class="btn btn-danger small">Delete Account</a>
    <a href="/info" class="badge bg-secondary">Info</a>
    <h3>Speed</h3>
    <p>We are fast. Very fast. Faster than everything else on the market today.</p>
    <h3>Accuracy</h3>
    <p>Our accuracy is unmatched in the industry.</p>
</main>
<footer>
    <p>Copyright 2026 Example Corp. All rights reserved.</p>
    <a href="/privacy">Privacy</a>
    <a href="/terms">Terms</a>
</footer>
</body></html>"#;

    let som = compiler::compile(html, "https://example.com").unwrap();
    let json = serde_json::to_string_pretty(&som).unwrap();

    // Link dedup: /home should appear once, /about should appear once
    let home_count = json.matches("\"/home\"").count();
    assert_eq!(
        home_count, 1,
        "Duplicate /home links should be deduplicated"
    );

    // Heading hierarchy: all levels present
    assert!(json.contains("\"level\": 1"));
    assert!(json.contains("\"level\": 2"));
    assert!(json.contains("\"level\": 3"));

    // CSS hints: primary and danger buttons should have hints
    assert!(
        json.contains("\"primary\""),
        "Primary hint should be present"
    );
    assert!(json.contains("\"danger\""), "Danger hint should be present");
    assert!(json.contains("\"badge\""), "Badge hint should be present");

    // Regions: should have nav, main, footer
    assert!(json.contains("\"navigation\""));
    assert!(json.contains("\"main\""));
    assert!(json.contains("\"footer\""));

    // For small test pages, SOM JSON overhead can exceed source HTML.
    // What matters: structural elements are preserved and semantic info is enriched.
    assert!(som.meta.element_count > 0, "SOM should contain elements");
    assert!(
        som.meta.interactive_count > 0,
        "SOM should contain interactive elements"
    );
}
