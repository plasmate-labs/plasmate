//! CDP cookie jar implementation.
//!
//! Provides a cookie store that the CDP layer can read/write directly,
//! supporting Network.getCookies, Network.setCookies, and related commands.
//! Also syncs with the reqwest cookie jar for actual HTTP requests.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::cookie::Jar;
use serde::{Deserialize, Serialize};
use url::Url;

/// A single cookie with all CDP-relevant attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    /// Expiration time as Unix timestamp (seconds). None = session cookie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<f64>,
    #[serde(rename = "httpOnly")]
    pub http_only: bool,
    pub secure: bool,
    #[serde(rename = "sameSite")]
    pub same_site: SameSite,
    /// Size in bytes (name.len() + value.len())
    pub size: usize,
}

/// SameSite cookie attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SameSite {
    #[serde(rename = "Strict")]
    Strict,
    #[serde(rename = "Lax")]
    #[default]
    Lax,
    #[serde(rename = "None")]
    None,
}

impl SameSite {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "strict" => SameSite::Strict,
            "none" => SameSite::None,
            _ => SameSite::Lax,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        }
    }
}

/// Cookie jar for CDP that maintains cookies and syncs with reqwest.
#[derive(Debug)]
pub struct CookieJar {
    /// Cookies indexed by (domain, path, name) for fast lookup and deduplication.
    cookies: HashMap<(String, String, String), Cookie>,
    /// Reference to the reqwest jar for syncing (HTTP requests use this).
    reqwest_jar: Arc<Jar>,
}

impl CookieJar {
    /// Create a new empty cookie jar with a reference to the reqwest jar.
    pub fn new(reqwest_jar: Arc<Jar>) -> Self {
        CookieJar {
            cookies: HashMap::new(),
            reqwest_jar,
        }
    }

    /// Get all cookies that match a URL (respecting domain, path, secure, expiry).
    pub fn get_cookies(&self, url: &str) -> Vec<Cookie> {
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return vec![],
        };

        let host = parsed.host_str().unwrap_or("");
        let path = parsed.path();
        let is_secure = parsed.scheme() == "https";
        let now = current_timestamp();

