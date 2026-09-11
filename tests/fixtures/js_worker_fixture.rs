use std::io::{Read, Write};
use std::time::Duration;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    if input.contains("__fixture_abort__") {
        std::process::abort();
    }
    if input.contains("__fixture_hang__") {
        std::thread::sleep(Duration::from_secs(60));
        return;
    }
    if input.contains("__fixture_output__") {
        std::io::stdout()
            .write_all(&vec![b'x'; 1024 * 1024])
            .unwrap();
        return;
    }
    if input.contains("__fixture_env__") && std::env::var_os("PLASMATE_TEST_SECRET").is_some() {
        eprintln!("secret environment leaked into worker");
        std::process::exit(17);
    }
    if input.contains("__fixture_dom_miss__") {
        println!(
            r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
        );
        return;
    }
    if input.contains("__fixture_html_id__") {
        if input.contains("getElementById") && input.contains("pay-now") {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"clicked\":true}}","effective_html":"<html><head><title>Pay</title></head><body><main><!-- __fixture_html_id__ --><button id='pay-now'>Pay</button></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_input_value__") {
        if input.contains("candidate.value")
            && input.contains("input[type=")
            && input.contains("Pay now")
        {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"clicked\":true}}","effective_html":"<html><head><title>Pay</title></head><body><main><!-- __fixture_input_value__ --><input type='submit' value='Pay now'></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_button_aria_label__") {
        if input.contains("getAttribute('aria-label')") && input.contains("Close") {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"clicked\":true}}","effective_html":"<html><head><title>Dialog</title></head><body><main><!-- __fixture_button_aria_label__ --><button aria-label='Close'></button></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_aria_tab__") {
        if input.contains(r#"[role=\"tab\"]"#) && input.contains("Overview") {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"clicked\":true}}","effective_html":"<html><head><title>Settings</title></head><body><main><!-- __fixture_aria_tab__ --><div role='tab'>Overview</div></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_aria_link__") {
        if input.contains(r#"[role=\"link\"]"#)
            && input.contains("Catalog")
            && !input.contains(r#"[role=\"option\"]"#)
            && !input.contains(r#"[role=\"treeitem\"]"#)
            && !input.contains(r#"[role=\"menuitem\"]"#)
        {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"clicked\":true}}","effective_html":"<html><head><title>Library</title></head><body><main><!-- __fixture_aria_link__ --><div role='link'>Catalog</div></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_aria_switch__") {
        if input.contains("role") && input.contains("switch") && input.contains("aria-checked") {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"toggled\":true,\"checked\":false}}","effective_html":"<html><head><title>Alerts</title></head><body><main><!-- __fixture_aria_switch__ --><button role='switch' id='alerts' aria-checked='false'>Email alerts</button></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element is not a checkbox, radio button, or details element\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_test_id__") {
        if input.contains("var testId")
            && input.contains("data-testid")
            && input.contains("data-qa")
            && input.contains("pay-now")
        {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"clicked\":true}}","effective_html":"<html><head><title>Pay</title></head><body><main><!-- __fixture_test_id__ --><button data-testid='pay-now'></button></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_compiled_href__") {
        if input.contains("var href")
            && input.contains("a[href]")
            && input.contains("javascript:void(0)")
            && !input.contains("area[href]")
        {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"clicked\":true}}","effective_html":"<html><head><title>Shop</title></head><body><main><!-- __fixture_compiled_href__ --><a href='javascript:void(0)'></a></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_same_document_fragment__") {
        if input.contains("var href") && input.contains("#pricing") && !input.contains("area[href]")
        {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"navigated\":true,\"href\":\"https://example.test/shop#pricing\"}}","effective_html":"<html><head><title>Shop</title></head><body><main><!-- __fixture_same_document_fragment__ --><a href='#pricing'>Pricing</a><h2 id='pricing'>Plans</h2></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_compiled_name__") {
        if input.contains("getAttribute('name')")
            && input.contains("fieldName")
            && input.contains("q")
        {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"typed\":true}}","effective_html":"<html><head><title>Search</title></head><body><main><!-- __fixture_compiled_name__ --><input name='q' placeholder='Search'><input name='other'></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_compiled_aria_label__") {
        if input.contains("getAttribute('aria-label')")
            && input.contains("fieldAriaLabel")
            && input.contains("Search")
            && input.contains("input, textarea")
            && !input.contains("getAttribute('placeholder')")
        {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"typed\":true}}","effective_html":"<html><head><title>Search</title></head><body><main><!-- __fixture_compiled_aria_label__ --><input type='search' aria-label='Search'><input aria-label='Other'></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element not found in DOM\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    if input.contains("__fixture_native_radio__") {
        if input.contains("type === 'radio'") {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"selected\":true,\"value\":\"email\"}}","effective_html":"<html><head><title>Contact</title></head><body><main><!-- __fixture_native_radio__ --><form><label><input type='radio' name='contact' value='email' checked> Email</label><label><input type='radio' name='contact' value='sms'> SMS</label></form></main></body></html>"}}}}"#
            );
        } else {
            println!(
                r#"{{"status":"evaluation","value":{{"result":"{{\"error\":\"Element is not a <select>\"}}","effective_html":"<html><body><p>mutated</p></body></html>"}}}}"#
            );
        }
        return;
    }
    println!(r#"{{"status":"evaluation","value":{{"result":"ok"}}}}"#);
}
