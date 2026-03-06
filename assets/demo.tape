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
Sleep 2s

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

# === Config (tab 3) ===
Type "3"
Sleep 1.5s

# === Costs (tab 6) ===
Type "6"
Sleep 2s

# === Analytics (tab 9) ===
Type "9"
Sleep 2s

# === Activity Security Audit (press 'a') ===
Type "a"
Sleep 2.5s

# Navigate violations
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 1.5s

# Toggle to Sessions view
Tab
Sleep 2s

# === Search tab (press Tab to navigate) ===
Tab
Sleep 1s

# Type a search query
Type "bug"
Sleep 2.5s

# Clear and try another
Ctrl+A
Type "authentication"
Sleep 2s

# === Back to Dashboard ===
Type "1"
Sleep 1.5s

# Quit
Type "q"
Sleep 500ms
