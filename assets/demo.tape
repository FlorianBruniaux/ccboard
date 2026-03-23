Output assets/demo.gif

Require ccboard

Set Shell "bash"
Set FontSize 16
Set Width 1400
Set Height 800
Set Padding 0
Set Framerate 15
Set PlaybackSpeed 1.0
Set Theme "Catppuccin Frappe"
Set TypingSpeed 80ms
Set WindowBar "Colorful"
Set BorderRadius 8

# Launch ccboard
Type "ccboard"
Enter

# Wait for SQLite cache load (fast after first run)
Sleep 6s

# === Dashboard (tab 1) ===
Sleep 2.5s

# === Sessions (tab 2) ===
Type "2"
Sleep 2s

# Navigate sessions list
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 1s

# === Conversation Viewer (Enter from Sessions) ===
Enter
Sleep 2.5s

# Scroll through messages
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 1.5s

# Back to Sessions
Escape
Sleep 1s

# === Analytics (tab 3) — heatmap + trends ===
Type "3"
Sleep 2.5s

# === Costs (tab 4) — budget gauge ===
Type "4"
Sleep 2s

# === Audit Log (tab 6) — security violations feed ===
Type "6"
Sleep 2.5s

# Navigate violations
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 1.5s

# === Config (tab 8) — 4-column diff ===
Type "8"
Sleep 2s

# === Hooks (tab 9) — syntax highlighting ===
Type "9"
Sleep 2s

# === Search (/) — FTS5 full-text search ===
Type "/"
Sleep 1.5s

# Type a search query
Type "bug"
Sleep 2.5s

# Clear and try another
Ctrl+A
Type "authentication"
Sleep 2s

# === Back to Dashboard ===
Type "1"
Sleep 2s

# Quit
Type "q"
Sleep 500ms
