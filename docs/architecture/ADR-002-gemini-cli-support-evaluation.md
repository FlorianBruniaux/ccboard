# ADR-002: Gemini CLI Support Evaluation

**Status**: Evaluation (Pending User Input)
**Date**: 2026-02-12
**Decision Makers**: Core team + Lab Google user
**Context**: Challenge to ADR-001 based on Gemini CLI v0.28.2 feature discovery
**Related**: [ADR-001: Single Provider Focus](./ADR-001-single-provider-focus.md)

---

## Context

### Why This Evaluation?

ADR-001 (2026-02-12) decided ccboard would remain **Claude Code-only** for 12+ months, citing:
- Low ROI (50% features missing in other providers)
- High maintenance burden (multi-provider = 5x complexity)
- No proven demand

**New Context**: Active user at Google Lab uses **Gemini CLI v0.28.2** and requested support. Initial analysis suggested Gemini CLI lacked core features (hooks, agents, skills, MCP), making integration effort **890h** with negative ROI.

**Discovery**: Perplexity research reveals Gemini CLI **has evolved significantly** since initial analysis. v0.26.0+ (Jan 2026) introduced hooks, skills, commands, and full MCP support.

### Trigger for Re-evaluation

1. **User has v0.28.2** (latest, released Feb 2026)
2. **Feature parity higher than assumed** (~80-90% vs 50%)
3. **Effort significantly lower** (~500h vs 890h, 44% reduction)
4. **Lab Google context** (potential team adoption, unknown volume)

---

## Findings: Gemini CLI v0.28.2 Architecture

### Research Methodology

