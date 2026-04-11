# Plasmate Video Demo Script

**Duration:** 60-90 seconds  
**Style:** Screen recording with voiceover  
**Tone:** Technical but accessible, confident

---

## HOOK (0:00 - 0:10)

**[Screen: Terminal, dark theme]**

> "What if your AI agent could read the web 25x faster... and cost 25x less?"

**[Type command: `plasmate fetch https://linear.app`]**

---

## THE PROBLEM (0:10 - 0:25)

**[Screen: Split view - messy HTML on left, token counter spinning on right]**

> "Right now, when your AI reads a webpage, it's drowning in HTML noise."

**[Show: Wall of HTML tags scrolling]**

> "2.2 megabytes of divs, spans, and styling garbage. That's over 500,000 tokens - burned on navigation menus, tracking scripts, and CSS classes your agent doesn't need."

**[Show: Dollar counter: $0.75 per page read]**

> "At GPT-4 pricing, that's 75 cents just to read ONE webpage."

---

## THE SOLUTION (0:25 - 0:45)

**[Screen: Terminal]**

> "Plasmate changes everything."

**[Run: `curl -s https://linear.app | wc -c` showing ~2.2MB]**

> "Here's Linear.app - 2.2 megabytes of raw HTML."

**[Run: `plasmate fetch https://linear.app | wc -c` showing ~21KB]**

> "And here's what Plasmate sees: 21 kilobytes. Same page. Same information. 105x smaller."

**[Run: `plasmate fetch https://linear.app | head -30`]**

> "Clean, structured JSON. Headlines, links, content - exactly what your agent needs. Nothing it doesn't."

---

## THE RESULT (0:45 - 1:00)

**[Screen: Side-by-side comparison graphic]**

> "We call it the Semantic Object Model - SOM for short. It's like a DOM, but built for AI."

**[Show stats: 2.2MB to 21KB, 105x compression, ~$0.007 per page]**

> "Your agents read faster, your token costs drop 95%, and your context window goes 25 times further."

---

## CALL TO ACTION (1:00 - 1:15)

**[Screen: Terminal with install command]**

> "Get started in seconds."

**[Type: `pip install plasmate`]**

> "Or run it as an MCP server for Claude, Cursor, or any AI tool."

**[Type: `plasmate mcp`]**

**[Screen: plasmate.app homepage]**

> "Plasmate. The browser engine for AI agents."

**[End card: plasmate.app logo, GitHub stars]**

---

## Recording Notes

- Use a clean terminal theme (dark background, readable font)
- Pre-load the Linear.app response so demo doesn't depend on network
- Keep typing speed moderate - let viewers follow along
- Pause briefly after each stat reveal for impact
