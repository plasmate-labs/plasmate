# Installation

Get Plasmate on your machine in seconds. Pick whichever package manager you prefer -  they all install the same `plasmate` CLI.

## Quick Install

```bash
# Python (recommended)
pip install plasmate

# Rust
cargo install plasmate

# Node.js
npm install -g plasmate

# macOS (Homebrew)
brew tap plasmate-labs/plasmate
brew install plasmate
```

## Verify

```bash
plasmate --version
plasmate fetch https://example.com
```

If you see SOM output, you're good to go.

## MCP Server (Claude Code / Cursor / Windsurf)

Plasmate ships an MCP server that exposes `fetch`, `navigate`, `click`, and `type` as tools for any MCP-compatible AI client.

### Claude Code (one-liner)

```bash
claude mcp add plasmate -- npx plasmate-mcp
```

### Claude Desktop / Cursor / Windsurf

Add to your MCP config (e.g. `~/.claude/settings.json` or the app's MCP settings):

```json
{
  "mcpServers": {
    "plasmate": {
      "command": "npx",
      "args": ["-y", "plasmate-mcp"]
    }
  }
}
```

Works with Claude Code, Claude Desktop, Cursor, Windsurf, and any MCP client. See the [MCP setup guide](integration-mcp) for full details.

## All Distribution Channels

| Channel | Command |
|---------|---------|
| PyPI | `pip install plasmate` |
| crates.io | `cargo install plasmate` |
| npm | `npm install plasmate` |
| Homebrew | `brew tap plasmate-labs/plasmate && brew install plasmate` |
| MCP Registry | `claude mcp add plasmate` |
| Docker | Coming soon |
| GitHub Releases | [Download](https://github.com/plasmate-labs/plasmate/releases) |

## Using as a Rust Library

Plasmate is a Rust crate. Add it to your `Cargo.toml`:

```toml
[dependencies]
plasmate = "0.4"
```

```rust
use plasmate::Plasmate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let p = Plasmate::new()?;
    let som = p.fetch("https://example.com").await?;
    println!("{}", som.text());
    Ok(())
}
```

## Next Steps

- [Quick Start](quickstart) -  fetch your first page
- [MCP Setup](integration-mcp) -  connect to Claude, Cursor, or Windsurf
- [Python SDK](sdk-python) -  use Plasmate from Python
- [Node.js SDK](sdk-node) -  use Plasmate from JavaScript
