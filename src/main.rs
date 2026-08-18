use clap::{Args, Parser, Subcommand};
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod mcp;

use plasmate::agent_workflow;
use plasmate::auth;
use plasmate::awp;
use plasmate::bench;
use plasmate::cache;
use plasmate::cdp;
use plasmate::coverage;
use plasmate::daemon;
use plasmate::js;
use plasmate::network;
use plasmate::plugin;
use plasmate::screenshot;
use plasmate::som;
use plasmate::webmcp;

#[derive(Parser)]
#[command(name = "plasmate")]
#[command(about = "Agent-native headless browser engine with Semantic Object Model")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// TLS configuration options (shared across fetch and serve commands).
#[derive(Args, Debug, Clone, Default)]
struct TlsArgs {
    /// Minimum TLS version (1.2 or 1.3)
    #[arg(long, value_name = "VERSION")]
    tls_min_version: Option<String>,
    /// Maximum TLS version (1.2 or 1.3)
    #[arg(long, value_name = "VERSION")]
    tls_max_version: Option<String>,
    /// Skip TLS certificate verification (like curl -k)
    #[arg(long, short = 'k')]
    insecure: bool,
    /// Path to PEM file with custom CA certificates
    #[arg(long, value_name = "FILE")]
    ca_cert: Option<String>,
    /// TLS 1.2 cipher suites (comma-separated IANA names, controls order in ClientHello)
    #[arg(long, value_name = "CIPHERS", value_delimiter = ',')]
    tls12_ciphers: Vec<String>,
    /// TLS 1.3 cipher suites (comma-separated IANA names)
    #[arg(long, value_name = "CIPHERS", value_delimiter = ',')]
    tls13_ciphers: Vec<String>,
    /// ALPN protocols to advertise (comma-separated, e.g., "h2,http/1.1")
    #[arg(long, value_name = "PROTOCOLS", value_delimiter = ',')]
    alpn: Vec<String>,
    /// Supported key exchange groups / curves (comma-separated, e.g., "x25519,secp256r1")
    #[arg(long, value_name = "GROUPS", value_delimiter = ',')]
    tls_groups: Vec<String>,
    /// Disable TLS Server Name Indication
    #[arg(long)]
    no_sni: bool,
    /// List available cipher suites and supported groups, then exit
    #[arg(long)]
    list_tls_options: bool,
}