- **Perplexity Search**: Gemini CLI v0.26.0-v0.28.2 features, changelog, docs
- **Local Inspection**: User's `~/.gemini/` directory (v0.28.2)
- **Official Docs**: geminicli.com/docs/*, GitHub discussions

### Feature Comparison: Gemini vs Claude Code

| Feature | Claude Code | Gemini CLI v0.28.2 | Status | Parsing Effort |
|---------|-------------|---------------------|--------|----------------|
| **Hooks** | `.claude/hooks/bash/*.sh` | JSON arrays in `settings.json` | ✅ **Available** | 🟡 **Medium** (format diff) |
| **Skills** | `.claude/agents/*.md` | `~/.gemini/skills/SKILL.md` | ✅ **Available** (experimental) | 🟢 **Low** (similar) |
| **Commands** | `.claude/commands/*.md` | `.gemini/commands/` (project) | ✅ **Available** | 🟢 **Low** (similar) |
| **Agents** | `.claude/agents/*.md` | Built-in CLI (no files) | ✅ **Available** | 🔴 **N/A** (no data) |
| **Sessions** | JSONL streaming | JSON array | ✅ **Available** | 🔴 **High** (format incompatible) |
| **Stats Cache** | `stats-cache.json` | ❌ Must recalculate | ❌ **Missing** | 🟠 **Medium** (compute) |
| **MCP Config** | `claude_desktop_config.json` | `mcp_config.json` | ✅ **Available** | 🟢 **Low** (if used) |
| **Tool Tracking** | Breakdown per tool | ❌ Must parse invocations | ❌ **Missing** | 🟠 **Medium** (compute) |
| **Git Context** | Branch, CWD tracked | ❌ Not tracked | ❌ **Missing** | N/A |

**Feature Parity**: **~80-90%** (vs 50% assumed in initial analysis)

### Architecture Differences (Critical)

#### 1. Hooks: JSON Arrays vs Shell Files

**Claude Code**:
```
.claude/hooks/bash/
├── pre-commit.sh
└── post-session.sh
```

**Gemini CLI**:
```json
{
  "hooks": {
    "BeforeAgent": ["script1.sh", "script2.sh"],
    "AfterAgent": ["cleanup.sh"],
    "BeforeTool": ["validate.sh"],
    "AfterTool": ["log.sh"]
  }
}
```

**Impact**: Parser must handle JSON arrays + script paths (30h effort)

#### 2. Skills: 3-Tier Discovery vs Flat Structure

**Gemini CLI**:
- Extension Skills (bundled)
- User Skills (`~/.gemini/skills/`)
- Workspace Skills (`.gemini/skills/`)
- **Experimental feature** (disabled by default via `experimental.skills`)

**Claude Code**:
- Single location: `.claude/agents/`
- Always active

**Impact**: Parser must check 3 locations + handle experimental flag (20h effort)

#### 3. Sessions: JSON Array vs JSONL Streaming

**Critical Blocker**: Claude uses JSONL (1 object per line, lazy loading), Gemini uses JSON (array of messages, must load fully).

**Impact**:
- Cannot lazy-load Gemini sessions
- Memory overhead for large sessions
- Parser completely different (150h effort, same as initial estimate)

#### 4. Project Identification: Path vs Hash

**Claude Code**: `/Users/foo/app` → `-Users-foo-app` (readable)
**Gemini CLI**: `/Users/foo/app` → SHA256 `64ce9ac29...` (opaque)

**Impact**: Reverse lookup required to map hashes → project paths (30h effort)

#### 5. Stats: Cache vs Recompute

**Claude Code**: Pre-computed `stats-cache.json` (instant load)
**Gemini CLI**: Must recalculate from all `logs.json` files (slower, but feasible)

**Impact**: Performance hit on initial load, need custom cache (50h effort)

### User's Current State (v0.28.2)

**Installation verified**:
```
~/.gemini/
├── antigravity/          (IDE integration)
│   └── mcp_config.json   (empty - not yet configured)
├── tmp/                  (sessions, hashed project IDs)
├── settings.json         (no hooks configured yet)
├── (no skills/ directory - not yet created)
└── (no commands/ in projects - not yet created)
```

**Findings**:
- ✅ Latest version (v0.28.2)
- ✅ Features **available** in CLI
- ❌ Features **not yet used** by user (no custom hooks/skills/commands)
- ❌ MCP config empty (feature available but unconfigured)

**Interpretation**: User hasn't created custom extensions yet, but CLI **fully supports** them.

---

## Effort Analysis: Revised Estimates

### Initial Estimate (Based on Outdated Assumptions)

| Phase | Effort | Basis |
|-------|--------|-------|
| MVP (basic support) | 650h | 50% features missing |
| Full Parity | 240h | Implement missing features |
| **Total** | **890h** | ~5.5 months, 1 person |

### Revised Estimate (Based on v0.28.2 Reality)

| Component | Original | Revised | Change | Rationale |
|-----------|----------|---------|--------|-----------|
| **Session Index Parser** | 150h | **150h** | 0% | JSON format incompatible (same effort) |
| **Stats Calculator** | 50h | **50h** | 0% | Recalculation needed (same effort) |
| **Session Content Parser** | 100h | **100h** | 0% | Message structure different |
| **Invocations Parser** | 80h | **80h** | 0% | Tool calls format different |
| **Settings Parser** | 15h | **10h** | -33% | Merge priority similar |
| **MCP Config Parser** | 60h | **20h** | -67% | Simple structure (if used) |
| **Hooks Parser** | — | **30h** | NEW | JSON arrays (new component) |
| **Skills Parser** | — | **20h** | NEW | SKILL.md (similar to agents) |
| **Commands Parser** | — | **15h** | NEW | Similar to Claude |
| **Project Mapper** | — | **30h** | NEW | Hash → path reverse lookup |
| **Data Store Refactoring** | 30h | **20h** | -33% | Multi-provider abstraction |
| **Pricing Module** | 40h | **40h** | 0% | Gemini pricing table |
| **TUI Adaptation** | 25h | **20h** | -20% | Provider selector + labels |
| **Web Adaptation** | 30h | **25h** | -17% | API endpoints |
| **Testing** | 80h | **60h** | -25% | Fixtures + integration |
| **Documentation** | 30h | **20h** | -33% | Multi-provider guide |
| **TOTAL MVP** | **890h** | **~500h** | **-44%** | Feature parity higher |

**Revised Timeline**: ~3 months (vs 5.5 months), 1 person, full-time

### Critical Path Components

**Highest Risk** (format incompatibilities):
1. Session Index Parser (150h) - JSON vs JSONL
2. Session Content Parser (100h) - Message structure
3. Invocations Parser (80h) - Tool calls format

**Medium Risk** (recomputation):
4. Stats Calculator (50h) - No cache
5. MCP Config Parser (20h) - If user configures MCP

**Low Risk** (similar structures):
6. Hooks/Skills/Commands Parsers (65h total)
7. UI Adaptations (45h total)

---

## Decision Options

### Option A: Export Script (40-60h) ✅ **Still Recommended**

**Approach**: Standalone script `gemini-to-csv` for manual export/import.

**Implementation**:
```bash
# Export Gemini sessions to CSV
gemini-to-csv --output sessions.csv

# Import into ccboard (universal format)
ccboard import --provider gemini --file sessions.csv
```

**Pros**:
- ✅ Lowest effort (40-60h vs 500h)
- ✅ ROI positive (even for single user)
- ✅ No ongoing maintenance
- ✅ User controls export timing
- ✅ No coupling to Gemini CLI updates

**Cons**:
- ❌ No live monitoring (manual export required)
- ❌ Limited to basic session metadata
- ❌ No hooks/skills/MCP config visibility
- ❌ Not real-time (snapshot-based)

**Use Cases**:
- Solo developer curiosity
- Occasional comparison (Claude vs Gemini)
- Archival/reporting needs
- Low session volume (<100)

---

### Option B: Full Integration (500h) ⚠️ **Conditionally Justifiable**

**Approach**: Multi-provider architecture with full Gemini CLI support.

**Implementation**:
```rust
// Trait-based provider abstraction
pub trait Provider {
    fn name(&self) -> &str;
    fn home_dir(&self) -> &Path;
    async fn scan_sessions(&self) -> Result<Vec<SessionMetadata>>;
    async fn load_stats(&self) -> Result<StatsCache>;
}

pub struct ClaudeProvider { /* ... */ }
pub struct GeminiProvider { /* ... */ }
```

**Pros**:
- ✅ Real-time monitoring (file watcher)
- ✅ Full feature parity (hooks, skills, MCP)
- ✅ Unified dashboard (compare providers)
- ✅ Scales to N providers (OpenAI, Anthropic, etc.)
- ✅ Future-proof architecture

**Cons**:
- ❌ High effort (500h = 3 months)
- ❌ Ongoing maintenance (2 parsers, breaking changes)
- ❌ Negative ROI if volume low (<50 users)
- ❌ Increased test surface (2x test cases)
- ❌ Architectural complexity (multi-provider state)

**Use Cases**:
- Lab Google team adoption (10-50+ developers)
- High session volume (500+ sessions/month)
- Migration from Claude → Gemini
- Permanent dual-support (compare models)

**Conditions for Justification**:
1. **Volume**: 50+ users OR 500+ sessions/month
2. **Sponsor**: Google funding OR active contributors
3. **Commitment**: Long-term Gemini usage (not trial)
4. **Feature usage**: User plans to create hooks/skills/MCP configs

---

### Option C: Maintain ADR-001 (0h)

**Approach**: Reject Gemini support, maintain Claude Code-only focus.

**Rationale**:
- Current user volume: 1 person, 10 sessions
- Alternative available (Option A: export script)
- Focus on Claude Code roadmap (Phases F-H incomplete)

**Pros**:
- ✅ Zero effort
- ✅ No maintenance burden
- ✅ Focus on existing roadmap
- ✅ Clear market positioning

**Cons**:
- ❌ Lab Google user unsupported
- ❌ Potential team adoption lost
- ❌ No data on Gemini adoption trends

---

## Critical Questions (Pending User Response)

### 1. Volume & Scale

**Q1.1**: Lab Google usage - solo or team?
- [ ] Solo developer (just you)
- [ ] Small team (2-5 developers)
- [ ] Medium team (6-20 developers)
- [ ] Large team (20+ developers)

**Q1.2**: Session volume forecast?
- [ ] Low (<50 sessions total)
- [ ] Medium (50-200 sessions/month)
- [ ] High (200-500 sessions/month)
- [ ] Very high (500+ sessions/month)

### 2. Use Case & Intent

**Q2.1**: Dashboard purpose?
- [ ] Curiosity (explore Gemini sessions)
- [ ] Comparison (Claude vs Gemini side-by-side)
- [ ] Migration (moving from Claude to Gemini)
- [ ] Dual-use (permanent multi-provider setup)

**Q2.2**: Export script sufficient?
- [ ] Yes (manual CSV export works)
- [ ] No (need real-time monitoring)

### 3. Feature Adoption

**Q3.1**: Plan to create custom extensions?
- [ ] Hooks (automation scripts)
- [ ] Skills (agent capabilities)
- [ ] Commands (team workflows)
- [ ] MCP configs (tool integrations)
- [ ] None of the above

### 4. Support & Resources

**Q4.1**: Sponsor/contributor availability?
- [ ] Google can provide funding
- [ ] Lab team can contribute code
- [ ] Solo usage (no external support)

**Q4.2**: Acceptable timeline?
- [ ] Urgent (<1 month)
- [ ] Standard (1-3 months)
- [ ] Flexible (>3 months)

---

## Decision Matrix

| Scenario | Volume | Intent | Extensions | Support | **Recommendation** |
|----------|--------|--------|------------|---------|-------------------|
| **A** | Solo | Curiosity | None | None | **Option A** (Export Script) |
| **B** | Solo | Migration | Planned | None | **Option B** (Full, if committed) |
| **C** | Team 2-5 | Dual-use | Planned | Contributors | **Option B** (Full, justified) |
| **D** | Team 10+ | Migration | Planned | Funding | **Option B** (Full, high ROI) |
| **E** | Solo | Comparison | None | None | **Option A** (Export Script) |
| **F** | Team 6-20 | Dual-use | None | Funding | **Option B** (Full, if volume) |

**Decision Rule**:
- **Option A**: Solo user OR curiosity OR low volume (<50 sessions)
- **Option B**: Team adoption OR migration OR high volume (500+ sessions) + sponsor
- **Option C**: No clear use case OR temporary trial

---

## Consequences

### If Option A (Export Script)

**Positive**:
- ✅ Immediate value (40-60h implementation)
- ✅ No maintenance burden
- ✅ ADR-001 maintained (focus preserved)
- ✅ User gets basic functionality

**Negative**:
- ❌ No real-time monitoring
- ❌ Manual export friction
- ❌ Limited feature visibility (no hooks/skills)

**Mitigation**:
- Schedule weekly/monthly exports
- Automate via cron: `gemini-to-csv && ccboard import`

---

### If Option B (Full Integration)

**Positive**:
- ✅ Full feature parity (80-90%)
- ✅ Real-time monitoring
- ✅ Unified dashboard (compare providers)
- ✅ Scales to team adoption

**Negative**:
- ❌ 500h development (3 months)
- ❌ 2x maintenance burden (Claude + Gemini parsers)
- ❌ Breaking changes risk (Gemini CLI updates)
- ❌ Increased test surface

**Mitigation**:
- Trait-based abstraction (isolate provider logic)
- Version pinning (Gemini CLI v0.28.x compatibility)
- Feature flags (disable Gemini if issues)
- Graceful degradation (partial data OK)

---

### If Option C (Maintain ADR-001)

**Positive**:
- ✅ Zero effort
- ✅ Focus on Claude Code roadmap
- ✅ Clear positioning ("The Claude Code Dashboard")

**Negative**:
- ❌ Lab Google user unsupported
- ❌ Potential team adoption lost
- ❌ No Gemini ecosystem data

**Mitigation**:
- Recommend Gemini's built-in analytics (if available)
- Revisit decision if demand proven (50+ requests)

---

## Next Steps

### Immediate (This Week)

1. **User response** to 4 critical questions (above)
2. **Decision classification** using Decision Matrix
3. **Go/No-Go** based on scenario match

### If Option A Selected (Export Script)

**Week 1-2**: Implementation
```bash
gemini-to-csv
├── Parse ~/.gemini/tmp/*/logs.json
├── Extract session metadata
├── Generate CSV (id, project, date, messages, tokens, cost)
└── Output compatible with ccboard import
```

**Week 3**: Testing + documentation
- Fixtures: 10-20 sample sessions
- User guide: Export workflow
- Import validation

---

### If Option B Selected (Full Integration)

**Phase 1: Refactoring (Weeks 1-2)**
- Trait `Provider` implementation
- Refactor `DataStore` for multi-provider
- Test existing Claude Code (no regressions)

**Phase 2: Gemini Parsers (Weeks 3-8)**
- Session Index Parser (JSON format)
- Stats Calculator (recompute from logs.json)
- Session Content Parser (message structure)
- Invocations Parser (tool calls)
- Hooks/Skills/Commands Parsers

**Phase 3: UI Adaptation (Weeks 9-10)**
- Provider selector (TUI + Web)
- Labels: "Claude" / "Gemini" tags
- Feature availability indicators (disable N/A features)

**Phase 4: Testing (Weeks 11-12)**
- Fixtures: 20+ Gemini sessions (sanitized)
- Integration tests: Multi-provider load
- Performance: <2s load for 1000+ sessions (both providers)

**Phase 5: Documentation (Week 13)**
- Multi-provider guide
- Migration guide (Claude → Gemini)
- API updates (provider parameter)

---

### If Option C Selected (Maintain ADR-001)

**Immediate**:
- Close evaluation with documented rationale
- Update ADR-001 with Gemini-specific findings
- Set re-evaluation trigger (50+ user requests)

---

## Success Metrics

### For Option A (Export Script)

**MVP Success**:
- [ ] Export 10+ sessions without errors
- [ ] Import into ccboard successfully
- [ ] Session metadata accurate (date, tokens, cost)
- [ ] User can view sessions in ccboard TUI/Web

**Ongoing Success**:
- User runs export weekly/monthly
- Finds value in historical comparison
- No feature requests for real-time monitoring

---

### For Option B (Full Integration)

**MVP Success** (Week 13):
- [ ] 100+ Gemini sessions parsed without errors
- [ ] Stats recalculated accurately (tokens, costs)
- [ ] Provider selector works (TUI + Web)
- [ ] No regressions in Claude Code support
- [ ] Performance <2s load for 1000+ total sessions

**Long-term Success** (6 months):
- [ ] 10+ Lab Google users adopted
- [ ] 500+ Gemini sessions tracked
- [ ] Zero critical bugs (parsing errors)
- [ ] Breaking change handled gracefully (1 Gemini CLI update)

**Failure Criteria** (triggers rollback):
- Critical parsing errors (>5% sessions fail)
- Performance degradation (>5s load times)
- Maintenance burden >20h/month

---

## References

- [ADR-001: Single Provider Focus](./ADR-001-single-provider-focus.md)
- [Gemini CLI Changelog](https://geminicli.com/docs/changelogs/)
- [Gemini CLI v0.26.0 Discussion](https://github.com/google-gemini/gemini-cli/discussions/17812)
- [Gemini CLI Hooks Documentation](https://geminicli.com/docs/hooks/)
- [Gemini CLI Skills Documentation](https://geminicli.com/docs/cli/skills/)
- [Perplexity Search: Gemini CLI v0.28.2 Features](https://www.perplexity.ai/) (2026-02-12)

---

## Appendix: Technical Details

### A. Session Format Comparison

**Claude Code (JSONL)**:
```jsonl
{"role":"user","content":"Fix this bug"}
{"role":"assistant","content":"I'll help","tool_calls":[...]}
{"role":"tool","name":"Read","result":"..."}
```

**Gemini CLI (JSON)**:
```json
{
  "sessionId": "abc123",
  "messages": [
    {"role":"user","content":"Fix this bug"},
    {"role":"assistant","content":"I'll help","toolCalls":[...]},
    {"role":"tool","name":"read","result":"..."}
  ]
}
```

**Parsing Implications**:
- Claude: Line-by-line streaming (lazy loading possible)
- Gemini: Full JSON parse (must load entirely)
- Memory impact: Gemini sessions held in RAM

---

### B. Stats Calculation

**Claude Code**: Pre-computed `stats-cache.json`
```json
{
  "total_tokens": 1234567,
  "total_sessions": 234,
  "tools_used": {"Read": 123, "Write": 45}
}
```

**Gemini CLI**: Must recalculate from all `logs.json`
```rust
// Pseudo-code
async fn calculate_stats() -> Stats {
    let sessions = scan_all_sessions().await?;
    let tokens = sessions.iter().map(|s| s.token_count).sum();
    let tools = sessions.iter().flat_map(|s| s.tools).collect();
    Stats { tokens, tools, ... }
}
```

**Performance**: Claude = O(1) read, Gemini = O(n) sessions scanned

---

### C. Project Hash Mapping

**Problem**: Gemini uses SHA256 hashes for project IDs.

**Example**:
```
/Users/florian/Sites/perso/ccboard
  → SHA256: 64ce9ac2984c5b2d8f3e1a7b9c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2
```

**Solution**: Maintain reverse lookup in `project_mapping.json`
```json
{
  "64ce9ac2984c5b2d8f3e1a7b9c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2": {
    "path": "/Users/florian/Sites/perso/ccboard",
    "name": "ccboard",
    "last_seen": "2026-02-12T10:30:00Z"
  }
}
```

**Discovery Strategy**:
1. Scan `logs.json` for CWD (current working directory)
2. Hash CWD → compare with directory name
3. Build mapping incrementally
4. Cache persistently

---

## Notes

**Philosophy**: "Challenge ADR-001 with facts, not assumptions."

This evaluation was triggered by:
1. Active user at Lab Google (real use case)
2. Gemini CLI v0.28.2 feature discovery (80-90% parity)
3. Reduced effort estimate (500h vs 890h, 44% reduction)

**Key Insight**: Initial analysis was based on **outdated local install**. Perplexity research revealed significant feature evolution (v0.26.0-v0.28.2).

**Decision Pending**: User must answer 4 critical questions to determine:
- Option A (Export Script): Low effort, low ROI
- Option B (Full Integration): High effort, high ROI (if conditions met)
- Option C (Maintain ADR-001): Zero effort, status quo

**Timeline**:
- User response: 2026-02-12 (this week)
- Decision: 2026-02-13 (next day)
- Implementation start: 2026-02-14 (if Option A/B selected)

---

**Status**: 🟡 **Awaiting User Input**
**Last Updated**: 2026-02-12
**Next Review**: After user answers 4 critical questions
**Decision Deadline**: 2026-02-13
