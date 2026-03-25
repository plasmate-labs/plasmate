# CrewAI Integration

Give your CrewAI agents web browsing superpowers — Plasmate provides structured SOM pages at **~10x fewer tokens** than raw HTML scraping.

Source: [`integrations/crewai/`](https://github.com/nicepkg/plasmate/tree/master/integrations/crewai)

## Installation

```bash
pip install plasmate crewai crewai-tools
```

## Quick Start

```python
from crewai import Agent, Task, Crew
from plasmate.integrations.crewai import PlasmateWebTool

# Create the Plasmate browsing tool
browse = PlasmateWebTool()

# Create an agent with web access
researcher = Agent(
    role="Web Researcher",
    goal="Find and summarize information from the web",
    backstory="Expert at extracting key information from web pages.",
    tools=[browse],
)

# Define a task
task = Task(
    description="Research the top stories on Hacker News and summarize them.",
    expected_output="A bullet-point summary of the top 5 stories.",
    agent=researcher,
)

# Run the crew
crew = Crew(agents=[researcher], tasks=[task])
result = crew.kickoff()
print(result)
```

## Available Tools

### `PlasmateWebTool`

Fetches a URL and returns SOM text. Drop-in replacement for `ScrapeWebsiteTool` with dramatically fewer tokens.

```python
from plasmate.integrations.crewai import PlasmateWebTool

tool = PlasmateWebTool()
# Agents invoke it automatically when they need web content
```

### `PlasmateBrowseTool`

Persistent browser session with navigate, click, and type actions for multi-step workflows.

```python
from plasmate.integrations.crewai import PlasmateBrowseTool

tool = PlasmateBrowseTool()
# Supports: navigate(url), click(index), type(index, text)
```

## Why Plasmate for CrewAI?

| | ScrapeWebsiteTool | PlasmateWebTool |
|---|---|---|
| **Output** | Raw HTML/text | Structured SOM |
| **Tokens per page** | ~20,000-40,000 | ~2,000-4,000 |
| **Interactive elements** | Lost | Indexed `[N]` |
| **Multi-step browsing** | ❌ | ✅ |
| **Dependencies** | requests + beautifulsoup | `plasmate` binary |

Over a typical crew run with 5-10 page loads, you save **50,000-150,000 tokens** — significant cost reduction at scale.

## Links

- [CrewAI Docs](https://docs.crewai.com)
- [Plasmate Python SDK](sdk-python)
