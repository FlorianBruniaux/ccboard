# ccboard Documentation

This directory contains active documentation for ccboard development and design decisions.

## Active Documentation

**Current State & Roadmap** ⭐
- [STATUS.md](STATUS.md) - Current project state (v0.7.0) and next actions - **START HERE**
- [ROADMAP-v0.8.md](ROADMAP-v0.8.md) - Quota tracking plan for v0.8.0
- [ROADMAP.md](ROADMAP.md) - Long-term features and planned improvements

**Architecture & Development**
- [PLAN.md](PLAN.md) - Complete architecture phases and development history (Phase 0→E, v0.4.0)

**Performance & Benchmarks**
- [performance-benchmark.md](performance-benchmark.md) - v0.4.0 optimization results (SQLite cache, Arc migration)
- [competitive-benchmark-2026-02-04.md](competitive-benchmark-2026-02-04.md) - Market positioning and competitive analysis

**Design Learnings (xlaude Analysis)**
- [xlaude-analysis.md](xlaude-analysis.md) - Deep dive into xlaude TUI design patterns
- [xlaude-vs-ccboard-comparison.md](xlaude-vs-ccboard-comparison.md) - Feature comparison and differentiation
- [xlaude-actionable-insights.md](xlaude-actionable-insights.md) - Actionable design insights for ccboard

## Archives

**archive/v0.7/** - v0.7.0 full-text search feature (2026-02-13)
- `PLAN-v0.7.0.md` - Detailed implementation plan
- `HANDOFF-v0.7.0.md` - Session handoff document

**archive/fixes/** - Bug fix documentation
- `PRICING_FIX.md` - Opus 4.5/4.6 pricing correction (2025-02)

**archive/templates/** - Reusable templates
- `SESSION-PROMPT.md` - Session handoff prompt template

**archive/** - Pre-v0.4.0 development artifacts and historical documentation

**archive-v05/** - Phase G (Web UI) development artifacts
- `web-phases/` - Wave 1-7 implementation plans and web architecture analysis
- `planning/` - Action plans and testing strategies
- `sessions/` - Status reports and completion summaries

## Usage

For project-wide documentation, see:
- [../README.md](../README.md) - Main project README
- [../CLAUDE.md](../CLAUDE.md) - Claude Code instructions
- [../ARCHITECTURE.md](../ARCHITECTURE.md) - Technical architecture overview
- [../CHANGELOG.md](../CHANGELOG.md) - Version history

## Notes

- Active documentation files are referenced by CLAUDE.md and should not be moved without updating references
- Archives preserve historical context and design decisions for future reference
- Phase-specific artifacts are archived after completion (e.g., archive-v05/ for Phase G)
