#!/usr/bin/env python3
"""
Paper 6: Token Economics sensitivity model.

Estimates annual token waste from HTML noise in agent-web interactions.
All inputs from public sources (Cloudflare Radar, HTTP Archive, WebTaskBench, provider pricing).
"""

import json
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from dataclasses import dataclass

@dataclass
class Params:
    """All model parameters with sources."""

    # Cloudflare: 81M req/sec average
    cloudflare_req_per_sec: float = 81_000_000
    # Cloudflare market share of global web traffic (conservative)
    cloudflare_market_share: float = 0.20
    # AI bot percentage of HTML traffic (Cloudflare Radar 2025)
    ai_bot_html_pct: float = 0.042
    # User-action fraction of AI crawling (estimated from 15x growth, still minority)
    user_action_fraction: float = 0.08
    # Fraction of user-action fetches actually consumed by LLM context
    llm_consumption_fraction: float = 0.35
    # Average HTML tokens per page (WebTaskBench)
    avg_html_tokens: int = 33_181
    # Average SOM tokens per page (WebTaskBench)
    avg_som_tokens: int = 8_301
    # Fraction of agents using Markdown preprocessing (framework survey)
    markdown_adoption: float = 0.40
    # Markdown waste reduction (Markdown still has some waste vs SOM)
    markdown_waste_reduction: float = 0.70
    # Fraction of agents truncating HTML
    truncation_adoption: float = 0.30
    # Truncation waste reduction
    truncation_waste_reduction: float = 0.30
    # Cache/dedup reduction
    cache_reduction: float = 0.15
    # Weighted average LLM input price ($/M tokens)
    avg_price_per_m_tokens: float = 0.93


def compute(p: Params) -> dict:
    """Run the model and return all intermediate values."""

    # Daily requests through Cloudflare
    cf_daily = p.cloudflare_req_per_sec * 86400

    # AI bot HTML requests through Cloudflare per day
    ai_daily_cf = cf_daily * p.ai_bot_html_pct

    # Scale to global
    ai_daily_global = ai_daily_cf / p.cloudflare_market_share

    # User-action subset
    user_action_daily = ai_daily_global * p.user_action_fraction

    # LLM-consumed pages per day
    llm_pages_daily = user_action_daily * p.llm_consumption_fraction

    # Per-page waste (HTML tokens minus SOM tokens)
    raw_waste_per_page = p.avg_html_tokens - p.avg_som_tokens

    # Adjust for current preprocessing
    # Agents using Markdown: waste is reduced by markdown_waste_reduction
    # Agents using truncation: waste is reduced by truncation_waste_reduction
    # Agents using neither: full waste
    neither_fraction = max(0, 1.0 - p.markdown_adoption - p.truncation_adoption)
    effective_waste = (
        p.markdown_adoption * raw_waste_per_page * (1 - p.markdown_waste_reduction)
        + p.truncation_adoption * raw_waste_per_page * (1 - p.truncation_waste_reduction)
        + neither_fraction * raw_waste_per_page
    )
    # Cache reduction
    effective_waste *= (1 - p.cache_reduction)

    # Daily and annual waste
    daily_waste_tokens = llm_pages_daily * effective_waste
    annual_waste_tokens = daily_waste_tokens * 365

    # Cost
    annual_cost = annual_waste_tokens * p.avg_price_per_m_tokens / 1_000_000

    return {
        "cf_daily_requests": cf_daily,
        "ai_daily_cf": ai_daily_cf,
        "ai_daily_global": ai_daily_global,
        "user_action_daily": user_action_daily,
        "llm_pages_daily": llm_pages_daily,
        "raw_waste_per_page": raw_waste_per_page,
        "effective_waste_per_page": effective_waste,
        "daily_waste_tokens": daily_waste_tokens,
        "annual_waste_tokens": annual_waste_tokens,
        "annual_cost_usd": annual_cost,
    }


def sensitivity_analysis(base: Params, outdir: str):
    """Vary each parameter and measure impact on annual cost."""

    base_result = compute(base)
    base_cost = base_result["annual_cost_usd"]

    params_to_vary = [
        ("user_action_fraction", 0.03, 0.15, "User-action fraction of AI crawling"),
        ("llm_consumption_fraction", 0.15, 0.60, "LLM consumption fraction"),
        ("markdown_adoption", 0.20, 0.70, "Markdown adoption rate"),
        ("avg_price_per_m_tokens", 0.30, 2.50, "Avg LLM price ($/M tokens)"),
        ("cloudflare_market_share", 0.10, 0.30, "Cloudflare market share"),
        ("cache_reduction", 0.05, 0.40, "Cache/dedup reduction"),
    ]

    labels = []
    low_deltas = []
    high_deltas = []

    for attr, low_val, high_val, label in params_to_vary:
        # Low value
        p_low = Params(**{k: getattr(base, k) for k in vars(base)})
        setattr(p_low, attr, low_val)
        cost_low = compute(p_low)["annual_cost_usd"]

        # High value
        p_high = Params(**{k: getattr(base, k) for k in vars(base)})
        setattr(p_high, attr, high_val)
        cost_high = compute(p_high)["annual_cost_usd"]

        labels.append(f"{label}\n({low_val} to {high_val})")
        low_deltas.append(cost_low - base_cost)
        high_deltas.append(cost_high - base_cost)

    # Sort by total range
    ranges = [abs(h - l) for h, l in zip(high_deltas, low_deltas)]
    order = sorted(range(len(ranges)), key=lambda i: ranges[i], reverse=True)

    labels = [labels[i] for i in order]
    low_deltas = [low_deltas[i] for i in order]
    high_deltas = [high_deltas[i] for i in order]

    # Tornado chart
    fig, ax = plt.subplots(figsize=(10, 5), dpi=160)
    y = range(len(labels))

    ax.barh(y, high_deltas, color="#E8853A", alpha=0.8, label="High value")
    ax.barh(y, low_deltas, color="#3D8FD4", alpha=0.8, label="Low value")

    ax.set_yticks(list(y))
    ax.set_yticklabels(labels, fontsize=8)
    ax.set_xlabel("Change in annual cost estimate (USD)")
    ax.set_title(f"Sensitivity analysis (base estimate: ${base_cost/1e9:.2f}B/year)")
    ax.axvline(0, color="black", linewidth=0.5)
    ax.legend(frameon=False)
    ax.grid(axis="x", alpha=0.2)
    fig.tight_layout()
    fig.savefig(f"{outdir}/sensitivity_tornado.png")
    fig.savefig(f"{outdir}/sensitivity_tornado.svg")
    plt.close(fig)


