#!/usr/bin/env python3
"""
WebTaskBench evaluation harness.
Runs all 100 tasks across 3 formats (HTML, Markdown, SOM) x 3 runs x 2 models.
Outputs structured JSON results for analysis.

Usage:
    python3 run_eval.py --model gpt-4o --format all --runs 3
    python3 run_eval.py --model claude-3-5-sonnet-20241022 --format som --runs 1  # quick test
"""

import argparse
import json
import os
import sys
import time
import hashlib
from pathlib import Path
from datetime import datetime

# Lazy imports for API clients
def get_openai_client():
    from openai import OpenAI
    return OpenAI()

def get_anthropic_client():
    import anthropic
    return anthropic.Anthropic()

SCRIPT_DIR = Path(__file__).parent
ROOT = SCRIPT_DIR.parent
CORPUS = ROOT / "corpus"
TASKS_FILE = ROOT / "tasks.json"
URLS_FILE = ROOT / "urls.json"
RESULTS_DIR = ROOT / "results"

SYSTEM_PROMPT_TEMPLATE = """You are a web research assistant. You will be given the content of a web page in {format_name} format. Answer the user's question based solely on the provided content.

Rules:
- Only use information present in the provided page content
- If the answer is not present in the content, say "Not found on this page"
- Be precise and concise
- For lists, use bullet points
- For tables, use markdown table format
- Do not use any external knowledge"""

FORMAT_NAMES = {
    "html": "raw HTML",
    "markdown": "cleaned Markdown",
    "som": "Semantic Object Model (SOM) JSON",
}

def load_page_content(url_id: str, fmt: str) -> str | None:
    """Load cached page content for a given URL ID and format."""
    ext = {"html": ".html", "markdown": ".md", "som": ".json"}[fmt]
    path = CORPUS / fmt / f"{url_id}{ext}"
    if not path.exists():
        return None
    content = path.read_text(errors="replace")
    # Truncate to avoid exceeding context windows
    MAX_CHARS = {"html": 120_000, "markdown": 60_000, "som": 60_000}
    if len(content) > MAX_CHARS[fmt]:
        content = content[:MAX_CHARS[fmt]] + "\n\n[TRUNCATED]"
    return content


def count_tokens(text: str) -> int:
    """Estimate token count using cl100k_base if available, else approximate."""
    try:
        import tiktoken
        enc = tiktoken.get_encoding("cl100k_base")
        return len(enc.encode(text))
    except ImportError:
        return len(text) // 4  # rough approximation


