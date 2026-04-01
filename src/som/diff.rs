use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{Element, Region, RegionRole, Som, SomMeta};

// ---------------------------------------------------------------------------
// Change types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextChange {
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttrChange {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Element diff
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElementDiff {
    pub id: String,
    pub change_type: ChangeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attr_changes: Option<Vec<AttrChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children_changes: Option<Vec<ElementDiff>>,
}

// ---------------------------------------------------------------------------
// Region diff
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegionDiff {
    pub id: String,
    pub change_type: ChangeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_changes: Option<Vec<ElementDiff>>,
}

// ---------------------------------------------------------------------------
// Page-level diff
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageDiff {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang_change: Option<TextChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_count_delta: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactive_count_delta: Option<i64>,
}

// ---------------------------------------------------------------------------
// Meta diff
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaDiff {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_bytes_delta: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub som_bytes_delta: Option<i64>,
}

// ---------------------------------------------------------------------------
// Summary stats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub elements_added: usize,
    pub elements_removed: usize,
    pub elements_modified: usize,
    pub regions_added: usize,
    pub regions_removed: usize,
    pub has_price_changes: bool,
    pub has_content_changes: bool,
    pub has_structural_changes: bool,
}

// ---------------------------------------------------------------------------
// Top-level diff
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SomDiff {
    pub page: PageDiff,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaDiff>,
    pub regions: Vec<RegionDiff>,
    pub summary: DiffSummary,
}

// ---------------------------------------------------------------------------
// Diff engine
// ---------------------------------------------------------------------------

/// Compare two SOM snapshots and produce a structured diff.
///
/// The algorithm is O(n) — elements and regions are indexed by their stable
/// `id` field using `HashMap` lookups.
pub fn diff_soms(old: &Som, new: &Som, ignore_meta: bool) -> SomDiff {
    let page = diff_page(old, new);
    let meta = if ignore_meta {
        None
    } else {
        diff_meta(&old.meta, &new.meta)
    };
    let (regions, mut summary) = diff_regions(&old.regions, &new.regions);

    // Detect price changes across all element text changes.
    summary.has_price_changes = detect_price_changes_in_regions(&regions);

    // Detect content changes (text mods in "main" regions).
    summary.has_content_changes = detect_content_changes(&regions, &old.regions, &new.regions);

    // Structural = added/removed regions or elements.
    summary.has_structural_changes = summary.regions_added > 0
        || summary.regions_removed > 0
        || summary.elements_added > 0
        || summary.elements_removed > 0;

    SomDiff {
        page,
        meta,
        regions,
        summary,
    }
}

// ---------------------------------------------------------------------------
// Page diff
// ---------------------------------------------------------------------------

fn diff_page(old: &Som, new: &Som) -> PageDiff {
    let title_change = if old.title != new.title {
        Some(TextChange {
            old: old.title.clone(),
            new: new.title.clone(),
        })
    } else {
        None
    };

    let url_change = if old.url != new.url {
        Some(TextChange {
            old: old.url.clone(),
            new: new.url.clone(),
        })
    } else {
        None
    };

    let lang_change = if old.lang != new.lang {
        Some(TextChange {
            old: old.lang.clone(),
            new: new.lang.clone(),
        })
    } else {
        None
    };

    let ec_delta = new.meta.element_count as i64 - old.meta.element_count as i64;
    let ic_delta = new.meta.interactive_count as i64 - old.meta.interactive_count as i64;

    PageDiff {
        title_change,
        url_change,
        lang_change,
        element_count_delta: if ec_delta != 0 { Some(ec_delta) } else { None },
        interactive_count_delta: if ic_delta != 0 {
            Some(ic_delta)
        } else {
            None
        },
    }
}

// ---------------------------------------------------------------------------
// Meta diff
// ---------------------------------------------------------------------------

fn diff_meta(old: &SomMeta, new: &SomMeta) -> Option<MetaDiff> {
    let html_delta = new.html_bytes as i64 - old.html_bytes as i64;
    let som_delta = new.som_bytes as i64 - old.som_bytes as i64;

    if html_delta == 0 && som_delta == 0 {
        return None;
    }

    Some(MetaDiff {
        html_bytes_delta: if html_delta != 0 {
            Some(html_delta)
        } else {
            None
        },
        som_bytes_delta: if som_delta != 0 {
            Some(som_delta)
        } else {
            None
        },
    })
}

