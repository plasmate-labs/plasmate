use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod awp;
mod bench;
mod cache;
mod js;
mod network;
mod som;

#[derive(Parser)]
#[command(name = "plasmate")]
#[command(about = "Agent-native headless browser engine with Semantic Object Model")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch a URL and output SOM JSON
    Fetch {
        /// URL to fetch
        url: String,
        /// Output file (defaults to stdout)
        #[arg(long, short)]
        output: Option<String>,
    },
    /// Start the AWP WebSocket server
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on
        #[arg(long, default_value = "9222")]
        port: u16,
    },
    /// Run SOM benchmarks against a list of URLs
    Bench {
        /// File containing URLs (one per line)
        #[arg(long, default_value = "bench/urls.txt")]
        urls: String,
        /// Output file for the report
        #[arg(long, default_value = "report.md")]
        output: String,
        /// Timeout per URL in milliseconds
        #[arg(long, default_value = "15000")]
        timeout: u64,
    },
    /// Throughput benchmark: fetch+compile N pages from a local server.
    /// Matches Lightpanda's benchmark methodology (local server, no external latency).
    ThroughputBench {
        /// Base URL of the local benchmark server
        #[arg(long, default_value = "http://127.0.0.1:8765")]
        base_url: String,
        /// Number of pages to fetch
        #[arg(long, default_value = "100")]
        pages: usize,
        /// Max concurrent fetches
        #[arg(long, default_value = "50")]
        concurrency: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure tracing to write to stderr, not stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch { url, output } => {
            cmd_fetch(&url, output.as_deref()).await?;
        }
        Commands::Serve { host, port } => {
            awp::server::start(&host, port).await?;
        }
        Commands::Bench {
            urls,
            output,
            timeout,
        } => {
            cmd_bench(&urls, &output, timeout).await?;
        }
        Commands::ThroughputBench {
            base_url,
            pages,
            concurrency,
        } => {
            cmd_throughput_bench(&base_url, pages, concurrency).await?;
        }
    }

    Ok(())
}

