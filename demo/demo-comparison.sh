#!/usr/bin/env bash
# Plasmate vs Chrome - Live Demo Script
# Run this while screen recording for a compelling comparison video

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

URL="${1:-https://news.ycombinator.com}"

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

CHROME_BIN="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"

echo -e "  Starting Chrome..."
CHROME_START=$(python3 -c "import time; print(time.time())")

CHROME_OUTPUT=$("$CHROME_BIN" --headless=new --dump-dom --no-sandbox --disable-gpu "$URL" 2>/dev/null || echo "Chrome failed")

CHROME_END=$(python3 -c "import time; print(time.time())")
CHROME_TIME=$(python3 -c "print(f'{${CHROME_END} - ${CHROME_START}:.2f}')")
CHROME_BYTES=$(echo "$CHROME_OUTPUT" | wc -c | tr -d ' ')
CHROME_TOKENS=$(echo "$CHROME_OUTPUT" | python3 -c "
import sys
try:
    import tiktoken
    enc = tiktoken.get_encoding('cl100k_base')
    print(len(enc.encode(sys.stdin.read())))
except:
    text = sys.stdin.read()
    print(len(text.split()) * 4 // 3)
" 2>/dev/null)
CHROME_LINES=$(echo "$CHROME_OUTPUT" | wc -l | tr -d ' ')

echo -e "  ${RED}Time:${NC}    ${CHROME_TIME}s"
echo -e "  ${RED}Output:${NC}  ${CHROME_BYTES} bytes"
echo -e "  ${RED}Lines:${NC}   ${CHROME_LINES}"
echo -e "  ${RED}Tokens:${NC}  ${CHROME_TOKENS}"
echo ""
echo -e "  ${RED}First 5 lines of output:${NC}"
echo "$CHROME_OUTPUT" | head -5 | sed 's/^/  | /'
echo "  | ..."

echo ""
sleep 2

# Plasmate
echo -e "${GREEN}${BOLD}  Plasmate SOM${NC}"
echo -e "  ${GREEN}────────────────${NC}"
echo ""

echo -e "  Starting Plasmate..."
PLASMATE_START=$(python3 -c "import time; print(time.time())")

PLASMATE_OUTPUT=$(plasmate som --url "$URL" --format json 2>/dev/null || echo "Plasmate failed")

PLASMATE_END=$(python3 -c "import time; print(time.time())")
PLASMATE_TIME=$(python3 -c "print(f'{${PLASMATE_END} - ${PLASMATE_START}:.2f}')")
PLASMATE_BYTES=$(echo "$PLASMATE_OUTPUT" | wc -c | tr -d ' ')
PLASMATE_TOKENS=$(echo "$PLASMATE_OUTPUT" | python3 -c "
import sys
try:
    import tiktoken
    enc = tiktoken.get_encoding('cl100k_base')
    print(len(enc.encode(sys.stdin.read())))
except:
    text = sys.stdin.read()
    print(len(text.split()) * 4 // 3)
" 2>/dev/null)
PLASMATE_LINES=$(echo "$PLASMATE_OUTPUT" | wc -l | tr -d ' ')

echo -e "  ${GREEN}Time:${NC}    ${PLASMATE_TIME}s"
echo -e "  ${GREEN}Output:${NC}  ${PLASMATE_BYTES} bytes"
echo -e "  ${GREEN}Lines:${NC}   ${PLASMATE_LINES}"
echo -e "  ${GREEN}Tokens:${NC}  ${PLASMATE_TOKENS}"
echo ""
echo -e "  ${GREEN}First 10 lines of SOM:${NC}"
echo "$PLASMATE_OUTPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d, indent=2)[:500])" 2>/dev/null | head -15 | sed 's/^/  | /'
echo "  | ..."

echo ""
sleep 2

# Summary
echo -e "${BOLD}${YELLOW}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}${YELLOW}  Results${NC}"
echo -e "${YELLOW}  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ "$CHROME_TOKENS" -gt 0 ] && [ "$PLASMATE_TOKENS" -gt 0 ]; then
  RATIO=$(python3 -c "print(f'{int($CHROME_TOKENS) / int($PLASMATE_TOKENS):.1f}')")
  SAVINGS=$(python3 -c "print(f'{(1 - int($PLASMATE_TOKENS) / int($CHROME_TOKENS)) * 100:.0f}')")

  echo -e "  ${RED}Chrome:${NC}   ${CHROME_TOKENS} tokens in ${CHROME_TIME}s"
  echo -e "  ${GREEN}Plasmate:${NC} ${PLASMATE_TOKENS} tokens in ${PLASMATE_TIME}s"
  echo ""
  echo -e "  ${BOLD}Compression: ${CYAN}${RATIO}x fewer tokens${NC}"
  echo -e "  ${BOLD}Token savings: ${CYAN}${SAVINGS}%${NC}"

  GPT4_CHROME=$(python3 -c "print(f'\${int($CHROME_TOKENS) * 30 / 1000000:.4f}')")
  GPT4_PLASMATE=$(python3 -c "print(f'\${int($PLASMATE_TOKENS) * 30 / 1000000:.4f}')")
  echo ""
  echo -e "  ${BOLD}Cost per page (GPT-4):${NC}"
  echo -e "    Chrome:   ${RED}$GPT4_CHROME${NC}"
  echo -e "    Plasmate: ${GREEN}$GPT4_PLASMATE${NC}"
fi

echo ""
echo -e "  ${CYAN}https://plasmate.app${NC}"
echo -e "  ${CYAN}Apache 2.0 | pip install plasmate${NC}"
echo ""
