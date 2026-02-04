# Screenshot Capture Guide

**Estimated time**: 10-15 minutes for all screenshots

---

## Prerequisites

Install screenshot tool:
```bash
# macOS (recommended: better quality)
# Built-in: Cmd+Shift+4 (select area)
# OR
brew install --cask shottr  # Advanced features

# Alternative: Terminal screenshot tool
brew install flameshot
```

**Terminal setup**:
- Size: 160x40 (for consistency with existing screenshots)
- Font size: 13-14pt
- Theme: Dracula or matching existing screenshots
- Launch: `ccboard` (ensure you have real data in ~/.claude)

---

## P0 Critical Screenshots (Before Distribution)

### 1. Analytics Tab Screenshot

**File**: `assets/screenshots/analytics.png`
**Priority**: 🔴 P0 CRITICAL

**Steps**:
```bash
# 1. Launch ccboard
ccboard

# 2. Jump to Analytics tab
# Press: 9

# 3. Wait for render (1 second)

# 4. Capture screenshot
# macOS: Cmd+Shift+4 → drag to select terminal window
# Save as: assets/screenshots/analytics.png

# Alternative keyboard shortcut:
# Cmd+Shift+4 → press Space → click terminal window
```

**What should be visible**:
- Tab header "9. Analytics" highlighted
- 4 sub-views: Trends | Forecast | Patterns | Insights
- Chart/graph visible (sparkline or bar chart)
- Data from last 7-30 days

**Expected content**:
```
┌─ Analytics ────────────────────────────────────┐
│ [Trends] [Forecast] [Patterns] [Insights]     │
│                                                │
│ Daily Token Usage (Last 7 Days)               │
│ ▁▂▄▅▇██▆                                       │
│                                                │
│ Forecast (Next 7 Days): +15% trend            │
│ R²: 0.87 (strong correlation)                 │
└────────────────────────────────────────────────┘
```

---

## P1 High Priority Screenshots

### 2. Hooks Test Mode Screenshot

**File**: `assets/screenshots/hooks-test-mode.png`
**Priority**: 🟡 P1 HIGH

**Steps**:
```bash
# 1. Launch ccboard
ccboard

# 2. Jump to Hooks tab
# Press: 4

# 3. Navigate to a hook (if you have hooks)
# Press: j or Down (to select a hook)

# 4. Test the hook
# Press: t

# 5. Test result modal appears
# Wait 1 second for modal to fully render

# 6. Capture screenshot
# Cmd+Shift+4 → select terminal
# Save as: assets/screenshots/hooks-test-mode.png
```

**What should be visible**:
- Hooks tab with list of hooks
- Test result modal/popup overlaid
- stdout/stderr output
- Exit code (0 = success, non-zero = error)

**Expected modal content**:
```
┌─ Test Result: pre-commit-hook.sh ──────────┐
│ Exit Code: 0                               │
│                                            │
│ stdout:                                    │
│ ✓ Running clippy...                       │
│ ✓ All checks passed                       │
│                                            │
│ stderr: (empty)                           │
│                                            │
│ Duration: 1.2s                            │
└────────────────────────────────────────────┘
```

**Fallback if no hooks**:
Skip this screenshot for now, mark as "TODO: Add after user has hooks configured"

---

### 3. Live Refresh Indicator Screenshot

**File**: `assets/screenshots/live-refresh.png`
**Priority**: 🟡 P1 HIGH

**Steps**:
```bash
# 1. Launch ccboard
ccboard

# 2. Jump to Sessions tab
# Press: 2

# 3. Wait for initial render

# 4. In another terminal, create a change:
# Open new terminal window
cd ~/.claude/projects/<any-project>
touch test-session-$(date +%s).jsonl
echo '{"role":"user","content":"test"}' > test-session-$(date +%s).jsonl

# 5. Switch back to ccboard terminal
# Within 2-3 seconds, you should see:
# - Green notification banner at top or bottom
# - Timestamp "2m ago" updating to "just now"
# - Session count incrementing

# 6. Capture screenshot during notification
# Cmd+Shift+4 → select terminal
# Save as: assets/screenshots/live-refresh.png
```

**What should be visible**:
- Sessions tab active
- Green notification banner: "✓ Data refreshed" or "✓ New session detected"
- Updated timestamp
- Possibly a toast notification in corner

**Expected notification**:
```
┌─ Sessions (3551) • just now ──────────────────┐
│ ✓ Data refreshed                              │
│                                               │
│ [Project tree]  [Session list]  [Detail]     │
└───────────────────────────────────────────────┘
```

