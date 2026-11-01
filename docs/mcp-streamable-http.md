# MCP Streamable HTTP

Plasmate exposes Streamable HTTP only when explicitly selected:

```bash
export PLASMATE_MCP_HTTP_TOKEN="$(openssl rand -hex 32)"
plasmate mcp --transport http --host 127.0.0.1 --port 9272
```

The single endpoint is `POST http://127.0.0.1:9272/mcp`. `plasmate mcp`
without transport flags remains the existing stdio server.

## Security contract

- Binding is restricted to loopback (`127.0.0.1`, `::1`, or `localhost`).
- Every method, including GET and DELETE, requires
  `Authorization: Bearer <capability-token>`.
- Tokens must contain at least 32 visible ASCII bytes without whitespace. If
  neither `--token` nor `PLASMATE_MCP_HTTP_TOKEN` is set, a new 256-bit token is
  printed to stderr.
- Requests with an `Origin` header receive HTTP 403 unless the exact HTTP(S)
  origin was added with `--allow-origin`. Requests from native clients normally
  omit `Origin` and remain available. Plasmate does not emit CORS response
  headers or handle browser preflight, so this allowlist is an Origin-security
  control, not a claim of direct browser-client support.
- Request bodies are capped at 1 MiB. Handler execution has a 60-second
  deadline, with a 65-second whole-request envelope that also bounds body
  delivery.
- Protocol sessions are capped at 128 and removed after 30 minutes idle.
- `Content-Type: application/json` is required. POST clients must advertise
  both `application/json` and `text/event-stream` in `Accept`, even though this
  implementation selects JSON response mode.

Do not expose the endpoint through a network proxy. Plasmate does not implement
TLS termination or MCP OAuth, and bearer tokens over cleartext HTTP are suitable
only for same-host use.

## MCP 2025-11-25

1. POST `initialize` without `Mcp-Session-Id`.
2. Read the unpredictable `Mcp-Session-Id` response header.
3. Send `notifications/initialized` with that session ID and
   `MCP-Protocol-Version: 2025-11-25`.
4. Include both headers on subsequent requests.
5. Send DELETE with both headers to terminate the protocol session.

HTTP initialization intentionally supports only `2025-11-25`; the pre-
Streamable-HTTP `2024-11-05` version is rejected instead of creating a session
that cannot be used coherently on this transport.

Missing session IDs return HTTP 400; unknown or terminated IDs return 404.
Supplying a client-chosen ID during initialize is rejected to prevent session
fixation. Browser session handles returned by tools remain explicit application
state and are separate from the transport session.

The 2025 specification allows a server to decline a standalone GET/SSE stream.
Plasmate returns HTTP 405 because it does not emit server-initiated requests or
notifications. POST responses use `application/json`; accepted notification
POSTs return HTTP 202 with no body.

## MCP 2026-07-28 transport (initially implemented against the release candidate)

The modern path is stateless: it ignores legacy `Mcp-Session-Id` headers and
does not implement initialize, GET, or DELETE semantics. Every POST must include:

- `MCP-Protocol-Version: 2026-07-28`, matching
  `params._meta.io.modelcontextprotocol/protocolVersion`;
- `Mcp-Method`, matching the JSON-RPC `method`;
- `Mcp-Name` for `tools/call`, matching `params.name` (including the specified
  Base64 sentinel decoding when a name is not header-safe);
- complete per-request client identity and capabilities metadata.

Header/body mismatches return HTTP 400 with MCP error `-32020`. Unsupported
versions return HTTP 400 with `-32022` and the supported version list. Unknown
methods return HTTP 404 with JSON-RPC `-32601`.

Plasmate does not advertise or implement Tasks or MCP Apps. It also does not
implement SSE subscriptions, resumability, or custom `Mcp-Param-*` header
annotations because no Plasmate tool schema declares such annotations.

## Specification status

This implementation was checked on 2026-07-19 against the official MCP
`2025-11-25` Streamable HTTP specification, the draft Streamable HTTP and
versioning pages for `2026-07-28`, and the release-candidate announcement.
The `2026-07-28` revision has since been published as the [dated MCP
specification](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http).
That publication does not by itself establish that Plasmate has passed a new
conformance run. Re-run transport tests and review headers, error handling,
and discovery behavior against the published revision before claiming full
`2026-07-28` conformance.
