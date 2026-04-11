# Plasmate Video Demo

Assets for recording the Plasmate product demo video.

## Contents

| File | Description |
|------|-------------|
| `script.md` | Full 60-90 second video script with voiceover text |
| `terminal-commands.sh` | Exact commands to run during recording |
| `comparison.html` | Side-by-side HTML vs SOM visual comparison |
| `slides.md` | 4 slides for video overlays/transitions |

## Quick Start

### 1. Build Plasmate

```bash
cd /Users/steve/git/plasmate
cargo build --release
```

### 2. Test the Demo Commands

```bash
# Verify compression ratio
curl -s https://linear.app | wc -c
# Expected: ~2,200,000 (2.2MB)

./target/release/plasmate fetch https://linear.app | wc -c
# Expected: ~21,000 (21KB)
```

### 3. Preview the Comparison Page

```bash
open website/video-demo/comparison.html
```

## Recording Setup

### Terminal

- **App:** iTerm2, Warp, or native Terminal
- **Theme:** Dark background, high contrast
- **Font:** SF Mono or Menlo, 16-18pt
- **Window size:** 1920x1080 or 2560x1440

### Screen Recording

- **Tool:** OBS Studio, ScreenFlow, or QuickTime
- **Resolution:** 1080p or 4K
- **Frame rate:** 30fps

### Audio

- **Microphone:** USB condenser or Rode Wireless
- **Environment:** Quiet room, no echo
- **Post-processing:** Light noise reduction, normalize levels

## Recording Checklist

- [ ] Terminal clean (no sensitive info visible)
- [ ] Plasmate built and working
- [ ] Internet connection stable
- [ ] Audio levels tested
- [ ] Script printed or on second monitor

## Suggested Recording Flow

1. **Open terminal** full screen, dark theme
2. **Read the hook** from script while typing first command
3. **Show HTML size** with curl command
4. **Show SOM size** with plasmate command
5. **Cut to comparison.html** for visual impact
6. **Back to terminal** for install commands
7. **End card** with plasmate.app

## Post-Production Notes

- Add lower-third text for key stats
- Speed up long command outputs (2-3x)
- Add subtle zoom on important numbers
- Include background music (royalty-free, subtle)
- Add captions/subtitles for accessibility

## Asset Locations

After recording, place final assets in:

```
website/
  video-demo/
    plasmate-demo.mp4      # Final video
    plasmate-demo.webm     # Web-optimized version
    thumbnail.png          # Video thumbnail
```

## Duration Target

- **Hook:** 10 seconds
- **Problem:** 15 seconds
- **Demo:** 25 seconds
- **Results:** 15 seconds
- **CTA:** 15 seconds
- **Total:** 60-90 seconds
