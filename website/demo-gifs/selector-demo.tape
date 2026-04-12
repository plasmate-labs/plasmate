# CSS Selector filtering demo
# Run: vhs selector-demo.tape

Output selector-demo.gif

Set Shell "zsh"
Set FontSize 14
Set Width 900
Set Height 450
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 50ms

Type "# Use CSS selectors to extract just what you need"
Enter
Sleep 800ms

Type "# Extract only the main content from a page"
Enter
Sleep 500ms

Type "plasmate fetch https://news.ycombinator.com --selector 'td.title' --format text | head -20"
Enter
Sleep 3s

Type "# Precise extraction = even fewer tokens!"
Enter
Sleep 2s
