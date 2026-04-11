#!/bin/bash
# Plasmate Video Demo - Terminal Commands
# Run these commands in sequence for the video recording

# ============================================
# SETUP (run before recording)
# ============================================

# Build plasmate if not already built
cargo build --release

# Verify plasmate is working
./target/release/plasmate --version

# Pre-fetch Linear.app to cache (optional, for faster demo)
curl -s https://linear.app > /tmp/linear-raw.html
./target/release/plasmate fetch https://linear.app > /tmp/linear-som.json

# ============================================
# DEMO COMMANDS (record these)
# ============================================

# --- Part 1: Show the problem (raw HTML size) ---

# Show raw HTML size from Linear.app
echo "=== Raw HTML Size ==="
curl -s https://linear.app | wc -c
# Expected output: ~2,200,000 (2.2MB)

# Show how messy raw HTML looks
echo "=== Raw HTML Preview ==="
curl -s https://linear.app | head -100

# --- Part 2: Show the solution (SOM compression) ---

# Show SOM output size
echo "=== SOM Size ==="
./target/release/plasmate fetch https://linear.app | wc -c
# Expected output: ~21,000 (21KB)

# Calculate compression ratio
echo "=== Compression Ratio ==="
HTML_SIZE=$(curl -s https://linear.app | wc -c)
SOM_SIZE=$(./target/release/plasmate fetch https://linear.app | wc -c)
echo "HTML: $HTML_SIZE bytes"
echo "SOM:  $SOM_SIZE bytes"
echo "Compression: $(echo "scale=0; $HTML_SIZE / $SOM_SIZE" | bc)x"

# --- Part 3: Show what SOM looks like ---

# Show the clean SOM JSON output
echo "=== SOM Preview (JSON) ==="
./target/release/plasmate fetch https://linear.app | head -50

# Show text-only extraction
echo "=== SOM Preview (Text) ==="
./target/release/plasmate fetch https://linear.app --text | head -30

# --- Part 4: Show the install/usage ---

# Python install (for pip users)
echo "=== Install ==="
pip install plasmate

# Or cargo install for Rust users
cargo install plasmate

# MCP mode for Claude/Cursor integration
./target/release/plasmate mcp

# ============================================
# ALTERNATE SITES FOR DEMO
# ============================================

# NYT (usually high compression)
# ./target/release/plasmate fetch https://nytimes.com | wc -c

# Hacker News (simpler site, still good compression)
# ./target/release/plasmate fetch https://news.ycombinator.com | wc -c

# Wikipedia (content-rich)
# ./target/release/plasmate fetch https://en.wikipedia.org/wiki/Artificial_intelligence | wc -c
