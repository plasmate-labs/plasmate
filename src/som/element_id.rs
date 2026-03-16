use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Generate a deterministic element ID from the element's properties.
///
/// Algorithm: `e_` + first 12 hex chars of `sha256("{origin}|{role}|{accessible_name}|{dom_path}")`
pub fn generate_element_id(
    origin: &str,
    role: &str,
    accessible_name: &str,
    dom_path: &str,
) -> String {
    let normalized_name = accessible_name
        .to_lowercase()
        .trim()
        .chars()
        .take(100)
        .collect::<String>();

    let input = format!("{}|{}|{}|{}", origin, role, normalized_name, dom_path);
    let hash = Sha256::digest(input.as_bytes());
    let hex_str = hex::encode(hash);
    format!("e_{}", &hex_str[..12])
}

/// Generate a region ID from the region role and index.
pub fn generate_region_id(role: &str, index: usize) -> String {
    if index == 0 {
        format!("r_{}", role)
    } else {
        format!("r_{}_{}", role, index)
    }
}

/// Tracks element IDs and handles collisions by appending a counter.
pub struct ElementIdTracker {
    seen: HashMap<String, usize>,
}

impl ElementIdTracker {
    pub fn new() -> Self {
        Self {
            seen: HashMap::new(),
        }
    }

    /// Register an element ID and return a unique version (handles collisions).
    pub fn register(&mut self, id: String) -> String {
        let count = self.seen.entry(id.clone()).or_insert(0);
        *count += 1;
        if *count == 1 {
            id
        } else {
            format!("{}_{}", id, count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_id() {
        let id1 = generate_element_id("https://example.com", "button", "Add to Cart", "0/3/1/0");
        let id2 = generate_element_id("https://example.com", "button", "Add to Cart", "0/3/1/0");
        assert_eq!(id1, id2);
        assert!(id1.starts_with("e_"));
        assert_eq!(id1.len(), 14); // "e_" + 12 hex chars
    }

    #[test]
    fn test_different_inputs_different_ids() {
        let id1 = generate_element_id("https://example.com", "button", "Submit", "0/1");
        let id2 = generate_element_id("https://example.com", "button", "Cancel", "0/2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_name_normalization() {
        let id1 = generate_element_id("https://example.com", "link", "  Hello World  ", "0/1");
        let id2 = generate_element_id("https://example.com", "link", "hello world", "0/1");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_collision_tracker() {
        let mut tracker = ElementIdTracker::new();
        let first = tracker.register("e_abc123def456".into());
        assert_eq!(first, "e_abc123def456");
        let second = tracker.register("e_abc123def456".into());
        assert_eq!(second, "e_abc123def456_2");
    }
}
