# Demo GIF Recordings

Animated GIF demos of Plasmate in action, created using [VHS](https://github.com/charmbracelet/vhs).

## Prerequisites

Install VHS:

```bash
brew install vhs
```

VHS also requires ffmpeg and a TTY font. On macOS:

```bash
brew install ffmpeg
```

## Recording

To generate a GIF from a tape file:

```bash
vhs fetch-basic.tape
```

This creates `fetch-basic.gif` in the current directory.

## Available Demos

| File | Description | Duration |
|------|-------------|----------|
| `fetch-basic.tape` | Basic fetch with text output | ~8s |
| `fetch-comparison.tape` | HTML vs SOM size comparison | ~9s |
| `mcp-demo.tape` | Starting the MCP server | ~7s |
| `selector-demo.tape` | CSS selector filtering | ~8s |

## Recording All Demos

```bash
for tape in *.tape; do
  vhs "$tape"
done
```

## Customization

Each tape file uses:
- **Theme**: Catppuccin Mocha (dark, modern)
- **Font Size**: 14
- **Dimensions**: 900x450 (or 900x400)
- **Typing Speed**: 50ms per character

Edit the `Set` commands at the top of each `.tape` file to customize.

## Tips

- Run `plasmate` commands manually first to verify output
- Adjust `Sleep` durations if output takes longer
- Use `Type@0ms ""` for instant (invisible) typing
- Test with `vhs --publish <tape>` to preview online

## Output

Generated GIFs can be used in:
- The plasmate.app homepage
- GitHub README
- Documentation
- Social media
