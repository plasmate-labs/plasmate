//! MCP (Model Context Protocol) server for Plasmate.
//!
//! Exposes Plasmate as an MCP tool server over stdio. Compatible with
//! Claude Desktop, Cursor, Windsurf, and other MCP clients.
//!
//! Phase 1 implements stateless tools:
//! - fetch_page: Fetch URL and return SOM JSON
//! - extract_text: Fetch URL and return plain text

pub mod server;
pub mod tools;

pub use server::run_server;