impl TlsArgs {
    fn to_tls_config(&self) -> Result<Option<network::tls::TlsConfig>, Box<dyn std::error::Error>> {
        use network::tls::{TlsConfig, TlsVersion};

        let config = TlsConfig {
            min_version: self
                .tls_min_version
                .as_deref()
                .map(TlsVersion::parse)
                .transpose()?,
            max_version: self
                .tls_max_version
                .as_deref()
                .map(TlsVersion::parse)
                .transpose()?,
            danger_accept_invalid_certs: self.insecure,
            ca_cert_path: self.ca_cert.as_ref().map(std::path::PathBuf::from),
            cipher_suites_tls12: self.tls12_ciphers.clone(),
            cipher_suites_tls13: self.tls13_ciphers.clone(),
            alpn_protocols: self.alpn.clone(),
            supported_groups: self.tls_groups.clone(),
            enable_sni: if self.no_sni { Some(false) } else { None },
        };

        if config.is_default() {
            Ok(None)
        } else {
            Ok(Some(config))
        }
    }
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
        /// Output format: "json" (default, full SOM), "text" (plain extracted
        /// text), "markdown" (structured Markdown), or "links" (one URL per
        /// line, deduplicated — for crawlers and research agents)
        #[arg(long, default_value = "json")]
        format: String,
        /// Override the default User-Agent string.
        /// Some sites (e.g. w3.org, mysql.com) return 403 for Chrome-like UAs but
        /// accept plain curl-style requests. Use this to pass a simpler UA when needed.
        #[arg(long)]
        user_agent: Option<String>,
        /// Filter output to a specific SOM region or element.
        ///
        /// Accepts semantic region roles (main, nav, navigation, aside, header,
        /// footer, form, dialog, content), element roles (button, link,
        /// text_input, select, etc.), action selectors (interactive,
        /// action:click, action:type, action:select), or an HTML id selector
        /// (#my-id).
        /// When a role is given, only regions of that role are included.
        /// When an element/action selector or id is given, only matching
        /// elements are kept, with parent containers preserved.
        /// Unrecognised selectors fall through gracefully (full SOM returned).
        ///
        /// Examples:
        ///   --selector main            (just the main content region)
        ///   --selector nav             (navigation links only)
        ///   --selector interactive     (only actionable elements)
        ///   --selector action:click    (only click targets)
        ///   --selector "#toc"          (elements with id="toc")
        ///   --selector main --format text   (main content as plain text)
        #[arg(long)]
        selector: Option<String>,
        /// Request timeout in milliseconds (default: 30000).
        #[arg(long, default_value = "30000")]
        timeout: u64,
        /// Skip fetching external <script src="..."> files (inline only)
        #[arg(long)]
        no_external: bool,
        /// Disable JavaScript execution entirely
        #[arg(long)]
        no_js: bool,
        /// Add custom HTTP headers (can be specified multiple times).
        ///
        /// Format: "Header-Name: value"
        ///
        /// Examples:
        ///   --header "Authorization: Bearer sk-..."
        ///   --header "X-Custom: value" --header "Accept-Language: en"
        #[arg(long, short = 'H')]
        header: Vec<String>,
        /// Load cookies from a stored auth profile (domain name)
        #[arg(long)]
        profile: Option<String>,
        #[command(flatten)]
        tls: TlsArgs,
        /// Load a Wasm plugin (can be specified multiple times)
        #[arg(long)]
        plugin: Vec<String>,
    },
    /// Start the WebSocket server
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on
        #[arg(long, default_value = "9222")]
        port: u16,
        /// Protocol: awp (default), cdp (supported CDP/Puppeteer subset), or both
        #[arg(long, default_value = "cdp")]
        protocol: String,
        /// Load cookies from stored auth profile(s) (comma-separated domain names)
        #[arg(long)]
        profile: Option<String>,
        #[command(flatten)]
        tls: TlsArgs,
        /// Load a Wasm plugin (can be specified multiple times)
        #[arg(long)]
        plugin: Vec<String>,
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
    /// Run the versioned, deterministic product benchmark suite.
    BenchmarkV1 {
        /// Output path for the machine-readable v1 JSON report.
        #[arg(long, default_value = "benchmark-v1.json")]
        output: String,
        /// Execute inline JavaScript before compiling fixture pages.
        #[arg(long)]
        js: bool,
        /// Maximum allowed cold end-to-end latency per successful task.
        #[arg(long, default_value = "2000")]
        max_cold_ms: u64,
        /// Maximum allowed warm end-to-end latency per successful task.
        #[arg(long, default_value = "2000")]
        max_warm_ms: u64,
        /// Hard wall timeout for each supervised JS task worker.
        #[arg(long, default_value = "15000")]
        worker_timeout_ms: u64,
        /// Address-space limit per supervised JS worker in MB (Linux; 0 disables).
        #[arg(long, default_value = "0")]
        worker_memory_mb: u64,
        /// Maximum stdout/stderr captured from each JS worker in KB.
        #[arg(long, default_value = "2048")]
        worker_output_kb: usize,
    },
    /// Internal deterministic benchmark worker. Input/output are JSON over stdio.
    #[command(name = "__benchmark-worker", hide = true)]
    BenchmarkWorker,
    /// Run deterministic task-success contracts through supervised agent workflows.
    AgentTaskBenchmarkV1 {
        /// Output path for the versioned machine-readable evidence report.
        #[arg(long, default_value = "agent-task-benchmark-v1.json")]
        output: std::path::PathBuf,
    },
    /// Validate an agent task benchmark report and its complete denominators.
    AgentTaskBenchmarkValidate {
        /// Evidence produced by `agent-task-benchmark-v1`.
        #[arg(long, default_value = "agent-task-benchmark-v1.json")]
        input: std::path::PathBuf,
    },
    /// Validate all independently versioned release artifacts without publishing.
    ReleaseValidate {
        /// Repository release manifest.
        #[arg(long, default_value = "release-manifest.json")]
        manifest: String,
        /// Optional path for the machine-readable validation report.
        #[arg(long)]
        output: Option<String>,
    },
    /// Discover and validate static ARD catalog signals for an HTTPS page or origin.
    ArdDiscover {
        /// Operator-supplied HTTPS origin or page URL.
        url: String,
        /// Optional path for the versioned JSON discovery report.
        #[arg(long, short)]
        output: Option<String>,
        /// Whole-discovery wall deadline in milliseconds (1 to 30000).
        #[arg(long, default_value = "10000")]
        timeout: u64,
    },
    /// Evaluate RFC 9309 robots.txt policy for one public target URL.
    CrawlPolicy {
        /// Public HTTP(S) target to evaluate.
        url: String,
        /// Explicit crawler product token used for group matching and HTTP identity.
        #[arg(long, default_value = "Plasmate")]
        product_token: String,
        /// Optional path for the versioned JSON policy report.
        #[arg(long, short)]
        output: Option<String>,
        /// Whole robots.txt request deadline in milliseconds (1 to 30000).
        #[arg(long, default_value = "10000")]
        timeout: u64,
    },
    /// Run the real-world coverage suite and write a public scorecard JSON
    Coverage {
        /// File containing URLs (one per line)
        #[arg(long, default_value = "bench/top100.txt")]
        urls: String,
        /// Output JSON file for the scorecard
        #[arg(long, default_value = "website/docs/coverage.json")]
        output: String,
        /// Timeout per URL in milliseconds
        #[arg(long, default_value = "15000")]
        timeout: u64,
        /// Max concurrent pages
        #[arg(long, default_value = "8")]
        concurrency: usize,
        /// Disable JavaScript execution
        #[arg(long)]
        no_js: bool,
        /// Skip fetching external <script src="..."> files (inline only)
        #[arg(long)]
        no_external: bool,
        /// V8 heap limit for JS execution (in MB). Only used when JS is enabled.
        #[arg(long, default_value = "256")]
        js_heap_mb: usize,
        /// Max external scripts fetched per page.
        #[arg(long, default_value = "20")]
        max_external_scripts: usize,
        /// Max bytes per fetched external script (in KB).
        #[arg(long, default_value = "50")]
        max_external_script_kb: usize,
        /// Max total bytes across fetched external scripts per page (in KB).
        #[arg(long, default_value = "1024")]
        max_external_total_kb: usize,
        /// Timeout per external script fetch in milliseconds.
        #[arg(long, default_value = "5000")]
        external_script_timeout_ms: u64,
        /// Address-space limit per isolated JS worker in MB (Linux; 0 disables).
        #[arg(long, default_value = "0")]
        worker_memory_mb: u64,
        /// Maximum stdout/stderr captured from each isolated JS worker in KB.
        #[arg(long, default_value = "256")]
        worker_output_kb: usize,
        /// Max URLs to run from the file
        #[arg(long, default_value = "100")]
        max: usize,
    },
    /// Validate a public coverage scorecard's schema and denominator invariants.
    CoverageValidate {
        /// Coverage JSON produced by `plasmate coverage`.
        #[arg(long, default_value = "website/docs/coverage.json")]
        input: String,
    },
    /// Internal single-page coverage worker. Input/output are JSON over stdio.
    #[command(name = "__coverage-worker", hide = true)]
    CoverageWorker,
    /// Internal process-isolated JavaScript worker. JSON protocol over stdio.
    #[command(name = "__js-worker", hide = true)]
    JsWorker,
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
    /// Capture a screenshot of a web page
    Screenshot {
        /// URL to screenshot
        url: String,
        /// Output file path (defaults to screenshot.png)
        #[arg(long, short, default_value = "screenshot.png")]
        output: String,
        /// Viewport width in pixels
        #[arg(long, default_value = "1280")]
        width: u32,
        /// Viewport height in pixels
        #[arg(long, default_value = "720")]
        height: u32,
        /// Image format: png, jpeg, webp
        #[arg(long, default_value = "png")]
        format: String,
        /// JPEG/WebP quality (1-100)
        #[arg(long)]
        quality: Option<u32>,
        /// Capture the full scrollable page
        #[arg(long)]
        full_page: bool,
    },
    /// Start a persistent daemon for fast repeated fetches
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Compile HTML to SOM without fetching (reads from file or stdin)
    Compile {
        /// HTML file to compile (reads from stdin if omitted)
        #[arg(long, short)]
        file: Option<String>,
        /// Page URL for stable ID generation (no network request is made)
        #[arg(long, default_value = "https://localhost")]
        url: String,
        /// Output file (defaults to stdout)
        #[arg(long, short)]
        output: Option<String>,
        /// Output format: "json" (default, full SOM), "text" (plain extracted
        /// text), or "markdown" (structured Markdown). Same as `plasmate fetch
        /// --format`.
        #[arg(long, default_value = "json")]
        format: String,
        /// Filter output to a specific SOM region, role, action surface, or
        /// element — same syntax as `plasmate fetch --selector` (e.g. `main`,
        /// `button`, `interactive`, `action:click`, `#my-id`).
        #[arg(long)]
        selector: Option<String>,
    },
    /// Start the MCP (Model Context Protocol) server
    Mcp {
        /// Transport: stdio (default) or http (authenticated Streamable HTTP)
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// Streamable HTTP bind host. Only loopback hosts are accepted.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Streamable HTTP port
        #[arg(long, default_value_t = mcp::streamable_http::DEFAULT_PORT)]
        port: u16,
        /// Streamable HTTP bearer capability token (prefer PLASMATE_MCP_HTTP_TOKEN)
        #[arg(long)]
        token: Option<String>,
        /// Exact HTTP Origin allowed to pass request validation (repeatable)
        #[arg(long = "allow-origin")]
        allowed_origins: Vec<String>,
    },
    /// Execute a validated stateful browser plan through a supervised MCP child
    AgentRun {
        /// Versioned JSON workflow plan
        #[arg(long)]
        plan: std::path::PathBuf,
        /// Machine-readable execution report (written atomically)
        #[arg(long, default_value = "agent-workflow-report.json")]
        report: std::path::PathBuf,
        /// Validate and report without spawning a process or making requests
        #[arg(long)]
        dry_run: bool,
        /// Permit separately approved evaluate steps
        #[arg(long)]
        allow_evaluate: bool,
        /// Permit separately approved set_cookies and clear_cookies steps
        #[arg(long)]
        allow_cookie_writes: bool,
        /// Approve one mutating step by exact plan ID (repeat for each step)
        #[arg(long = "confirm-step")]
        confirm_steps: Vec<String>,
    },
    /// Manage authentication profiles for cookie-based browsing
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    /// Compare two SOM snapshots and output a structured diff
    Diff {
        /// Path to the old (baseline) SOM JSON file
        old: String,
        /// Path to the new SOM JSON file
        new: String,
        /// Output format: json (default), text, or summary
        #[arg(long, default_value = "json")]
        format: String,
        /// Skip SomMeta changes (html_bytes, som_bytes)
        #[arg(long)]
        ignore_meta: bool,
        /// Write output to a file instead of stdout
        #[arg(long, short)]
        output: Option<String>,
        /// Filter both snapshots to a specific region before diffing.
        /// Same syntax as `plasmate fetch --selector` (e.g. `main`, `nav`,
        /// `button`, `interactive`, `action:click`, `#my-id`). Useful for
        /// diffing only the content region or action surface and ignoring
        /// navigation or footer churn.
        #[arg(long)]
        selector: Option<String>,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Store cookies for a domain
    Set {
        /// Domain (e.g., x.com, github.com)
        domain: String,
        /// Cookie string: "name1=val1; name2=val2"
        #[arg(long)]
        cookies: Option<String>,
        /// X/Twitter ct0 CSRF token (shorthand for --cookies)
        #[arg(long)]
        ct0: Option<String>,
        /// X/Twitter auth_token (shorthand for --cookies)
        #[arg(long)]
        auth_token: Option<String>,
        /// Cookie expiry TTL in seconds from now
        #[arg(long)]
        expires: Option<i64>,
    },
    /// List stored profiles (domains only, never cookie values)
    List,
    /// Delete a stored profile
    Revoke {
        /// Domain to revoke
        domain: String,
    },
    /// Show profile info (domain, cookie count, fingerprint - never values)
    Info {
        /// Domain to inspect
        domain: String,
        /// Show encryption status only
        #[arg(long)]
        encrypt: bool,
        /// Verify profile can be decrypted
        #[arg(long)]
        decrypt: bool,
    },
    /// Start local HTTP bridge server for extension push
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "9271")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon (keeps browser warm for fast fetches)
    Start {
        /// Port to listen on
        #[arg(long, default_value = "9224")]
        port: u16,
    },
    /// Stop a running daemon
    Stop,
    /// Check daemon status
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    plasmate::process_supervisor::prepare_current_process()?;
    // Configure tracing to write to stderr, not stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch {
            url,
            output,
            format,
            user_agent,
            selector,
            timeout,
            no_external,
            no_js,
            header,
            profile,
            tls,
            plugin: plugin_paths,
        } => {
            if tls.list_tls_options {
                print_tls_options();
                return Ok(());
            }
            // Set global TLS config if any TLS flags were provided
            if let Some(tls_config) = tls.to_tls_config()? {
                info!(tls = %tls_config.summary(), "TLS configuration");
                network::tls::set_global(tls_config);
            }
            let mut plugins = load_plugins(&plugin_paths)?;
            // Parse custom headers
            let extra_headers = parse_header_args(&header);
            cmd_fetch(
                &url,
                output.as_deref(),
                &format,
                user_agent.as_deref(),
                selector.as_deref(),
                timeout,
                !no_external,
                no_js,
                profile.as_deref(),
                &extra_headers,
                plugins.as_mut(),
            )
            .await?;
        }
        Commands::Serve {
            host,
            port,
            protocol,
            profile,
            tls,
            plugin: plugin_paths,
        } => {
            if !network::security::is_loopback_bind_host(&host) {
                return Err(format!(
                    "refusing unauthenticated server exposure on '{host}'; bind to 127.0.0.1, ::1, or localhost"
                )
                .into());
            }
            if tls.list_tls_options {
                print_tls_options();
                return Ok(());
            }
            // Set global TLS config if any TLS flags were provided
            if let Some(tls_config) = tls.to_tls_config()? {
                info!(tls = %tls_config.summary(), "TLS configuration for all sessions");
                network::tls::set_global(tls_config);
            }
            let plugins = load_plugins(&plugin_paths)?;
            let plugins = plugins.map(|pm| Arc::new(tokio::sync::Mutex::new(pm)));

            // Set global auth profiles for all sessions
            if let Some(ref profile_str) = profile {
                let domains: Vec<String> = profile_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !domains.is_empty() {
                    info!(profiles = ?domains, "Loading auth profiles for server sessions");
                    auth::config::set_profiles(domains);
                }
            }

            match protocol.as_str() {
                "awp" => {
                    info!("Starting AWP protocol server");
                    awp::server::start(&host, port, plugins).await?;
                }
                "cdp" => {
                    info!("Starting server with Plasmate's supported CDP subset");
                    info!("  Custom domain: Plasmate.getSom, Plasmate.getStructuredData, Plasmate.getInteractiveElements, Plasmate.getMarkdown");
                    cdp::server::start(&host, port, plugins).await?;
                }
                "both" => {
                    // CDP on main port, AWP on main port + 1
                    let awp_port = port + 1;
                    info!("Starting dual-protocol server");
                    info!("  CDP compatibility endpoint: ws://{}:{}", host, port);
                    info!("  AWP (native):               ws://{}:{}", host, awp_port);
                    let host_awp = host.clone();
                    let awp_plugins = plugins.clone();
                    let awp_handle = tokio::spawn(async move {
                        if let Err(e) = awp::server::start(&host_awp, awp_port, awp_plugins).await {
                            eprintln!("AWP server error: {}", e);
                        }
                    });
                    let cdp_handle = tokio::spawn(async move {
                        if let Err(e) = cdp::server::start(&host, port, plugins).await {
                            eprintln!("CDP server error: {}", e);
                        }
                    });
                    tokio::select! {
                        _ = cdp_handle => {}
                        _ = awp_handle => {}
                    }
                }
                _ => {
                    eprintln!("Unknown protocol: {}. Use: awp, cdp, or both", protocol);
                    std::process::exit(1);
                }
            }
        }
        Commands::Bench {
            urls,
            output,
            timeout,
        } => {
            cmd_bench(&urls, &output, timeout).await?;
        }
        Commands::BenchmarkV1 {
            output,
            js,
            max_cold_ms,
            max_warm_ms,
            worker_timeout_ms,
            worker_memory_mb,
            worker_output_kb,
        } => {
            let options = plasmate::benchmark::v1::BenchmarkOptions {
                js_enabled: js,
                max_cold_ms,
                max_warm_ms,
                worker_timeout_ms,
                worker_memory_bytes: worker_memory_mb.saturating_mul(1024 * 1024),
                worker_output_bytes: worker_output_kb.saturating_mul(1024),
                worker_executable: None,
            };
            let report = plasmate::benchmark::v1::run_deterministic_suite(&options).await?;
            std::fs::write(&output, serde_json::to_string_pretty(&report)?)?;
            println!(
                "Benchmark v1: {}/{} task contracts passed; threshold gate: {}",
                report.summary.tasks_passed,
                report.summary.inputs_total,
                if report.threshold_evaluation.passed {
                    "pass"
                } else {
                    "fail"
                }
            );
            if !report.threshold_evaluation.passed {
                for violation in &report.threshold_evaluation.violations {
                    eprintln!("threshold violation: {violation}");
                }
                std::process::exit(2);
            }
        }
        Commands::BenchmarkWorker => {
            use std::io::Read;
            let mut input = Vec::new();
            std::io::stdin().read_to_end(&mut input)?;
            let request: plasmate::benchmark::v1::BenchmarkWorkerRequest =
                serde_json::from_slice(&input)?;
            let result = plasmate::benchmark::v1::run_worker(request).await?;
            println!("{}", serde_json::to_string(&result)?);
        }
        Commands::AgentTaskBenchmarkV1 { output } => {
            let report = plasmate::benchmark::agent_v1::run_suite(
                &plasmate::benchmark::agent_v1::BenchmarkOptions::default(),
            )
            .await?;
            plasmate::benchmark::agent_v1::write_report(&output, &report)?;
            plasmate::benchmark::agent_v1::validate_evidence(&report)
                .map_err(std::io::Error::other)?;
            println!(
                "Agent task benchmark v1: {}/{} contracts passed; observed workflows: {} succeeded, {} failed, {} crashed, {} timed out; gate: {}",
                report.summary.task_contracts_passed,
                report.summary.tasks_total,
                report.summary.observed_succeeded,
                report.summary.observed_failed,
                report.summary.observed_crash,
                report.summary.observed_timeout,
                if report.gate.passed { "pass" } else { "fail" }
            );
            if !report.gate.passed {
                for violation in &report.gate.violations {
                    eprintln!("agent task benchmark violation: {violation}");
                }
                std::process::exit(2);
            }
        }
        Commands::AgentTaskBenchmarkValidate { input } => {
            let report = plasmate::benchmark::agent_v1::read_report(&input)?;
            plasmate::benchmark::agent_v1::validate_evidence(&report)
                .map_err(std::io::Error::other)?;
            println!(
                "agent task benchmark evidence is valid and gate-passing: {}",
                input.display()
            );
            if !report.gate.passed {
                for violation in &report.gate.violations {
                    eprintln!("agent task benchmark violation: {violation}");
                }
                std::process::exit(2);
            }
        }
        Commands::ReleaseValidate { manifest, output } => {
            let repository_root = std::env::current_dir()?;
            let report = plasmate::release_manifest::validate(&repository_root, &manifest)?;
            let json = serde_json::to_string_pretty(&report)?;
            if let Some(output) = output {
                std::fs::write(output, &json)?;
            } else {
                println!("{json}");
            }
            if !report.valid {
                std::process::exit(2);
            }
        }
        Commands::ArdDiscover {
            url,
            output,
            timeout,
        } => {
            let report = plasmate::ard::discover(&url, timeout).await?;
            let json = serde_json::to_string_pretty(&report)?;
            if let Some(output) = output {
                std::fs::write(output, &json)?;
            } else {
                println!("{json}");
            }
            if report.summary.catalogs_accepted == 0 {
                std::process::exit(2);
            }
        }
        Commands::CrawlPolicy {
            url,
            product_token,
            output,
            timeout,
        } => {
            let report = plasmate::crawl_policy::evaluate(&url, &product_token, timeout).await?;
            let json = serde_json::to_string_pretty(&report)?;
            if json.len() > plasmate::crawl_policy::MAX_SERIALIZED_OUTPUT_BYTES {
                return Err(std::io::Error::other(
                    "crawl-policy CLI envelope exceeded its output safety bound",
                )
                .into());
            }
            if let Some(output) = output {
                std::fs::write(output, &json)?;
            } else {
                println!("{json}");
            }
            if !report.decision.allowed {
                std::process::exit(2);
            }
        }
        Commands::Coverage {
            urls,
            output,
            timeout,
            concurrency,
            no_js,
            no_external,
            js_heap_mb,
            max_external_scripts,
            max_external_script_kb,
            max_external_total_kb,
            external_script_timeout_ms,
            worker_memory_mb,
            worker_output_kb,
            max,
        } => {
            cmd_coverage(
                &urls,
                &output,
                timeout,
                concurrency,
                no_js,
                no_external,
                js_heap_mb,
                max_external_scripts,
                max_external_script_kb,
                max_external_total_kb,
                external_script_timeout_ms,
                worker_memory_mb,
                worker_output_kb,
                max,
            )
            .await?;
        }
        Commands::CoverageValidate { input } => {
            let bytes = std::fs::read(&input)?;
            let report: coverage::runner::CoverageReport = serde_json::from_slice(&bytes)?;
            coverage::runner::validate_evidence(&report).map_err(std::io::Error::other)?;
            println!("coverage evidence is valid: {input}");
        }
        Commands::CoverageWorker => {
            use std::io::Read;
            let mut input = Vec::new();
            std::io::stdin().read_to_end(&mut input)?;
            let request: coverage::runner::CoverageWorkerRequest = serde_json::from_slice(&input)?;
            let result = coverage::runner::run_worker(request).await;
            println!("{}", serde_json::to_string(&result)?);
        }
        Commands::JsWorker => {
            use std::io::Read;
            let mut input = Vec::new();
            std::io::stdin()
                .take((plasmate::js::worker::MAX_WORKER_INPUT_BYTES as u64).saturating_add(1))
                .read_to_end(&mut input)?;
            let response = if input.len() > plasmate::js::worker::MAX_WORKER_INPUT_BYTES {
                plasmate::js::worker::JsWorkerResponse::Error {
                    code: "request_too_large".to_string(),
                    message: "JavaScript worker request exceeded its input bound".to_string(),
                }
            } else {
                match serde_json::from_slice::<plasmate::js::worker::JsWorkerRequest>(&input) {
                    Ok(request) => plasmate::js::worker::run_worker_request(request),
                    Err(error) => plasmate::js::worker::JsWorkerResponse::Error {
                        code: "invalid_request".to_string(),
                        message: error.to_string(),
                    },
                }
            };
            println!("{}", serde_json::to_string(&response)?);
        }
        Commands::ThroughputBench {
            base_url,
            pages,
            concurrency,
        } => {
            cmd_throughput_bench(&base_url, pages, concurrency).await?;
        }
        Commands::Screenshot {
            url,
            output,
            width,
            height,
            format,
            quality,
            full_page,
        } => {
            cmd_screenshot(&url, &output, width, height, &format, quality, full_page).await?;
        }
        Commands::Daemon { action } => match action {
            DaemonAction::Start { port } => {
                daemon::run_daemon(port).await?;
            }
            DaemonAction::Stop => {
                if let Some(port) = daemon::daemon_port() {
                    let client = reqwest::Client::new();
                    let resp = client
                        .post(format!("http://127.0.0.1:{}/shutdown", port))
                        .send()
                        .await;
                    match resp {
                        Ok(_) => eprintln!("Daemon stopped."),
                        Err(e) => eprintln!("Failed to stop daemon: {}", e),
                    }
                } else {
                    eprintln!("No daemon is running.");
                }
            }
            DaemonAction::Status => {
                if let Some(port) = daemon::daemon_port() {
                    let client = reqwest::Client::new();
                    match client
                        .get(format!("http://127.0.0.1:{}/health", port))
                        .send()
                        .await
                    {
                        Ok(resp) => {
                            let body = resp.text().await.unwrap_or_default();
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                                let uptime = json["uptime_seconds"].as_u64().unwrap_or_default();
                                let requests = json["requests_served"].as_u64().unwrap_or_default();
                                let cache = &json["cache"];
                                eprintln!("Daemon running on port {}", port);
                                eprintln!("Uptime: {}s, requests served: {}", uptime, requests);
                                eprintln!(
                                        "Cache: {} entries ({} full, {} selector), hits: {}, misses: {}, stale: {}, evictions: {}",
                                        cache["entries"].as_u64().unwrap_or_default(),
                                        cache["full_entries"].as_u64().unwrap_or_default(),
                                        cache["selector_entries"].as_u64().unwrap_or_default(),
                                        cache["hits"].as_u64().unwrap_or_default(),
                                        cache["misses"].as_u64().unwrap_or_default(),
                                        cache["stale_hits"].as_u64().unwrap_or_default(),
                                        cache["evictions"].as_u64().unwrap_or_default()
                                    );
                                eprintln!(
                                    "Bytes: {} SOM cached, {} HTML avoided",
                                    cache["total_som_bytes"].as_u64().unwrap_or_default(),
                                    cache["total_html_bytes_avoided"]
                                        .as_u64()
                                        .unwrap_or_default()
                                );
                            } else {
                                eprintln!("Daemon running on port {} {}", port, body);
                            }
                        }
                        Err(_) => {
                            eprintln!(
                                "Daemon PID file exists but daemon is not responding on port {}.",
                                port
                            );
                        }
                    }
                } else {
                    eprintln!("No daemon is running.");
                }
            }
        },
        Commands::Compile {
            file,
            url,
            output,
            format,
            selector,
        } => {
            cmd_compile(file, url, output, &format, selector.as_deref())?;
        }
        Commands::Mcp {
            transport,
            host,
            port,
            token,
            allowed_origins,
        } => match transport.as_str() {
            "stdio" => mcp::run_server().await?,
            "http" | "streamable-http" => {
                mcp::run_http_server(mcp::McpHttpConfig {
                    host,
                    port,
                    token,
                    allowed_origins,
                })
                .await?;
            }
            _ => return Err("unsupported MCP transport; expected 'stdio' or 'http'".into()),
        },
        Commands::AgentRun {
            plan,
            report,
            dry_run,
            allow_evaluate,
            allow_cookie_writes,
            confirm_steps,
        } => {
            let options = agent_workflow::WorkflowOptions {
                dry_run,
                allow_evaluate,
                allow_cookie_writes,
                confirm_steps,
            };
            let workflow = agent_workflow::load_and_validate(&plan, &options)?;
            let execution = agent_workflow::execute(workflow, &options)?;
            agent_workflow::write_report(&report, &execution)?;
            println!(
                "workflow={} status={:?} succeeded={}/{} report={}",
                execution.workflow,
                execution.status,
                execution.summary.succeeded,
                execution.summary.total,
                report.display()
            );
            if !execution.succeeded() {
                return Err("agent workflow failed; inspect the redacted report".into());
            }
        }
        Commands::Auth { action } => {
            cmd_auth(action).await?;
        }
        Commands::Diff {
            old,
            new,
            format,
            ignore_meta,
            output,
            selector,
        } => {
            cmd_diff(
                &old,
                &new,
                &format,
                ignore_meta,
                output.as_deref(),
                selector.as_deref(),
            )?;
        }
    }

    Ok(())
}

