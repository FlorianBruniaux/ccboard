# ADR-001: Single Provider Focus (Claude Code Only)

**Status**: Accepted
**Date**: 2026-02-12
**Decision Makers**: Core team
**Context**: Architecture decision on multi-provider support

---

## Context

Question raised: Should ccboard become a generic AI assistant dashboard supporting multiple providers (Gemini, Copilot, Cursor, etc.) instead of focusing exclusively on Claude Code?

### Current Architecture

ccboard is tightly coupled to Claude Code's data structures:
- `~/.claude/` directory structure
- `stats-cache.json` proprietary format
- Session JSONL format (Claude API messages)
- MCP configuration (`claude_desktop_config.json`)
- Hooks/agents/skills in `.claude/` structure

### Other Providers Have Different Structures

| Provider | Storage | Format | Accessibility |
|----------|---------|--------|---------------|
| **Claude Code** | `~/.claude/` | JSONL sessions, JSON stats | Local files |
| **GitHub Copilot** | VSCode/JetBrains logs | Extension logs | Not persistently stored |
| **Gemini** | Cloud-based | API logs | Cloud-side only |
| **Cursor** | SQLite database | Custom schema | Different structure |
| **Aider** | `.aider/` | Git-based format | Custom format |

---

## Decision

**ccboard will remain Claude Code-only for the foreseeable future (12+ months).**

Multi-provider support is **deferred** pending:
1. ccboard feature completeness for Claude Code
2. Demonstrated user demand (50+ requests for other providers)
3. 1000+ daily active users on Claude Code version
4. Available development resources or active contributors

---

## Rationale

### Why Claude Code-Only Is Better Now

**1. Focus & Quality**
- Current roadmap incomplete: Conversation viewer (Phase F), Plan tracking (Phase H), MCP server mode not yet delivered
- Better to have one excellent tool than multiple mediocre integrations
- Claude Code users = early adopters who value depth over breadth

**2. Maintenance Burden**
- Multi-provider = 5x complexity minimum
- Each provider requires:
  - Custom parser (different formats)
  - Provider-specific tests
  - Ongoing maintenance as providers evolve
- Current capacity: solo maintainer, limited contributors

**3. Low ROI**
- Gemini/Copilot have integrated dashboards
- 80% of effort would serve 20% of users
- No proven demand from existing user base

**4. Technical Debt**
- Premature abstraction = over-engineering
- Format incompatibilities require translation layers
- Performance impact (multiple parsers, increased memory)

---

## Consequences

### Positive

✅ **Faster feature delivery** for Claude Code users
✅ **Simpler codebase** with lower maintenance burden
✅ **Better UX** through deep Claude Code integration
✅ **Clear positioning** in market ("The Claude Code dashboard")

### Negative

❌ **Smaller addressable market** (Claude users only)
❌ **Missed opportunity** if multi-AI usage becomes dominant
❌ **Future refactoring cost** if decision reversed later

### Mitigation Strategy

**Prepare for potential future generalization** without implementing it:

**Phase 1: Internal Abstraction** (Low cost, high flexibility)
```rust
// Trait-based architecture allows future providers
trait SessionParser {
    fn parse(&self, path: &Path) -> Result<Session>;
}

struct ClaudeSessionParser;  // Only implementation for now
impl SessionParser for ClaudeSessionParser { ... }
```

**Phase 2: Universal Export Format** (Medium cost, broad utility)
```rust
// Export to standard formats
ccboard export --format json > sessions.json
ccboard export --format csv > sessions.csv
```

**Phase 3: MCP Server Mode** (Already planned)
```rust
// Expose ccboard data via Model Context Protocol
// Other AI assistants can read ccboard data as MCP resource
```

---

## Alternatives Considered

### Option A: Multi-Provider from Day 1
- **Pros**: Larger market, universal positioning
- **Cons**: 3-4 weeks refactoring, ongoing complexity, unproven demand
- **Rejected**: Premature optimization, high opportunity cost

### Option B: Plugin Architecture
- **Pros**: Community can add providers, modular design
- **Cons**: Even more complex, API stability burden, low contributor count
- **Rejected**: Over-engineering for current scale

### Option C: Universal Format Translation
- **Pros**: Support all providers via import/export
- **Cons**: Lossy translation, no live monitoring, poor UX
- **Rejected**: Doesn't solve core value proposition

---

## Success Metrics

**Conditions to revisit this decision:**

1. **User Demand**: 50+ GitHub issues/requests for multi-provider support
2. **Scale**: 1000+ daily active users on Claude Code version
3. **Feature Completeness**: All Phase A-H features delivered for Claude
4. **Resources**: Active contributors or funding for multi-provider work

**Re-evaluation Date**: 2027-02-01 (12 months)

---

## References

- [PLAN.md](../../PLAN.md) - ccboard roadmap (Phases A-H)
- [CLAUDE.md](../../CLAUDE.md) - Architecture overview
- GitHub Copilot logs: No persistent session storage
- Cursor architecture: SQLite-based, proprietary schema
- Aider: Git-based session tracking in `.aider/`

---

## Notes

**Philosophy**: "Mieux vaut un outil excellent pour 1 provider qu'un outil moyen pour 5."

This decision aligns with ccboard's core value: **deep visibility into Claude Code workflows**, not generic AI monitoring.

If multi-provider support becomes necessary, the internal abstractions (Phase 1 mitigation) will make future refactoring significantly cheaper than a full rewrite.

---

## Related Decisions

- [ADR-002: Gemini CLI Support Evaluation](./ADR-002-gemini-cli-support-evaluation.md) (2026-02-12) - Evaluation of Gemini CLI v0.28.2 support following feature discovery. Confirmed ADR-001 based on technical blockers (JSON vs JSONL format, hash-based project IDs, stats recomputation overhead). Recommended export script alternative (40-60h) over full integration (500h).

---

**Last Updated**: 2026-02-12
**Next Review**: 2027-02-01
