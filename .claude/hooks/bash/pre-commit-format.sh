#!/bin/bash
# Auto-format Rust code before commits
# Hook: PreToolUse for git commit

set -e

echo "🦀 Running Rust pre-commit checks..."

# Format code
cargo fmt --all

# Check for warnings
if ! cargo clippy --all-targets -- -D warnings; then
    echo "❌ Clippy found warnings. Fix them before committing."
    exit 1
fi

echo "✅ Pre-commit checks passed"