fn cmd_compile(
    file: Option<String>,
    url: String,
    output: Option<String>,
    format: &str,
    selector: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;

    // Read HTML from file or stdin
    let html = if let Some(path) = file {
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path, e))?
    } else {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed to read stdin: {}", e))?;
        buf
    };

    if html.is_empty() {
        return Err("No HTML input provided. Use --file <path> or pipe HTML to stdin.".into());
    }

    // Compile HTML to SOM
    let compiled = som::compiler::compile(&html, &url)?;

    // Apply selector filter (if requested)
    let filtered;
    let som_to_render = if let Some(sel) = selector {
        filtered = apply_selector(&compiled, sel);
        &filtered
    } else {
        &compiled
    };

    // Render to requested format
    let out = render_som_output(som_to_render, format)?;

    // Write output
    if let Some(out_path) = output {
        std::fs::write(&out_path, &out)?;
        eprintln!(
            "Wrote SOM to {} ({} bytes, {:.1}x compression)",
            out_path,
            compiled.meta.som_bytes,
            compiled.meta.html_bytes as f64 / compiled.meta.som_bytes as f64
        );
    } else {
        println!("{}", out);
    }

    Ok(())
}

fn cmd_diff(
    old_path: &str,
    new_path: &str,
    format: &str,
    ignore_meta: bool,
    output: Option<&str>,
    selector: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let old_json = std::fs::read_to_string(old_path)
        .map_err(|e| format!("Failed to read {}: {}", old_path, e))?;
    let new_json = std::fs::read_to_string(new_path)
        .map_err(|e| format!("Failed to read {}: {}", new_path, e))?;

    let old_som: som::types::Som = serde_json::from_str(&old_json)
        .map_err(|e| format!("Failed to parse {}: {}", old_path, e))?;
    let new_som: som::types::Som = serde_json::from_str(&new_json)
        .map_err(|e| format!("Failed to parse {}: {}", new_path, e))?;

    // Apply selector to both snapshots before diffing (if requested)
    let (effective_old, effective_new) = if let Some(sel) = selector {
        (apply_selector(&old_som, sel), apply_selector(&new_som, sel))
    } else {
        (old_som, new_som)
    };

    let diff = som::diff::diff_soms(&effective_old, &effective_new, ignore_meta);

    let result = match format {
        "text" => som::diff::render_text(&diff),
        "summary" => {
            let mut s = som::diff::render_summary(&diff.summary);
            s.push('\n');
            s
        }
        "json" => serde_json::to_string_pretty(&diff)?,
        other => {
            eprintln!(
                "Error: unknown format '{}'. Use: json, text, or summary",
                other
            );
            std::process::exit(1);
        }
    };

    match output {
        Some(path) => {
            std::fs::write(path, &result)?;
            eprintln!("Diff written to {}", path);
        }
        None => {
            print!("{}", result);
        }
    }

    Ok(())
}