**Alternative method** (if file creation doesn't trigger):
```bash
# Press r (global refresh)
# Capture the toast notification that appears
```

---

### 4. Toast Notifications Screenshot

**File**: `assets/screenshots/toast-notifications.png`
**Priority**: 🟡 P1 HIGH

**Steps**:
```bash
# 1. Launch ccboard
ccboard

# 2. Trigger multiple toasts quickly:
# Press: r (refresh) → should show "✓ Refreshed" toast
# Wait 500ms
# Press: x (export) → may show error toast if no data selected
# Wait 500ms
# Press: ? (help modal) → Escape → may show info toast

# 3. Capture screenshot while 2-3 toasts stacked
# Cmd+Shift+4 → select terminal
# Save as: assets/screenshots/toast-notifications.png
```

**What should be visible**:
- 2-3 toast notifications stacked (bottom-right corner typical)
- Different types: success (green), error (red), info (blue)

**Expected toast stack**:
```
                              ┌─────────────────┐
                              │ ✓ Refreshed    │
                              └─────────────────┘
                              ┌─────────────────┐
                              │ ⚠ No selection │
                              └─────────────────┘
                              ┌─────────────────┐
                              │ ℹ Help closed  │
                              └─────────────────┘
```

**Alternative method** (guaranteed toasts):
```bash
# Method 1: Clear cache (shows warning toast)
ccboard clear-cache
# Then launch: ccboard
# Shows "⚠ Cache cleared" toast

# Method 2: Corrupt a file temporarily
echo "invalid json" >> ~/.claude/stats-cache.json.bak
ccboard
# Shows "⚠ Stats unavailable" toast
# Don't forget to restore: rm ~/.claude/stats-cache.json.bak
```

---

## P2 Nice-to-Have (Optional)

### 5. Comparison Infographic

**File**: `assets/comparison-chart.png`
**Priority**: 🟢 P2 OPTIONAL

**Tool**: Not terminal screenshot, use graphic design tool

**Options**:
- Canva template
- Figma
- Or: Convert markdown table to PNG with `mdpdf` or similar

**Alternative**: Skip graphic, markdown table in README sufficient

---

### 6. Performance Benchmark Chart

**File**: `assets/perf-benchmark.png`
**Priority**: 🟢 P2 OPTIONAL

**Generate from Criterion benchmark**:
```bash
# Run benchmarks
cargo bench

# Criterion generates HTML report
open target/criterion/report/index.html

# Take screenshot of chart (89x speedup graph)
# Save as: assets/perf-benchmark.png
```

**Alternative**: Skip, README text "89x speedup" sufficient

---

## Quick Checklist

Before distribution, verify:

- [ ] `assets/screenshots/analytics.png` (P0)
- [ ] `assets/screenshots/hooks-test-mode.png` (P1, if hooks exist)
- [ ] `assets/screenshots/live-refresh.png` (P1)
- [ ] `assets/screenshots/toast-notifications.png` (P1)
- [ ] All screenshots 2x retina (high DPI)
- [ ] Consistent terminal size (160x40)
- [ ] Consistent theme (Dracula)

---

## Troubleshooting

### Terminal not 160x40

Check size:
```bash
tput cols  # Should show 160
tput lines # Should show 40
```

Resize terminal:
- iTerm2: Cmd+D (split) then adjust manually
- Terminal.app: Preferences → Profiles → Window → Columns/Rows

### ccboard shows no data

Ensure you have Claude Code sessions:
```bash
ls -la ~/.claude/projects/*/
# Should see .jsonl files
```

If empty, create test data:
```bash
mkdir -p ~/.claude/projects/test-project
echo '{"role":"user","content":"test"}' > ~/.claude/projects/test-project/test.jsonl
```

### Can't trigger toasts

Use refresh key `r` multiple times quickly (500ms apart)

### Screenshot quality low

Use:
- Cmd+Shift+4 → Space → Click window (native macOS, best quality)
- Avoid: Cmd+Shift+3 (full screen, includes desktop)
- Export at 2x resolution

---

## Batch Script (Semi-Automated)

Save as `capture-screenshots.sh`:

```bash
#!/bin/bash
set -e

ASSETS_DIR="assets/screenshots"
mkdir -p "$ASSETS_DIR"

echo "Screenshot Capture Helper"
echo "=========================="
echo ""
echo "This script will guide you through capturing screenshots."
echo "After each prompt, take the screenshot manually with Cmd+Shift+4"
echo ""

# Analytics
echo "[1/4] Analytics Tab"
echo "→ Launch ccboard, press 9, capture screenshot"
echo "→ Save as: $ASSETS_DIR/analytics.png"
read -p "Press Enter when done..."

# Hooks test mode
echo "[2/4] Hooks Test Mode"
echo "→ Press 4 (Hooks tab), press j to select hook, press t to test"
echo "→ Save as: $ASSETS_DIR/hooks-test-mode.png"
read -p "Press Enter when done (or skip if no hooks)..."

# Live refresh
echo "[3/4] Live Refresh"
echo "→ Press 2 (Sessions tab), press r to refresh, capture toast"
echo "→ Save as: $ASSETS_DIR/live-refresh.png"
read -p "Press Enter when done..."

# Toast notifications
echo "[4/4] Toast Notifications"
echo "→ Press r multiple times (500ms apart), capture stacked toasts"
echo "→ Save as: $ASSETS_DIR/toast-notifications.png"
read -p "Press Enter when done..."

echo ""
echo "✓ Screenshot capture complete!"
echo ""
echo "Verify files:"
ls -lh "$ASSETS_DIR"/*.png | grep -E "analytics|hooks-test|live-refresh|toast"
```

Run with:
```bash
chmod +x capture-screenshots.sh
./capture-screenshots.sh
```

---

**Estimated total time**: 10-15 minutes for all P0+P1 screenshots

**Priority order**:
1. Analytics (P0) - 2 minutes
2. Toast notifications (P1) - 3 minutes (easiest, trigger with r key)
3. Live refresh (P1) - 3 minutes
4. Hooks test mode (P1) - 2 minutes (skip if no hooks)
