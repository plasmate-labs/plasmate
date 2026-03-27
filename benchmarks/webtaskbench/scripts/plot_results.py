#!/usr/bin/env python3
"""Generate paper-ready plots from WebTaskBench JSONL results.

Usage:
  python3 plot_results.py \
    --gpt results/eval-gpt-4o-20260326-175217.jsonl \
    --claude results/eval-claude-sonnet-4-20250514-20260326-234453.jsonl \
    --out /path/to/outdir

Outputs PNG + SVG.
"""

import argparse
import json
from pathlib import Path
from collections import defaultdict

import matplotlib
import matplotlib.pyplot as plt

COLORS = {
    "html": "#6B6560",      # ash
    "markdown": "#3D8FD4",  # arc
    "som": "#E8853A",       # ember
}

FORMATS = ["html", "markdown", "som"]
CATS = ["extraction", "comparison", "navigation", "summarization", "adversarial"]


def load_ok(path: Path):
    rows = []
    with open(path) as f:
        for line in f:
            r = json.loads(line)
            if r.get("status") == "ok":
                rows.append(r)
    return rows


def mean(xs):
    xs = list(xs)
    return sum(xs) / len(xs) if xs else 0.0


def plot_tokens_by_format(rows, title, outdir: Path, basename: str):
    by_fmt = defaultdict(list)
    for r in rows:
        by_fmt[r["format"]].append(r.get("input_tokens", 0))

    vals = [mean(by_fmt[f]) for f in FORMATS]

    fig, ax = plt.subplots(figsize=(6.2, 3.6), dpi=160)
    bars = ax.bar(FORMATS, vals, color=[COLORS[f] for f in FORMATS])

    ax.set_title(title)
    ax.set_ylabel("Average input tokens")
    ax.set_ylim(0, max(vals) * 1.15)

    for b, v in zip(bars, vals):
        ax.text(b.get_x() + b.get_width() / 2, v + max(vals) * 0.02, f"{int(round(v)):,}",
                ha="center", va="bottom", fontsize=9)

    ax.grid(axis="y", alpha=0.25)
    fig.tight_layout()

    fig.savefig(outdir / f"{basename}.png")
    fig.savefig(outdir / f"{basename}.svg")
    plt.close(fig)


def plot_latency_by_format(rows_gpt, rows_claude, outdir: Path):
    def agg(rows):
        by_fmt = defaultdict(list)
        for r in rows:
            by_fmt[r["format"]].append(r.get("latency_s", 0))
        return [mean(by_fmt[f]) for f in FORMATS]

    gpt = agg(rows_gpt)
    claude = agg(rows_claude)

    x = range(len(FORMATS))
    width = 0.38

    fig, ax = plt.subplots(figsize=(7.4, 3.8), dpi=160)

    ax.bar([i - width / 2 for i in x], gpt, width, label="GPT-4o", color="#222222")
    ax.bar([i + width / 2 for i in x], claude, width, label="Claude Sonnet 4", color="#777777")

    ax.set_title("Average latency by format")
    ax.set_ylabel("Seconds")
    ax.set_xticks(list(x), FORMATS)
    ax.grid(axis="y", alpha=0.25)
    ax.legend(frameon=False)

    # Annotate
    for i, v in enumerate(gpt):
        ax.text(i - width / 2, v + max(claude) * 0.02, f"{v:.1f}s", ha="center", va="bottom", fontsize=8)
    for i, v in enumerate(claude):
        ax.text(i + width / 2, v + max(claude) * 0.02, f"{v:.1f}s", ha="center", va="bottom", fontsize=8)

    fig.tight_layout()
    fig.savefig(outdir / "latency_by_format.png")
    fig.savefig(outdir / "latency_by_format.svg")
    plt.close(fig)


def plot_category_token_ratios(rows, outdir: Path):
    # Plot compression ratios per category: HTML/SOM and HTML/Markdown
    by_cat_fmt = defaultdict(lambda: defaultdict(list))
    for r in rows:
        by_cat_fmt[r.get("category", "unknown")][r["format"]].append(r.get("input_tokens", 0))

    html_som = []
    html_md = []
    for cat in CATS:
        html = mean(by_cat_fmt[cat]["html"])
        som = mean(by_cat_fmt[cat]["som"])
        md = mean(by_cat_fmt[cat]["markdown"])
        html_som.append(html / som if som else 0)
        html_md.append(html / md if md else 0)

    x = range(len(CATS))
    width = 0.38

    fig, ax = plt.subplots(figsize=(8.2, 3.9), dpi=160)

    ax.bar([i - width / 2 for i in x], html_som, width, label="HTML / SOM", color=COLORS["som"])
    ax.bar([i + width / 2 for i in x], html_md, width, label="HTML / Markdown", color=COLORS["markdown"])

    ax.set_title("Token compression ratios by category")
    ax.set_ylabel("Compression ratio (higher is better)")
    ax.set_xticks(list(x), [c.replace("summarization", "summary") for c in CATS], rotation=18, ha="right")
    ax.grid(axis="y", alpha=0.25)
    ax.legend(frameon=False)

    fig.tight_layout()
    fig.savefig(outdir / "compression_by_category.png")
    fig.savefig(outdir / "compression_by_category.svg")
    plt.close(fig)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gpt", required=True)
    ap.add_argument("--claude", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)

    rows_gpt = load_ok(Path(args.gpt))
    rows_claude = load_ok(Path(args.claude))

    plot_tokens_by_format(rows_gpt, "Average input tokens by format", outdir, "tokens_by_format")
    plot_latency_by_format(rows_gpt, rows_claude, outdir)
    plot_category_token_ratios(rows_gpt, outdir)

    print("Wrote plots to", outdir)


if __name__ == "__main__":
    main()
