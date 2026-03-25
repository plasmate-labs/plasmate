# Smolagents (HuggingFace) Integration

Use Plasmate as a web browsing tool in HuggingFace's smolagents framework — structured SOM output for **~10x fewer tokens** than raw HTML.

Source: [`integrations/smolagents/`](https://github.com/nicepkg/plasmate/tree/master/integrations/smolagents)

## Installation

```bash
pip install plasmate smolagents
```

## Quick Start

```python
from smolagents import CodeAgent, HfApiModel
from plasmate.integrations.smolagents import PlasmateWebTool

# Create a Plasmate browsing tool
web_tool = PlasmateWebTool()

# Create an agent with web access
agent = CodeAgent(
    tools=[web_tool],
    model=HfApiModel("Qwen/Qwen2.5-Coder-32B-Instruct"),
)

result = agent.run(
    "Go to https://news.ycombinator.com and list the top 3 stories"
)
print(result)
```

## The Tool

### `PlasmateWebTool`

A smolagents `Tool` that fetches a URL and returns structured SOM content.

```python
from plasmate.integrations.smolagents import PlasmateWebTool

tool = PlasmateWebTool(
    binary="plasmate",   # Path to binary
    timeout=30,          # Timeout in seconds
    budget=None,         # Optional token budget
)
```

**Inputs:** `url` (string) — the URL to fetch

**Output:** SOM text with indexed interactive elements

## Why Plasmate for Smolagents?

Smolagents is designed for lightweight, code-generating agents. Plasmate is a natural fit:

- **Compact output** — SOM text fits well in smolagents' code-generation context windows
- **No heavy dependencies** — no Chrome, no Playwright, just the `plasmate` binary
- **Works with any model** — HuggingFace Hub models, OpenAI, Anthropic, local models
- **Open source** — both smolagents and Plasmate are Apache 2.0

## Links

- [Smolagents Docs](https://huggingface.co/docs/smolagents)
- [HuggingFace Hub](https://huggingface.co)
- [Plasmate Python SDK](sdk-python)
