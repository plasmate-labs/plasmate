# CrewAI Integration

Give your CrewAI agents structured SOM pages instead of raw HTML scraping. The resulting context size depends on the page and model tokenizer.

This repository does not ship a `plasmate.integrations.crewai` package. Wrap the shipped [Python SDK](sdk-python) as a CrewAI tool.

## Installation

```bash
pip install plasmate crewai
```

Requires the `plasmate` binary on your PATH.

## Quick Start

```python
from typing import Type

from crewai import Agent, Task, Crew
from crewai.tools import BaseTool
from pydantic import BaseModel, Field
from plasmate import Plasmate


class FetchPageInput(BaseModel):
    url: str = Field(..., description="Public URL to fetch as SOM")


class PlasmateFetchTool(BaseTool):
    name: str = "plasmate_fetch"
    description: str = (
        "Fetch a web page and return structured SOM instead of raw HTML."
    )
    args_schema: Type[BaseModel] = FetchPageInput

    def _run(self, url: str) -> str:
        with Plasmate() as client:
            return str(client.fetch_page(url))


researcher = Agent(
    role="Web Researcher",
    goal="Find and summarize information from the web",
    backstory="Expert at extracting key information from web pages.",
    tools=[PlasmateFetchTool()],
)

task = Task(
    description="Research the top stories on Hacker News and summarize them.",
    expected_output="A bullet-point summary of the top 5 stories.",
    agent=researcher,
)

crew = Crew(agents=[researcher], tasks=[task])
result = crew.kickoff()
print(result)
```

## Fetch vs scrape

Use `Plasmate.fetch_page` in place of HTML scrapers when the crew needs semantic regions and actions. Persistent click/type flows use `open_page` on the same client; see the Python SDK.

SOM removes non-semantic markup, but token and cost differences vary with the
pages, prompts, and model tokenizer. Benchmark the full crew workflow before
planning context or spend.

## Links

- [CrewAI Docs](https://docs.crewai.com)
- [Plasmate Python SDK](sdk-python)