async fn cmd_auth(action: AuthAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        AuthAction::Set {
            domain,
            cookies,
            ct0,
            auth_token,
            expires,
        } => {
            let mut cookie_map = std::collections::HashMap::new();

            // Parse --cookies string with optional TTL
            if let Some(cookie_str) = cookies {
                cookie_map.extend(auth::store::parse_cookie_string_with_ttl(
                    &cookie_str,
                    expires,
                ));
            }

            // X/Twitter shorthand flags
            if let Some(ct0_val) = ct0 {
                let entry = auth::store::CookieEntry::with_expiry(
                    ct0_val,
                    expires.map(|ttl| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64 + ttl)
                            .unwrap_or(0)
                    }),
                );
                cookie_map.insert("ct0".to_string(), entry);
            }
            if let Some(auth_val) = auth_token {
                let entry = auth::store::CookieEntry::with_expiry(
                    auth_val,
                    expires.map(|ttl| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64 + ttl)
                            .unwrap_or(0)
                    }),
                );
                cookie_map.insert("auth_token".to_string(), entry);
            }

            if cookie_map.is_empty() {
                eprintln!("No cookies provided. Use --cookies, --ct0, or --auth-token");
                std::process::exit(1);
            }

            let profile = auth::store::CookieProfile {
                domain: domain.clone(),
                cookies: cookie_map,
                created_at: Some({
                    let dur = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default();
                    format!("{}", dur.as_secs())
                }),
                notes: None,
            };

            auth::store::store_profile(&profile)?;
            let fp = auth::store::profile_fingerprint(&profile);
            eprintln!(
                "✓ Stored {} cookie(s) for {} [{}]",
                profile.cookies.len(),
                domain,
                fp
            );
        }
        AuthAction::List => {
            let profiles = auth::store::list_profiles()?;
            if profiles.is_empty() {
                eprintln!("No stored profiles. Use `plasmate auth set <domain> --cookies ...`");
            } else {
                eprintln!("Stored profiles:");
                for domain in profiles {
                    if let Ok(Some(p)) = auth::store::load_profile(&domain) {
                        let fp = auth::store::profile_fingerprint(&p);
                        // Calculate expiry status
                        let expiry_status = calculate_profile_expiry_status(&p);
                        eprintln!(
                            "  {} ({} cookies) [{}] {}",
                            domain,
                            p.cookies.len(),
                            fp,
                            expiry_status
                        );
                    } else {
                        eprintln!("  {}", domain);
                    }
                }
            }
        }
        AuthAction::Revoke { domain } => {
            if auth::store::revoke_profile(&domain)? {
                eprintln!("✓ Revoked profile for {}", domain);
            } else {
                eprintln!("No profile found for {}", domain);
            }
        }
        AuthAction::Info {
            domain,
            encrypt,
            decrypt,
        } => {
            // Handle encryption status check
            if encrypt || decrypt {
                match auth::store::is_profile_encrypted(&domain)? {
                    Some(is_encrypted) => {
                        if encrypt {
                            eprintln!(
                                "Profile '{}': {}",
                                domain,
                                if is_encrypted {
                                    "encrypted"
                                } else {
                                    "plaintext"
                                }
                            );
                        }
                        if decrypt {
                            // Try to load (decrypt) the profile
                            match auth::store::load_profile(&domain) {
                                Ok(Some(_)) => {
                                    eprintln!("✓ Profile '{}' decrypted successfully", domain);
                                }
                                Ok(None) => {
                                    eprintln!("No profile found for {}", domain);
                                }
                                Err(e) => {
                                    eprintln!("✗ Failed to decrypt profile '{}': {}", domain, e);
                                }
                            }
                        }
                    }
                    None => {
                        eprintln!("No profile found for {}", domain);
                    }
                }
                return Ok(());
            }

            // Regular info display
            match auth::store::load_profile(&domain)? {
                Some(profile) => {
                    let fp = auth::store::profile_fingerprint(&profile);
                    let is_encrypted = auth::store::is_profile_encrypted(&domain)?;
                    eprintln!("Domain:      {}", profile.domain);
                    eprintln!("Cookies:     {}", profile.cookies.len());
                    eprintln!("Fingerprint: {}", fp);
                    if let Some(encrypted) = is_encrypted {
                        eprintln!("Encrypted:   {}", if encrypted { "yes" } else { "no" });
                    }
                    if let Some(ts) = &profile.created_at {
                        eprintln!("Created:     {}", ts);
                    }
                    eprintln!();
                    eprintln!("Cookies:");
                    for (name, entry) in &profile.cookies {
                        let status = auth::store::cookie_expiry_status(entry.expires_at);
                        eprintln!("  {} - {}", name, status);
                    }
                }
                None => {
                    eprintln!("No profile found for {}", domain);
                }
            }
        }
        AuthAction::Serve { port } => {
            eprintln!("Starting auth bridge server on 127.0.0.1:{}", port);
            eprintln!("Endpoints:");
            eprintln!("  GET  /api/status  - Server status and stored profiles");
            eprintln!("  POST /api/cookies - Store cookies from extension");
            eprintln!("  GET  /api/wait    - Long-poll until domain cookies arrive");
            eprintln!();
            auth::bridge::start(port).await?;
        }
    }
    Ok(())
}

