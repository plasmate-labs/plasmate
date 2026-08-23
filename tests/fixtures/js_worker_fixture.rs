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
        std::io::stdout().write_all(&vec![b'x'; 1024 * 1024]).unwrap();
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
    println!(r#"{{"status":"evaluation","value":{{"result":"ok"}}}}"#);
}
