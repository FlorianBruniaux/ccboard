# Cross-Platform Validation Guide

This document outlines platform-specific considerations and validation procedures for ccboard.

## Supported Platforms

| Platform | Architecture | Status | Notes |
|----------|--------------|--------|-------|
| **macOS** | x86_64 (Intel) | ✅ Tested | Primary development platform |
| **macOS** | aarch64 (Apple Silicon) | ✅ Tested | Native M1/M2/M3 support |
| **Linux** | x86_64 | 🟡 CI only | Ubuntu, Debian, Fedora, Arch |
| **Linux** | aarch64 | 🟡 CI only | Raspberry Pi, ARM servers |
| **Windows** | x86_64 | 🟡 CI only | Windows 10/11, MSVC toolchain |

**Legend**:
- ✅ Tested = Manually validated on real hardware
- 🟡 CI only = Automated tests pass, no manual validation
- ❌ Unsupported = Known issues or not tested

---

## Platform-Specific Considerations

### macOS

#### Paths
- `~/.claude` resolves correctly via `dirs::home_dir()`
- Symlink rejection works with `std::fs::symlink_metadata()`
- File manager reveal uses `open -R <path>`

#### Terminal
- Unicode/emoji rendering: ✅ Full support (Braille spinner, icons)
- Color depth: ✅ 24-bit color
- Terminal apps tested:
  - iTerm2: ✅ Perfect
  - Terminal.app: ✅ Full support
  - Alacritty: ✅ Fast rendering
  - Kitty: ✅ Perfect

#### File Editor
- `$VISUAL` / `$EDITOR` priority works
- Fallback: `nano` (preinstalled)
- Tested editors: vim, nvim, helix, VSCode (`code -w`)

#### Known Issues
- None currently

---

### Linux

#### Paths
- `~/.claude` resolves via `dirs::home_dir()` (uses `$HOME`)
- Symlink rejection: ✅ `std::fs::symlink_metadata()` is cross-platform
- File manager reveal: `xdg-open <directory>` (opens parent folder)

#### Terminal
- Unicode/emoji: ✅ Most modern terminals support
- Color depth: Varies by terminal (8/16/256/24-bit)
- Terminal compatibility:
  - GNOME Terminal: ✅ Expected to work
  - Konsole (KDE): ✅ Expected to work
  - Alacritty: ✅ Expected to work
  - Kitty: ✅ Expected to work
  - xterm: ⚠️ Limited unicode/color support

#### File Editor
- `$VISUAL` / `$EDITOR` priority works
- Fallback: `nano` (usually preinstalled)
- Common editors: vim, nvim, emacs, helix

#### Edge Cases to Test

**1. Network Filesystems (NFS, SMB)**
```bash
# mtime might be unreliable on network FS
# Mitigation: Cache invalidation still works (worst case: slower)
```

**2. Case-Sensitive vs Case-Insensitive Filesystems**
```bash
# Most Linux: case-sensitive (default)
# macOS APFS: case-insensitive by default
# Code handles both via Path normalization
```

**3. Permissions**
```bash
# Test restrictive permissions
chmod 000 ~/.claude/stats-cache.json
ccboard  # Should handle gracefully with LoadReport

# Restore
chmod 644 ~/.claude/stats-cache.json
```

**4. Non-UTF8 Paths**
```bash
# Rust Path/PathBuf handle OsStr (non-UTF8 safe)
# Use .to_string_lossy() for display
```

#### Known Issues
- File manager reveal opens parent directory (not file selection like macOS `open -R`)
  - Expected behavior: User navigates to file manually
  - Workaround: None needed (acceptable UX)

---

### Windows

#### Paths
- `~/.claude` → `C:\Users\<username>\.claude`
- Path separator: Backslash `\` (Rust `std::path` handles automatically)
- Symlink rejection: ✅ Works (requires admin on old Windows, but detection is safe)

#### Terminal
- Unicode/emoji: ⚠️ Limited in cmd.exe and PowerShell
  - Braille spinner may render as `?` or boxes
  - Mitigation: Use ASCII fallback spinner on Windows
- Color depth: 16 colors (cmd.exe), 256+ (Windows Terminal)
- Terminal apps:
  - Windows Terminal: ✅ Full unicode/color support
  - cmd.exe: ⚠️ Limited unicode/color
  - PowerShell: ⚠️ Better than cmd, still limited
  - Alacritty: ✅ Full support
  - Kitty: ❌ Not available on Windows

#### File Editor
- `$VISUAL` / `$EDITOR` priority (PowerShell: `$env:EDITOR`)
- Fallback: `notepad.exe` (always available)
- Common editors: VSCode (`code.exe -w`), Notepad++, vim (Git Bash)

#### Edge Cases to Test

**1. Path Separators**
```rust
// ✅ Good: Use std::path::Path (handles \\ automatically)
let path = Path::new("C:\\Users\\name\\.claude");