/// Calculate overall expiry status for a profile's cookies
fn calculate_profile_expiry_status(profile: &auth::store::CookieProfile) -> &'static str {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut has_expired = false;
    let mut has_expiring_soon = false;

    for entry in profile.cookies.values() {
        if let Some(exp) = entry.expires_at {
            if exp < now {
                has_expired = true;
            } else if exp < now + 86400 {
                has_expiring_soon = true;
            }
        }
    }

    if has_expired {
        "✗ expired"
    } else if has_expiring_soon {
        "⚠ expires soon"
    } else {
        "✓ valid"
    }
}

fn print_tls_options() {
    eprintln!("Available TLS cipher suites (ring provider):");
    eprintln!();
    for suite in network::tls::available_cipher_suites() {
        eprintln!("  {}", suite);
    }
    eprintln!();
    eprintln!("Available key exchange groups:");
    eprintln!();
    for group in network::tls::available_kx_groups() {
        eprintln!("  {}", group);
    }
    eprintln!();
    eprintln!("Usage examples:");
    eprintln!("  plasmate fetch URL --tls-min-version 1.3");
    eprintln!("  plasmate fetch URL --insecure");
    eprintln!(
        "  plasmate fetch URL --tls13-ciphers TLS13_AES_256_GCM_SHA384,TLS13_AES_128_GCM_SHA256"
    );
    eprintln!("  plasmate fetch URL --alpn h2,http/1.1 --tls-groups x25519,secp256r1");
    eprintln!("  plasmate serve --tls-min-version 1.2 --ca-cert /path/to/ca.pem");
}

