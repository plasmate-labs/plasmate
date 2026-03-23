"""Helper functions for the Plasmate Browser Use integration."""

from typing import Any, Optional


def estimate_tokens(text: str) -> int:
    """Estimate token count for a string (roughly 4 chars per token)."""
    return len(text.encode("utf-8")) // 4


def token_count_comparison(
    som_data: dict[str, Any],
    som_text: Optional[str] = None,
) -> dict[str, Any]:
    """Compare token counts between raw HTML and SOM representation.

    Useful for benchmarking and demonstrating the token savings Plasmate
    provides over traditional browser backends.

    Args:
        som_data: A SOM document dict (as returned by Plasmate).
        som_text: Pre-rendered SOM text. If not provided, token estimate
            is based on the SOM bytes from metadata.

    Returns:
        Dict with token counts and savings ratio::

            {
                "html_bytes": 87234,
                "som_bytes": 4521,
                "html_tokens_est": 21808,
                "som_tokens_est": 1130,
                "byte_ratio": 19.3,
                "token_ratio": 19.3,
                "token_savings_pct": 94.8,
            }
    """
    meta = som_data.get("meta", {})
    html_bytes = meta.get("html_bytes", 0)
    som_bytes = meta.get("som_bytes", 0)

    if som_text is not None:
        som_text_bytes = len(som_text.encode("utf-8"))
    else:
        som_text_bytes = som_bytes

    # Token estimation: ~4 chars per token (standard heuristic)
    html_tokens = html_bytes // 4
    som_tokens = som_text_bytes // 4

    byte_ratio = html_bytes / som_bytes if som_bytes else 0
    token_ratio = html_tokens / som_tokens if som_tokens else 0
    savings_pct = (1 - som_tokens / html_tokens) * 100 if html_tokens else 0

    return {
        "html_bytes": html_bytes,
        "som_bytes": som_bytes,
        "som_text_bytes": som_text_bytes,
        "html_tokens_est": html_tokens,
        "som_tokens_est": som_tokens,
        "byte_ratio": round(byte_ratio, 1),
        "token_ratio": round(token_ratio, 1),
        "token_savings_pct": round(savings_pct, 1),
    }