// ---------------------------------------------------------------------------
// Region diff
// ---------------------------------------------------------------------------

fn diff_regions(old: &[Region], new: &[Region]) -> (Vec<RegionDiff>, DiffSummary) {
    let old_map: HashMap<&str, &Region> = old.iter().map(|r| (r.id.as_str(), r)).collect();
    let new_map: HashMap<&str, &Region> = new.iter().map(|r| (r.id.as_str(), r)).collect();

    let mut diffs = Vec::new();
    let mut summary = DiffSummary {
        total_changes: 0,
        elements_added: 0,
        elements_removed: 0,
        elements_modified: 0,
        regions_added: 0,
        regions_removed: 0,
        has_price_changes: false,
        has_content_changes: false,
        has_structural_changes: false,
    };

    // Removed regions (in old but not new).
    for r in old {
        if !new_map.contains_key(r.id.as_str()) {
            summary.regions_removed += 1;
            summary.total_changes += 1;
            diffs.push(RegionDiff {
                id: r.id.clone(),
                change_type: ChangeType::Removed,
                role_change: None,
                label_change: None,
                element_changes: None,
            });
        }
    }

    // Added or modified regions.
    for r in new {
        if let Some(old_r) = old_map.get(r.id.as_str()) {
            // Region exists in both — check for changes.
            let role_change = if old_r.role != r.role {
                Some(TextChange {
                    old: format!("{:?}", old_r.role),
                    new: format!("{:?}", r.role),
                })
            } else {
                None
            };

            let label_change = if old_r.label != r.label {
                Some(TextChange {
                    old: old_r.label.clone().unwrap_or_default(),
                    new: r.label.clone().unwrap_or_default(),
                })
            } else {
                None
            };

            let (elem_diffs, elem_counts) = diff_elements(&old_r.elements, &r.elements);

            summary.elements_added += elem_counts.0;
            summary.elements_removed += elem_counts.1;
            summary.elements_modified += elem_counts.2;
            summary.total_changes += elem_counts.0 + elem_counts.1 + elem_counts.2;

            let has_changes = role_change.is_some()
                || label_change.is_some()
                || !elem_diffs.is_empty();

            if has_changes {
                if role_change.is_some() || label_change.is_some() {
                    summary.total_changes += 1;
                }
                diffs.push(RegionDiff {
                    id: r.id.clone(),
                    change_type: ChangeType::Modified,
                    role_change,
                    label_change,
                    element_changes: if elem_diffs.is_empty() {
                        None
                    } else {
                        Some(elem_diffs)
                    },
                });
            }
        } else {
            // New region.
            summary.regions_added += 1;
            summary.total_changes += 1;
            diffs.push(RegionDiff {
                id: r.id.clone(),
                change_type: ChangeType::Added,
                role_change: None,
                label_change: None,
                element_changes: None,
            });
        }
    }

    (diffs, summary)
}

// ---------------------------------------------------------------------------
// Element diff  (returns diffs + (added, removed, modified) counts)
// ---------------------------------------------------------------------------

fn diff_elements(old: &[Element], new: &[Element]) -> (Vec<ElementDiff>, (usize, usize, usize)) {
    let old_map: HashMap<&str, &Element> = old.iter().map(|e| (e.id.as_str(), e)).collect();
    let new_map: HashMap<&str, &Element> = new.iter().map(|e| (e.id.as_str(), e)).collect();

    let mut diffs = Vec::new();
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut modified = 0usize;

    // Removed elements.
    for e in old {
        if !new_map.contains_key(e.id.as_str()) {
            removed += 1;
            diffs.push(ElementDiff {
                id: e.id.clone(),
                change_type: ChangeType::Removed,
                text_change: None,
                role_change: None,
                attr_changes: None,
                actions_change: None,
                hints_change: None,
                children_changes: None,
            });
        }
    }

    // Added or modified elements.
    for e in new {
        if let Some(old_e) = old_map.get(e.id.as_str()) {
            let ediff = diff_single_element(old_e, e);
            if let Some(d) = ediff {
                modified += 1;
                diffs.push(d);
            }
        } else {
            added += 1;
            diffs.push(ElementDiff {
                id: e.id.clone(),
                change_type: ChangeType::Added,
                text_change: None,
                role_change: None,
                attr_changes: None,
                actions_change: None,
                hints_change: None,
                children_changes: None,
            });
        }
    }

    (diffs, (added, removed, modified))
}

