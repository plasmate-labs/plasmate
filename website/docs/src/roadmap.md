# Roadmap

Plasmate's roadmap is public and standards-first. We ship compression and correctness before scale.

## Completed (v0.1.1)

- SOM compiler with 9.4x median compression across 38 sites
- V8 JavaScript execution with full DOM shim
- AWP WebSocket server
- CDP compatibility (Puppeteer connects out of the box)
- MCP server mode (stdio JSON-RPC)
- Cookie management
- Published on crates.io, npm, PyPI
- Docker image (GHCR multi-arch)

## v0.2: Standards & Adoption (Current)

- [x] CDP polish (Page.setContent, Accessibility.getFullAXTree, error handling)
- [x] SOM Specification v1.0 with JSON Schema and conformance test suite
- [x] Benchmark expansion to 100+ URLs across 13 categories
- [x] Node.js SDK with full TypeScript types and query helpers
- [x] Python SDK with Pydantic models and query helpers
- [x] Go SDK with structs, client, and query helpers
- [x] Browser Use integration
- [x] LangChain integration
- [ ] Benchmark blog post and interactive results page
- [ ] Framework PRs submitted upstream (Browser Use, LangChain)

## v0.3: Production Engine (Next)

- SOM Cache (in-memory + on-disk, differential updates)
- Parallel Session Manager (500+ concurrent sessions per 8GB)
- Stealth networking (TLS fingerprint control, proxy rotation)
- Wasm Skill System + marketplace
- Network request interception

## v0.4+: Commercial Platform

- Plasmate Cloud (fleet orchestration)
- Managed proxy network
- Enterprise SSO, SOC 2
- Skill marketplace
