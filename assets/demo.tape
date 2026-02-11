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

# Wait for loading to complete (conservative: 28s)
Sleep 28s

# === Dashboard (tab 1) - already here ===
Sleep 1.5s

# === Sessions (tab 2) ===
Type "2"
Sleep 1.5s

# === Config (tab 3) ===
Type "3"
Sleep 1.5s

# === Hooks (tab 4) ===
Type "4"
Sleep 1.5s

# === Capabilities (tab 5) ===
Type "5"
Sleep 1.5s

# === Costs (tab 6) ===
Type "6"
Sleep 1.5s

# === History (tab 7) ===
Type "7"
Sleep 1.5s

# === MCP (tab 8) ===
Type "8"
Sleep 1.5s

# === Analytics (tab 9) ===
Type "9"
Sleep 2s

# === Back to Sessions for search demo ===
Type "2"
Sleep 1s

# Search
Type "/"
Sleep 500ms
Type "bug"
Sleep 2s

# Close search / back
Escape
Sleep 500ms

# Quit
Type "q"
Sleep 500ms
