//! MCP (Model Context Protocol) server for Plasmate.
//!
//! Exposes Plasmate as an MCP tool server over stdio. Compatible with
//! Claude Desktop, Cursor, Windsurf, and other MCP clients.
//!
//! Phase 1 implements stateless tools:
//! - fetch_page: Fetch URL and return SOM JSON
//! - extract_text: Fetch URL and return plain text
//!
//! Phase 2 implements stateful tools:
//! - open_page: Open a page in a persistent session
//! - evaluate: Run JavaScript in the session
//! - click: Click an element by SOM ID
//! - close_page: Close a session

pub mod server;
pub mod sessions;
pub mod tools;

pub use server::run_server;
