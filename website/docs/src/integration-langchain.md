# LangChain Integration

LangChain tools and document loader for Plasmate -  browse the web with **~10x fewer tokens** than raw HTML.

Plasmate's SOM output compiles web pages into compact, structured representations that preserve interactive elements and content hierarchy. This integration provides LangChain-native tools for stateless fetching, persistent browsing, and batch document loading.

Source: [`integrations/langchain/`](https://github.com/nicepkg/plasmate/tree/master/integrations/langchain)

## Installation

```bash
pip install langchain-plasmate
```

Requires the `plasmate` binary on your PATH:

```bash
curl -fsSL https://plasmate.app/install.sh | sh
```

## Quick Start

### Fetch a page (stateless)

```python
from langchain_plasmate import PlasmateFetchTool

fetch = PlasmateFetchTool()
result = fetch.invoke("https://news.ycombinator.com")
print(result)
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
    print(f"{doc.metadata['title']} -  {doc.metadata['element_count']} elements")
    print(doc.page_content[:200])
    print()
```

## Available Tools

### `PlasmateFetchTool`

Stateless page fetch. Each call creates a fresh connection, fetches the URL, and returns SOM text. Best for one-off page reads.

```python
from langchain_plasmate import PlasmateFetchTool

fetch = PlasmateFetchTool()
result = fetch.invoke("https://example.com")
```

### `PlasmateNavigateTool`

Opens a URL in a persistent browser session. Use with `PlasmateClickTool` and `PlasmateTypeTool` for multi-step browsing workflows.

```python
from langchain_plasmate import get_plasmate_tools

tools = get_plasmate_tools()
navigate, click, type_tool, fetch = tools
```

### `PlasmateClickTool`

Clicks an interactive element by its SOM element ID (e.g., `e_a1b2c3d4e5f6`). Requires an active session from `PlasmateNavigateTool`.

### `PlasmateTypeTool`

Types text into a form input or textarea by SOM element ID. Takes `element_id` and `text` as input.

### `get_plasmate_tools()`

Returns all four tools with a shared Plasmate client and browser session:

```python
from langchain_plasmate import get_plasmate_tools
from plasmate import Plasmate

# Default client
tools = get_plasmate_tools()

# Custom client
client = Plasmate(binary="/path/to/plasmate", timeout=60)
tools = get_plasmate_tools(client=client)
```

## PlasmateSOMLoader

Loads web pages as LangChain `Document` objects with SOM text as `page_content`.

```python
PlasmateSOMLoader(
    urls=["https://example.com"],
    budget=2000,          # optional token budget per page
    javascript=True,      # enable JS execution (default)
    client=None,          # optional Plasmate instance
)
```

Document metadata includes: `url`, `title`, `lang`, `html_bytes`, `som_bytes`, `element_count`, `interactive_count`, and optionally `description`, `open_graph`, `json_ld`.

## Token Efficiency

Compared to LangChain's `WebBaseLoader` which passes raw HTML:

| Source | Hacker News | Example.com | News Article |
|--------|-------------|-------------|--------------|
| Raw HTML (WebBaseLoader) | ~22,000 tokens | ~400 tokens | ~45,000 tokens |
| **SOM (PlasmateSOMLoader)** | **~1,500 tokens** | **~80 tokens** | **~3,000 tokens** |
| **Savings** | **~15x** | **~5x** | **~15x** |

SOM preserves all interactive elements, headings, and content structure while stripping scripts, styles, hidden elements, and layout-only markup.