/// Load Wasm plugins from the given paths. Returns None if no plugins specified.
fn load_plugins(
    paths: &[String],
) -> Result<Option<plugin::PluginManager>, Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Ok(None);
    }
    let mut pm = plugin::PluginManager::new().map_err(|e| e.to_string())?;
    for p in paths {
        let manifest = pm
            .load(std::path::Path::new(p))
            .map_err(|e| e.to_string())?;
        info!(
            name = %manifest.name,
            version = %manifest.version,
            "Loaded plugin"
        );
    }
    Ok(Some(pm))
}

/// Parse `--header "Key: Value"` arguments into a `HashMap`.
fn parse_header_args(args: &[String]) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for arg in args {
        if let Some(pos) = arg.find(':') {
            let key = arg[..pos].trim().to_string();
            let val = arg[pos + 1..].trim().to_string();
            if !key.is_empty() {
                map.insert(key, val);
            }
        } else {
            eprintln!(
                "Warning: ignoring malformed header (expected 'Name: value'): {}",
                arg
            );
        }
    }
    map
}

#[allow(clippy::too_many_arguments)]
async fn cmd_fetch(
    url: &str,
    output: Option<&str>,
    format: &str,
    user_agent: Option<&str>,
    selector: Option<&str>,
    timeout_ms: u64,
    external_scripts: bool,
    no_js: bool,
    profile: Option<&str>,
    extra_headers: &std::collections::HashMap<String, String>,
    mut plugins: Option<&mut plugin::PluginManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the daemon is running and delegate to it
    if plugins.is_none() {
        if let Some(port) = daemon::daemon_port() {
            info!(port, "Delegating to daemon");
            match daemon::daemon_fetch(port, url, no_js, profile, selector).await {
                Ok(som) => {
                    let out = render_som_output(&som, format)?;
                    println!("{}", out);
                    return Ok(());
                }
                Err(e) => {
                    info!(error = %e, "Daemon fetch failed, falling back to direct fetch");
                }
            }
        }
    }

    let jar = Arc::new(reqwest::cookie::Jar::default());

    // Load auth cookies if a profile is specified
    if let Some(domain) = profile {
        if !auth::store::load_into_jar(domain, &jar)? {
            eprintln!(
                "Warning: no auth profile found for '{}', continuing without cookies",
                domain
            );
        }
    }

    let tls_config = network::tls::global();
    let headers_opt = if extra_headers.is_empty() {
        None
    } else {
        Some(extra_headers)
    };
    let client = network::fetch::build_client_h1_fallback_with_headers(
        user_agent,
        jar,
        tls_config,
        headers_opt,
    )?;

    // Plugin hook: pre_navigate
    let effective_url = if let Some(pm) = plugins.as_deref_mut() {
        pm.run_pre_navigate(url).map_err(|e| e.to_string())?
    } else {
        url.to_string()
    };

    info!(url = %effective_url, "Fetching");
    let result = network::fetch::fetch_url(&client, &effective_url, timeout_ms).await?;
    info!(
        url = %result.url,
        status = result.status,
        html_bytes = result.html_bytes,
        load_ms = result.load_ms,
        "Fetched"
    );

    // Process through async JS pipeline (supports external script fetching)
    let pipeline_config = js::pipeline::PipelineConfig {
        execute_js: !no_js,
        fetch_external_scripts: external_scripts && !no_js,
        ..Default::default()
    };

    let page_result = if let Some(pm) = plugins {
        js::pipeline::process_page_async_with_plugins(
            &result.html,
            &result.url,
            &pipeline_config,
            &client,
            pm,
        )
        .await?
    } else {
        js::pipeline::process_page_async(&result.html, &result.url, &pipeline_config, &client)
            .await?
    };

    if let Some(ref report) = page_result.js_report {
        info!(
            scripts = report.total,
            ok = report.succeeded,
            err = report.failed,
            "JS execution"
        );
        if let Some(failure) = &report.containment_failure {
            warn!(
                kind = ?failure.kind,
                code = %failure.code,
                message = %failure.message,
                source_som_fallback = true,
                "JavaScript worker was contained; returning the source SOM fallback"
            );
        }
    }

    info!(
        extract_us = page_result.timing.extract_scripts_us,
        js_us = page_result.timing.js_execution_us,
        som_us = page_result.timing.som_compile_us,
        total_us = page_result.timing.total_us,
        "Pipeline complete"
    );

    let filtered_som;
    let som_to_render = if let Some(sel) = selector {
        filtered_som = apply_selector(&page_result.som, sel);
        &filtered_som
    } else {
        &page_result.som
    };

    let out = render_som_output(som_to_render, format)?;

    match output {
        Some(path) => {
            std::fs::write(path, &out)?;
            info!(path, som_bytes = page_result.som.meta.som_bytes, "Written");
        }
        None => {
            println!("{}", out);
        }
    }

    Ok(())
}

