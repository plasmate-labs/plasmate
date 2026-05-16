//! In-memory + on-disk SOM cache.
//!
//! Key = URL (normalized) plus optional SOM selector.
//! Value = CacheEntry { content_hash, som_json, metadata }.
//! On revisit: fetch HTML, compute hash, compare. If match -> instant SOM return.

use crate::som::{filter::apply_selector, types::Som};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, info};
use url::Url;

/// A cached SOM snapshot for a URL.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The compiled SOM as JSON bytes.
    pub som_json: Vec<u8>,
    /// xxHash of the HTML content that produced this SOM.
    pub content_hash: u64,
    /// When this entry was created.
    pub created_at: Instant,
    /// When this entry was last accessed.
    pub last_accessed: Instant,
    /// Original HTML byte count.
    pub html_bytes: usize,
    /// SOM byte count.
    pub som_bytes: usize,
    /// Effective HTML after JS execution, when stored by an interactive path.
    pub effective_html: Option<String>,
    /// Number of times this entry has been served from cache.
    pub hit_count: u64,
}

/// Cache lookup result.
#[derive(Debug)]
pub enum CacheLookup {
    /// Exact match: content hash matches, SOM is valid.
    Hit(CacheEntry),
    /// Stale: URL was cached but content hash differs (page changed).
    Stale { old_hash: u64, new_hash: u64 },
    /// Miss: URL was never cached.
    Miss,
}

/// Configuration for the SOM cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Max entries in the hot (in-memory) cache.
    pub max_hot_entries: usize,
    /// Max age before an entry is considered expired.
    pub max_age: Duration,
    /// Whether to enable prefetching of links found in SOM.
    pub prefetch_enabled: bool,
    /// Max concurrent prefetch tasks.
    pub prefetch_concurrency: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_hot_entries: 1000,
            max_age: Duration::from_secs(300), // 5 minutes
            prefetch_enabled: true,
            prefetch_concurrency: 4,
        }
    }
}