        self.cookies
            .values()
            .filter(|cookie| {
                // Check expiration
                if let Some(expires) = cookie.expires {
                    if expires <= now {
                        return false;
                    }
                }

                // Check secure flag
                if cookie.secure && !is_secure {
                    return false;
                }

                // Check domain match
                if !domain_matches(host, &cookie.domain) {
                    return false;
                }

                // Check path match
                if !path_matches(path, &cookie.path) {
                    return false;
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Get all cookies in the jar (for Network.getAllCookies).
    pub fn get_all_cookies(&self) -> Vec<Cookie> {
        let now = current_timestamp();
        self.cookies
            .values()
            .filter(|cookie| {
                // Filter out expired cookies
                if let Some(expires) = cookie.expires {
                    if expires <= now {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Set multiple cookies at once.
    pub fn set_cookies(&mut self, cookies: Vec<Cookie>) {
        for cookie in cookies {
            self.set_cookie(cookie);
        }
    }

    /// Set a single cookie, replacing any existing cookie with the same key.
    pub fn set_cookie(&mut self, cookie: Cookie) {
        let key = (
            cookie.domain.clone(),
            cookie.path.clone(),
            cookie.name.clone(),
        );

        // Sync to reqwest jar for actual HTTP requests
        self.sync_to_reqwest(&cookie);

        self.cookies.insert(key, cookie);
    }

    /// Remove a specific cookie by name, domain, and path.
    pub fn remove_cookie(&mut self, name: &str, domain: &str, path: Option<&str>) -> bool {
        let path = path.unwrap_or("/");
        let key = (domain.to_string(), path.to_string(), name.to_string());
        self.cookies.remove(&key).is_some()
    }

    /// Remove cookies matching the filter criteria (for Network.deleteCookies).
    /// Returns the number of cookies removed.
    pub fn delete_cookies(
        &mut self,
        name: &str,
        url: Option<&str>,
        domain: Option<&str>,
        path: Option<&str>,
    ) -> usize {
        let mut to_remove = Vec::new();

        // Parse URL if provided to extract domain/path defaults
        let (url_domain, url_path) = if let Some(url_str) = url {
            if let Ok(parsed) = Url::parse(url_str) {
                (
                    parsed.host_str().map(|s| s.to_string()),
                    Some(parsed.path().to_string()),
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Determine effective domain and path
        let effective_domain = domain.map(|s| s.to_string()).or(url_domain);
        let effective_path = path.map(|s| s.to_string()).or(url_path);

        for (key, cookie) in &self.cookies {
            // Name must match
            if cookie.name != name {
                continue;
            }

            // Domain must match if specified
            if let Some(ref d) = effective_domain {
                if !domain_matches(d, &cookie.domain) && &cookie.domain != d {
                    continue;
                }
            }

            // Path must match if specified
            if let Some(ref p) = effective_path {
                if &cookie.path != p && !path_matches(p, &cookie.path) {
                    continue;
                }
            }

            to_remove.push(key.clone());
        }

        let count = to_remove.len();
        for key in to_remove {
            self.cookies.remove(&key);
        }
        count
    }

    /// Clear all cookies from the jar.
    pub fn clear(&mut self) {
        self.cookies.clear();
        // Note: We can't clear the reqwest jar, but it will be rebuilt on next navigation
    }

    /// Get the number of cookies in the jar.
    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    /// Check if the jar is empty.
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    /// Sync a cookie to the reqwest jar for HTTP requests.
    fn sync_to_reqwest(&self, cookie: &Cookie) {
        // Build cookie string in Set-Cookie format
        let mut cookie_str = format!("{}={}", cookie.name, cookie.value);

        // Domain (without leading dot for reqwest)
        let domain = cookie.domain.trim_start_matches('.');
        cookie_str.push_str(&format!("; Domain={}", domain));

        // Path
        cookie_str.push_str(&format!("; Path={}", cookie.path));

        // Secure
        if cookie.secure {
            cookie_str.push_str("; Secure");
        }

        // HttpOnly
        if cookie.http_only {
            cookie_str.push_str("; HttpOnly");
        }

        // SameSite
        cookie_str.push_str(&format!("; SameSite={}", cookie.same_site.as_str()));

        // Expires
        if let Some(expires) = cookie.expires {
            // Convert Unix timestamp to HTTP date format
            if let Some(dt) = unix_to_http_date(expires) {
                cookie_str.push_str(&format!("; Expires={}", dt));
            }
        }

        // Create a URL for the cookie domain
        let scheme = if cookie.secure { "https" } else { "http" };
        if let Ok(url) = Url::parse(&format!("{}://{}/", scheme, domain)) {
            self.reqwest_jar.add_cookie_str(&cookie_str, &url);
        }
    }

    /// Parse a Set-Cookie header and add the cookie to the jar.
    pub fn parse_set_cookie(&mut self, header_value: &str, request_url: &str) {
        if let Some(cookie) = parse_set_cookie_header(header_value, request_url) {
            self.set_cookie(cookie);
        }
    }
}

/// Parse a Set-Cookie header into a Cookie struct.
pub fn parse_set_cookie_header(header_value: &str, request_url: &str) -> Option<Cookie> {
    let parsed_url = Url::parse(request_url).ok()?;
    let default_domain = parsed_url.host_str()?.to_string();
    let default_path = get_default_path(parsed_url.path());

    let parts: Vec<&str> = header_value.split(';').collect();
    if parts.is_empty() {
        return None;
    }

    // First part is name=value
    let name_value = parts[0].trim();
    let (name, value) = if let Some(eq_pos) = name_value.find('=') {
        (
            name_value[..eq_pos].trim().to_string(),
            name_value[eq_pos + 1..].trim().to_string(),
        )
    } else {
        return None;
    };

    if name.is_empty() {
        return None;
    }

    let mut cookie = Cookie {
        name: name.clone(),
        value: value.clone(),
        domain: default_domain,
        path: default_path,
        expires: None,
        http_only: false,
        secure: false,
        same_site: SameSite::Lax,
        size: name.len() + value.len(),
    };

    // Parse attributes
    for part in parts.iter().skip(1) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (attr_name, attr_value) = if let Some(eq_pos) = part.find('=') {
            (
                part[..eq_pos].trim().to_lowercase(),
                Some(part[eq_pos + 1..].trim()),
            )
        } else {
            (part.to_lowercase(), None)
        };

        match attr_name.as_str() {
            "domain" => {
                if let Some(v) = attr_value {
                    // Store domain with leading dot for subdomain matching
                    cookie.domain = if v.starts_with('.') {
                        v.to_string()
                    } else {
                        format!(".{}", v)
                    };
                }
            }
            "path" => {
                if let Some(v) = attr_value {
                    cookie.path = v.to_string();
                }
            }
            "expires" => {
                if let Some(v) = attr_value {
                    if let Some(ts) = parse_cookie_date(v) {
                        cookie.expires = Some(ts);
                    }
                }
            }
            "max-age" => {
                if let Some(v) = attr_value {
                    if let Ok(seconds) = v.parse::<i64>() {
                        let now = current_timestamp();
                        cookie.expires = Some(now + seconds as f64);
                    }
                }
            }
            "secure" => {
                cookie.secure = true;
            }
            "httponly" => {
                cookie.http_only = true;
            }
            "samesite" => {
                if let Some(v) = attr_value {
                    cookie.same_site = SameSite::from_str(v);
                }
            }
            _ => {}
        }
    }

    Some(cookie)
}

/// Check if a host matches a cookie domain.
fn domain_matches(host: &str, cookie_domain: &str) -> bool {
    let cookie_domain = cookie_domain.trim_start_matches('.');

    // Exact match
    if host.eq_ignore_ascii_case(cookie_domain) {
        return true;
    }

    // Subdomain match: host ends with .cookie_domain
    if host.len() > cookie_domain.len() {
        let suffix = &host[host.len() - cookie_domain.len()..];
        let prefix_char = host.chars().nth(host.len() - cookie_domain.len() - 1);
        if suffix.eq_ignore_ascii_case(cookie_domain) && prefix_char == Some('.') {
            return true;
        }
    }

    false
}

/// Check if a request path matches a cookie path.
fn path_matches(request_path: &str, cookie_path: &str) -> bool {
    // Exact match
    if request_path == cookie_path {
        return true;
    }

    // Request path starts with cookie path
    if request_path.starts_with(cookie_path) {
        // Cookie path ends with /
        if cookie_path.ends_with('/') {
            return true;
        }
        // Character after cookie path is /
        if request_path.chars().nth(cookie_path.len()) == Some('/') {
            return true;
        }
    }

    false
}

/// Get the default path for a cookie based on the request URL path.
fn get_default_path(request_path: &str) -> String {
    if request_path.is_empty() || !request_path.starts_with('/') {
        return "/".to_string();
    }

    // Find the last / before any query string
    let path = request_path.split('?').next().unwrap_or(request_path);
    if let Some(last_slash) = path.rfind('/') {
        if last_slash == 0 {
            return "/".to_string();
        }
        return path[..last_slash].to_string();
    }

    "/".to_string()
}

/// Get current Unix timestamp in seconds.
fn current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Convert Unix timestamp to HTTP date format.
fn unix_to_http_date(timestamp: f64) -> Option<String> {
    use std::time::Duration;
    let secs = timestamp as u64;
    let system_time = UNIX_EPOCH + Duration::from_secs(secs);

    // Format: "Wed, 09 Jun 2021 10:18:14 GMT"
    // We'll use a simple approach since we don't have chrono
    let duration = system_time.duration_since(UNIX_EPOCH).ok()?;
    let secs = duration.as_secs();

    // Days since Unix epoch
    let days = secs / 86400;
    let day_of_week = ((days + 4) % 7) as usize; // Jan 1 1970 was Thursday (4)

    let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    // Calculate year/month/day from days since epoch
    let (year, month, day) = days_to_ymd(days);

    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    Some(format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
        weekdays[day_of_week], day, months[month as usize], year, hours, minutes, seconds
    ))
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (i32, u32, u32) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (year as i32, m, d)
}

/// Parse a cookie date string to Unix timestamp.
fn parse_cookie_date(date_str: &str) -> Option<f64> {
    // Common formats:
    // "Wed, 09 Jun 2021 10:18:14 GMT"
    // "Wednesday, 09-Jun-21 10:18:14 GMT"
    // "Wed Jun  9 10:18:14 2021"

    // Try parsing RFC 2822 style: "Wed, 09 Jun 2021 10:18:14 GMT"
    let parts: Vec<&str> = date_str.split_whitespace().collect();

    if parts.len() >= 4 {
        let months = [
            "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
        ];

        // Try format: "Day, DD Mon YYYY HH:MM:SS GMT"
        if parts.len() >= 5 {
            let day: u32 = parts[1].trim_matches(',').parse().ok()?;
            let month = months
                .iter()
                .position(|&m| parts[2].to_lowercase().starts_with(m))?
                as u32;
            let year: i32 = parts[3].parse().ok()?;
            let time_parts: Vec<&str> = parts[4].split(':').collect();

            if time_parts.len() >= 3 {
                let hours: u32 = time_parts[0].parse().ok()?;
                let minutes: u32 = time_parts[1].parse().ok()?;
                let seconds: u32 = time_parts[2].parse().ok()?;

                return Some(ymd_hms_to_timestamp(
                    year,
                    month + 1,
                    day,
                    hours,
                    minutes,
                    seconds,
                ));
            }
        }
    }

    // Fallback: try parsing as Unix timestamp directly
    date_str.parse::<f64>().ok()
}

/// Convert year/month/day/hour/minute/second to Unix timestamp.
fn ymd_hms_to_timestamp(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> f64 {
    // Days from 1970-01-01 to year-month-day
    let y = if month <= 2 { year - 1 } else { year } as i64;
    let m = if month <= 2 { month + 12 } else { month } as i64;
    let d = day as i64;

    let days = d - 32045
        + (1461 * (y + 4800 + (m - 14) / 12)) / 4
        + (367 * (m - 2 - 12 * ((m - 14) / 12))) / 12
        - (3 * ((y + 4900 + (m - 14) / 12) / 100)) / 4
        - 2440588; // Julian day number of Unix epoch

    let seconds = days * 86400 + hour as i64 * 3600 + min as i64 * 60 + sec as i64;
    seconds as f64
}

/// Create a Cookie from CDP parameters (for Network.setCookie).
pub fn cookie_from_cdp_params(params: &serde_json::Value) -> Option<Cookie> {
    let name = params.get("name")?.as_str()?.to_string();
    let value = params.get("value")?.as_str()?.to_string();

    // Domain is required or derived from URL
    let domain = params
        .get("domain")
        .and_then(|v| v.as_str())
        .map(|s| {
            if s.starts_with('.') {
                s.to_string()
            } else {
                format!(".{}", s)
            }
        })
        .or_else(|| {
            params
                .get("url")
                .and_then(|v| v.as_str())
                .and_then(|url| Url::parse(url).ok())
                .and_then(|u| u.host_str().map(|h| format!(".{}", h)))
        })?;

    let path = params
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("/")
        .to_string();

    let expires = params.get("expires").and_then(|v| v.as_f64());

    let http_only = params
        .get("httpOnly")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let secure = params
        .get("secure")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let same_site = params
        .get("sameSite")
        .and_then(|v| v.as_str())
        .map(SameSite::from_str)
        .unwrap_or(SameSite::Lax);

    Some(Cookie {
        size: name.len() + value.len(),
        name,
        value,
        domain,
        path,
        expires,
        http_only,
        secure,
        same_site,
    })
}

/// Convert a Cookie to CDP JSON format.
impl Cookie {
    pub fn to_cdp_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "value": self.value,
            "domain": self.domain,
            "path": self.path,
            "expires": self.expires.unwrap_or(-1.0),
            "size": self.size,
            "httpOnly": self.http_only,
            "secure": self.secure,
            "session": self.expires.is_none(),
            "sameSite": self.same_site.as_str(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jar() -> CookieJar {
        let reqwest_jar = Arc::new(Jar::default());
        CookieJar::new(reqwest_jar)
    }

    #[test]
    fn test_set_and_get_cookie() {
        let mut jar = create_test_jar();

        let cookie = Cookie {
            name: "session".to_string(),
            value: "abc123".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: false,
            same_site: SameSite::Lax,
            size: 13,
        };

        jar.set_cookie(cookie);

        let cookies = jar.get_cookies("http://example.com/page");
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "session");
        assert_eq!(cookies[0].value, "abc123");
    }

    #[test]
    fn test_domain_matching() {
        let mut jar = create_test_jar();

        let cookie = Cookie {
            name: "test".to_string(),
            value: "value".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 9,
        };

        jar.set_cookie(cookie);

        // Should match example.com
        assert_eq!(jar.get_cookies("http://example.com/").len(), 1);

        // Should match subdomain
        assert_eq!(jar.get_cookies("http://www.example.com/").len(), 1);
        assert_eq!(jar.get_cookies("http://sub.example.com/").len(), 1);

        // Should NOT match different domain
        assert_eq!(jar.get_cookies("http://example.org/").len(), 0);
        assert_eq!(jar.get_cookies("http://notexample.com/").len(), 0);
    }

    #[test]
    fn test_path_matching() {
        let mut jar = create_test_jar();

        let cookie = Cookie {
            name: "test".to_string(),
            value: "value".to_string(),
            domain: ".example.com".to_string(),
            path: "/app".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 9,
        };

        jar.set_cookie(cookie);

        // Should match /app
        assert_eq!(jar.get_cookies("http://example.com/app").len(), 1);

        // Should match subpaths
        assert_eq!(jar.get_cookies("http://example.com/app/page").len(), 1);
        assert_eq!(jar.get_cookies("http://example.com/app/sub/page").len(), 1);

        // Should NOT match different paths
        assert_eq!(jar.get_cookies("http://example.com/").len(), 0);
        assert_eq!(jar.get_cookies("http://example.com/other").len(), 0);
        assert_eq!(jar.get_cookies("http://example.com/application").len(), 0);
    }

    #[test]
    fn test_secure_cookie() {
        let mut jar = create_test_jar();

        let cookie = Cookie {
            name: "secure_cookie".to_string(),
            value: "secret".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: true,
            same_site: SameSite::Lax,
            size: 19,
        };

        jar.set_cookie(cookie);

        // Should NOT match http
        assert_eq!(jar.get_cookies("http://example.com/").len(), 0);

        // Should match https
        assert_eq!(jar.get_cookies("https://example.com/").len(), 1);
    }

    #[test]
    fn test_cookie_expiration() {
        let mut jar = create_test_jar();

        // Expired cookie (1 second in the past)
        let expired_cookie = Cookie {
            name: "expired".to_string(),
            value: "old".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: Some(current_timestamp() - 1.0),
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 10,
        };

        // Valid cookie (1 hour in the future)
        let valid_cookie = Cookie {
            name: "valid".to_string(),
            value: "new".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: Some(current_timestamp() + 3600.0),
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 8,
        };

        jar.set_cookie(expired_cookie);
        jar.set_cookie(valid_cookie);

        let cookies = jar.get_cookies("http://example.com/");
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "valid");
    }

    #[test]
    fn test_remove_cookie() {
        let mut jar = create_test_jar();

        jar.set_cookie(Cookie {
            name: "test".to_string(),
            value: "value".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 9,
        });

        assert_eq!(jar.len(), 1);

        let removed = jar.remove_cookie("test", ".example.com", Some("/"));
        assert!(removed);
        assert_eq!(jar.len(), 0);
    }

    #[test]
    fn test_delete_cookies() {
        let mut jar = create_test_jar();

        jar.set_cookie(Cookie {
            name: "session".to_string(),
            value: "abc".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 10,
        });

        jar.set_cookie(Cookie {
            name: "session".to_string(),
            value: "def".to_string(),
            domain: ".other.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 10,
        });

        assert_eq!(jar.len(), 2);

        // Delete only the example.com cookie
        let count = jar.delete_cookies("session", None, Some(".example.com"), None);
        assert_eq!(count, 1);
        assert_eq!(jar.len(), 1);

        let remaining = jar.get_all_cookies();
        assert_eq!(remaining[0].domain, ".other.com");
    }

    #[test]
    fn test_clear() {
        let mut jar = create_test_jar();

        jar.set_cookie(Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 2,
        });

        jar.set_cookie(Cookie {
            name: "b".to_string(),
            value: "2".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: false,
            secure: false,
            same_site: SameSite::Lax,
            size: 2,
        });

        assert_eq!(jar.len(), 2);

        jar.clear();
        assert!(jar.is_empty());
    }

    #[test]
    fn test_parse_set_cookie_header() {
        let cookie = parse_set_cookie_header(
            "session=abc123; Domain=example.com; Path=/; HttpOnly; Secure; SameSite=Strict",
            "https://example.com/login",
        )
        .unwrap();

        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, ".example.com");
        assert_eq!(cookie.path, "/");
        assert!(cookie.http_only);
        assert!(cookie.secure);
        assert_eq!(cookie.same_site, SameSite::Strict);
    }

    #[test]
    fn test_cookie_from_cdp_params() {
        let params = serde_json::json!({
            "name": "test",
            "value": "value",
            "domain": "example.com",
            "path": "/app",
            "httpOnly": true,
            "secure": true,
            "sameSite": "None"
        });

        let cookie = cookie_from_cdp_params(&params).unwrap();
        assert_eq!(cookie.name, "test");
        assert_eq!(cookie.value, "value");
        assert_eq!(cookie.domain, ".example.com");
        assert_eq!(cookie.path, "/app");
        assert!(cookie.http_only);
        assert!(cookie.secure);
        assert_eq!(cookie.same_site, SameSite::None);
    }
}
