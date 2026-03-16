use std::sync::Arc;
use std::time::Instant;

use reqwest::cookie::Jar;
use tracing::{info, warn};

use crate::network::fetch;
use crate::som::compiler;

/// Result for a single URL benchmark.
#[derive(Debug)]
pub struct BenchResult {
    pub url: String,
    pub status: String,
    pub html_bytes: usize,
    pub som_bytes: usize,
    pub element_count: usize,
    pub interactive_count: usize,
    pub fetch_ms: u64,
    pub parse_ms: u64,
    pub error: Option<String>,
}

/// Aggregate stats across all benchmarks.
pub struct BenchReport {
    pub results: Vec<BenchResult>,
    pub date: String,
}

impl BenchReport {
    fn successful(&self) -> Vec<&BenchResult> {
        self.results.iter().filter(|r| r.status == "ok").collect()
    }

    /// Generate a Markdown report.
    pub fn to_markdown(&self) -> String {
        let total = self.results.len();
        let ok = self.successful();
        let ok_count = ok.len();

        let mut md = String::new();
        md.push_str("# Plasmate SOM Benchmark Report\n\n");
        md.push_str(&format!("Date: {}\n", self.date));
        md.push_str("Engine: plasmate v0.1.0\n");
        md.push_str(&format!("URLs tested: {}\n", total));
        md.push_str(&format!(
            "Successful: {} ({:.0}%)\n\n",
            ok_count,
            if total > 0 {
                (ok_count as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        ));

        if !ok.is_empty() {
            md.push_str("## Summary\n\n");
            md.push_str("| Metric | Mean | Median | P95 |\n");
            md.push_str("|---|---|---|---|\n");

            let html_bytes: Vec<f64> = ok.iter().map(|r| r.html_bytes as f64).collect();
            let som_bytes: Vec<f64> = ok.iter().map(|r| r.som_bytes as f64).collect();
            let ratios: Vec<f64> = ok
                .iter()
                .filter(|r| r.som_bytes > 0)
                .map(|r| r.html_bytes as f64 / r.som_bytes as f64)
                .collect();
            let html_tokens: Vec<f64> = ok.iter().map(|r| r.html_bytes as f64 / 4.0).collect();
            let som_tokens: Vec<f64> = ok.iter().map(|r| r.som_bytes as f64 / 4.0).collect();
            let elements: Vec<f64> = ok.iter().map(|r| r.element_count as f64).collect();
            let interactive: Vec<f64> = ok.iter().map(|r| r.interactive_count as f64).collect();
            let fetch_times: Vec<f64> = ok.iter().map(|r| r.fetch_ms as f64).collect();
            let parse_times: Vec<f64> = ok.iter().map(|r| r.parse_ms as f64).collect();

            md.push_str(&format_stat_row("HTML bytes", &html_bytes));
            md.push_str(&format_stat_row("SOM bytes", &som_bytes));
            md.push_str(&format_stat_row_ratio("Byte ratio", &ratios));
            md.push_str(&format_stat_row("HTML tokens (est)", &html_tokens));
            md.push_str(&format_stat_row("SOM tokens (est)", &som_tokens));
            md.push_str(&format_stat_row_ratio("Token ratio", &ratios));
            md.push_str(&format_stat_row("Elements found", &elements));
            md.push_str(&format_stat_row("Interactive found", &interactive));
            md.push_str(&format_stat_row("Fetch time (ms)", &fetch_times));
            md.push_str(&format_stat_row("Parse+SOM time (ms)", &parse_times));
        }

        md.push_str("\n## Per-URL Results\n\n");
        md.push_str("| URL | HTML bytes | SOM bytes | Ratio | Grade | Elements | Interactive | Fetch ms | Parse ms | Status |\n");
        md.push_str("|---|---|---|---|---|---|---|---|---|---|\n");

        for r in &self.results {
            let ratio_val = if r.som_bytes > 0 {
                r.html_bytes as f64 / r.som_bytes as f64
            } else {
                0.0
            };
            let ratio = if r.som_bytes > 0 {
                format!("{:.1}x", ratio_val)
            } else {
                "N/A".into()
            };
            let grade = ratio_to_grade(ratio_val);
            let short_url = shorten_url(&r.url);
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                short_url,
                format_number(r.html_bytes),
                format_number(r.som_bytes),
                ratio,
                grade,
                r.element_count,
                r.interactive_count,
                r.fetch_ms,
                r.parse_ms,
                r.status
            ));
        }

        // Add total summary at the bottom
        md.push_str("\n## Grade Distribution\n\n");
        let mut grade_counts = [0usize; 5]; // A, B, C, D, F
        for r in &ok {
            if r.som_bytes > 0 {
                let ratio = r.html_bytes as f64 / r.som_bytes as f64;
                match ratio_to_grade(ratio).as_str() {
                    "A" => grade_counts[0] += 1,
                    "B" => grade_counts[1] += 1,
                    "C" => grade_counts[2] += 1,
                    "D" => grade_counts[3] += 1,
                    _ => grade_counts[4] += 1,
                }
            } else {
                grade_counts[4] += 1; // F for no SOM
            }
        }
        md.push_str("| Grade | Count | Criteria |\n");
        md.push_str("|---|---|---|\n");
        md.push_str(&format!("| A | {} | >15x ratio |\n", grade_counts[0]));
        md.push_str(&format!("| B | {} | 8-15x ratio |\n", grade_counts[1]));
        md.push_str(&format!("| C | {} | 3-8x ratio |\n", grade_counts[2]));
        md.push_str(&format!("| D | {} | 1-3x ratio |\n", grade_counts[3]));
        md.push_str(&format!("| F | {} | <1x ratio |\n", grade_counts[4]));

        md
    }

