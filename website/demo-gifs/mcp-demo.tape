# MCP Server demo - Model Context Protocol integration
# Run: vhs mcp-demo.tape

Output mcp-demo.gif

Set Shell "zsh"
Set FontSize 14
Set Width 900
Set Height 400
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 50ms

Type "# Start Plasmate as an MCP server"
Enter
Sleep 800ms

Type "plasmate mcp"
Enter
Sleep 1500ms

Type@0ms ""
Ctrl+C
Sleep 500ms

Type "# MCP server ready for Claude, Cursor, or any MCP client"
Enter
Sleep 500ms

Type "# Add to your claude_desktop_config.json:"
Enter
Sleep 500ms

Type 'echo \'{"mcpServers":{"plasmate":{"command":"plasmate","args":["mcp"]}}}\''
Enter
Sleep 2s
