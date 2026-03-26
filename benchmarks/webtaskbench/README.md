# WebTaskBench

A benchmark for measuring how web page representation format affects AI agent task performance.

## The Question

Does giving an LLM structured SOM output instead of raw HTML or cleaned markdown actually make it **better at tasks**, not just cheaper?

## Design

100 tasks across 5 categories, evaluated in 3 formats:

| Category | Tasks | What it measures |
|----------|-------|------------------|
| Factual Extraction | 20 | Can the agent find specific facts? |
| Multi-fact Comparison | 20 | Can it extract, filter, and compare? |
| Navigation/Structure | 20 | Does it understand page layout? |
| Summarization | 20 | Can it comprehend and distill? |
| Adversarial/Noise | 20 | Can it ignore ads, popups, and boilerplate? |

Each task is run against:
- **Raw HTML** (full HTTP response)
- **Cleaned Markdown** (Readability extraction)
- **SOM** (Plasmate Semantic Object Model)

## Metrics

- **Accuracy** (correct answers / total)
- **Token consumption** (input tokens per task)
- **Latency** (time to response)
- **Hallucination rate** (unsupported claims / total claims)
- **Refusal rate** (false "not found" responses)

## Quick Start

```bash
# 1. Build the corpus (fetch all pages in 3 formats)
./scripts/fetch_corpus.sh

# 2. Run evaluation (dry run first)
python3 scripts/run_eval.py --model gpt-4o --dry-run

# 3. Run for real (all formats, 3 runs)
python3 scripts/run_eval.py --model gpt-4o --format all --runs 3

# 4. Run a single category
python3 scripts/run_eval.py --model gpt-4o --category adversarial --runs 1

# 5. Replicate with Claude
python3 scripts/run_eval.py --model claude-3-5-sonnet-20241022 --format all --runs 3
```

## File Structure

```
webtaskbench/
  urls.json              # 50 URLs with categories
  tasks.json             # 100 tasks with rubrics
  corpus/                # Cached pages (git-ignored)
    html/                #   Raw HTML
    markdown/            #   Cleaned markdown
    som/                 #   Plasmate SOM JSON
  scripts/
    fetch_corpus.sh      # Build the corpus
    run_eval.py          # Evaluation harness
  results/               # JSONL output files (git-ignored)
  README.md
```

## Requirements

- Python 3.10+
- `openai` and/or `anthropic` Python packages
- `tiktoken` for token counting
- Plasmate CLI (`plasmate fetch`) for SOM generation
- API keys: `OPENAI_API_KEY` and/or `ANTHROPIC_API_KEY`

## Paper

This benchmark supports the paper: *"Does Format Matter? Agent Task Performance Across Web Representations"*

## License

Apache 2.0