/// Render a SOM to the requested output format.
///
/// - `"json"` (default): pretty-printed SOM JSON.
/// - `"text"`: plain text extracted from all regions — no JSON overhead.
///   Useful for already-minimal pages where the SOM structure would add more
///   tokens than it saves, or for piping into plain-text tools.
/// - `"markdown"`: structured Markdown — headings, paragraphs, links, images,
///   lists and separators are mapped to their Markdown equivalents. Useful for
///   LLM context where light structure helps without full JSON overhead.
/// - `"links"`: one URL per line, deduplicated, order-preserving. Useful for
///   crawlers, sitemaps, and research agents that need to discover outbound links.
fn render_som_output(
    som: &som::types::Som,
    format: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        "text" => {
            let mut parts: Vec<&str> = Vec::new();
            if !som.title.is_empty() {
                parts.push(&som.title);
            }
            for region in &som.regions {
                for el in &region.elements {
                    collect_text(el, &mut parts);
                }
            }
            Ok(parts.join("\n"))
        }
        "markdown" => {
            let mut out = String::new();
            if !som.title.is_empty() {
                out.push_str(&format!("# {}\n\n", som.title));
            }
            for region in &som.regions {
                for el in &region.elements {
                    render_element_markdown(el, &mut out, 0);
                }
            }
            Ok(out)
        }
        "links" => {
            let mut urls: Vec<String> = Vec::new();
            for region in &som.regions {
                for el in &region.elements {
                    collect_links(el, &mut urls);
                }
            }
            // Deduplicate while preserving order
            let mut seen = std::collections::HashSet::new();
            urls.retain(|u| seen.insert(u.clone()));
            Ok(urls.join("\n"))
        }
        _ => Ok(serde_json::to_string_pretty(som)?),
    }
}

/// Recursively collect visible text from a SOM element tree.
fn collect_text<'a>(el: &'a som::types::Element, parts: &mut Vec<&'a str>) {
    if let Some(ref text) = el.text {
        let t = text.trim();
        if !t.is_empty() {
            parts.push(t);
        }
    }
    if let Some(ref children) = el.children {
        for child in children {
            collect_text(child, parts);
        }
    }
    if let Some(ref shadow) = el.shadow {
        for child in &shadow.elements {
            collect_text(child, parts);
        }
    }
}

/// Recursively collect link URLs from a SOM element tree.
fn collect_links(el: &som::types::Element, urls: &mut Vec<String>) {
    if el.role == som::types::ElementRole::Link {
        if let Some(ref attrs) = el.attrs {
            if let Some(href) = attrs.get("href").and_then(|v| v.as_str()) {
                if !href.is_empty() && href != "#" {
                    urls.push(href.to_string());
                }
            }
        }
    }
    if let Some(ref children) = el.children {
        for child in children {
            collect_links(child, urls);
        }
    }
    if let Some(ref shadow) = el.shadow {
        for child in &shadow.elements {
            collect_links(child, urls);
        }
    }
}

/// Recursively render a SOM element to Markdown.
fn render_element_markdown(el: &som::types::Element, out: &mut String, depth: usize) {
    use som::types::ElementRole;

    match el.role {
        ElementRole::Heading => {
            // Map depth to heading level: h2 at depth 0, up to h6
            let hashes = "#".repeat((depth + 2).min(6));
            if let Some(ref t) = el.text {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    out.push_str(&format!("{} {}\n\n", hashes, trimmed));
                }
            }
        }
        ElementRole::Paragraph => {
            if let Some(ref t) = el.text {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    out.push_str(trimmed);
                    out.push_str("\n\n");
                }
            }
        }
        ElementRole::Link => {
            if let Some(ref t) = el.text {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    let href = el
                        .attrs
                        .as_ref()
                        .and_then(|a| a.get("href"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("#");
                    out.push_str(&format!("[{}]({})\n", trimmed, href));
                }
            }
        }
        ElementRole::Button => {
            if let Some(ref t) = el.text {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    out.push_str(&format!("**[{}]**\n", trimmed));
                }
            }
        }
        ElementRole::Image => {
            let alt = el.label.as_deref().or(el.text.as_deref()).unwrap_or("");
            let src = el
                .attrs
                .as_ref()
                .and_then(|a| a.get("src"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            out.push_str(&format!("![{}]({})\n", alt, src));
        }
        ElementRole::List => {
            if let Some(ref children) = el.children {
                for child in children {
                    if let Some(ref t) = child.text {
                        let trimmed = t.trim();
                        if !trimmed.is_empty() {
                            out.push_str(&format!("- {}\n", trimmed));
                        }
                    }
                }
                out.push('\n');
            } else if let Some(ref t) = el.text {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    out.push_str(&format!("- {}\n\n", trimmed));
                }
            }
        }
        ElementRole::Table => {
            // Tables are complex structures; emit their text content for now
            if let Some(ref t) = el.text {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    out.push_str(trimmed);
                    out.push_str("\n\n");
                }
            }
        }
        ElementRole::Separator => {
            out.push_str("---\n\n");
        }
        _ => {
            if let Some(ref t) = el.text {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    out.push_str(trimmed);
                    out.push('\n');
                }
            }
            if let Some(ref children) = el.children {
                for child in children {
                    render_element_markdown(child, out, depth + 1);
                }
            }
        }
    }
}

