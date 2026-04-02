# Publishing Plasmate to the MCP Registry

The [MCP Registry](https://registry.modelcontextprotocol.io/) is the official discovery surface
for MCP servers — used by Claude, Cursor, and other MCP-compatible clients. Plasmate is not
yet listed there. This guide covers the one-time setup to publish.

## Prerequisites

- Maintainer access to the `plasmate-labs` GitHub org (auth is tied to org ownership)
- `node` / `npm` installed

## Step 1 — Install mcp-publisher

```bash
curl -L "https://github.com/modelcontextprotocol/registry/releases/latest/download/mcp-publisher_$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/').tar.gz" \
  | tar xz mcp-publisher \
  && sudo mv mcp-publisher /usr/local/bin/
```

Verify:

```bash
mcp-publisher --help
```

## Step 2 — Authenticate

The `server.json` name is `io.github.plasmate-labs/plasmate`, so you must authenticate
as a member of the `plasmate-labs` GitHub org:

```bash
mcp-publisher login github
# Visit the printed URL, enter the device code, authorize the app
```

## Step 3 — Publish

From the repo root (where `server.json` lives):

```bash
mcp-publisher publish
```

Expected output:

```
Publishing to https://registry.modelcontextprotocol.io...
✓ Successfully published
✓ Server io.github.plasmate-labs/plasmate version 0.4.0
```

## Step 4 — Verify

```bash
curl "https://registry.modelcontextprotocol.io/v0.1/servers?search=io.github.plasmate-labs/plasmate"
```

## What was changed in this PR

- `server.json` added at repo root — defines the registry entry (name, description, npm + PyPI packages, stdio transport, `plasmate mcp` invocation)
- `sdk/python/README.md` — added `<!-- mcp-name: io.github.plasmate-labs/plasmate -->` comment required for PyPI ownership verification
- `package.json` already has `mcpName: "io.github.plasmate-labs/plasmate"` — npm ownership already verified ✓

## Re-publishing after version bumps

After each release, update the `version` field in `server.json` to match and re-run:

```bash
mcp-publisher publish
```

Consider automating this with the
[GitHub Actions publishing guide](https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/github-actions.mdx).
