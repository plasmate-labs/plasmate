use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod awp;
mod bench;
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
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
    }

    Ok(())
}

async fn cmd_fetch(url: &str, output: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let client = network::fetch::build_client(None, jar)?;

    info!(url, "Fetching");
    let result = network::fetch::fetch_url(&client, url, 30000).await?;
    info!(
        url = %result.url,
        status = result.status,
        html_bytes = result.html_bytes,
        load_ms = result.load_ms,
        "Fetched"
    );

    let som = som::compiler::compile(&result.html, &result.url)?;
    let json = serde_json::to_string_pretty(&som)?;

    match output {
        Some(path) => {
            std::fs::write(path, &json)?;
            info!(path, som_bytes = som.meta.som_bytes, "SOM written");
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

    let ok = report.results.iter().filter(|r| r.status == "ok").count();
    println!(
        "Benchmark complete: {}/{} URLs succeeded",
        ok,
        report.results.len()
    );

    Ok(())
}
