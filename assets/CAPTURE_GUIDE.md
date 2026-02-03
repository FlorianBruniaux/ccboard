# Screenshot & GIF Capture Guide

## Requirements

### Option 1: Manual Screenshots (macOS built-in)
- **Tool**: `Cmd+Shift+4` (macOS screenshot)
- **Time**: ~10 minutes
- **Quality**: Good for static images

### Option 2: Automated Recording (Professional)
- **Tools**:
  - `asciinema` (terminal recording)
  - `agg` (asciinema-to-gif converter)
- **Time**: ~5 minutes setup + 2 minutes recording
- **Quality**: Perfect for animated demos

## Installation (Option 2)

```bash
# Install asciinema
brew install asciinema

# Install agg (asciinema-to-gif)
cargo install --git https://github.com/asciinema/agg
```

## Screenshots Needed (1920x1080 terminal, 100x30 chars)

### 1. Dashboard Tab
**Filename**: `01_dashboard.png`
**Command**:
```bash
ccboard
# Wait for load completion
# Press '1' to ensure Dashboard tab active
# Capture full terminal window
```
**Shows**: Stats cards, 7-day sparkline, model usage gauges

---

### 2. Sessions Tab with Project Tree
**Filename**: `02_sessions.png`
**Command**:
```bash
ccboard
# Press '2' for Sessions tab
# Navigate to show project tree expanded
# Capture
```
**Shows**: Dual-pane (projects | sessions), metadata preview

---

### 3. Help Modal
**Filename**: `03_help_modal.png`
**Command**:
```bash
ccboard
# Press '?' to open help modal
# Capture centered modal overlay
```
**Shows**: Keybindings, global + tab-specific shortcuts

---

### 4. Search Highlighting (Sessions Tab)
**Filename**: `04_search_highlighting.png`
**Command**:
```bash
ccboard
# Press '2' for Sessions
# Press '/' to activate search
# Type 'rtk' or common search term
# Capture with yellow highlighted matches
```
**Shows**: Yellow background matches in preview text

---

### 5. Loading Spinner (Cold Cache)
**Filename**: `05_loading_spinner.png`
**Command**:
```bash
# Clear cache first
rm -f ~/.claude/cache/session-metadata.db*

# Start ccboard - capture immediately during load
ccboard
# Capture within first 2 seconds (spinner animating)
```
**Shows**: Braille dot animation, "Loading..." message

---

## Animated GIF Demo (Option 2 - asciinema)

### Recording Script

Create `assets/record_demo.sh`:

```bash
#!/bin/bash
set -e

# Clear cache for spinner demo
rm -f ~/.claude/cache/session-metadata.db*

# Record session
asciinema rec demo.cast \
  --overwrite \
  --command "bash -c 'sleep 1; ccboard'" \
  --idle-time-limit 2 \
  --title "ccboard - Claude Code Dashboard"

# Convert to GIF (optimized)
agg \
  --font-family "JetBrains Mono,Monaco,Menlo,monospace" \
  --theme monokai \
  --font-size 14 \
  --line-height 1.4 \
  --cols 100 \
  --rows 30 \
  --speed 1.5 \
  --fps 20 \
  demo.cast \
  assets/gifs/demo.gif

echo "✓ GIF created: assets/gifs/demo.gif"
```

### Manual Recording Steps

1. **Start recording**:
   ```bash
   asciinema rec demo.cast --idle-time-limit 2
   ```

2. **Demo sequence** (30-45s total):
   ```
   [Wait 2s for cold cache load with spinner]
   [Press '1' - Dashboard]
   [Wait 1s]
   [Press '2' - Sessions tab]
   [Press '/' - Search]
   [Type 'rtk']
   [Wait 1s - show highlighting]
   [Press 'ESC' - clear search]
   [Press '?' - Help modal]
   [Wait 2s - show keybindings]
   [Press 'ESC' - close help]
   [Press 'q' - quit]
   ```

3. **Convert to GIF**:
   ```bash
   agg \
     --font-family "JetBrains Mono" \
     --theme monokai \
     --font-size 14 \
     --cols 100 \
     --rows 30 \
     --speed 1.5 \
     --fps 20 \
     demo.cast \
     assets/gifs/demo.gif
   ```

## Optimization

### Reduce GIF Size (<5MB target)

```bash
# Use gifsicle for compression
brew install gifsicle

gifsicle -O3 --colors 256 \
  assets/gifs/demo.gif \
  -o assets/gifs/demo_optimized.gif
```

## Validation

- **Screenshot dimensions**: 1920x1080 or similar 16:9
- **GIF file size**: < 5MB (mobile-friendly)
- **GIF frame rate**: 15-20 fps (smooth but not bloated)
- **GIF duration**: 30-45s max (attention span)
- **Terminal size**: 100x30 minimum (readable)

## Usage in README

```markdown
![Dashboard](assets/screenshots/01_dashboard.png)
![Sessions](assets/screenshots/02_sessions.png)
![Help Modal](assets/screenshots/03_help_modal.png)
![Search](assets/screenshots/04_search_highlighting.png)

## Quick Demo

![Demo](assets/gifs/demo_optimized.gif)
```