def call_llm(model: str, system: str, user: str, max_tokens: int = 2048) -> dict:
    """Call an LLM and return response + metadata."""
    start = time.time()
    input_tokens = count_tokens(system + user)

    if model.startswith("gpt") or model.startswith("o1") or model.startswith("o3"):
        client = get_openai_client()
        resp = client.chat.completions.create(
            model=model,
            messages=[
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            temperature=0.0,
            max_tokens=max_tokens,
        )
        answer = resp.choices[0].message.content or ""
        output_tokens = count_tokens(answer)
    elif "claude" in model:
        client = get_anthropic_client()
        resp = client.messages.create(
            model=model,
            max_tokens=max_tokens,
            system=system,
            messages=[{"role": "user", "content": user}],
            temperature=0.0,
        )
        answer = resp.content[0].text if resp.content else ""
        output_tokens = count_tokens(answer)
    else:
        raise ValueError(f"Unknown model: {model}")

    elapsed = time.time() - start

    return {
        "answer": answer,
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "latency_s": round(elapsed, 3),
        "model": model,
    }


def run_task(task: dict, fmt: str, model: str) -> dict:
    """Run a single task in a given format and return the result."""
    url_id = task["url_id"]
    content = load_page_content(url_id, fmt)

    if content is None:
        return {
            "task_id": task["id"],
            "format": fmt,
            "model": model,
            "status": "skipped",
            "reason": f"No cached {fmt} content for {url_id}",
        }

    system = SYSTEM_PROMPT_TEMPLATE.format(format_name=FORMAT_NAMES[fmt])
    user = f"Page content:\n\n{content}\n\n---\n\nQuestion: {task['question']}"

    try:
        result = call_llm(model, system, user)
    except Exception as e:
        return {
            "task_id": task["id"],
            "format": fmt,
            "model": model,
            "status": "error",
            "reason": str(e),
        }

    return {
        "task_id": task["id"],
        "category": task["category"],
        "format": fmt,
        "model": model,
        "url_id": url_id,
        "question": task["question"],
        "answer": result["answer"],
        "input_tokens": result["input_tokens"],
        "output_tokens": result["output_tokens"],
        "latency_s": result["latency_s"],
        "answer_type": task["answer_type"],
        "rubric": task["rubric"],
        "status": "ok",
    }


def main():
    parser = argparse.ArgumentParser(description="WebTaskBench evaluation harness")
    parser.add_argument("--model", default="gpt-4o", help="Model to evaluate")
    parser.add_argument("--format", default="all", choices=["all", "html", "markdown", "som"],
                        help="Format to test (default: all)")
    parser.add_argument("--runs", type=int, default=3, help="Number of runs per task")
    parser.add_argument("--tasks", default=None, help="Comma-separated task IDs (default: all)")
    parser.add_argument("--category", default=None,
                        choices=["extraction", "comparison", "navigation", "summarization", "adversarial"],
                        help="Only run tasks in this category")
    parser.add_argument("--dry-run", action="store_true", help="Print plan without calling APIs")
    args = parser.parse_args()

    tasks = json.loads(TASKS_FILE.read_text())
    urls = {u["id"]: u for u in json.loads(URLS_FILE.read_text())}

    # Filter tasks
    if args.tasks:
        task_ids = set(args.tasks.split(","))
        tasks = [t for t in tasks if t["id"] in task_ids]
    if args.category:
        tasks = [t for t in tasks if t["category"] == args.category]

    formats = ["html", "markdown", "som"] if args.format == "all" else [args.format]

    total_calls = len(tasks) * len(formats) * args.runs
    print(f"WebTaskBench Evaluation")
    print(f"  Model:   {args.model}")
    print(f"  Formats: {', '.join(formats)}")
    print(f"  Tasks:   {len(tasks)}")
    print(f"  Runs:    {args.runs}")
    print(f"  Total API calls: {total_calls}")
    print()

    if args.dry_run:
        print("DRY RUN - no API calls made")
        for t in tasks[:5]:
            for fmt in formats:
                content = load_page_content(t["url_id"], fmt)
                tokens = count_tokens(content) if content else 0
                print(f"  {t['id']} / {fmt}: {tokens:,} input tokens")
        return

    RESULTS_DIR.mkdir(exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    model_slug = args.model.replace("/", "-").replace(".", "-")
    output_file = RESULTS_DIR / f"eval-{model_slug}-{timestamp}.jsonl"

    print(f"Results -> {output_file}")
    print()

    completed = 0
    with open(output_file, "w") as f:
        for run_idx in range(args.runs):
            for task in tasks:
                for fmt in formats:
                    result = run_task(task, fmt, args.model)
                    result["run"] = run_idx + 1
                    result["timestamp"] = datetime.now().isoformat()
                    f.write(json.dumps(result) + "\n")
                    f.flush()

                    completed += 1
                    status = result.get("status", "?")
                    tokens = result.get("input_tokens", 0)
                    latency = result.get("latency_s", 0)

                    print(f"  [{completed}/{total_calls}] {task['id']}/{fmt}/run{run_idx+1}: "
                          f"{status} ({tokens:,} tok, {latency:.1f}s)")

                    # Rate limit: pause between calls
                    if status == "ok":
                        time.sleep(0.5)

    print(f"\nDone. Results in {output_file}")
    print(f"  Total calls: {completed}")


if __name__ == "__main__":
    main()
