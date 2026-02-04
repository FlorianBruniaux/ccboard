# VHS Script for ccboard Demo GIF
# Generate with: vhs demo.tape
# Install VHS: brew install vhs (macOS) or https://github.com/charmbracelet/vhs

Output assets/demo.gif
Set Shell bash
Set FontSize 13
Set Width 160
Set Height 40
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 50ms
Set FrameRate 30

# Launch ccboard
Type "ccboard"
Enter
Sleep 2.5s

# Dashboard (Tab 1) - already visible
Sleep 1.5s

# Navigate to Sessions (Tab 2)
Type "2"
Sleep 1.5s

# Scroll down in sessions list
Down
Down
Down
Sleep 1s

# Search in Sessions
Type "/"
Sleep 300ms
Type "auth"
Sleep 1.2s
Escape
Escape
Sleep 800ms

# Navigate to Config (Tab 3)
Type "3"
Sleep 1.5s

# Scroll to see different config levels
Down
Down
Down
Sleep 1s

# Navigate to Hooks (Tab 4)
Type "4"
Sleep 1.5s

# Navigate to Agents (Tab 5)
Type "5"
Sleep 1.5s

# Switch to Commands sub-tab
Tab
Sleep 1s

# Switch to Skills sub-tab
Tab
Sleep 1s

# Back to Agents
Tab
Sleep 800ms

# Navigate to Costs (Tab 6)
Type "6"
Sleep 1.5s

# Cycle through cost views
Tab
Sleep 1s
Tab
Sleep 1s
Tab
Sleep 1s

# Navigate to History (Tab 7)
Type "7"
Sleep 1.5s

# Navigate to MCP (Tab 8)
Type "8"
Sleep 1.5s

# Navigate to Analytics (Tab 9)
Type "9"
Sleep 1.5s

# Cycle through analytics sub-views
Tab
Sleep 1s
Tab
Sleep 1s
Tab
Sleep 1s

# Show Help Modal
Type "?"
Sleep 2.5s
Escape
Sleep 800ms

# Command Palette
Type ":"
Sleep 300ms
Type "dash"
Sleep 800ms
Enter
Sleep 1.5s

# Global refresh
Type "r"
Sleep 1.5s

# Quit
Type "q"
Sleep 500ms
