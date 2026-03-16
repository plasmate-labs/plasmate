//! Fetch external <script src="..."> files for execution.
//!
//! Resolves relative URLs against the page URL, fetches in parallel,
//! and returns script sources in document order for execution.

use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

use super::extract::ScriptBlock;

/// A script ready for execution (either inline or fetched external).
#[derive(Debug, Clone)]
pub struct ResolvedScript {
    pub source: String,
    pub label: String,
    pub index: usize,
}

/// Fetch external scripts and merge with inline scripts in document order.
///
/// Limits:
/// - Max 20 external scripts fetched (common pages have 5-15)
/// - 5 second timeout per script
/// - 50KB max per script (skip huge bundles - they're usually framework code)
/// - Only .js files (skip .mjs, .tsx, etc. that need transpilation)
pub async fn resolve_scripts(
    scripts: &[ScriptBlock],
    page_url: &str,
    client: &Client,
) -> Vec<ResolvedScript> {
    let base_url = Url::parse(page_url).ok();
    let mut resolved = Vec::new();
    let mut fetch_count = 0;
    const MAX_EXTERNAL: usize = 20;
    const MAX_SCRIPT_BYTES: usize = 50_000;
    const FETCH_TIMEOUT_MS: u64 = 5000;

    // Collect external scripts that need fetching
    let mut to_fetch: Vec<(usize, String)> = Vec::new();

    for script in scripts {
        if script.is_inline {
            resolved.push(ResolvedScript {
                source: script.source.clone(),
                label: script.label.clone(),
                index: script.index,
            });
        } else if fetch_count < MAX_EXTERNAL {
            // Resolve URL
            let url = if script.label.starts_with("http://") || script.label.starts_with("https://") {
                script.label.clone()
            } else if let Some(ref base) = base_url {
                base.join(&script.label)
                    .map(|u| u.to_string())
                    .unwrap_or_default()
            } else {
                continue;
            };

            if url.is_empty() {
                continue;
            }

            // Skip module scripts, TypeScript, etc.
            let lower = url.to_lowercase();
            if lower.contains(".mjs") || lower.contains(".tsx") || lower.contains(".ts?") {
                debug!(url, "Skipping non-JS script");
                continue;
            }

            to_fetch.push((script.index, url));
            fetch_count += 1;
        }
    }

    // Fetch external scripts concurrently
    if !to_fetch.is_empty() {
        let fetches = to_fetch.iter().map(|(idx, url)| {
            let client = client.clone();
            let url = url.clone();
            let idx = *idx;
            async move {
                let result = tokio::time::timeout(
                    Duration::from_millis(FETCH_TIMEOUT_MS),
                    client.get(&url)
                        .header("Accept", "application/javascript, text/javascript, */*")
                        .send(),
                )
                .await;

                match result {
                    Ok(Ok(resp)) if resp.status().is_success() => {
                        match resp.text().await {
                            Ok(text) if text.len() <= MAX_SCRIPT_BYTES => {
                                debug!(url, bytes = text.len(), "Fetched external script");
                                Some(ResolvedScript {
                                    source: text,
                                    label: url,
                                    index: idx,
                                })
                            }
                            Ok(text) => {
                                debug!(url, bytes = text.len(), "Script too large, skipping");
                                None
                            }
                            Err(e) => {
                                debug!(url, error = %e, "Failed to read script body");
                                None
                            }
                        }
                    }
                    Ok(Ok(resp)) => {
                        debug!(url, status = resp.status().as_u16(), "Script fetch failed");
                        None
                    }
                    Ok(Err(e)) => {
                        debug!(url, error = %e, "Script fetch error");
                        None
                    }
                    Err(_) => {
                        debug!(url, "Script fetch timeout");
                        None
                    }
                }
            }
        });

        let results: Vec<Option<ResolvedScript>> =
            futures_util::future::join_all(fetches).await;

        for result in results.into_iter().flatten() {
            resolved.push(result);
        }
    }

    // Sort by document order
    resolved.sort_by_key(|s| s.index);
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_scripts_pass_through() {
        let scripts = vec![
            ScriptBlock {
                source: "var x = 1;".to_string(),
                label: "inline-0".to_string(),
                is_inline: true,
                index: 0,
            },
            ScriptBlock {
                source: "var y = 2;".to_string(),
                label: "inline-1".to_string(),
                is_inline: true,
                index: 1,
            },
        ];

        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = Client::new();
        let resolved = rt.block_on(resolve_scripts(&scripts, "https://example.com", &client));

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].source, "var x = 1;");
        assert_eq!(resolved[1].source, "var y = 2;");
    }

    #[test]
    fn test_document_order_preserved() {
        let scripts = vec![
            ScriptBlock {
                source: "first();".to_string(),
                label: "inline-0".to_string(),
                is_inline: true,
                index: 0,
            },
            ScriptBlock {
                source: String::new(),
                label: "https://httpbin.org/html".to_string(), // Will fail but that's OK
                is_inline: false,
                index: 1,
            },
            ScriptBlock {
                source: "third();".to_string(),
                label: "inline-2".to_string(),
                is_inline: true,
                index: 2,
            },
        ];

        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = Client::new();
        let resolved = rt.block_on(resolve_scripts(&scripts, "https://example.com", &client));

        // At minimum, inline scripts should be in order
        let inline: Vec<_> = resolved.iter().filter(|s| s.label.starts_with("inline")).collect();
        assert_eq!(inline.len(), 2);
        assert!(inline[0].index < inline[1].index);
    }
}