def token_flow_chart(result: dict, params: Params, outdir: str):
    """Bar chart showing token flow: total HTML -> noise vs content."""

    fig, ax = plt.subplots(figsize=(7, 4), dpi=160)

    total = params.avg_html_tokens
    content = params.avg_som_tokens
    noise = total - content

    bars = ax.bar(["Content\n(preserved in SOM)", "Noise\n(wasted tokens)"],
                  [content, noise],
                  color=["#3D8FD4", "#E8853A"])

    ax.set_ylabel("Tokens per page")
    ax.set_title(f"Token composition of an average web page ({total:,} total)")
    ax.set_ylim(0, total * 1.15)

    for b, v in zip(bars, [content, noise]):
        pct = v / total * 100
        ax.text(b.get_x() + b.get_width() / 2, v + total * 0.02,
                f"{v:,}\n({pct:.0f}%)", ha="center", va="bottom", fontsize=9)

    ax.grid(axis="y", alpha=0.2)
    fig.tight_layout()
    fig.savefig(f"{outdir}/token_composition.png")
    fig.savefig(f"{outdir}/token_composition.svg")
    plt.close(fig)


def annual_cost_scenarios(outdir: str):
    """Bar chart showing cost under conservative, middle, aggressive scenarios."""

    scenarios = {
        "Conservative": Params(user_action_fraction=0.04, llm_consumption_fraction=0.20,
                               markdown_adoption=0.55, avg_price_per_m_tokens=0.50),
        "Middle": Params(),  # defaults
        "Aggressive": Params(user_action_fraction=0.12, llm_consumption_fraction=0.50,
                            markdown_adoption=0.25, avg_price_per_m_tokens=1.50),
    }

    names = list(scenarios.keys())
    costs = [compute(p)["annual_cost_usd"] / 1e9 for p in scenarios.values()]

    fig, ax = plt.subplots(figsize=(6, 4), dpi=160)
    colors = ["#3D8FD4", "#E8853A", "#d46060"]
    bars = ax.bar(names, costs, color=colors)

    ax.set_ylabel("Annual cost ($ billions)")
    ax.set_title("Estimated annual cost of HTML noise in agent workloads")
    ax.set_ylim(0, max(costs) * 1.2)

    for b, v in zip(bars, costs):
        ax.text(b.get_x() + b.get_width() / 2, v + max(costs) * 0.02,
                f"${v:.1f}B", ha="center", va="bottom", fontsize=10, fontweight="bold")

    ax.grid(axis="y", alpha=0.2)
    fig.tight_layout()
    fig.savefig(f"{outdir}/annual_cost_scenarios.png")
    fig.savefig(f"{outdir}/annual_cost_scenarios.svg")
    plt.close(fig)


if __name__ == "__main__":
    import os

    outdir = os.path.join(os.path.dirname(__file__), "figures")
    os.makedirs(outdir, exist_ok=True)

    # Obsidian output too
    obsidian_dir = os.path.expanduser(
        "~/Library/Mobile Documents/iCloud~md~obsidian/Documents/"
        "DBH-Vault/Projects/Plasmate/papers/assets/paper6"
    )
    os.makedirs(obsidian_dir, exist_ok=True)

    base = Params()
    result = compute(base)

    print("=== Paper 6: Token Economics Model ===\n")
    print(f"Cloudflare daily requests:     {result['cf_daily_requests']:,.0f}")
    print(f"AI bot daily (Cloudflare):     {result['ai_daily_cf']:,.0f}")
    print(f"AI bot daily (global est):     {result['ai_daily_global']:,.0f}")
    print(f"User-action daily (global):    {result['user_action_daily']:,.0f}")
    print(f"LLM-consumed pages/day:        {result['llm_pages_daily']:,.0f}")
    print(f"Raw waste per page:            {result['raw_waste_per_page']:,} tokens")
    print(f"Effective waste per page:       {result['effective_waste_per_page']:,.0f} tokens")
    print(f"Daily waste:                   {result['daily_waste_tokens']:,.0f} tokens")
    print(f"Annual waste:                  {result['annual_waste_tokens']:.2e} tokens")
    print(f"Annual cost:                   ${result['annual_cost_usd']:,.0f}")
    print(f"Annual cost (billions):        ${result['annual_cost_usd']/1e9:.2f}B")

    # Generate figures
    sensitivity_analysis(base, outdir)
    token_flow_chart(result, base, outdir)
    annual_cost_scenarios(outdir)

    # Copy to obsidian
    for f in os.listdir(outdir):
        if f.endswith(".png"):
            import shutil
            shutil.copy(os.path.join(outdir, f), os.path.join(obsidian_dir, f))

    print(f"\nFigures written to {outdir} and {obsidian_dir}")
