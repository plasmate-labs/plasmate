//! Deterministic, advisory Robots Exclusion Protocol evaluation.
//!
//! This module never changes ordinary page-fetch behavior. It fetches exactly
//! one origin's `/robots.txt` through a public-network-only client and reports
//! what RFC 9309 says for a caller-declared product token.

use std::sync::Arc;

use reqwest::cookie::Jar;
#[cfg(test)]
use reqwest::Client;
use serde::Serialize;
use url::Url;

use crate::network::fetch::{self, FetchError, FetchLimits, FetchResult, PublicOnlyClient};

pub const RESULT_SCHEMA_VERSION: &str = "plasmate.crawl-policy.v1";
pub const SPEC_CHECKED_AT: &str = "2026-07-19";
pub const MAX_ROBOTS_BYTES: usize = 500 * 1024;
pub const MAX_SERIALIZED_OUTPUT_BYTES: usize = 128 * 1024;
const MAX_URL_BYTES: usize = 4096;
const MAX_PRODUCT_TOKEN_BYTES: usize = 64;
const MAX_TIMEOUT_MS: u64 = 30_000;
const MAX_LINE_BYTES: usize = MAX_ROBOTS_BYTES;
const MAX_RULE_DIAGNOSTIC_BYTES: usize = 8 * 1024;
const MAX_RULES: usize = 65_536;
const MAX_ADVISORIES: usize = 32;