// ❌ Bad: Hardcode Unix paths
let path = "~/.claude";  // Won't work on Windows
```

**2. File Manager Reveal**
```cmd
REM Opens Explorer with file selected
explorer /select,C:\Users\name\.claude\sessions\file.jsonl
```

**3. Symlinks**
```powershell
# Requires admin privileges on Windows < 10 build 14972
# Detection still works safely without admin
```

**4. Line Endings**
- JSONL files: `\r\n` (Windows) vs `\n` (Unix)
- Rust BufReader handles both automatically
- No special handling needed

#### Known Issues

**1. Unicode Rendering in cmd.exe/PowerShell**
- **Issue**: Braille spinner (`⠋⠙⠹⠸`) may render as `????`
- **Status**: Won't Fix — Use Windows Terminal for full Unicode support
- **Recommendation**: Install [Windows Terminal](https://aka.ms/terminal) (free, supports Unicode + 24-bit color)

**2. ANSI Color Support**
- **Issue**: cmd.exe has limited color support
- **Status**: Crossterm handles this automatically with `enable_raw_mode()`
- **No action needed**: Already works

---

## Validation Checklist

### Automated (CI)

All platforms run these checks automatically on every PR:

- ✅ `cargo fmt --check` (formatting)
- ✅ `cargo clippy -- -D warnings` (linting)
- ✅ `cargo test --all` (344 tests)
- ✅ `cargo build --release` (release build)

**CI Matrix**:
- ubuntu-latest (x86_64-unknown-linux-gnu)
- macos-latest (x86_64-apple-darwin + aarch64-apple-darwin)
- windows-latest (x86_64-pc-windows-msvc)

### Manual (Pre-Release)

Before each release, validate manually on at least one machine per OS:

#### macOS ✅
```bash
# 1. Install from source
cargo install --path crates/ccboard

# 2. Run TUI
ccboard

# 3. Test all tabs (1-7)
# 4. Test search highlighting (/)
# 5. Test help modal (?)
# 6. Test file editing (e) - verify $EDITOR opens
# 7. Test file reveal (o) - verify Finder opens with file selected
# 8. Test web mode
ccboard web --port 3333

# 9. Test stats mode
ccboard stats
```

#### Linux (Ubuntu/Debian) 🟡
```bash
# 1. Install from binary (GitHub Releases)
wget https://github.com/FlorianBruniaux/ccboard/releases/download/v0.2.0/ccboard-linux-x86_64.tar.gz
tar xzf ccboard-linux-x86_64.tar.gz
sudo mv ccboard /usr/local/bin/

# 2. Verify installation
ccboard --version

# 3. Run TUI
ccboard

# 4. Test unicode rendering
#    - Braille spinner visible during cold cache load?
#    - Icons render correctly in tabs?

# 5. Test file manager reveal (o)
#    - Does xdg-open work?
#    - Opens parent directory? (expected behavior)

# 6. Test restrictive permissions
chmod 000 ~/.claude/stats-cache.json
ccboard  # Should show partial data with error in LoadReport
chmod 644 ~/.claude/stats-cache.json

# 7. Test editor integration
export EDITOR=vim
ccboard
# Press 'e' on a file → vim should open
```

#### Windows 10/11 🟡
```powershell
# 1. Install from binary (GitHub Releases)
# Download ccboard-windows-x86_64.zip
# Extract to C:\Program Files\ccboard\
# Add to PATH

# 2. Verify installation
ccboard --version

# 3. Run TUI in Windows Terminal (recommended)
ccboard

# 4. Test unicode rendering
#    - Braille spinner shows correctly? (Windows Terminal: yes, cmd.exe: no)
#    - If cmd.exe: Report issue (expected limitation)

# 5. Test file manager reveal (o)
#    - Explorer opens with file selected?

# 6. Test editor integration
$env:EDITOR = "code -w"  # VSCode
ccboard
# Press 'e' on a file → VSCode should open

# 7. Test with PowerShell vs cmd.exe
#    - Both should work (colors may differ)
```

---

## Platform-Specific Bugs Reporting

If you encounter platform-specific issues, please report with:

1. **Platform details**:
   ```bash
   ccboard --version
   rustc --version
   uname -a  # Linux/macOS
   systeminfo | findstr /B /C:"OS Name" /C:"OS Version"  # Windows
   ```

2. **Terminal info**:
   - Terminal application (iTerm2, GNOME Terminal, Windows Terminal, etc.)
   - Unicode support test: Can you see `⠋⠙⠹⠸` correctly?

3. **Reproduction steps**:
   - Exact command run
   - Expected behavior
   - Actual behavior
   - Screenshots if UI-related

4. **Logs**:
   ```bash
   RUST_LOG=ccboard=debug ccboard 2> debug.log
   # Attach debug.log
   ```

---

## Continuous Improvement

### Phase A.4 Validation Results

After manual testing on each platform, update this section:

| Platform | Tested By | Date | Issues Found | Status |
|----------|-----------|------|--------------|--------|
| macOS Intel | @FlorianBruniaux | 2026-02-10 | None | ✅ Pass |
| macOS ARM64 | @FlorianBruniaux | 2026-02-10 | None | ✅ Pass |
| Linux x86_64 | - | - | - | 🟡 CI only |
| Windows 10 | - | - | - | 🟡 CI only |

### Future Platform Support

- **BSD** (FreeBSD, OpenBSD): Untested, likely works (Rust std support)
- **ARM Linux** (Raspberry Pi): CI builds available, untested
- **musl Linux**: TODO - Add CI job for static linking

---

## References

- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [crossterm Platform Support](https://docs.rs/crossterm/latest/crossterm/#supported-terminals)
- [dirs crate Platform Differences](https://docs.rs/dirs/latest/dirs/)
