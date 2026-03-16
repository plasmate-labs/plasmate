//! JavaScript execution via V8.
//!
//! Provides a minimal JS runtime sufficient for agent browsing:
//! - Execute inline `<script>` tags
//! - Basic DOM query/mutation bridge
//! - setTimeout/setInterval (with budget limits)
//!
//! We intentionally skip layout, paint, Canvas, WebGL, Workers,
//! and other APIs that agents never need.

pub mod extract;
pub mod pipeline;
pub mod runtime;