    /// Print a brief summary to stdout (for console output).
    pub fn print_summary(&self) {
        let ok = self.successful();
        let ok_count = ok.len();
        let total = self.results.len();

        println!("=== Plasmate SOM Benchmark Summary ===");
        println!("URLs tested: {}, Successful: {} ({:.0}%)",
            total, ok_count,
            if total > 0 { (ok_count as f64 / total as f64) * 100.0 } else { 0.0 }
        );

        if !ok.is_empty() {
            let ratios: Vec<f64> = ok
                .iter()
                .filter(|r| r.som_bytes > 0)
                .map(|r| r.html_bytes as f64 / r.som_bytes as f64)
                .collect();

            if !ratios.is_empty() {
                let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
                let median = percentile(&ratios, 0.5);
                println!("Compression ratio: mean {:.1}x, median {:.1}x", mean, median);
            }

            // Print per-URL results
            println!("\nPer-URL Results:");
            println!("{:<50} {:>10} {:>10} {:>8} {:>6}",
                "URL", "HTML", "SOM", "Ratio", "Grade");
            println!("{}", "-".repeat(90));

            for r in &self.results {
                let ratio_val = if r.som_bytes > 0 {
                    r.html_bytes as f64 / r.som_bytes as f64
                } else {
                    0.0
                };
                let ratio_str = if r.som_bytes > 0 {
                    format!("{:.1}x", ratio_val)
                } else {
                    "N/A".into()
                };
                let grade = ratio_to_grade(ratio_val);
                let short_url: String = shorten_url(&r.url).chars().take(48).collect();
                let status_marker = if r.status == "ok" { "" } else { " [ERR]" };
                println!("{:<50} {:>10} {:>10} {:>8} {:>6}{}",
                    short_url,
                    format_number(r.html_bytes),
                    format_number(r.som_bytes),
                    ratio_str,
                    grade,
                    status_marker
                );
            }
        }
    }
}

/// Convert compression ratio to letter grade.
fn ratio_to_grade(ratio: f64) -> String {
    if ratio > 15.0 {
        "A".into()
    } else if ratio >= 8.0 {
        "B".into()
    } else if ratio >= 3.0 {
        "C".into()
    } else if ratio >= 1.0 {
        "D".into()
    } else {
        "F".into()
    }
}