async fn cmd_fetch(url: &str, output: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let client = network::fetch::build_client(None, jar)?;

    // Initialize SOM cache
    let som_cache = cache::store::SomCache::new(cache::store::CacheConfig::default());

    info!(url, "Fetching");
    let result = network::fetch::fetch_url(&client, url, 30000).await?;
    info!(
        url = %result.url,
        status = result.status,
        html_bytes = result.html_bytes,
        load_ms = result.load_ms,
        "Fetched"
    );

    // Check SOM cache
    let content_hash = cache::store::SomCache::content_hash(result.html.as_bytes());
    let (json, cache_status) = match som_cache.lookup(&result.url, content_hash) {
        cache::store::CacheLookup::Hit(entry) => {
            info!(url = %result.url, hit_count = entry.hit_count, "SOM cache HIT");
            (String::from_utf8(entry.som_json).unwrap_or_default(), "hit")
        }
        _ => {
            let fetch_start = std::time::Instant::now();
            let som = som::compiler::compile(&result.html, &result.url)?;
            let compile_ms = fetch_start.elapsed().as_millis();
            let json = serde_json::to_string_pretty(&som)?;

            // Store in cache for next time
            som_cache.store(&result.url, content_hash, json.as_bytes().to_vec(), result.html_bytes);
            info!(compile_ms, "SOM compiled and cached");

            (json, "miss")
        }
    };

    match output {
        Some(path) => {
            std::fs::write(path, &json)?;
            info!(path, cache = cache_status, "SOM written");
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

async fn cmd_bench(
    urls_file: &str,
    output: &str,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(urls_file)?;
    let urls = bench::runner::parse_urls_file(&content);

    info!(count = urls.len(), "Running benchmarks");
    let report = bench::runner::run(&urls, timeout).await;

    let md = report.to_markdown();
    std::fs::write(output, &md)?;
    info!(output, "Benchmark report written");

    // Print summary to stdout
    report.print_summary();

    Ok(())
}

async fn cmd_throughput_bench(
    base_url: &str,
    pages: usize,
    concurrency: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    let jar = Arc::new(reqwest::cookie::Jar::default());
    let client = network::fetch::build_client_h1_fallback(None, jar)?;

    // Generate URLs
    let urls: Vec<String> = (1..=pages).map(|i| format!("{}/{}", base_url, i)).collect();

    eprintln!("=== Plasmate Throughput Benchmark ===");
    eprintln!("Pages: {}", pages);
    eprintln!("Concurrency: {}", concurrency);
    eprintln!("Server: {}", base_url);
    eprintln!();

    // --- Sequential benchmark ---
    eprintln!("--- Sequential (1 at a time) ---");
    let start = Instant::now();
    let mut total_html_bytes = 0usize;
    let mut total_som_bytes = 0usize;
    let mut total_elements = 0usize;
    let mut compile_time_us = 0u128;

    for url in &urls {
        let result = network::fetch::fetch_url(&client, url, 10000).await?;
        total_html_bytes += result.html_bytes;

        let compile_start = Instant::now();
        let compiled = som::compiler::compile(&result.html, &result.url)?;
        compile_time_us += compile_start.elapsed().as_micros();

        total_som_bytes += compiled.meta.som_bytes;
        total_elements += compiled.meta.element_count;
    }

    let seq_elapsed = start.elapsed();
    let seq_ms = seq_elapsed.as_millis();
    let seq_per_page = seq_ms as f64 / pages as f64;

    eprintln!("Total time: {}ms ({:.1}ms/page)", seq_ms, seq_per_page);
    eprintln!("SOM compile time: {}ms ({:.1}us/page)", compile_time_us / 1000, compile_time_us as f64 / pages as f64);
    eprintln!("HTML bytes: {} ({}/page)", total_html_bytes, total_html_bytes / pages);
    eprintln!("SOM bytes: {} ({}/page)", total_som_bytes, total_som_bytes / pages);
    eprintln!("Elements: {} ({}/page)", total_elements, total_elements / pages);
    eprintln!();

    // --- Parallel benchmark ---
    eprintln!("--- Parallel ({} concurrent) ---", concurrency);
    let start = Instant::now();

    let results = network::fetch::fetch_urls_parallel(&client, &urls, 10000, concurrency).await;

    let fetch_elapsed = start.elapsed();
    let mut par_html_bytes = 0usize;
    let mut par_som_bytes = 0usize;
    let mut par_elements = 0usize;
    let mut par_compile_us = 0u128;
    let mut success_count = 0usize;

    for result in results {
        if let Ok(r) = result {
            par_html_bytes += r.html_bytes;
            let compile_start = Instant::now();
            if let Ok(compiled) = som::compiler::compile(&r.html, &r.url) {
                par_compile_us += compile_start.elapsed().as_micros();
                par_som_bytes += compiled.meta.som_bytes;
                par_elements += compiled.meta.element_count;
                success_count += 1;
            }
        }
    }

    let par_ms = fetch_elapsed.as_millis();
    let par_per_page = par_ms as f64 / pages as f64;

    eprintln!("Total time: {}ms ({:.1}ms/page effective)", par_ms, par_per_page);
    eprintln!("SOM compile time: {}ms ({:.1}us/page)", par_compile_us / 1000, par_compile_us as f64 / success_count as f64);
    eprintln!("Successful: {}/{}", success_count, pages);
    eprintln!();

    // --- Memory usage ---
    eprintln!("--- Summary ---");
    eprintln!("Sequential:  {}ms total, {:.1}ms/page", seq_ms, seq_per_page);
    eprintln!("Parallel:    {}ms total, {:.1}ms/page effective", par_ms, par_per_page);
    eprintln!("Speedup:     {:.1}x", seq_ms as f64 / par_ms as f64);
    eprintln!();
    eprintln!("Comparison (Lightpanda claims for 100 local pages):");
    eprintln!("  Lightpanda: 2,300ms sequential");
    eprintln!("  Chrome:     25,200ms sequential");
    eprintln!("  Plasmate:   {}ms sequential", seq_ms);

    Ok(())
}
