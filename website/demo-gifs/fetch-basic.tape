# Basic Plasmate fetch demo showing clean structured output
# Run: vhs fetch-basic.tape

Output fetch-basic.gif

Set Shell "zsh"
Set FontSize 14
Set Width 900
Set Height 450
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 50ms

Type "# Plasmate: The browser engine for AI agents"
Enter
Sleep 1s

Type "# Let's fetch Hacker News and see the clean output"
Enter
Sleep 500ms

Type "plasmate fetch https://news.ycombinator.com --format text | head -30"
Enter
Sleep 4s

Type "# Clean, structured text - perfect for LLMs!"
Enter
Sleep 2s
