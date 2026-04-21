---
model: haiku
description: Diagnose dev environment — checks Cargo build, clippy, deps, binary. Auto-suggest on Rust/Cargo errors.
---

# /diagnose

Check the development environment and suggest fixes.

## When to use

- **Auto-suggested** when Claude detects these error patterns:
  - `error[E...]` (rustc errors) → compilation failure
  - `cannot find crate` → missing dependency or `cargo fetch` needed
  - `error: failed to run custom build command` → build script failure
  - `CARGO_MANIFEST_DIR` issues → wrong working directory
  - `thread 'main' panicked` → runtime panic in dev

- **Manually** after a `git pull` or at session start

## Execution

### 1. Parallel checks

Run these commands in parallel:

```bash
# Git status
git status --short && git branch --show-current
```

```bash
# Cargo.lock freshness
if [ ! -f "Cargo.lock" ]; then
  echo "❌ MISSING: Cargo.lock"
elif [ "Cargo.toml" -nt "Cargo.lock" ]; then
  echo "⚠️ OUTDATED: cargo fetch or cargo build needed"
else
  echo "✅ OK: Cargo.lock"
fi
```

```bash
# Check compilation
cargo check --all 2>&1 | tail -5
```

```bash
# Check for outdated dependencies
cargo outdated --depth 1 2>/dev/null | head -15 || echo "(cargo-outdated not installed)"
```

### 2. Clippy (optional, if errors suspected)

```bash
cargo clippy --all-targets 2>&1 | grep -E "^error|^warning" | head -20
```

### 3. Test suite (optional, if regressions suspected)

```bash
cargo test --all 2>&1 | tail -10
```

## Output format

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 Environment Diagnostic
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 Cargo.lock:    ✅ OK
🔧 cargo check:   ⚠️  3 errors (see below)
📎 Clippy:        ✅ OK
🧪 Tests:         ✅ OK (47 passed)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Suggested actions

Use `AskUserQuestion` if issues detected:

```
question: "Issues detected. Which fixes should I apply?"
header: "Fixes"
multiSelect: true
options:
  - label: "cargo fetch"
    description: "Fetch missing dependencies"
  - label: "cargo build --all"
    description: "Build all workspace crates"
  - label: "cargo clippy --all-targets --fix"
    description: "Auto-fix clippy lints"
  - label: "Fix all (recommended)"
    description: "cargo fetch && cargo build --all && cargo clippy --all-targets"
```

## Fix execution

If user selects "Fix all":

```bash
cargo fetch && cargo build --all && cargo clippy --all-targets
```

Otherwise, run selected commands sequentially.

## Auto-detection

**IMPORTANT**: Claude should suggest `/diagnose` automatically when it sees these errors:

| Error | Pattern | Likely Cause |
|-------|---------|--------------|
| Rustc compile error | `error[E...]` | Compilation failure |
| Missing crate | `cannot find crate for` | Missing dep / cargo fetch needed |
| Build script failure | `failed to run custom build command` | sys-crate or build.rs issue |
| Clippy failures | `warning: ... denied` | Clippy lint violations |
| Test failures | `FAILED` in cargo test output | Regression in tests |

Example auto-suggestion:
```
This "cannot find crate for `parking_lot`" error suggests a missing
dependency. I suggest running `/diagnose` to check the environment state.
```