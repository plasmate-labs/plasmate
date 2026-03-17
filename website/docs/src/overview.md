# Plasmate Documentation

Plasmate is an open-source, agent-native headless browser engine written in Rust.

It compiles HTML into a **Semantic Object Model (SOM)** - a structured, token-efficient representation that AI agents can reason about directly. It replaces CDP with a purpose-built **Agent Web Protocol (AWP)**.

## Quick Links

| Document | Description |
|----------|-------------|
| [Quick Start](quickstart) | Install Plasmate and run your first fetch in 60 seconds |
| [Product Spec](spec) | Full architecture, market analysis, and technical vision |
| [AWP Protocol](awp) | Agent Web Protocol draft specification |
| [AWP MVP v0.1](awp-mvp) | The 7-method subset shipped in v0.1 |
| [SOM Reference](som) | Semantic Object Model structure and element addressing |
| [PoC Build Brief](poc) | Scope and acceptance criteria for the proof of concept |
| [Roadmap (v0.2)](roadmap) | V8 integration, CDP compatibility, SOM cache, parallel sessions |
| [Brand Guide](brand) | Colors, typography, voice, and the pixie dust system |

## Key Numbers

- **4-5 ms per page** throughput (100-page local benchmark, 231KB average)
- **10.4x compression** on Wikipedia (HTML to SOM token reduction)
- **~30 MB RSS** memory baseline per page
- **43 MB** single binary (macOS arm64)
- **184 tests** passing
- **Apache 2.0** license

## Architecture at a Glance

```
Agent (Python/JS/Rust)
  |
  |-- AWP (native, 7 methods)
  |-- CDP (legacy compatibility)
  |
Plasmate Engine
  |-- HTML Parser (html5ever)
  |-- SOM Compiler (regions, elements, budgets)
  |-- V8 JS Runtime (script execution, DOM shim)
  |-- Network Layer (reqwest, connection pooling, HTTP/2)
  |-- Session Manager (cookie persistence, navigation history)
```

## Source

- GitHub: [plasmate-labs/plasmate](https://github.com/plasmate-labs/plasmate)
- License: Apache 2.0
- Website: [plasmate.app](https://plasmate.app)
