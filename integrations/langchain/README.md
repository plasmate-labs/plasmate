# langchain-plasmate

LangChain integration for [Plasmate](https://github.com/nicepkg/plasmate) — an agent-native headless browser that returns **SOM (Semantic Object Model)** output instead of raw HTML.

SOM compiles web pages into compact, structured representations that preserve interactive elements, content hierarchy, and navigation landmarks while stripping scripts, styles, and layout noise. This typically saves **~10x tokens** compared to raw HTML.

## Installation

```bash
pip install langchain-plasmate
```

Requires the `plasmate` binary on your PATH. See the [main Plasmate README](../../README.md) for installation.

## Quick Start

### Fetch a page (stateless)

```python
from langchain_plasmate import PlasmateFetchTool

fetch = PlasmateFetchTool()
result = fetch.invoke("https://news.ycombinator.com")
print(result)
```

Output:
```
Page: Hacker News
URL: https://news.ycombinator.com

## navigation "Main menu"
  [e_a1b2c3d4e5f6] link "Hacker News" -> /
  [e_b2c3d4e5f6a7] link "new" -> /newest
  [e_c3d4e5f6a7b8] link "past" -> /front
  ...

## main
  h1: Hacker News
  [e_d4e5f6a7b8c9] link "Show HN: Something Cool" -> https://example.com
  [e_e5f6a7b8c9d0] link "42 comments" -> /item?id=12345
  ...

---
87,234 → 4,521 bytes (19.3x) | 156 elements, 89 interactive
```

### Agent with browsing tools

```python
from langchain.agents import AgentExecutor, create_tool_calling_agent
from langchain_core.prompts import ChatPromptTemplate
from langchain_openai import ChatOpenAI
from langchain_plasmate import get_plasmate_tools

tools = get_plasmate_tools()
llm = ChatOpenAI(model="gpt-4o")

prompt = ChatPromptTemplate.from_messages([
    ("system", "You can browse the web using Plasmate tools."),
    ("human", "{input}"),
    ("placeholder", "{agent_scratchpad}"),
])

agent = create_tool_calling_agent(llm, tools, prompt)
executor = AgentExecutor(agent=agent, tools=tools)

result = executor.invoke({
    "input": "Go to Hacker News and tell me the top 3 stories"
})
print(result["output"])
```

### Document loader

```python
from langchain_plasmate import PlasmateSOMLoader

loader = PlasmateSOMLoader([
    "https://example.com",
    "https://news.ycombinator.com",
])
docs = loader.load()

for doc in docs:
    print(f"{doc.metadata['title']} — {doc.metadata['element_count']} elements")
    print(doc.page_content[:200])
    print()
```

## Token Comparison

| Source | Hacker News | Example.com | Typical News Article |
|--------|-------------|-------------|---------------------|
| Raw HTML (WebBaseLoader) | ~22,000 tokens | ~400 tokens | ~45,000 tokens |
| **SOM (PlasmateSOMLoader)** | **~1,500 tokens** | **~80 tokens** | **~3,000 tokens** |
| **Savings** | **~15x** | **~5x** | **~15x** |

SOM preserves all interactive elements, headings, and content structure while stripping scripts, styles, hidden elements, and layout-only markup.

## Tools Reference

### `PlasmateFetchTool`

Stateless page fetch — returns SOM text for a single URL.

| Field | Value |
|-------|-------|
| Name | `plasmate_fetch` |
| Input | `url` (string) |
| Output | SOM text with element IDs |

### `PlasmateNavigateTool`

Opens a URL in a persistent browser session. Subsequent click/type calls target this page.

| Field | Value |
|-------|-------|
| Name | `plasmate_navigate` |
| Input | `url` (string) |
| Output | SOM text with element IDs |

### `PlasmateClickTool`

Clicks an interactive element by its SOM element ID (e.g., `e_a1b2c3d4e5f6`).

| Field | Value |
|-------|-------|
| Name | `plasmate_click` |
| Input | `element_id` (string) |
| Output | Updated SOM text |

### `PlasmateTypeTool`

Types text into a form input or textarea by its SOM element ID.

| Field | Value |
|-------|-------|
| Name | `plasmate_type` |
| Input | `element_id` (string), `text` (string) |
| Output | Updated SOM text |

### `get_plasmate_tools()`

Returns all four tools with a shared Plasmate client and browser session.

```python
from langchain_plasmate import get_plasmate_tools
from plasmate import Plasmate

# Default client
tools = get_plasmate_tools()

# Custom client
client = Plasmate(binary="/path/to/plasmate", timeout=60)
tools = get_plasmate_tools(client=client)
```

## Loader Reference

### `PlasmateSOMLoader`

Loads web pages as LangChain `Document` objects with SOM text as content.

```python
PlasmateSOMLoader(
    urls=["https://example.com"],
    budget=2000,          # optional token budget per page
    javascript=True,      # enable JS execution (default)
    client=None,          # optional Plasmate instance
)
```

**Document metadata** includes: `url`, `title`, `lang`, `html_bytes`, `som_bytes`, `element_count`, `interactive_count`, and optionally `description`, `open_graph`, `json_ld`.

## Configuration

All tools accept an optional `Plasmate` client instance for custom configuration:

```python
from plasmate import Plasmate
from langchain_plasmate import PlasmateFetchTool

client = Plasmate(binary="/opt/plasmate/bin/plasmate", timeout=60)
fetch = PlasmateFetchTool(client=client)
```

## License

MIT
