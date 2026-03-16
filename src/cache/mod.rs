//! SOM Cache - the paradigm shift.
//!
//! Why re-parse a page you already understand?
//!
//! Traditional browsers re-render every page from scratch on every visit.
//! Plasmate caches the compiled SOM and returns it instantly on revisits.
//!
//! Three tiers:
//! - Hot (in-memory LRU): instant, last N pages
//! - Warm (on-disk): thousands of pages, <1ms
//! - Prewarmed: background fetches predicted next pages
//!
//! For an agent navigating 50 pages in a workflow, pages 2-50 are often
//! revisits or predictable next pages. SOM Cache makes those free.

pub mod store;
