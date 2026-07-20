use std::io::{Read, Write};
use std::time::Duration;

fn input_url(request: &str) -> String {
    let marker = "\"input_url\":\"";
    let start = request.find(marker).expect("input_url") + marker.len();
    let remainder = &request[start..];
    let end = remainder.find('"').expect("input_url terminator");
    remainder[..end].to_string()
}

fn valid_result(url: &str, status: &str) -> String {
    let (failure_kind, error) = match status {
        "ok" | "blocked" => ("null", "null"),
        "failed" => ("\"http_error\"", "\"ordinary page error\""),
        _ => panic!("unsupported fixture status"),
    };
    format!(
        "{{\"input_url\":\"{url}\",\"final_url\":null,\"status\":\"{status}\",\"http_status\":null,\"content_type\":null,\"title\":null,\"html_bytes\":null,\"som_bytes\":null,\"compression_ratio\":null,\"element_count\":null,\"interactive_count\":null,\"fetch_ms\":null,\"pipeline_ms\":null,\"js_total_scripts\":null,\"js_succeeded\":null,\"js_failed\":null,\"failure_kind\":{failure_kind},\"error\":{error}}}"
    )
}

fn main() {
    let mut request = String::new();
    std::io::stdin().read_to_string(&mut request).unwrap();
    let url = input_url(&request);

    if url.contains("__fixture_exit__") {
        std::process::exit(23);
    }
    if url.contains("__fixture_abort__") {
        std::process::abort();
    }
    if url.contains("__fixture_hang__") {
        std::thread::sleep(Duration::from_secs(60));
        return;
    }
    if url.contains("__fixture_output__") {
        std::io::stdout().write_all(&vec![b'x'; 1024 * 1024]).unwrap();
        return;
    }
    if url.contains("__fixture_malformed__") {
        println!("not-json");
        return;
    }
    if url.contains("__fixture_resource__") {
        eprintln!("FATAL ERROR: Allocation failed - JavaScript heap out of memory");
        std::process::abort();
    }
    if url.contains("__fixture_page_error__") {
        println!("{}", valid_result(&url, "failed"));
        return;
    }
    if url.contains("__fixture_blocked__") {
        println!("{}", valid_result(&url, "blocked"));
        return;
    }

    println!("{}", valid_result(&url, "ok"));
}
