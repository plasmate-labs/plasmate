#!/usr/bin/env bash
# Plasmate vs Chrome - Live Demo Script
# Run this while screen recording for a compelling comparison video
# Usage: ./demo-comparison.sh [URL]

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

URL="${1:-https://www.bbc.com/news}"

count_tokens() {
  uv run --with tiktoken --quiet python3 -c "
import sys, tiktoken
enc = tiktoken.get_encoding('cl100k_base')
print(len(enc.encode(sys.stdin.read())))
" 2>/dev/null
}

clear
echo ""
echo -e "${BOLD}${CYAN}  Plasmate vs Chrome Headless${NC}"
echo -e "${CYAN}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "  Target: ${YELLOW}$URL${NC}"
echo ""
sleep 2

# Chrome
echo -e "${RED}${BOLD}  Chrome Headless${NC}"
echo -e "  ${RED}─────────────────${NC}"
echo ""
echo -e "  Starting Chrome..."

CHROME_BIN="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
CHROME_START=$(python3 -c "import time; print(time.time())")
CHROME_OUTPUT=$("$CHROME_BIN" --headless=new --dump-dom --no-sandbox --disable-gpu "$URL" 2>/dev/null || echo "")
CHROME_END=$(python3 -c "import time; print(time.time())")
CHROME_TIME=$(python3 -c "print(f'{${CHROME_END} - ${CHROME_START}:.2f}')")
CHROME_BYTES=$(echo "$CHROME_OUTPUT" | wc -c | tr -d ' ')
CHROME_TOKENS=$(echo "$CHROME_OUTPUT" | count_tokens)

echo -e "  ${RED}Time:${NC}    ${CHROME_TIME}s"
echo -e "  ${RED}Output:${NC}  ${CHROME_BYTES} bytes"
echo -e "  ${RED}Tokens:${NC}  ${CHROME_TOKENS}"
echo ""
echo -e "  ${RED}Output preview (raw HTML):${NC}"
echo "$CHROME_OUTPUT" | head -3 | cut -c1-120 | sed 's/^/  | /'
echo "  | ... (${CHROME_BYTES} bytes total)"
echo ""
sleep 2

# Plasmate
echo -e "${GREEN}${BOLD}  Plasmate SOM${NC}"
echo -e "  ${GREEN}────────────────${NC}"
echo ""
echo -e "  Starting Plasmate..."

PLASMATE_START=$(python3 -c "import time; print(time.time())")
PLASMATE_OUTPUT=$(plasmate fetch "$URL" 2>/dev/null || echo "Plasmate failed")
PLASMATE_END=$(python3 -c "import time; print(time.time())")
PLASMATE_TIME=$(python3 -c "print(f'{${PLASMATE_END} - ${PLASMATE_START}:.2f}')")
PLASMATE_BYTES=$(echo "$PLASMATE_OUTPUT" | wc -c | tr -d ' ')
PLASMATE_TOKENS=$(echo "$PLASMATE_OUTPUT" | count_tokens)

echo -e "  ${GREEN}Time:${NC}    ${PLASMATE_TIME}s"
echo -e "  ${GREEN}Output:${NC}  ${PLASMATE_BYTES} bytes"
echo -e "  ${GREEN}Tokens:${NC}  ${PLASMATE_TOKENS}"
echo ""
echo -e "  ${GREEN}Output preview (structured SOM):${NC}"
echo "$PLASMATE_OUTPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d, indent=2)[:600])" 2>/dev/null | head -18 | sed 's/^/  | /'
echo "  | ..."
echo ""
sleep 2

# Summary
echo -e "${BOLD}${YELLOW}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}${YELLOW}  Results${NC}"
echo -e "${YELLOW}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ "${CHROME_TOKENS:-0}" -gt 0 ] && [ "${PLASMATE_TOKENS:-0}" -gt 0 ]; then
  RATIO=$(python3 -c "print(f'{int($CHROME_TOKENS) / int($PLASMATE_TOKENS):.1f}')")
  SAVINGS=$(python3 -c "print(f'{(1 - int($PLASMATE_TOKENS) / int($CHROME_TOKENS)) * 100:.0f}')")
  SPEED=$(python3 -c "print(f'{float($CHROME_TIME) / max(float($PLASMATE_TIME), 0.01):.0f}')")

  printf "  %-12s %10s tokens   %6ss\n" "Chrome:" "$CHROME_TOKENS" "$CHROME_TIME"
  printf "  %-12s %10s tokens   %6ss\n" "Plasmate:" "$PLASMATE_TOKENS" "$PLASMATE_TIME"
  echo ""
  echo -e "  ${BOLD}Token compression:  ${CYAN}${RATIO}x fewer tokens${NC}"
  echo -e "  ${BOLD}Token savings:      ${CYAN}${SAVINGS}%${NC}"
  echo -e "  ${BOLD}Speed improvement:  ${CYAN}${SPEED}x faster${NC}"

  GPT4_CHROME=$(python3 -c "print(f'{int($CHROME_TOKENS) * 30 / 1000000:.4f}')")
  GPT4_PLASMATE=$(python3 -c "print(f'{int($PLASMATE_TOKENS) * 30 / 1000000:.4f}')")
  MONTHLY_CHROME=$(python3 -c "print(f'{int($CHROME_TOKENS) * 30 / 1000000 * 1000000:.0f}')")
  MONTHLY_PLASMATE=$(python3 -c "print(f'{int($PLASMATE_TOKENS) * 30 / 1000000 * 1000000:.0f}')")
  echo ""
  echo -e "  ${BOLD}Cost per page (GPT-4, \$30/1M tokens):${NC}"
  echo -e "    Chrome:   ${RED}\$${GPT4_CHROME}${NC}"
  echo -e "    Plasmate: ${GREEN}\$${GPT4_PLASMATE}${NC}"
  echo ""
  echo -e "  ${BOLD}At 1M pages/month:${NC}"
  echo -e "    Chrome:   ${RED}\$${MONTHLY_CHROME}${NC}"
  echo -e "    Plasmate: ${GREEN}\$${MONTHLY_PLASMATE}${NC}"
fi

echo ""
echo -e "  ${CYAN}https://plasmate.app | Apache 2.0${NC}"
echo -e "  ${CYAN}pip install plasmate | cargo install plasmate${NC}"
echo ""
