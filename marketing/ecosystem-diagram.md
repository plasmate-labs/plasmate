# Plasmate Ecosystem Diagram

## Mermaid Diagram (for docs/README)

```mermaid
flowchart TB
    subgraph core["Core"]
        plasmate["Plasmate Engine"]
        som["SOM Compiler"]
        mcp["MCP Server"]
    end

    subgraph ai["AI Frameworks"]
        langchain["LangChain"]
        llamaindex["LlamaIndex"]
        crewai["CrewAI"]
        autogen["AutoGen"]
        haystack["Haystack"]
        dspy["DSPy"]
        semantic["Semantic Kernel"]
        vercel["Vercel AI SDK"]
    end

    subgraph visual["Visual Builders"]
        langflow["Langflow"]
        flowise["Flowise"]
        dify["Dify"]
    end

    subgraph automation["Automation"]
        n8n["n8n"]
        zapier["Zapier"]
        make["Make.com"]
        activepieces["Activepieces"]
        temporal["Temporal"]
    end

    subgraph scraping["Web Scraping"]
        scrapy["Scrapy"]
        crawl4ai["Crawl4AI"]
        firecrawl["Firecrawl"]
        scrapegraph["ScrapeGraphAI"]
    end

    subgraph database["Databases"]
        supabase["Supabase"]
        prisma["Prisma"]
        planetscale["PlanetScale"]
        airtable["Airtable"]
    end

    subgraph devtools["Developer Tools"]
        vscode["VS Code"]
        cursor["Cursor"]
        raycast["Raycast"]
        copilot["GitHub Copilot"]
        cloudflare["Cloudflare Workers"]
    end

    subgraph selfhosted["Self-Hosted LLMs"]
        openwebui["Open WebUI"]
        openai["OpenAI GPT Actions"]
    end

    plasmate --> ai
    plasmate --> visual
    plasmate --> automation
    plasmate --> scraping
    plasmate --> database
    plasmate --> devtools
    plasmate --> selfhosted
```

## ASCII Art Version (for terminals/plain text)

```
                              ┌─────────────────┐
                              │    PLASMATE     │
                              │  Browser Engine │
                              └────────┬────────┘
                                       │
        ┌──────────────────────────────┼──────────────────────────────┐
        │                              │                              │
        ▼                              ▼                              ▼
┌───────────────┐            ┌─────────────────┐            ┌─────────────────┐
│ AI FRAMEWORKS │            │ VISUAL BUILDERS │            │   AUTOMATION    │
├───────────────┤            ├─────────────────┤            ├─────────────────┤
│ LangChain     │            │ Langflow        │            │ n8n             │
│ LlamaIndex    │            │ Flowise         │            │ Zapier          │
│ CrewAI        │            │ Dify            │            │ Make.com        │
│ AutoGen       │            └─────────────────┘            │ Temporal        │
│ Haystack      │                                           └─────────────────┘
│ DSPy          │
│ Semantic Kern │
│ Vercel AI     │
└───────────────┘

        ┌──────────────────────────────┼──────────────────────────────┐
        │                              │                              │
        ▼                              ▼                              ▼
┌───────────────┐            ┌─────────────────┐            ┌─────────────────┐
│  WEB SCRAPING │            │    DATABASES    │            │  DEVELOPER TOOLS│
├───────────────┤            ├─────────────────┤            ├─────────────────┤
│ Scrapy        │            │ Supabase        │            │ VS Code         │
│ Crawl4AI      │            │ Prisma          │            │ Cursor          │
│ Firecrawl     │            │ PlanetScale     │            │ Raycast         │
│ ScrapeGraphAI │            │ Airtable        │            │ GitHub Copilot  │
└───────────────┘            └─────────────────┘            │ Cloudflare      │
                                                            └─────────────────┘
```

## Badge for README

```markdown
![Integrations](https://img.shields.io/badge/integrations-60%2B-brightgreen)
```

## Category Stats

| Category | Count |
|----------|-------|
| AI Frameworks | 8 |
| Visual Builders | 3 |
| Automation | 5 |
| Web Scraping | 4 |
| Databases | 4 |
| Developer Tools | 5 |
| Self-Hosted LLMs | 2 |
| **Total** | **31 integrations** |
| + Quickstarts, Examples, Tools | **60 repos** |