/// Compare two elements with the same ID. Returns `None` if identical.
fn diff_single_element(old: &Element, new: &Element) -> Option<ElementDiff> {
    let text_change = if old.text != new.text {
        Some(TextChange {
            old: old.text.clone().unwrap_or_default(),
            new: new.text.clone().unwrap_or_default(),
        })
    } else {
        None
    };

    let role_change = if old.role != new.role {
        Some(TextChange {
            old: old.role.as_str().to_string(),
            new: new.role.as_str().to_string(),
        })
    } else {
        None
    };

    let attr_changes = diff_attrs(&old.attrs, &new.attrs);

    let actions_change = {
        let old_actions = old
            .actions
            .as_ref()
            .map(|a| a.join(","))
            .unwrap_or_default();
        let new_actions = new
            .actions
            .as_ref()
            .map(|a| a.join(","))
            .unwrap_or_default();
        if old_actions != new_actions {
            Some(TextChange {
                old: old_actions,
                new: new_actions,
            })
        } else {
            None
        }
    };

    let hints_change = {
        let old_hints = old
            .hints
            .as_ref()
            .map(|h| h.join(","))
            .unwrap_or_default();
        let new_hints = new
            .hints
            .as_ref()
            .map(|h| h.join(","))
            .unwrap_or_default();
        if old_hints != new_hints {
            Some(TextChange {
                old: old_hints,
                new: new_hints,
            })
        } else {
            None
        }
    };

    // Children diff.
    let children_changes = match (&old.children, &new.children) {
        (Some(old_c), Some(new_c)) => {
            let (cdiffs, _) = diff_elements(old_c, new_c);
            if cdiffs.is_empty() {
                None
            } else {
                Some(cdiffs)
            }
        }
        (None, Some(new_c)) if !new_c.is_empty() => {
            let cdiffs: Vec<ElementDiff> = new_c
                .iter()
                .map(|e| ElementDiff {
                    id: e.id.clone(),
                    change_type: ChangeType::Added,
                    text_change: None,
                    role_change: None,
                    attr_changes: None,
                    actions_change: None,
                    hints_change: None,
                    children_changes: None,
                })
                .collect();
            Some(cdiffs)
        }
        (Some(old_c), None) if !old_c.is_empty() => {
            let cdiffs: Vec<ElementDiff> = old_c
                .iter()
                .map(|e| ElementDiff {
                    id: e.id.clone(),
                    change_type: ChangeType::Removed,
                    text_change: None,
                    role_change: None,
                    attr_changes: None,
                    actions_change: None,
                    hints_change: None,
                    children_changes: None,
                })
                .collect();
            Some(cdiffs)
        }
        _ => None,
    };

    let has_changes = text_change.is_some()
        || role_change.is_some()
        || attr_changes.is_some()
        || actions_change.is_some()
        || hints_change.is_some()
        || children_changes.is_some();

    if !has_changes {
        return None;
    }

    Some(ElementDiff {
        id: old.id.clone(),
        change_type: ChangeType::Modified,
        text_change,
        role_change,
        attr_changes,
        actions_change,
        hints_change,
        children_changes,
    })
}

// ---------------------------------------------------------------------------
// Attribute diff
// ---------------------------------------------------------------------------

