#!/usr/bin/env python3
"""
Analyze WebTaskBench results and produce paper-ready tables.
Usage:
    python3 analyze.py results/eval-gpt-4o-*.jsonl results/eval-claude-*.jsonl
"""

import json
import sys
from pathlib import Path
from collections import defaultdict

def load_results(paths):
    results = []
    for p in paths:
        with open(p) as f:
            for line in f:
                r = json.loads(line)
                if r.get("status") == "ok":
                    results.append(r)
    return results

def analyze(results):
    # Group by model
    by_model = defaultdict(list)
    for r in results:
        by_model[r["model"]].append(r)

    for model, model_results in sorted(by_model.items()):
        print(f"\n{'='*70}")
        print(f"MODEL: {model}")
        print(f"{'='*70}")
        print(f"Total successful calls: {len(model_results)}")

        # --- Table 1: Format-level summary ---
        print(f"\n--- Table 1: Token Consumption by Format ---")
        by_fmt = defaultdict(lambda: {"count": 0, "input_tokens": 0, "output_tokens": 0, "latency": 0})
        for r in model_results:
            fmt = r["format"]
            by_fmt[fmt]["count"] += 1
            by_fmt[fmt]["input_tokens"] += r.get("input_tokens", 0)
            by_fmt[fmt]["output_tokens"] += r.get("output_tokens", 0)
            by_fmt[fmt]["latency"] += r.get("latency_s", 0)

        print(f"{'Format':<12} {'n':<6} {'Avg In Tok':<14} {'Avg Out Tok':<14} {'Avg Latency':<12} {'Total In Tok':<14}")
        print("-" * 72)
        html_avg = 0
        for fmt in ["html", "markdown", "som"]:
            d = by_fmt[fmt]
            n = d["count"]
            if n == 0: continue
            avg_in = d["input_tokens"] // n
            avg_out = d["output_tokens"] // n
            avg_lat = d["latency"] / n
            if fmt == "html": html_avg = avg_in
            ratio = f"({html_avg/avg_in:.1f}x)" if html_avg and fmt != "html" else ""
            print(f"{fmt:<12} {n:<6} {avg_in:>10,}    {avg_out:>10,}    {avg_lat:>8.1f}s    {d['input_tokens']:>12,}  {ratio}")

        # --- Table 2: By category x format ---
        print(f"\n--- Table 2: Avg Input Tokens by Category x Format ---")
        cats = ["extraction", "comparison", "navigation", "summarization", "adversarial"]
        by_cat_fmt = defaultdict(lambda: defaultdict(lambda: {"count": 0, "tokens": 0, "latency": 0}))
        for r in model_results:
            cat = r.get("category", "unknown")
            fmt = r["format"]
            by_cat_fmt[cat][fmt]["count"] += 1
            by_cat_fmt[cat][fmt]["tokens"] += r.get("input_tokens", 0)
            by_cat_fmt[cat][fmt]["latency"] += r.get("latency_s", 0)

        print(f"{'Category':<16} {'HTML':<12} {'Markdown':<12} {'SOM':<12} {'HTML/SOM':<10}")
        print("-" * 62)
        for cat in cats:
            vals = {}
            for fmt in ["html", "markdown", "som"]:
                d = by_cat_fmt[cat][fmt]
                vals[fmt] = d["tokens"] // max(d["count"], 1)
            ratio = f"{vals['html']/vals['som']:.1f}x" if vals["som"] > 0 else "n/a"
            print(f"{cat:<16} {vals['html']:>8,}    {vals['markdown']:>8,}    {vals['som']:>8,}    {ratio}")

        # --- Table 3: Latency by category x format ---
        print(f"\n--- Table 3: Avg Latency (s) by Category x Format ---")
        print(f"{'Category':<16} {'HTML':<10} {'Markdown':<10} {'SOM':<10}")
        print("-" * 46)
        for cat in cats:
            vals = {}
            for fmt in ["html", "markdown", "som"]:
                d = by_cat_fmt[cat][fmt]
                vals[fmt] = d["latency"] / max(d["count"], 1)
            print(f"{cat:<16} {vals['html']:>6.1f}s    {vals['markdown']:>6.1f}s    {vals['som']:>6.1f}s")

        # --- Table 4: Output token analysis (answer verbosity) ---
        print(f"\n--- Table 4: Avg Output Tokens by Format ---")
        for fmt in ["html", "markdown", "som"]:
            d = by_fmt[fmt]
            n = d["count"]
            if n == 0: continue
            avg_out = d["output_tokens"] // n
            print(f"  {fmt:<12} {avg_out:>6,} tokens")

        # --- Cost estimate ---
        print(f"\n--- Cost Estimate ---")
        # GPT-4o: $2.50/1M input, $10/1M output
        # Claude 3.5 Sonnet: $3/1M input, $15/1M output
        if "gpt" in model:
            in_rate, out_rate = 2.50, 10.00
        else:
            in_rate, out_rate = 3.00, 15.00

        for fmt in ["html", "markdown", "som"]:
            d = by_fmt[fmt]
            in_cost = d["input_tokens"] * in_rate / 1_000_000
            out_cost = d["output_tokens"] * out_rate / 1_000_000
            total = in_cost + out_cost
            print(f"  {fmt:<12} ${total:>7.2f}  (in: ${in_cost:.2f}, out: ${out_cost:.2f})")

        total_in = sum(d["input_tokens"] for d in by_fmt.values())
        total_out = sum(d["output_tokens"] for d in by_fmt.values())
        total_cost = total_in * in_rate / 1_000_000 + total_out * out_rate / 1_000_000
        print(f"  {'TOTAL':<12} ${total_cost:>7.2f}")


    # --- Cross-model comparison ---
    if len(by_model) > 1:
        print(f"\n{'='*70}")
        print(f"CROSS-MODEL COMPARISON")
        print(f"{'='*70}")
        print(f"\n{'Model':<35} {'Format':<12} {'Avg In Tok':<14} {'Avg Lat':<10}")
        print("-" * 71)
        for model in sorted(by_model.keys()):
            for fmt in ["html", "markdown", "som"]:
                fmtr = [r for r in by_model[model] if r["format"] == fmt]
                if not fmtr: continue
                avg_in = sum(r.get("input_tokens", 0) for r in fmtr) // len(fmtr)
                avg_lat = sum(r.get("latency_s", 0) for r in fmtr) / len(fmtr)
                print(f"{model:<35} {fmt:<12} {avg_in:>10,}    {avg_lat:>6.1f}s")
            print()


if __name__ == "__main__":
    if len(sys.argv) < 2:
        # Auto-discover
        results_dir = Path(__file__).parent.parent / "results"
        files = sorted(results_dir.glob("eval-*.jsonl"))
        if not files:
            print("No result files found. Pass paths as arguments.")
            sys.exit(1)
        print(f"Auto-discovered {len(files)} result files")
    else:
        files = [Path(p) for p in sys.argv[1:]]

    results = load_results(files)
    print(f"Loaded {len(results)} successful results from {len(files)} files")
    analyze(results)
