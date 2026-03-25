# AutoGen Integration

Add structured web browsing to your Microsoft AutoGen multi-agent conversations -  Plasmate delivers **~10x fewer tokens** than raw HTML.

Source: [`integrations/autogen/`](https://github.com/nicepkg/plasmate/tree/master/integrations/autogen)

## Installation

```bash
pip install plasmate pyautogen
```

## Quick Start

```python
import autogen
from plasmate.integrations.autogen import plasmate_fetch

# Configure the LLM
config_list = [{"model": "gpt-4o", "api_key": "sk-..."}]

# Create an assistant with web browsing
assistant = autogen.AssistantAgent(
    name="WebResearcher",
    llm_config={"config_list": config_list},
)

# Create a user proxy that can execute the tool
user_proxy = autogen.UserProxyAgent(
    name="User",
    human_input_mode="NEVER",
    code_execution_config={"work_dir": "workspace"},
)

# Register the Plasmate tool
autogen.register_function(
    plasmate_fetch,
    caller=assistant,
    executor=user_proxy,
    name="plasmate_fetch",
    description="Fetch a web page and return structured SOM content.",
)

# Start the conversation
user_proxy.initiate_chat(
    assistant,
    message="Fetch https://news.ycombinator.com and summarize the top 5 stories.",
)
```

## Available Functions

### `plasmate_fetch(url: str) -> str`

Stateless fetch -  returns SOM text for a single URL.

### `plasmate_browse(url: str, actions: list) -> str`

Multi-step browsing session. Actions can be `navigate`, `click`, or `type`.

```python
from plasmate.integrations.autogen import plasmate_browse

result = plasmate_browse(
    url="https://github.com",
    actions=[
        {"type": "type", "index": 1, "text": "plasmate"},
        {"type": "click", "index": 2},
    ],
)
```

## Why Plasmate for AutoGen?

- **Token-efficient** -  multi-agent conversations amplify token costs; SOM keeps them manageable
- **Structured output** -  agents parse SOM content more reliably than raw HTML
- **No browser required** -  runs headless, perfect for server-based AutoGen deployments
- **Tool-compatible** -  works with AutoGen's function calling and tool registration

## Links

- [AutoGen Docs](https://microsoft.github.io/autogen/)
- [Plasmate Python SDK](sdk-python)