/// Thread-safe SOM cache.
pub struct SomCache {
    config: CacheConfig,
    /// URL -> CacheEntry, protected by RwLock for concurrent access.
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Cache statistics.
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub stale_hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_som_bytes_served: u64,
    pub total_html_bytes_avoided: u64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct CacheSnapshot {
    pub hits: u64,
    pub stale_hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_som_bytes_served: u64,
    pub total_html_bytes_avoided: u64,
    pub entries: usize,
    pub full_entries: usize,
    pub selector_entries: usize,
    pub effective_html_entries: usize,
    pub total_som_bytes: usize,
    pub total_html_bytes: usize,
    pub total_effective_html_bytes: usize,
    pub max_hot_entries: usize,
}

impl SomCache {
    pub fn new(config: CacheConfig) -> Self {
        info!(
            max_entries = config.max_hot_entries,
            "SOM cache initialized"
        );
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Compute content hash for HTML bytes.
    pub fn content_hash(html: &[u8]) -> u64 {
        xxhash_rust::xxh3::xxh3_64(html)
    }

    /// Look up a URL in the cache, checking content hash.
    pub fn lookup(&self, url: &str, content_hash: u64) -> CacheLookup {
        self.lookup_with_selector(url, content_hash, None)
    }

    /// Look up a URL plus selector in the cache, checking content hash.
    ///
    /// A `None` or blank selector uses the full-page cache entry. Non-id
    /// selectors are case-insensitive (`interactive`, `ACTION:CLICK`), while
    /// `#id` selectors preserve case so HTML id lookups keep browser semantics.
    pub fn lookup_with_selector(
        &self,
        url: &str,
        content_hash: u64,
        selector: Option<&str>,
    ) -> CacheLookup {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        let key = cache_key(url, selector);

        if let Some(entry) = entries.get_mut(&key) {
            // Check if expired
            if entry.created_at.elapsed() > self.config.max_age {
                stats.misses += 1;
                entries.remove(&key);
                return CacheLookup::Miss;
            }

            if entry.content_hash == content_hash {
                // Exact hit
                entry.last_accessed = Instant::now();
                entry.hit_count += 1;
                stats.hits += 1;
                stats.total_som_bytes_served += entry.som_bytes as u64;
                stats.total_html_bytes_avoided += entry.html_bytes as u64;
                debug!(url, hit_count = entry.hit_count, "SOM cache hit");
                return CacheLookup::Hit(entry.clone());
            } else {
                // Content changed
                stats.stale_hits += 1;
                let old_hash = entry.content_hash;
                return CacheLookup::Stale {
                    old_hash,
                    new_hash: content_hash,
                };
            }
        }

        stats.misses += 1;
        CacheLookup::Miss
    }

    /// Look up by URL only (skip content hash check). Used for prefetch/instant mode.
    pub fn lookup_any(&self, url: &str) -> Option<CacheEntry> {
        self.lookup_any_with_selector(url, None)
    }

    /// Look up by URL plus selector only (skip content hash check).
    pub fn lookup_any_with_selector(
        &self,
        url: &str,
        selector: Option<&str>,
    ) -> Option<CacheEntry> {
        let mut entries = self.entries.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        let key = cache_key(url, selector);
        if let Some(entry) = entries.get_mut(&key) {
            if entry.created_at.elapsed() <= self.config.max_age {
                entry.last_accessed = Instant::now();
                entry.hit_count += 1;
                stats.hits += 1;
                stats.total_som_bytes_served += entry.som_bytes as u64;
                stats.total_html_bytes_avoided += entry.html_bytes as u64;
                return Some(entry.clone());
            }
            entries.remove(&key);
        }
        stats.misses += 1;
        None
    }

    /// Store a compiled SOM in the cache.
    pub fn store(&self, url: &str, content_hash: u64, som_json: Vec<u8>, html_bytes: usize) {
        self.store_with_selector(url, content_hash, None, som_json, html_bytes);
    }

    /// Store a compiled or filtered SOM in the cache for a URL plus selector.
    pub fn store_with_selector(
        &self,
        url: &str,
        content_hash: u64,
        selector: Option<&str>,
        som_json: Vec<u8>,
        html_bytes: usize,
    ) {
        self.store_with_selector_and_effective_html(
            url,
            content_hash,
            selector,
            som_json,
            html_bytes,
            None,
        );
    }

    /// Store a full-page SOM with effective HTML for later session restoration.
    pub fn store_page_state(
        &self,
        url: &str,
        content_hash: u64,
        som_json: Vec<u8>,
        html_bytes: usize,
        effective_html: String,
    ) {
        self.store_with_selector_and_effective_html(
            url,
            content_hash,
            None,
            som_json,
            html_bytes,
            Some(effective_html),
        );
    }

    fn store_with_selector_and_effective_html(
        &self,
        url: &str,
        content_hash: u64,
        selector: Option<&str>,
        som_json: Vec<u8>,
        html_bytes: usize,
        effective_html: Option<String>,
    ) {
        let som_bytes = som_json.len();
        let entry = CacheEntry {
            som_json,
            content_hash,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            html_bytes,
            som_bytes,
            effective_html,
            hit_count: 0,
        };

        let key = cache_key(url, selector);
        let mut entries = self.entries.write().unwrap();

        // Evict if at capacity (LRU by last_accessed). Replacing an existing
        // key should not evict an unrelated cache entry.
        if !entries.contains_key(&key) && entries.len() >= self.config.max_hot_entries {
            self.evict_lru(&mut entries);
        }

        entries.insert(key, entry);
        debug!(url, selector = ?selector, html_bytes, som_bytes, "SOM cached");
    }

    /// Return a selector-specific cache entry, deriving it from a fresh full-SOM
    /// cache hit when the selector entry has not been materialized yet.
    pub fn lookup_or_filter_selector(
        &self,
        url: &str,
        content_hash: u64,
        selector: Option<&str>,
    ) -> CacheLookup {
        match self.lookup_with_selector(url, content_hash, selector) {
            CacheLookup::Miss if normalized_selector(selector).is_some() => {}
            other => return other,
        }

        let full_entry = match self.lookup(url, content_hash) {
            CacheLookup::Hit(entry) => entry,
            CacheLookup::Stale { old_hash, new_hash } => {
                return CacheLookup::Stale { old_hash, new_hash };
            }
            CacheLookup::Miss => return CacheLookup::Miss,
        };

        let Some(selector) = normalized_selector(selector) else {
            return CacheLookup::Hit(full_entry);
        };

        let Ok(som) = serde_json::from_slice::<Som>(&full_entry.som_json) else {
            return CacheLookup::Hit(full_entry);
        };
        let filtered = apply_selector(&som, &selector);
        let Ok(filtered_json) = serde_json::to_vec(&filtered) else {
            return CacheLookup::Hit(full_entry);
        };

        self.store_with_selector(
            url,
            content_hash,
            Some(&selector),
            filtered_json,
            full_entry.html_bytes,
        );

        self.lookup_with_selector(url, content_hash, Some(&selector))
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Cache statistics plus current entry inventory for daemon status output.
    pub fn snapshot(&self) -> CacheSnapshot {
        let entries = self.entries.read().unwrap();
        let stats = self.stats.read().unwrap().clone();
        let mut snapshot = CacheSnapshot {
            hits: stats.hits,
            stale_hits: stats.stale_hits,
            misses: stats.misses,
            evictions: stats.evictions,
            total_som_bytes_served: stats.total_som_bytes_served,
            total_html_bytes_avoided: stats.total_html_bytes_avoided,
            entries: entries.len(),
            max_hot_entries: self.config.max_hot_entries,
            ..CacheSnapshot::default()
        };

        for (key, entry) in entries.iter() {
            if key.contains("::selector=") {
                snapshot.selector_entries += 1;
            } else {
                snapshot.full_entries += 1;
            }
            if let Some(effective_html) = &entry.effective_html {
                snapshot.effective_html_entries += 1;
                snapshot.total_effective_html_bytes += effective_html.len();
            }
            snapshot.total_som_bytes += entry.som_bytes;
            snapshot.total_html_bytes += entry.html_bytes;
        }

        snapshot
    }

    /// Number of entries currently cached.
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Whether the cache currently has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    /// Extract all link hrefs from cached SOM JSON for prefetching.
    pub fn extract_prefetch_urls(&self, som_json: &[u8]) -> Vec<String> {
        if let Ok(som) = serde_json::from_slice::<serde_json::Value>(som_json) {
            let mut urls = Vec::new();
            let mut seen = std::collections::HashSet::new();
            if let Some(regions) = som.get("regions").and_then(|r| r.as_array()) {
                for region in regions {
                    if let Some(elements) = region.get("elements").and_then(|e| e.as_array()) {
                        for element in elements {
                            collect_prefetch_urls(element, &mut urls, &mut seen);
                        }
                    }
                }
            }
            urls
        } else {
            Vec::new()
        }
    }

    fn evict_lru(&self, entries: &mut HashMap<String, CacheEntry>) {
        // Find the least recently accessed entry
        if let Some((key, _)) = entries
            .iter()
            .min_by_key(|(_, e)| e.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            entries.remove(&key);
            let mut stats = self.stats.write().unwrap();
            stats.evictions += 1;
        }
    }
}

fn normalize_url(url: &str) -> String {
    let trimmed = url.trim();
    if let Ok(mut parsed) = Url::parse(trimmed) {
        parsed.set_fragment(None);
        if parsed.path() != "/" {
            let path = parsed.path().trim_end_matches('/').to_string();
            parsed.set_path(&path);
        }
        return parsed.to_string();
    }

    trimmed
        .split('#')
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches('/')
        .to_string()
}

fn normalized_selector(selector: Option<&str>) -> Option<String> {
    let selector = selector?.trim();
    if selector.is_empty() {
        return None;
    }
    if selector.starts_with('#') {
        Some(selector.to_string())
    } else {
        Some(selector.to_ascii_lowercase())
    }
}

fn cache_key(url: &str, selector: Option<&str>) -> String {
    let url_key = normalize_url(url);
    match normalized_selector(selector) {
        Some(selector) => format!("{url_key}::selector={selector}"),
        None => url_key,
    }
}

fn collect_prefetch_urls(
    element: &serde_json::Value,
    urls: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    if element.get("role").and_then(|r| r.as_str()) == Some("link") {
        if let Some(href) = element
            .get("attrs")
            .and_then(|a| a.get("href"))
            .and_then(|h| h.as_str())
        {
            if is_http_url(href) && seen.insert(href.to_string()) {
                urls.push(href.to_string());
            }
        }
    }

    if let Some(children) = element.get("children").and_then(|c| c.as_array()) {
        for child in children {
            collect_prefetch_urls(child, urls, seen);
        }
    }

    if let Some(shadow_elements) = element
        .get("shadow")
        .and_then(|s| s.get("elements"))
        .and_then(|e| e.as_array())
    {
        for shadow_element in shadow_elements {
            collect_prefetch_urls(shadow_element, urls, seen);
        }
    }
}

fn is_http_url(href: &str) -> bool {
    Url::parse(href)
        .map(|url| matches!(url.scheme(), "http" | "https"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_miss() {
        let cache = SomCache::new(CacheConfig::default());
        let result = cache.lookup("https://example.com", 12345);
        assert!(matches!(result, CacheLookup::Miss));
    }

    #[test]
    fn test_cache_hit() {
        let cache = SomCache::new(CacheConfig::default());
        let som = b"test som json".to_vec();
        cache.store("https://example.com", 12345, som.clone(), 1000);

        let result = cache.lookup("https://example.com", 12345);
        match result {
            CacheLookup::Hit(entry) => {
                assert_eq!(entry.som_json, som);
                assert_eq!(entry.html_bytes, 1000);
                assert_eq!(entry.hit_count, 1);
            }
            _ => panic!("Expected cache hit"),
        }
    }

    #[test]
    fn test_cache_stale() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store("https://example.com", 12345, b"old som".to_vec(), 1000);

        let result = cache.lookup("https://example.com", 99999);
        match result {
            CacheLookup::Stale { old_hash, new_hash } => {
                assert_eq!(old_hash, 12345);
                assert_eq!(new_hash, 99999);
            }
            _ => panic!("Expected stale hit"),
        }
    }

    #[test]
    fn test_url_normalization() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store("https://Example.Com/", 111, b"som".to_vec(), 100);

        // Should match with different casing and trailing slash
        let result = cache.lookup("https://example.com", 111);
        assert!(matches!(result, CacheLookup::Hit(_)));
    }

    #[test]
    fn test_url_normalization_preserves_path_case() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store(
            "https://Example.Com/CaseSensitive",
            111,
            b"som".to_vec(),
            100,
        );

        let result = cache.lookup("https://example.com/casesensitive", 111);
        assert!(matches!(result, CacheLookup::Miss));
    }

    #[test]
    fn test_selector_cache_keys_are_distinct_and_normalized() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store("https://example.com/app", 111, b"full".to_vec(), 100);
        cache.store_with_selector(
            "https://example.com/app",
            111,
            Some(" ACTION:CLICK "),
            b"click".to_vec(),
            100,
        );

        match cache.lookup_with_selector("https://example.com/app", 111, Some("action:click")) {
            CacheLookup::Hit(entry) => assert_eq!(entry.som_json, b"click".to_vec()),
            _ => panic!("Expected selector cache hit"),
        }
        match cache.lookup("https://example.com/app", 111) {
            CacheLookup::Hit(entry) => assert_eq!(entry.som_json, b"full".to_vec()),
            _ => panic!("Expected full cache hit"),
        }
    }