#[derive(Debug, Clone, Serialize)]
pub struct CrawlPolicyReport {
    pub schema_version: &'static str,
    pub spec_snapshot: SpecSnapshot,
    pub target_url: String,
    pub product_token: String,
    pub source: SourceReport,
    pub decision: DecisionReport,
    pub parsing: ParsingReport,
    pub advisories: Vec<AdvisoryDirective>,
    pub trust: TrustReport,
    pub limitations: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecSnapshot {
    pub standard: &'static str,
    pub checked_at: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceReport {
    pub requested_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    pub classification: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub content_bytes: usize,
    pub checks_total: usize,
    pub checks_completed: usize,
    pub checks_failed: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionReport {
    pub allowed: bool,
    pub reason: &'static str,
    pub groups_total: usize,
    pub groups_selected: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_specificity_bytes: Option<usize>,
    pub selected_user_agents: Vec<String>,
    pub rules_considered: usize,
    pub rules_matched: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_rule: Option<MatchedRule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedRule {
    pub group_index: usize,
    pub directive: &'static str,
    pub pattern: String,
    pub normalized_pattern: String,
    pub pattern_bytes: usize,
    pub pattern_truncated: bool,
    pub specificity_octets: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ParsingReport {
    pub lines_seen: usize,
    pub lines_parsed: usize,
    pub lines_ignored: usize,
    pub lines_over_limit: usize,
    pub invalid_utf8_replacements: usize,
    pub groups_seen: usize,
    pub rules_seen: usize,
    pub rules_retained: usize,
    pub rules_omitted_at_limit: usize,
    pub empty_disallow_rules_ignored: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdvisoryDirective {
    pub group_index: usize,
    pub name: String,
    pub value: String,
    pub normative_for_permission: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrustReport {
    pub classification: &'static str,
    pub verification: &'static str,
    pub data_handling: &'static str,
}

#[derive(Debug, Clone)]
struct Group {
    index: usize,
    user_agents: Vec<String>,
    rules: Vec<Rule>,
    advisories: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct Rule {
    directive: RuleDirective,
    pattern: String,
    pattern_bytes: usize,
    pattern_truncated: bool,
    normalized: String,
    specificity: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RuleDirective {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Copy)]
enum PolicyClient<'a> {
    Product(&'a PublicOnlyClient),
    #[cfg(test)]
    Fixture(&'a Client),
}

pub async fn evaluate(
    target: &str,
    product_token: &str,
    timeout_ms: u64,
) -> Result<CrawlPolicyReport, String> {
    let token = validate_product_token(product_token)?;
    let client =
        fetch::build_client_public_only_with_user_agent(Arc::new(Jar::default()), Some(&token))
            .map_err(|error| error.to_string())?;
    evaluate_with_client(target, &token, timeout_ms, PolicyClient::Product(&client)).await
}

async fn evaluate_with_client(
    target: &str,
    product_token: &str,
    timeout_ms: u64,
    client: PolicyClient<'_>,
) -> Result<CrawlPolicyReport, String> {
    let target_url = validate_target(target)?;
    let timeout_ms = validate_timeout(timeout_ms)?;
    let robots_url = robots_url(&target_url)?;
    let limits = FetchLimits {
        max_compressed_bytes: MAX_ROBOTS_BYTES,
        max_body_bytes: MAX_ROBOTS_BYTES,
        max_redirects: 5,
    };
    let fetched = match client {
        PolicyClient::Product(client) => {
            fetch::fetch_url_public_only_same_origin_with_limits(
                client,
                robots_url.as_str(),
                timeout_ms,
                limits,
            )
            .await
        }
        #[cfg(test)]
        PolicyClient::Fixture(client) => {
            fetch::fetch_url_for_local_fixture_same_origin_with_limits(
                client,
                robots_url.as_str(),
                timeout_ms,
                limits,
            )
            .await
        }
    };

    let mut report = match fetched {
        Ok(result) => evaluate_document(&target_url, product_token, &robots_url, result),
        Err(error) => evaluate_fetch_failure(&target_url, product_token, &robots_url, error),
    };
    ensure_output_bound(&mut report)?;
    Ok(report)
}

fn evaluate_document(
    target: &Url,
    product_token: &str,
    robots_url: &Url,
    fetched: FetchResult,
) -> CrawlPolicyReport {
    let (groups, parsing) = parse_robots(&fetched.html);
    let (mut decision, advisories) = decide(target, product_token, &groups);
    if parsing.lines_over_limit > 0 {
        decision.allowed = false;
        decision.reason = "robots_invalid_line_limit";
        decision.matched_rule = None;
    }
    base_report(
        target,
        product_token,
        SourceReport {
            requested_url: robots_url.to_string(),
            final_url: Some(fetched.url),
            http_status: Some(fetched.status),
            classification: "available",
            content_type: Some(bound_string(&fetched.content_type, 256)),
            content_bytes: fetched.html_bytes,
            checks_total: 1,
            checks_completed: 1,
            checks_failed: 0,
        },
        decision,
        parsing,
        advisories,
    )
}

fn evaluate_fetch_failure(
    target: &Url,
    product_token: &str,
    robots_url: &Url,
    error: FetchError,
) -> CrawlPolicyReport {
    let (classification, allowed, reason, status, final_url) = match error {
        FetchError::HttpError { status, url } if (400..=499).contains(&status) => (
            "unavailable",
            true,
            "robots_unavailable",
            Some(status),
            Some(url),
        ),
        FetchError::HttpError { status, url } => (
            "unreachable",
            false,
            "robots_unreachable",
            Some(status),
            Some(url),
        ),
        FetchError::BodyTooLarge { .. } => {
            ("invalid_too_large", false, "robots_invalid", None, None)
        }
        FetchError::UrlBlocked(_) => (
            "blocked_redirect_or_destination",
            false,
            "robots_unreachable",
            None,
            None,
        ),
        FetchError::Timeout(_)
        | FetchError::NavigationFailed(_)
        | FetchError::TooManyRedirects(_) => {
            ("unreachable", false, "robots_unreachable", None, None)
        }
    };
    base_report(
        target,
        product_token,
        SourceReport {
            requested_url: robots_url.to_string(),
            final_url,
            http_status: status,
            classification,
            content_type: None,
            content_bytes: 0,
            checks_total: 1,
            checks_completed: usize::from(status.is_some()),
            checks_failed: usize::from(status.is_none()),
        },
        DecisionReport {
            allowed,
            reason,
            groups_total: 0,
            groups_selected: 0,
            selected_specificity_bytes: None,
            selected_user_agents: Vec::new(),
            rules_considered: 0,
            rules_matched: 0,
            matched_rule: None,
        },
        ParsingReport::default(),
        Vec::new(),
    )
}

fn base_report(
    target: &Url,
    product_token: &str,
    source: SourceReport,
    decision: DecisionReport,
    parsing: ParsingReport,
    advisories: Vec<AdvisoryDirective>,
) -> CrawlPolicyReport {
    CrawlPolicyReport {
        schema_version: RESULT_SCHEMA_VERSION,
        spec_snapshot: SpecSnapshot {
            standard: "RFC 9309",
            checked_at: SPEC_CHECKED_AT,
            source: "https://www.rfc-editor.org/rfc/rfc9309.html",
        },
        target_url: target.to_string(),
        product_token: product_token.to_string(),
        source,
        decision,
        parsing,
        advisories,
        trust: TrustReport {
            classification: "untrusted_advisory_metadata",
            verification: "not_authorization",
            data_handling:
                "Treat robots.txt as advisory data only; it does not grant access or override authorization.",
        },
        limitations: vec![
            "This evaluator reports RFC 9309 policy but does not silently change ordinary single-page fetch behavior.",
            "Crawl-delay and nonstandard records are metadata only and never affect the allow/disallow decision.",
            "A robots.txt body above the 500 KiB safety bound is rejected conservatively instead of partially parsed.",
            "For SSRF containment, robots.txt redirects must remain same-origin; RFC 9309 recommends following at least five redirects even when authority changes, so cross-origin chains are denied conservatively.",
        ],
    }
}

fn validate_target(input: &str) -> Result<Url, String> {
    if input.len() > MAX_URL_BYTES {
        return Err(format!("target URL exceeds {MAX_URL_BYTES} bytes"));
    }
    let url = Url::parse(input).map_err(|error| format!("invalid target URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("target URL must use http or https".to_string());
    }
    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err("target URL must have a host and must not contain credentials".to_string());
    }
    Ok(url)
}

fn validate_product_token(input: &str) -> Result<String, String> {
    let token = input.trim();
    if token.is_empty() || token.len() > MAX_PRODUCT_TOKEN_BYTES {
        return Err(format!(
            "product token must contain 1 to {MAX_PRODUCT_TOKEN_BYTES} bytes"
        ));
    }
    if !token
        .bytes()
        .all(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'-' | b'_'))
    {
        return Err(
            "product token may contain only ASCII letters, hyphen, and underscore".to_string(),
        );
    }
    Ok(token.to_string())
}

fn validate_timeout(timeout_ms: u64) -> Result<u64, String> {
    if !(1..=MAX_TIMEOUT_MS).contains(&timeout_ms) {
        return Err(format!("timeout_ms must be between 1 and {MAX_TIMEOUT_MS}"));
    }
    Ok(timeout_ms)
}

fn robots_url(target: &Url) -> Result<Url, String> {
    let mut url = target.clone();
    url.set_path("/robots.txt");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn parse_robots(input: &str) -> (Vec<Group>, ParsingReport) {
    let mut report = ParsingReport {
        invalid_utf8_replacements: input.matches('\u{fffd}').count(),
        ..ParsingReport::default()
    };
    let mut groups = Vec::new();
    let mut current: Option<Group> = None;
    let mut advisories_retained = 0usize;

    for (line_index, raw_line) in input.split('\n').enumerate() {
        report.lines_seen += 1;
        let mut line = raw_line.trim_end_matches('\r');
        if line_index == 0 {
            line = line.strip_prefix('\u{feff}').unwrap_or(line);
        }
        if line.len() > MAX_LINE_BYTES {
            report.lines_over_limit += 1;
            report.lines_ignored += 1;
            continue;
        }
        line = line
            .split_once('#')
            .map_or(line, |(before, _)| before)
            .trim();
        if line.is_empty() {
            report.lines_ignored += 1;
            continue;
        }
        let Some((raw_name, raw_value)) = line.split_once(':') else {
            report.lines_ignored += 1;
            continue;
        };
        let name = raw_name.trim().to_ascii_lowercase();
        let value = raw_value.trim();
        match name.as_str() {
            "user-agent" => {
                let agent = parse_user_agent_value(value);
                if agent.is_empty() {
                    report.lines_ignored += 1;
                    continue;
                }
                let starts_new = current
                    .as_ref()
                    .is_some_and(|group| !group.rules.is_empty());
                if starts_new {
                    groups.push(current.take().expect("current group exists"));
                }
                let index = groups.len();
                current
                    .get_or_insert_with(|| Group {
                        index,
                        user_agents: Vec::new(),
                        rules: Vec::new(),
                        advisories: Vec::new(),
                    })
                    .user_agents
                    .push(bound_string(&agent, 256));
                report.lines_parsed += 1;
            }
            "allow" | "disallow" => {
                let Some(group) = current.as_mut() else {
                    report.lines_ignored += 1;
                    continue;
                };
                report.rules_seen += 1;
                if value.is_empty() {
                    if name == "disallow" {
                        report.empty_disallow_rules_ignored += 1;
                    }
                    report.lines_parsed += 1;
                    continue;
                }
                if report.rules_retained >= MAX_RULES {
                    report.rules_omitted_at_limit += 1;
                    report.lines_ignored += 1;
                    continue;
                }
                let normalized = normalize_path(value, true);
                group.rules.push(Rule {
                    directive: if name == "allow" {
                        RuleDirective::Allow
                    } else {
                        RuleDirective::Disallow
                    },
                    specificity: pattern_specificity(&normalized),
                    pattern: bound_string(value, MAX_RULE_DIAGNOSTIC_BYTES),
                    pattern_bytes: value.len(),
                    pattern_truncated: value.len() > MAX_RULE_DIAGNOSTIC_BYTES,
                    normalized,
                });
                report.rules_retained += 1;
                report.lines_parsed += 1;
            }
            _ => {
                if let Some(group) = current.as_mut() {
                    if advisories_retained < MAX_ADVISORIES {
                        group
                            .advisories
                            .push((bound_string(&name, 128), bound_string(value, 512)));
                        advisories_retained += 1;
                    }
                }
                report.lines_parsed += 1;
            }
        }
    }
    if let Some(group) = current {
        groups.push(group);
    }
    report.groups_seen = groups.len();
    (groups, report)
}

fn decide(
    target: &Url,
    product_token: &str,
    groups: &[Group],
) -> (DecisionReport, Vec<AdvisoryDirective>) {
    if target.path() == "/robots.txt" {
        return (
            DecisionReport {
                allowed: true,
                reason: "robots_txt_implicitly_allowed",
                groups_total: groups.len(),
                groups_selected: 0,
                selected_specificity_bytes: None,
                selected_user_agents: Vec::new(),
                rules_considered: 0,
                rules_matched: 0,
                matched_rule: None,
            },
            Vec::new(),
        );
    }
    let token = product_token.to_ascii_lowercase();
    let exact_groups = groups
        .iter()
        .filter(|group| {
            group
                .user_agents
                .iter()
                .any(|agent| agent.eq_ignore_ascii_case(&token))
        })
        .collect::<Vec<_>>();
    let (selected, best_specificity) = if exact_groups.is_empty() {
        let wildcard_groups = groups
            .iter()
            .filter(|group| group.user_agents.iter().any(|agent| agent == "*"))
            .collect::<Vec<_>>();
        let specificity = if wildcard_groups.is_empty() {
            None
        } else {
            Some(0)
        };
        (wildcard_groups, specificity)
    } else {
        (exact_groups, Some(token.len()))
    };
    let selected_agents = selected
        .iter()
        .flat_map(|group| group.user_agents.iter().cloned())
        .take(64)
        .collect::<Vec<_>>();
    let advisories = selected
        .iter()
        .flat_map(|group| {
            group
                .advisories
                .iter()
                .map(|(name, value)| AdvisoryDirective {
                    group_index: group.index,
                    name: name.clone(),
                    value: value.clone(),
                    normative_for_permission: false,
                })
        })
        .take(MAX_ADVISORIES)
        .collect::<Vec<_>>();
    let target_path = target_path(target);
    let mut matching = Vec::new();
    let mut considered = 0usize;
    for group in &selected {
        for rule in &group.rules {
            considered += 1;
            if pattern_matches(&rule.normalized, &target_path) {
                matching.push((group.index, rule));
            }
        }
    }
    matching.sort_by(|(left_group, left), (right_group, right)| {
        right
            .specificity
            .cmp(&left.specificity)
            .then_with(|| directive_rank(right.directive).cmp(&directive_rank(left.directive)))
            .then_with(|| left_group.cmp(right_group))
            .then_with(|| left.pattern.cmp(&right.pattern))
    });
    let matched_rule = matching.first().map(|(group_index, rule)| MatchedRule {
        group_index: *group_index,
        directive: match rule.directive {
            RuleDirective::Allow => "allow",
            RuleDirective::Disallow => "disallow",
        },
        pattern: rule.pattern.clone(),
        normalized_pattern: bound_string(&rule.normalized, MAX_RULE_DIAGNOSTIC_BYTES),
        pattern_bytes: rule.pattern_bytes,
        pattern_truncated: rule.pattern_truncated
            || rule.normalized.len() > MAX_RULE_DIAGNOSTIC_BYTES,
        specificity_octets: rule.specificity,
    });
    let allowed = match &matched_rule {
        None => true,
        Some(rule) => rule.directive == "allow",
    };
    let reason = if matched_rule.is_none() {
        "no_matching_rule"
    } else if allowed {
        "allow_rule"
    } else {
        "disallow_rule"
    };
    (
        DecisionReport {
            allowed,
            reason,
            groups_total: groups.len(),
            groups_selected: selected.len(),
            selected_specificity_bytes: best_specificity,
            selected_user_agents: selected_agents,
            rules_considered: considered,
            rules_matched: matching.len(),
            matched_rule,
        },
        advisories,
    )
}

fn directive_rank(directive: RuleDirective) -> usize {
    usize::from(directive == RuleDirective::Allow)
}

fn parse_user_agent_value(value: &str) -> String {
    let raw = value.trim();
    if raw == "*" {
        return raw.to_string();
    }
    if !raw.is_empty()
        && raw
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'-' | b'_'))
    {
        raw.to_string()
    } else {
        String::new()
    }
}

fn target_path(url: &Url) -> String {
    // RFC 9309 evaluates rules against the URI path. The query is not part
    // of the path and must not make an otherwise allowed resource match a
    // disallow rule.
    normalize_path(url.path(), false)
}

fn normalize_path(input: &str, preserve_pattern_tokens: bool) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                let decoded = high * 16 + low;
                if is_unreserved(decoded) {
                    output.push(decoded as char);
                } else {
                    output.push('%');
                    output.push(hex_upper(decoded >> 4));
                    output.push(hex_upper(decoded & 0x0f));
                }
                index += 3;
                continue;
            }
        }
        if byte.is_ascii() {
            if byte == b'%' {
                output.push_str("%25");
            } else if preserve_pattern_tokens && matches!(byte, b'*' | b'$') {
                output.push(byte as char);
            } else if !preserve_pattern_tokens && matches!(byte, b'*' | b'$') {
                output.push('%');
                output.push(hex_upper(byte >> 4));
                output.push(hex_upper(byte & 0x0f));
            } else {
                output.push(byte as char);
            }
        } else {
            output.push('%');
            output.push(hex_upper(byte >> 4));
            output.push(hex_upper(byte & 0x0f));
        }
        index += 1;
    }
    output
}

fn pattern_specificity(pattern: &str) -> usize {
    let bytes = pattern.as_bytes();
    let end_anchor = bytes.last() == Some(&b'$');
    let mut index = 0usize;
    let mut count = 0usize;
    while index < bytes.len() - usize::from(end_anchor) {
        if bytes[index] == b'*' {
            index += 1;
        } else if bytes[index] == b'%' && index + 2 < bytes.len() {
            count += 1;
            index += 3;
        } else {
            count += 1;
            index += 1;
        }
    }
    count
}

fn pattern_matches(pattern: &str, target: &str) -> bool {
    let mut pattern = pattern.as_bytes();
    let anchored_end = pattern.last() == Some(&b'$');
    if anchored_end {
        pattern = &pattern[..pattern.len() - 1];
    }
    let target = target.as_bytes();
    let mut pattern_index = 0usize;
    let mut target_index = 0usize;
    let mut last_star = None;
    let mut star_target_index = 0usize;
    while target_index < target.len() {
        if pattern_index == pattern.len() {
            return !anchored_end;
        }
        if pattern[pattern_index] == target[target_index] {
            pattern_index += 1;
            target_index += 1;
        } else if pattern[pattern_index] == b'*' {
            last_star = Some(pattern_index);
            pattern_index += 1;
            star_target_index = target_index;
        } else if let Some(star) = last_star {
            star_target_index += 1;
            target_index = star_target_index;
            pattern_index = star + 1;
        } else {
            return false;
        }
    }
    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn hex_upper(value: u8) -> char {
    b"0123456789ABCDEF"[value as usize] as char
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

fn bound_string(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }
    let mut end = max_bytes.min(input.len());
    while !input.is_char_boundary(end) {
        end -= 1;
    }
    input[..end].to_string()
}

fn ensure_output_bound(report: &mut CrawlPolicyReport) -> Result<(), String> {
    enforce_serialized_output_limit(report, |candidate| {
        serde_json::to_vec(candidate)
            .map(|bytes| bytes.len())
            .map_err(|error| error.to_string())
    })
}

/// Reduce optional diagnostic detail until the caller's complete serialized
/// envelope fits. Callers can measure plain CLI JSON or an adapted MCP result.
pub fn enforce_serialized_output_limit<F>(
    report: &mut CrawlPolicyReport,
    measure: F,
) -> Result<(), String>
where
    F: Fn(&CrawlPolicyReport) -> Result<usize, String>,
{
    while measure(report)? > MAX_SERIALIZED_OUTPUT_BYTES {
        if report.advisories.pop().is_some() {
            continue;
        }
        if report.decision.selected_user_agents.pop().is_some() {
            continue;
        }
        return Err("crawl-policy report exceeds its serialized output bound".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn decision(robots: &str, target: &str, token: &str) -> DecisionReport {
        let (groups, _) = parse_robots(robots);
        decide(&Url::parse(target).unwrap(), token, &groups).0
    }

    #[test]
    fn exact_groups_merge_wildcards_are_ignored_and_allow_wins_equal_rule_tie() {
        let robots = r#"
User-agent: *
Disallow: /
User-agent: plasmate
Disallow: /private
User-agent: PLASMATE
Allow: /private$
User-agent: Mate
Disallow: /substring-must-not-match
"#;
        let exact = decision(robots, "https://example.com/private", "Plasmate");
        assert!(exact.allowed);
        assert_eq!(exact.groups_selected, 2);
        assert_eq!(exact.rules_considered, 2);
        assert_eq!(exact.matched_rule.unwrap().directive, "allow");
        let child = decision(robots, "https://example.com/private/child", "Plasmate");
        assert!(!child.allowed);
        assert_eq!(child.matched_rule.unwrap().directive, "disallow");
        assert!(
            decision(
                robots,
                "https://example.com/substring-must-not-match",
                "Plasmate"
            )
            .allowed
        );
    }

    #[test]
    fn wildcard_groups_merge_only_when_no_exact_group_exists() {
        let robots = r#"
User-agent: *
Disallow: /shared
User-agent: *
Allow: /shared$
User-agent: Other
Disallow: /
"#;
        let wildcard = decision(robots, "https://example.com/shared", "Plasmate");
        assert!(wildcard.allowed);
        assert_eq!(wildcard.groups_selected, 2);
        assert_eq!(wildcard.rules_considered, 2);
        let exact = decision(robots, "https://example.com/anything", "Other");
        assert!(!exact.allowed);
        assert_eq!(exact.groups_selected, 1);
        assert_eq!(exact.rules_considered, 1);
    }

    #[test]
    fn malformed_user_agent_is_ignored_and_cannot_override_wildcard_policy() {
        let robots = r#"
User-agent: *
Disallow: /guarded
User-agent: Plasmate/1.0
User-agent: Plasmate extra
User-agent: Other
Allow: /guarded
"#;
        let result = decision(robots, "https://example.com/guarded", "Plasmate");
        assert!(!result.allowed);
        assert_eq!(result.groups_selected, 1);
        assert_eq!(result.rules_considered, 1);
    }

    #[test]
    fn malformed_user_agent_does_not_sever_the_current_valid_group() {
        let robots = r#"
User-agent: *
Allow: /
User-agent: Bad/1.0
Disallow: /private
"#;
        let result = decision(robots, "https://example.com/private", "Plasmate");
        assert!(!result.allowed);
        assert_eq!(result.groups_selected, 1);
        assert_eq!(result.rules_considered, 2);
        assert_eq!(result.matched_rule.unwrap().directive, "disallow");
    }

    #[test]
    fn query_string_is_not_used_for_robots_path_matching() {
        let robots = "User-agent: Plasmate\nDisallow: /blocked\n";

        assert!(decision(
            robots,
            "https://example.com/search?q=/blocked",
            "Plasmate"
        )
        .allowed);
        assert!(!decision(
            robots,
            "https://example.com/blocked?q=allowed",
            "Plasmate"
        )
        .allowed);
    }

    #[test]
    fn valid_rule_longer_than_eight_kib_is_matched_but_diagnostic_is_bounded() {
        let pattern = format!("{}/blocked", "*".repeat(9 * 1024));
        let robots = format!("User-agent: Plasmate\nDisallow: {pattern}\n");
        let (groups, parsing) = parse_robots(&robots);
        assert_eq!(parsing.lines_over_limit, 0);
        assert_eq!(parsing.rules_retained, 1);
        let (result, _) = decide(
            &Url::parse("https://example.com/blocked").unwrap(),
            "Plasmate",
            &groups,
        );
        assert!(!result.allowed);
        let matched = result.matched_rule.unwrap();
        assert!(matched.pattern_bytes > 8 * 1024);
        assert!(matched.pattern_truncated);
        assert!(matched.pattern.len() <= MAX_RULE_DIAGNOSTIC_BYTES);
        assert!(matched.normalized_pattern.len() <= MAX_RULE_DIAGNOSTIC_BYTES);
        assert!(serde_json::to_vec(&matched).unwrap().len() < 64 * 1024);
    }

    #[test]
    fn wildcard_anchor_and_percent_encoding_follow_octet_semantics() {
        let robots = r#"
User-agent: Plasmate
Disallow: /caf%C3%A9/*?key=value$
Allow: /caf%C3%A9/public
Disallow: /literal-%2A
"#;
        assert!(
            !decision(
                robots,
                "https://example.com/caf%C3%A9/x?key=value",
                "Plasmate"
            )
            .allowed
        );
        assert!(
            decision(
                robots,
                "https://example.com/caf%C3%A9/x?key=value2",
                "Plasmate"
            )
            .allowed
        );
        assert!(!decision(robots, "https://example.com/literal-%2A", "Plasmate").allowed);
        assert!(!decision(robots, "https://example.com/literal-*", "Plasmate").allowed);
        assert!(decision(robots, "https://example.com/literal-anything", "Plasmate").allowed);
    }

    #[tokio::test]
    async fn fixture_fetches_only_root_robots_and_evaluates_the_declared_identity() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0u8; 4096];
            let read = stream.read(&mut request).await.unwrap();
            request.truncate(read);
            let body = b"User-agent: Plasmate\nDisallow: /private\n";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                String::from_utf8_lossy(body)
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            String::from_utf8_lossy(&request).into_owned()
        });
        let client = Client::builder()
            .user_agent("Plasmate")
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let target = format!("http://{address}/private");
        let report =
            evaluate_with_client(&target, "Plasmate", 2_000, PolicyClient::Fixture(&client))
                .await
                .unwrap();
        assert!(!report.decision.allowed);
        assert_eq!(report.source.classification, "available");
        assert_eq!(report.source.checks_total, 1);
        let request = server.await.unwrap().to_ascii_lowercase();
        assert!(request.starts_with("get /robots.txt http/1.1"));
        assert!(request.contains("user-agent: plasmate"));
    }

    #[test]
    fn bom_comments_empty_disallow_unknown_records_and_line_limits_are_safe() {
        let oversized = "x".repeat(MAX_LINE_BYTES + 1);
        let robots = format!(
            "\u{feff}User-Agent: Alpha # comment\nSitemap: https://example/x\nUser-agent: Beta\nDisallow:\nCrawl-delay: 5\nDisallow: /no\n{oversized}\n"
        );
        let (groups, parsing) = parse_robots(&robots);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].user_agents, vec!["Alpha", "Beta"]);
        assert_eq!(parsing.empty_disallow_rules_ignored, 1);
        assert_eq!(parsing.lines_over_limit, 1);
        let (report, advisories) = decide(
            &Url::parse("https://example.com/no").unwrap(),
            "Beta",
            &groups,
        );
        assert!(!report.allowed);
        assert_eq!(advisories[0].name, "sitemap");
        assert!(!advisories[0].normative_for_permission);
    }

    #[test]
    fn missing_is_unavailable_but_server_and_network_failures_are_unreachable() {
        let target = Url::parse("https://example.com/a").unwrap();
        let robots = Url::parse("https://example.com/robots.txt").unwrap();
        let missing = evaluate_fetch_failure(
            &target,
            "Plasmate",
            &robots,
            FetchError::HttpError {
                status: 404,
                url: robots.to_string(),
            },
        );
        assert!(missing.decision.allowed);
        assert_eq!(missing.source.classification, "unavailable");
        let server = evaluate_fetch_failure(
            &target,
            "Plasmate",
            &robots,
            FetchError::HttpError {
                status: 503,
                url: robots.to_string(),
            },
        );
        assert!(!server.decision.allowed);
        assert_eq!(server.source.classification, "unreachable");
    }

    #[test]
    fn product_token_and_urls_are_strictly_bounded() {
        assert!(validate_product_token("Plasmate").is_ok());
        assert!(validate_product_token("bad token").is_err());
        assert!(validate_product_token("../bad").is_err());
        assert!(validate_product_token("Plasmate1").is_err());
        assert!(validate_product_token("Plasmate.Bot").is_err());
        assert!(validate_target("file:///etc/passwd").is_err());
        assert!(validate_target("https://user:pass@example.com").is_err());
    }

    #[test]
    fn output_is_bounded_even_with_escape_heavy_advisories() {
        let mut report = base_report(
            &Url::parse("https://example.com/").unwrap(),
            "Plasmate",
            SourceReport {
                requested_url: "https://example.com/robots.txt".to_string(),
                final_url: None,
                http_status: None,
                classification: "available",
                content_type: None,
                content_bytes: 0,
                checks_total: 1,
                checks_completed: 1,
                checks_failed: 0,
            },
            DecisionReport {
                allowed: true,
                reason: "no_matching_rule",
                groups_total: 1,
                groups_selected: 1,
                selected_specificity_bytes: Some(8),
                selected_user_agents: vec!["\\\"".repeat(128); 64],
                rules_considered: 0,
                rules_matched: 0,
                matched_rule: None,
            },
            ParsingReport::default(),
            (0..32)
                .map(|index| AdvisoryDirective {
                    group_index: index,
                    name: "crawl-delay".to_string(),
                    value: "\\\"".repeat(256),
                    normative_for_permission: false,
                })
                .collect(),
        );
        ensure_output_bound(&mut report).unwrap();
        assert!(serde_json::to_vec(&report).unwrap().len() <= MAX_SERIALIZED_OUTPUT_BYTES);
    }
}
