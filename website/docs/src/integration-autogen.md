# AutoGen Integration

Add structured web browsing to your Microsoft AutoGen multi-agent conversations. Plasmate returns SOM instead of raw HTML; the size difference depends on the page and tokenizer.

Published AutoGen samples imported `plasmate.integrations.autogen`. That module is not in this repository. Point AutoGen at the [native MCP server](integration-mcp), or have an AutoGen-owned tool call the shipped [Python SDK](sdk-python). Do not install a framework-specific Plasmate AutoGen package.

## Installation

```bash
curl -fsSL https://plasmate.app/install.sh | sh
pip install plasmate
```

Requires the `plasmate` binary on PATH.

## Why Plasmate for AutoGen?

- **Compact by design** -  SOM removes markup that does not contribute to the semantic page model; measure token use with your pages and model
- **Structured output** -  SOM exposes explicit regions, roles, and actions instead of raw markup
- **No Chrome installation required** -  runs headlessly in server-based AutoGen deployments
- **Tool-compatible** -  works with AutoGen's function calling once the agent attaches MCP or its own SDK wrapper

## Links

- [AutoGen Docs](https://microsoft.github.io/autogen/)
- [Native MCP Server](integration-mcp)
- [Plasmate Python SDK](sdk-python)
