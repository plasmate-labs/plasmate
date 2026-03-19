# Roadmap

Plasmate's roadmap is public and tracked in GitHub.

- Source of truth: the `## Roadmap` section in the repo README
- This docs page mirrors that list for convenience

## Current Roadmap

- [x] MCP server mode (`plasmate mcp` over stdio)
- [x] MCP Phase 2: stateful tools (open_page, click, evaluate, close_page)
- [x] Docker image (GHCR multi-arch)
- [ ] Full V8 DOM mutation bridge (re-snapshot SOM after JS changes)
- [ ] Network interception (Fetch domain)
- [ ] Expose cookie APIs (CDP Network.getCookies/setCookies, MCP cookie import/export)
- [ ] Proxy support (per-session config, SOCKS)
- [ ] Real-world top-100 site coverage testing
- [ ] Web Platform Tests integration

## Notes

- The older "v0.2 Full Engine" roadmap is kept for historical context in `ROADMAP-v0.2.md`.
- The coverage scorecard lives at the `Coverage` page in these docs.