fn diff_attrs(
    old: &Option<serde_json::Value>,
    new: &Option<serde_json::Value>,
) -> Option<Vec<AttrChange>> {
    let empty_obj = serde_json::Value::Object(serde_json::Map::new());
    let old_obj = old.as_ref().unwrap_or(&empty_obj);
    let new_obj = new.as_ref().unwrap_or(&empty_obj);

    if old_obj == new_obj {
        return None;
    }

    let old_map = old_obj.as_object();
    let new_map = new_obj.as_object();

    let (old_map, new_map) = match (old_map, new_map) {
        (Some(o), Some(n)) => (o, n),
        _ => {
            // Non-object attrs — treat as wholesale change if different.
            if old_obj != new_obj {
                return Some(vec![AttrChange {
                    key: "_value".to_string(),
                    old: old.clone(),
                    new: new.clone(),
                }]);
            }
            return None;
        }
    };

    let mut changes = Vec::new();

    // Keys removed.
    for (k, v) in old_map {
        if !new_map.contains_key(k) {
            changes.push(AttrChange {
                key: k.clone(),
                old: Some(v.clone()),
                new: None,
            });
        }
    }

    // Keys added or modified.
    for (k, new_v) in new_map {
        match old_map.get(k) {
            Some(old_v) if old_v != new_v => {
                changes.push(AttrChange {
                    key: k.clone(),
                    old: Some(old_v.clone()),
                    new: Some(new_v.clone()),
                });
            }
            None => {
                changes.push(AttrChange {
                    key: k.clone(),
                    old: None,
                    new: Some(new_v.clone()),
                });
            }
            _ => {}
        }
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

// ---------------------------------------------------------------------------
// Price change detection
// ---------------------------------------------------------------------------

/// Detect price-like patterns in text changes.
/// Matches: $X.XX, $X,XXX.XX, €X.XX, £X.XX, and bare decimals that look
/// like prices.
fn is_price_text(text: &str) -> bool {
    // Simple check: contains a currency symbol followed by digits, or a
    // digit pattern like "XX.XX" near a currency symbol.
    let price_re = regex::Regex::new(r"[$€£¥]\s*[\d,]+\.?\d*").unwrap();
    price_re.is_match(text)
}

fn detect_price_changes_in_regions(regions: &[RegionDiff]) -> bool {
    for r in regions {
        if let Some(ref elems) = r.element_changes {
            if detect_price_changes_in_elements(elems) {
                return true;
            }
        }
    }
    false
}

fn detect_price_changes_in_elements(elements: &[ElementDiff]) -> bool {
    for e in elements {
        if let Some(ref tc) = e.text_change {
            if is_price_text(&tc.old) || is_price_text(&tc.new) {
                return true;
            }
        }
        if let Some(ref children) = e.children_changes {
            if detect_price_changes_in_elements(children) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Content change detection (text modifications in "main" region)
// ---------------------------------------------------------------------------

fn detect_content_changes(
    diffs: &[RegionDiff],
    old_regions: &[Region],
    new_regions: &[Region],
) -> bool {
    // Build a set of region IDs whose role is Main.
    let main_ids: std::collections::HashSet<&str> = old_regions
        .iter()
        .chain(new_regions.iter())
        .filter(|r| r.role == RegionRole::Main)
        .map(|r| r.id.as_str())
        .collect();

    for d in diffs {
        if !main_ids.contains(d.id.as_str()) {
            continue;
        }
        if let Some(ref elems) = d.element_changes {
            for e in elems {
                if e.text_change.is_some() {
                    return true;
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Human-readable text output
// ---------------------------------------------------------------------------

/// Render a SomDiff as human-readable text.
pub fn render_text(diff: &SomDiff) -> String {
    let mut out = String::new();

    // Page changes.
    if let Some(ref tc) = diff.page.title_change {
        out.push_str(&format!("Title: \"{}\" → \"{}\"\n", tc.old, tc.new));
    }
    if let Some(ref uc) = diff.page.url_change {
        out.push_str(&format!("URL: {} → {}\n", uc.old, uc.new));
    }
    if let Some(ref lc) = diff.page.lang_change {
        out.push_str(&format!("Language: {} → {}\n", lc.old, lc.new));
    }
    if let Some(d) = diff.page.element_count_delta {
        out.push_str(&format!("Element count delta: {:+}\n", d));
    }
    if let Some(d) = diff.page.interactive_count_delta {
        out.push_str(&format!("Interactive count delta: {:+}\n", d));
    }

    // Meta changes.
    if let Some(ref m) = diff.meta {
        if let Some(d) = m.html_bytes_delta {
            out.push_str(&format!("HTML bytes delta: {:+}\n", d));
        }
        if let Some(d) = m.som_bytes_delta {
            out.push_str(&format!("SOM bytes delta: {:+}\n", d));
        }
    }

    if !out.is_empty() {
        out.push('\n');
    }

    // Region changes.
    for r in &diff.regions {
        match r.change_type {
            ChangeType::Added => {
                out.push_str(&format!("[+] Region {}\n", r.id));
            }
            ChangeType::Removed => {
                out.push_str(&format!("[-] Region {}\n", r.id));
            }
            ChangeType::Modified => {
                out.push_str(&format!("[~] Region {}\n", r.id));
                if let Some(ref rc) = r.role_change {
                    out.push_str(&format!("    Role: {} → {}\n", rc.old, rc.new));
                }
                if let Some(ref lc) = r.label_change {
                    out.push_str(&format!("    Label: \"{}\" → \"{}\"\n", lc.old, lc.new));
                }
                if let Some(ref elems) = r.element_changes {
                    render_element_diffs(&mut out, elems, 4);
                }
            }
        }
    }

    // Summary line.
    out.push('\n');
    out.push_str(&render_summary(&diff.summary));
    out.push('\n');

    out
}

fn render_element_diffs(out: &mut String, diffs: &[ElementDiff], indent: usize) {
    let pad: String = " ".repeat(indent);
    for e in diffs {
        match e.change_type {
            ChangeType::Added => {
                out.push_str(&format!("{}[+] Element {}\n", pad, e.id));
            }
            ChangeType::Removed => {
                out.push_str(&format!("{}[-] Element {}\n", pad, e.id));
            }
            ChangeType::Modified => {
                out.push_str(&format!("{}[~] Element {}\n", pad, e.id));
                if let Some(ref tc) = e.text_change {
                    out.push_str(&format!(
                        "{}    Text: \"{}\" → \"{}\"\n",
                        pad, tc.old, tc.new
                    ));
                }
                if let Some(ref rc) = e.role_change {
                    out.push_str(&format!("{}    Role: {} → {}\n", pad, rc.old, rc.new));
                }
                if let Some(ref attrs) = e.attr_changes {
                    for a in attrs {
                        let old_str = a
                            .old
                            .as_ref()
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "(none)".into());
                        let new_str = a
                            .new
                            .as_ref()
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "(none)".into());
                        out.push_str(&format!(
                            "{}    Attr {}: {} → {}\n",
                            pad, a.key, old_str, new_str
                        ));
                    }
                }
                if let Some(ref ac) = e.actions_change {
                    out.push_str(&format!(
                        "{}    Actions: [{}] → [{}]\n",
                        pad, ac.old, ac.new
                    ));
                }
                if let Some(ref hc) = e.hints_change {
                    out.push_str(&format!(
                        "{}    Hints: [{}] → [{}]\n",
                        pad, hc.old, hc.new
                    ));
                }
                if let Some(ref children) = e.children_changes {
                    render_element_diffs(out, children, indent + 4);
                }
            }
        }
    }
}

/// One-line summary string.
pub fn render_summary(s: &DiffSummary) -> String {
    let mut parts = Vec::new();
    if s.elements_added > 0 {
        parts.push(format!("{} added", s.elements_added));
    }
    if s.elements_removed > 0 {
        parts.push(format!("{} removed", s.elements_removed));
    }
    if s.elements_modified > 0 {
        parts.push(format!("{} modified", s.elements_modified));
    }
    if s.regions_added > 0 {
        parts.push(format!("{} region(s) added", s.regions_added));
    }
    if s.regions_removed > 0 {
        parts.push(format!("{} region(s) removed", s.regions_removed));
    }
    if s.has_price_changes {
        parts.push("price change detected".to_string());
    }
    if parts.is_empty() {
        "no changes".to_string()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
#[path = "diff_tests.rs"]
mod tests;
