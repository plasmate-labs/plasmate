# Side-by-side comparison: Raw HTML vs Plasmate SOM
# Run: vhs fetch-comparison.tape

Output fetch-comparison.gif

Set Shell "zsh"
Set FontSize 14
Set Width 900
Set Height 450
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 50ms

Type "# Raw HTML size vs Plasmate output"
Enter
Sleep 800ms

Type "# First, let's see the raw HTML size..."
Enter
Sleep 500ms

Type "curl -s https://linear.app | wc -c"
Enter
Sleep 2s

Type "# Now with Plasmate..."
Enter
Sleep 500ms

Type "plasmate fetch https://linear.app | wc -c"
Enter
Sleep 2s

Type "# Up to 16x smaller - fewer tokens, lower cost!"
Enter
Sleep 2s
