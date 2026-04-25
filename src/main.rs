#![allow(dead_code, unused_imports, unused_variables, unused_assignments)]
use clap::{Args, Parser, Subcommand};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod mcp;

use plasmate::auth;
use plasmate::awp;
use plasmate::bench;
use plasmate::cdp;
use plasmate::coverage;
use plasmate::daemon;
use plasmate::js;
use plasmate::network;
use plasmate::plugin;
use plasmate::screenshot;
use plasmate::session;
use plasmate::som;

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
        /// footer, form, dialog, content) or an HTML id selector (#my-id).
        /// When a role is given, only regions of that role are included.
        /// When an id is given, only elements whose html_id matches are kept.
        /// Unrecognised selectors fall through gracefully (full SOM returned).
        ///
        /// Examples:
        ///   --selector main            (just the main content region)
        ///   --selector nav             (navigation links only)
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
        /// Protocol: awp (default), cdp (Puppeteer/Playwright compatible), or both
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
        /// Max URLs to run from the file
        #[arg(long, default_value = "100")]
        max: usize,
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
        /// Filter output to a specific SOM region or element — same syntax as
        /// `plasmate fetch --selector` (e.g. `main`, `nav`, `#my-id`).
        #[arg(long)]
        selector: Option<String>,
    },
    /// Start the MCP (Model Context Protocol) server over stdio
    Mcp,
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
        /// `#my-id`). Useful for diffing only the content region and ignoring
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
                    info!("Starting CDP-compatible server (Puppeteer/Playwright ready)");
                    info!("  Custom domain: Plasmate.getSom, Plasmate.getStructuredData, Plasmate.getInteractiveElements, Plasmate.getMarkdown");
                    cdp::server::start(&host, port, plugins).await?;
                }
                "both" => {
                    // CDP on main port, AWP on main port + 1
                    let awp_port = port + 1;
                    info!("Starting dual-protocol server");
                    info!("  CDP (Puppeteer/Playwright): ws://{}:{}", host, port);
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
                max,
            )
            .await?;
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
            cmd_screenshot(&url, &output, width, height, &format, quality, full_page)?;
        }
        Commands::Daemon { action } => {
            match action {
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
                                eprintln!("Daemon running on port {} {}", port, body);
                            }
                            Err(_) => {
                                eprintln!("Daemon PID file exists but daemon is not responding on port {}.", port);
                            }
                        }
                    } else {
                        eprintln!("No daemon is running.");
                    }
                }
            }
        }
        Commands::Compile {
            file,
            url,
            output,
            format,
            selector,
        } => {
            cmd_compile(file, url, output, &format, selector.as_deref())?;
        }
        Commands::Mcp => {
            mcp::run_server().await?;
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
            eprintln!("Warning: ignoring malformed header (expected 'Name: value'): {}", arg);
        }
    }
    map
}

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
            match daemon::daemon_fetch(port, url, no_js, profile).await {
                Ok(som) => {
                    let effective_som = if let Some(sel) = selector {
                        apply_selector(&som, sel)
                    } else {
                        som
                    };
                    let out = render_som_output(&effective_som, format)?;
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
    let headers_opt = if extra_headers.is_empty() { None } else { Some(extra_headers) };
    let client = network::fetch::build_client_h1_fallback_with_headers(user_agent, jar, tls_config, headers_opt)?;

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
                    if let Some(ref text) = el.text {
                        let t = text.trim();
                        if !t.is_empty() {
                            parts.push(t);
                        }
                    }
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
        "json" | _ => Ok(serde_json::to_string_pretty(som)?),
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
    max_urls: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(urls_file)?;
    let urls = coverage::runner::parse_urls_file(&content);

    let mut opts = coverage::runner::CoverageOptions::default();
    opts.timeout_ms = timeout_ms;
    opts.concurrency = concurrency;
    opts.execute_js = !no_js;
    opts.fetch_external_scripts = !no_external;

    opts.js_max_heap_bytes = js_heap_mb.saturating_mul(1024 * 1024);

    opts.max_external_scripts = max_external_scripts;
    opts.max_external_script_bytes = max_external_script_kb.saturating_mul(1024);
    opts.max_external_total_bytes = max_external_total_kb.saturating_mul(1024);
    opts.external_script_timeout_ms = external_script_timeout_ms;

    opts.max_urls = Some(max_urls);

    info!(count = urls.len(), "Running coverage suite");
    let report = coverage::runner::run(&urls, &opts).await;

    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(output, json)?;
    info!(output, "Coverage scorecard written");

    println!(
        "Coverage: ok {} / {} ({:.1}%), blocked {}, failed {}, median compression {:.1}x",
        report.summary.ok,
        report.summary.urls_total,
        report.summary.parsed_percent,
        report.summary.blocked,
        report.summary.failed,
        report.summary.median_ratio
    );

    Ok(())
}

fn cmd_screenshot(
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

    match screenshot::capture_url(url, &opts) {
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
            eprintln!("token-efficient representation of the page content.");
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