fn shorten_url(url: &str) -> String {
    url.replace("https://", "")
        .replace("http://", "")
        .chars()
        .take(50)
        .collect()
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_stat_row(label: &str, values: &[f64]) -> String {
    if values.is_empty() {
        return format!("| {} | - | - | - |\n", label);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let median = percentile(values, 0.5);
    let p95 = percentile(values, 0.95);
    format!(
        "| {} | {} | {} | {} |\n",
        label,
        format_number(mean as usize),
        format_number(median as usize),
        format_number(p95 as usize)
    )
}

fn format_stat_row_ratio(label: &str, values: &[f64]) -> String {
    if values.is_empty() {
        return format!("| {} | - | - | - |\n", label);
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let median = percentile(values, 0.5);
    let p95 = percentile(values, 0.95);
    format!(
        "| {} | {:.1}x | {:.1}x | {:.1}x |\n",
        label, mean, median, p95
    )
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Run benchmarks for a list of URLs.
pub async fn run(urls: &[String], timeout_ms: u64) -> BenchReport {
    let jar = Arc::new(Jar::default());
    let client = fetch::build_client(None, jar).expect("Failed to build HTTP client");

    let mut results = Vec::new();

    for url in urls {
        info!(url, "Benchmarking");
        let result = bench_single(&client, url, timeout_ms).await;
        match &result.error {
            Some(e) => warn!(url, error = %e, "Benchmark failed"),
            None => info!(
                url,
                html_bytes = result.html_bytes,
                som_bytes = result.som_bytes,
                ratio = format!("{:.1}x", if result.som_bytes > 0 {
                    result.html_bytes as f64 / result.som_bytes as f64
                } else { 0.0 }),
                "Benchmark complete"
            ),
        }
        results.push(result);
    }

    let date = chrono_like_date();

    BenchReport { results, date }
}

async fn bench_single(client: &reqwest::Client, url: &str, timeout_ms: u64) -> BenchResult {
    let fetch_start = Instant::now();
    let fetch_result = match fetch::fetch_url(client, url, timeout_ms).await {
        Ok(r) => r,
        Err(e) => {
            return BenchResult {
                url: url.to_string(),
                status: "error".into(),
                html_bytes: 0,
                som_bytes: 0,
                element_count: 0,
                interactive_count: 0,
                fetch_ms: fetch_start.elapsed().as_millis() as u64,
                parse_ms: 0,
                error: Some(e.to_string()),
            };
        }
    };
    let fetch_ms = fetch_start.elapsed().as_millis() as u64;

    let parse_start = Instant::now();
    match compiler::compile(&fetch_result.html, &fetch_result.url) {
        Ok(som) => {
            let parse_ms = parse_start.elapsed().as_millis() as u64;
            BenchResult {
                url: fetch_result.url,
                status: "ok".into(),
                html_bytes: fetch_result.html_bytes,
                som_bytes: som.meta.som_bytes,
                element_count: som.meta.element_count,
                interactive_count: som.meta.interactive_count,
                fetch_ms,
                parse_ms,
                error: None,
            }
        }
        Err(e) => BenchResult {
            url: url.to_string(),
            status: "error".into(),
            html_bytes: fetch_result.html_bytes,
            som_bytes: 0,
            element_count: 0,
            interactive_count: 0,
            fetch_ms,
            parse_ms: parse_start.elapsed().as_millis() as u64,
            error: Some(e.to_string()),
        },
    }
}

fn chrono_like_date() -> String {
    // Simple date without chrono dependency
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Rough date calc (good enough for a report header)
    let days = secs / 86400;
    let years = 1970 + days / 365;
    let remaining_days = days % 365;
    let month = remaining_days / 30 + 1;
    let day = remaining_days % 30 + 1;
    format!("{}-{:02}-{:02}", years, month.min(12), day.min(31))
}

/// Parse a urls.txt file into a list of URLs.
pub fn parse_urls_file(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect()
}