    #[test]
    fn test_id_selector_cache_keys_preserve_case() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store_with_selector(
            "https://example.com/app",
            111,
            Some("#SaveButton"),
            b"save".to_vec(),
            100,
        );

        assert!(matches!(
            cache.lookup_with_selector("https://example.com/app", 111, Some("#savebutton")),
            CacheLookup::Miss
        ));
        assert!(matches!(
            cache.lookup_with_selector("https://example.com/app", 111, Some("#SaveButton")),
            CacheLookup::Hit(_)
        ));
    }

    #[test]
    fn test_selector_lookup_derives_from_full_som_cache() {
        let cache = SomCache::new(CacheConfig::default());
        let som = serde_json::json!({
            "som_version": "1",
            "url": "https://example.com/app",
            "title": "App",
            "lang": "en",
            "regions": [{
                "id": "r_main",
                "role": "main",
                "elements": [
                    {"id": "e_copy", "role": "paragraph", "text": "Welcome"},
                    {"id": "e_save", "role": "button", "text": "Save", "actions": ["click"]}
                ]
            }],
            "meta": {
                "html_bytes": 100,
                "som_bytes": 100,
                "element_count": 2,
                "interactive_count": 1
            }
        });
        cache.store(
            "https://example.com/app",
            111,
            serde_json::to_vec(&som).unwrap(),
            100,
        );

        let derived =
            cache.lookup_or_filter_selector("https://example.com/app", 111, Some("interactive"));
        match derived {
            CacheLookup::Hit(entry) => {
                let filtered: serde_json::Value = serde_json::from_slice(&entry.som_json).unwrap();
                let elements = filtered["regions"][0]["elements"].as_array().unwrap();
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0]["id"], "e_save");
            }
            _ => panic!("Expected derived selector cache hit"),
        }

        assert!(matches!(
            cache.lookup_with_selector("https://example.com/app", 111, Some("INTERACTIVE")),
            CacheLookup::Hit(_)
        ));
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            max_hot_entries: 2,
            ..Default::default()
        };
        let cache = SomCache::new(config);
        cache.store("https://a.com", 1, b"a".to_vec(), 100);
        cache.store("https://b.com", 2, b"b".to_vec(), 100);
        cache.store("https://c.com", 3, b"c".to_vec(), 100);

        // Cache should have 2 entries (one was evicted)
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_replace_does_not_evict_other_entry() {
        let config = CacheConfig {
            max_hot_entries: 2,
            ..Default::default()
        };
        let cache = SomCache::new(config);
        cache.store("https://a.com", 1, b"a".to_vec(), 100);
        cache.store("https://b.com", 2, b"b".to_vec(), 100);
        cache.store("https://b.com", 3, b"b2".to_vec(), 100);

        assert_eq!(cache.len(), 2);
        assert!(matches!(
            cache.lookup("https://a.com", 1),
            CacheLookup::Hit(_)
        ));
        match cache.lookup("https://b.com", 3) {
            CacheLookup::Hit(entry) => assert_eq!(entry.som_json, b"b2".to_vec()),
            _ => panic!("Expected replaced cache hit"),
        }
    }

    #[test]
    fn test_content_hash() {
        let h1 = SomCache::content_hash(b"hello world");
        let h2 = SomCache::content_hash(b"hello world");
        let h3 = SomCache::content_hash(b"different content");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_cache_stats() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store("https://example.com", 111, b"som".to_vec(), 1000);

        let _ = cache.lookup("https://example.com", 111); // hit
        let _ = cache.lookup("https://example.com", 111); // hit
        let _ = cache.lookup("https://other.com", 222); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_lookup_any_updates_stats() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store("https://example.com", 111, b"som".to_vec(), 1000);

        let hit = cache.lookup_any("https://example.com");
        assert!(hit.is_some());
        assert!(cache.lookup_any("https://missing.example").is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total_som_bytes_served, 3);
        assert_eq!(stats.total_html_bytes_avoided, 1000);
    }

    #[test]
    fn test_cache_snapshot_counts_full_and_selector_entries() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store_page_state(
            "https://example.com",
            111,
            b"full".to_vec(),
            1000,
            "<html><body>full</body></html>".to_string(),
        );
        cache.store_with_selector(
            "https://example.com",
            111,
            Some("interactive"),
            b"selector".to_vec(),
            1000,
        );
        let _ = cache.lookup_with_selector("https://example.com", 111, Some("INTERACTIVE"));

        let snapshot = cache.snapshot();
        assert_eq!(snapshot.entries, 2);
        assert_eq!(snapshot.full_entries, 1);
        assert_eq!(snapshot.selector_entries, 1);
        assert_eq!(snapshot.effective_html_entries, 1);
        assert_eq!(snapshot.total_som_bytes, 12);
        assert_eq!(snapshot.total_html_bytes, 2000);
        assert_eq!(snapshot.total_effective_html_bytes, 30);
        assert_eq!(snapshot.hits, 1);
        assert_eq!(
            snapshot.max_hot_entries,
            CacheConfig::default().max_hot_entries
        );
    }

    #[test]
    fn test_page_state_cache_entry_restores_effective_html() {
        let cache = SomCache::new(CacheConfig::default());
        cache.store_page_state(
            "https://example.com/app",
            111,
            b"{\"regions\":[]}".to_vec(),
            100,
            "<html><body>hydrated</body></html>".to_string(),
        );

        match cache.lookup("https://example.com/app", 111) {
            CacheLookup::Hit(entry) => {
                assert_eq!(
                    entry.effective_html.as_deref(),
                    Some("<html><body>hydrated</body></html>")
                );
            }
            _ => panic!("Expected page-state cache hit"),
        }
    }

    #[test]
    fn test_prefetch_url_extraction() {
        let cache = SomCache::new(CacheConfig::default());
        let som = serde_json::json!({
            "regions": [{
                "elements": [
                    {"role": "link", "attrs": {"href": "https://example.com/page1"}},
                    {"role": "link", "attrs": {"href": "https://example.com/page2"}},
                    {"role": "button", "text": "Click"},
                    {"role": "link", "attrs": {"href": "/relative"}}
                ]
            }]
        });
        let json = serde_json::to_vec(&som).unwrap();
        let urls = cache.extract_prefetch_urls(&json);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com/page1".to_string()));
        assert!(urls.contains(&"https://example.com/page2".to_string()));
    }

    #[test]
    fn test_prefetch_url_extraction_walks_nested_and_shadow_elements() {
        let cache = SomCache::new(CacheConfig::default());
        let som = serde_json::json!({
            "regions": [{
                "elements": [{
                    "role": "section",
                    "children": [
                        {"role": "link", "attrs": {"href": "https://example.com/nested"}},
                        {"role": "link", "attrs": {"href": "https://example.com/nested"}}
                    ],
                    "shadow": {
                        "mode": "open",
                        "elements": [
                            {"role": "link", "attrs": {"href": "https://example.com/shadow"}},
                            {"role": "link", "attrs": {"href": "mailto:team@example.com"}}
                        ]
                    }
                }]
            }]
        });
        let json = serde_json::to_vec(&som).unwrap();
        let urls = cache.extract_prefetch_urls(&json);

        assert_eq!(
            urls,
            vec![
                "https://example.com/nested".to_string(),
                "https://example.com/shadow".to_string()
            ]
        );
    }
}
