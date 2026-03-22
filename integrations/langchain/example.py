#!/usr/bin/env python3
"""Example: LangChain agent browsing Hacker News with Plasmate.

Demonstrates how Plasmate's SOM output saves ~10x tokens compared to raw HTML
while giving the agent everything it needs to understand and interact with pages.

Prerequisites:
    pip install langchain-plasmate langchain-openai langchain
    # Ensure `plasmate` binary is on PATH

Usage:
    export OPENAI_API_KEY=sk-...
    python example.py
"""

from langchain.agents import AgentExecutor, create_tool_calling_agent
from langchain_core.prompts import ChatPromptTemplate
from langchain_openai import ChatOpenAI

from langchain_plasmate import PlasmateFetchTool, PlasmateSOMLoader, get_plasmate_tools


def main() -> None:
    # --- 1. Token efficiency demo -------------------------------------------
    print("=" * 60)
    print("Token efficiency: PlasmateSOMLoader vs raw HTML")
    print("=" * 60)

    loader = PlasmateSOMLoader(["https://news.ycombinator.com"])
    docs = loader.load()
    doc = docs[0]

    html_bytes = doc.metadata["html_bytes"]
    som_bytes = doc.metadata["som_bytes"]
    som_tokens = len(doc.page_content) // 4  # ~4 chars/token estimate
    html_tokens = html_bytes // 4

    print(f"  HTML size:  {html_bytes:>8,} bytes  (~{html_tokens:,} tokens)")
    print(f"  SOM size:   {som_bytes:>8,} bytes")
    print(f"  SOM text:   {len(doc.page_content):>8,} chars  (~{som_tokens:,} tokens)")
    print(f"  Savings:    ~{html_tokens / max(som_tokens, 1):.0f}x fewer tokens")
    print(f"  Elements:   {doc.metadata['element_count']} total, "
          f"{doc.metadata['interactive_count']} interactive")
    print()

    # --- 2. Agent with browsing tools ----------------------------------------
    print("=" * 60)
    print("Agent: 'Go to Hacker News and tell me the top 3 stories'")
    print("=" * 60)

    tools = get_plasmate_tools()
    llm = ChatOpenAI(model="gpt-4o")
    prompt = ChatPromptTemplate.from_messages([
        (
            "system",
            "You are a helpful assistant that can browse the web using Plasmate. "
            "When you fetch a page, you receive a compact semantic structure with "
            "element IDs. Use these IDs with click/type tools to interact with pages.",
        ),
        ("human", "{input}"),
        ("placeholder", "{agent_scratchpad}"),
    ])

    agent = create_tool_calling_agent(llm, tools, prompt)
    executor = AgentExecutor(agent=agent, tools=tools, verbose=True)

    result = executor.invoke({
        "input": "Go to Hacker News and tell me the top 3 stories"
    })

    print()
    print("Agent response:")
    print(result["output"])

    # --- 3. Standalone fetch tool -------------------------------------------
    print()
    print("=" * 60)
    print("Standalone PlasmateFetchTool")
    print("=" * 60)

    fetch = PlasmateFetchTool()
    som_text = fetch.invoke("https://news.ycombinator.com")
    # Show first 40 lines
    lines = som_text.split("\n")
    for line in lines[:40]:
        print(f"  {line}")
    if len(lines) > 40:
        print(f"  ... ({len(lines) - 40} more lines)")


if __name__ == "__main__":
    main()