/// Delegate to `som::filter::apply_selector` (shared with MCP tools).
fn apply_selector(som: &som::types::Som, selector: &str) -> som::types::Som {
    som::filter::apply_selector(som, selector)
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

#[allow(clippy::too_many_arguments)]
async fn cmd_coverage(
    urls_file: &str,
    output: &str,
    timeout_ms: u64,
    concurrency: usize,
    no_js: bool,
    no_external: bool,
    js_heap_mb: usize,
    max_external_scripts: usize,
    max_external_script_kb: usize,
    max_external_total_kb: usize,
    external_script_timeout_ms: u64,
    worker_memory_mb: u64,
    worker_output_kb: usize,
    max_urls: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(urls_file)?;
    let urls = coverage::runner::parse_urls_file(&content);

    let opts = coverage::runner::CoverageOptions {
        timeout_ms,
        concurrency,
        execute_js: !no_js,
        fetch_external_scripts: !no_external,
        js_max_heap_bytes: js_heap_mb.saturating_mul(1024 * 1024),
        max_external_scripts,
        max_external_script_bytes: max_external_script_kb.saturating_mul(1024),
        max_external_total_bytes: max_external_total_kb.saturating_mul(1024),
        external_script_timeout_ms,
        worker_memory_bytes: worker_memory_mb.saturating_mul(1024 * 1024),
        worker_output_bytes: worker_output_kb.saturating_mul(1024),
        max_urls: Some(max_urls),
        ..Default::default()
    };

    info!(count = urls.len(), "Running coverage suite");
    let report = coverage::runner::run(&urls, &opts).await;
    coverage::runner::validate_evidence(&report).map_err(std::io::Error::other)?;

    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(output, json)?;
    info!(output, "Coverage scorecard written");

    let parseable = report
        .summary
        .urls_total
        .saturating_sub(report.summary.blocked);
    let overall_percent = if report.summary.urls_total == 0 {
        0.0
    } else {
        report.summary.ok as f64 / report.summary.urls_total as f64 * 100.0
    };
    println!(
        "Coverage: overall {} / {} ({:.1}%); parseable-site {} / {} ({:.1}%, excludes blocked); blocked {}; failed {}; worker crashes {}; worker resource exhaustions {}; worker exits {}; infrastructure failures {}; median compression {:.1}x",
        report.summary.ok,
        report.summary.urls_total,
        overall_percent,
        report.summary.ok,
        parseable,
        report.summary.parsed_percent,
        report.summary.blocked,
        report.summary.failed,
        report.summary.worker_crashes,
        report.summary.worker_resource_exhaustions,
        report.summary.worker_exits,
        report.summary.infrastructure_failures,
        report.summary.median_ratio
    );

    if report.summary.infrastructure_failures > 0 {
        return Err(std::io::Error::other(format!(
            "coverage completed with {} isolated worker infrastructure failure(s); report was written to {output}",
            report.summary.infrastructure_failures
        ))
        .into());
    }

    Ok(())
}

async fn cmd_screenshot(
    url: &str,
    output: &str,
    width: u32,
    height: u32,
    format: &str,
    quality: Option<u32>,
    full_page: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let opts = screenshot::ScreenshotOptions {
        width,
        height,
        format: screenshot::Format::from_str(format),
        quality,
        full_page,
    };

    let jar = Arc::new(reqwest::cookie::Jar::default());
    let client = network::fetch::build_client_h1_fallback(None, jar, network::tls::global())?;
    let fetched = network::fetch::fetch_url(&client, url, 15_000).await?;

    match screenshot::capture_html(&fetched.html, &fetched.url, &opts) {
        Ok(data) => {
            std::fs::write(output, &data)?;
            eprintln!("✓ Screenshot saved to {} ({} bytes)", output, data.len());
        }
        Err(screenshot::ScreenshotError::ChromeNotFound) => {
            eprintln!("Chrome/Chromium not found.");
            eprintln!();
            eprintln!("Install Google Chrome or Chromium for screenshot support.");
            eprintln!("Screenshots delegate rendering to a headless Chrome subprocess.");
            eprintln!();
            eprintln!("For structured content extraction without Chrome, use:");
            eprintln!("  plasmate fetch {}", url);
            eprintln!();
            eprintln!("This returns the Semantic Object Model (SOM) — a structured,");
            eprintln!("structured representation of the supported page content.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Screenshot failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn cmd_throughput_bench(
    base_url: &str,
    pages: usize,
    concurrency: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    let jar = Arc::new(reqwest::cookie::Jar::default());
    let client = network::fetch::build_client_h1_fallback(None, jar, None)?;

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
    eprintln!(
        "SOM compile time: {}ms ({:.1}us/page)",
        compile_time_us / 1000,
        compile_time_us as f64 / pages as f64
    );
    eprintln!(
        "HTML bytes: {} ({}/page)",
        total_html_bytes,
        total_html_bytes / pages
    );
    eprintln!(
        "SOM bytes: {} ({}/page)",
        total_som_bytes,
        total_som_bytes / pages
    );
    eprintln!(
        "Elements: {} ({}/page)",
        total_elements,
        total_elements / pages
    );
    eprintln!();

    // --- Parallel benchmark ---
    eprintln!("--- Parallel ({} concurrent) ---", concurrency);
    let start = Instant::now();

    let results = network::fetch::fetch_urls_parallel(&client, &urls, 10000, concurrency).await;

    let fetch_elapsed = start.elapsed();
    let mut par_compile_us = 0u128;
    let mut success_count = 0usize;

    for r in results.into_iter().flatten() {
        let compile_start = Instant::now();
        if som::compiler::compile(&r.html, &r.url).is_ok() {
            par_compile_us += compile_start.elapsed().as_micros();
            success_count += 1;
        }
    }

    let par_ms = fetch_elapsed.as_millis();
    let par_per_page = par_ms as f64 / pages as f64;

    eprintln!(
        "Total time: {}ms ({:.1}ms/page effective)",
        par_ms, par_per_page
    );
    eprintln!(
        "SOM compile time: {}ms ({:.1}us/page)",
        par_compile_us / 1000,
        par_compile_us as f64 / success_count as f64
    );
    eprintln!("Successful: {}/{}", success_count, pages);
    eprintln!();

    // --- Memory usage ---
    eprintln!("--- Summary ---");
    eprintln!(
        "Sequential:  {}ms total, {:.1}ms/page",
        seq_ms, seq_per_page
    );
    eprintln!(
        "Parallel:    {}ms total, {:.1}ms/page effective",
        par_ms, par_per_page
    );
    eprintln!("Speedup:     {:.1}x", seq_ms as f64 / par_ms as f64);
    eprintln!();
    eprintln!("Comparison (Lightpanda claims for 100 local pages):");
    eprintln!("  Lightpanda: 2,300ms sequential");
    eprintln!("  Chrome:     25,200ms sequential");
    eprintln!("  Plasmate:   {}ms sequential", seq_ms);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use plasmate::som::types::{
        Element, ElementRole, Region, RegionRole, ShadowRoot, Som, SomMeta,
    };

    #[test]
    fn links_format_includes_shadow_dom_links() {
        let som = Som {
            som_version: "0.1".to_string(),
            url: "https://example.test".to_string(),
            title: "Shadow links".to_string(),
            lang: "en".to_string(),
            regions: vec![Region {
                id: "r_main".to_string(),
                role: RegionRole::Main,
                label: None,
                action: None,
                method: None,
                target: None,
                enctype: None,
                novalidate: None,
                accept_charset: None,
                autocomplete: None,
                elements: vec![Element {
                    id: "e_host".to_string(),
                    role: ElementRole::Section,
                    html_id: None,
                    text: None,
                    label: None,
                    actions: None,
                    attrs: None,
                    children: None,
                    hints: None,
                    shadow: Some(ShadowRoot {
                        mode: "open".to_string(),
                        elements: vec![Element {
                            id: "e_shadow_link".to_string(),
                            role: ElementRole::Link,
                            html_id: None,
                            text: Some("Docs".to_string()),
                            label: None,
                            actions: Some(vec!["click".to_string()]),
                            attrs: Some(serde_json::json!({"href": "/docs"})),
                            children: None,
                            hints: None,
                            shadow: None,
                        }],
                    }),
                }],
            }],
            meta: SomMeta {
                html_bytes: 1,
                som_bytes: 1,
                element_count: 2,
                interactive_count: 1,
            },
            structured_data: None,
        };

        let links = render_som_output(&som, "links").expect("links output should render");

        assert_eq!(links, "/docs");
    }

    #[test]
    fn text_format_includes_nested_and_shadow_content() {
        let som = Som {
            som_version: "0.1".to_string(),
            url: "https://example.test".to_string(),
            title: "Settings".to_string(),
            lang: "en".to_string(),
            regions: vec![Region {
                id: "r_main".to_string(),
                role: RegionRole::Main,
                label: None,
                action: None,
                method: None,
                target: None,
                enctype: None,
                novalidate: None,
                accept_charset: None,
                autocomplete: None,
                elements: vec![Element {
                    id: "e_host".to_string(),
                    role: ElementRole::Section,
                    html_id: None,
                    text: None,
                    label: None,
                    actions: None,
                    attrs: None,
                    children: Some(vec![Element {
                        id: "e_nested".to_string(),
                        role: ElementRole::Paragraph,
                        html_id: None,
                        text: Some("Nested paragraph".to_string()),
                        label: None,
                        actions: None,
                        attrs: None,
                        children: None,
                        hints: None,
                        shadow: None,
                    }]),
                    hints: None,
                    shadow: Some(ShadowRoot {
                        mode: "open".to_string(),
                        elements: vec![Element {
                            id: "e_shadow_text".to_string(),
                            role: ElementRole::Paragraph,
                            html_id: None,
                            text: Some("Shadow copy".to_string()),
                            label: None,
                            actions: None,
                            attrs: None,
                            children: None,
                            hints: None,
                            shadow: None,
                        }],
                    }),
                }],
            }],
            meta: SomMeta {
                html_bytes: 1,
                som_bytes: 1,
                element_count: 3,
                interactive_count: 0,
            },
            structured_data: None,
        };

        let text = render_som_output(&som, "text").expect("text output should render");

        assert_eq!(text, "Settings\nNested paragraph\nShadow copy");
    }
}
