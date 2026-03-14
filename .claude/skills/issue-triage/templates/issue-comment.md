# Issue Comment Templates

Use these templates to generate GitHub issue comments. Select the appropriate template based on the recommended action from Phase 2. Comments are posted in **English** (international audience).

---

## Template 1 -- Acknowledgment + Request Info

Use when: issue is valid but missing information to act on it (reproduction steps, version, install method, environment).

```markdown
## Issue Triage

**Category**: {Bug | Feature | Enhancement | Question}
**Priority**: {P0 | P1 | P2 | P3}
**Effort estimate**: {XS | S | M | L | XL}

### Assessment

{1-2 sentences: what this issue is about and why it matters. Be direct.}

### Missing Information

To move forward, we need the following:

- {Specific missing info 1 -- e.g., "ccboard version (`ccboard --version` output)"}
- {Specific missing info 2 -- e.g., "Install method used (Homebrew / cargo install / install script / build from source)"}
- {Specific missing info 3 -- e.g., "OS and architecture (macOS ARM64, Linux x86_64, etc.)"}

### Next Steps

{What happens once the info is provided -- e.g., "Once confirmed, we'll prioritize this for the next release."}

---
*Triaged via [ccboard](https://github.com/FlorianBruniaux/ccboard) `/issue-triage`*
```

---

## Template 2 -- Duplicate

Use when: this issue is a duplicate of an existing open (or recently closed) issue.

```markdown
## Duplicate Issue

This issue covers the same problem as #{original_number}: **{original_title}**.

### Overlap

{1-2 sentences explaining the overlap -- what's identical or nearly identical between the two issues.}

If your situation differs in an important way (different install method, different OS, different error output), please reopen and add that context. Otherwise, follow the original issue for updates.

---
*Triaged via [ccboard](https://github.com/FlorianBruniaux/ccboard) `/issue-triage`*
```

---

## Template 3 -- Close (Stale)

Use when: issue has had no activity for >90 days and there's been no engagement.

```markdown
## Closing: No Activity

This issue has been open for {N} days without activity. To keep the backlog actionable, we're closing it.

If this is still relevant:
- Reopen and add context about your current setup and ccboard version
- Or reference this issue in a new one if the problem has evolved

Thanks for taking the time to report it.

---
*Triaged via [ccboard](https://github.com/FlorianBruniaux/ccboard) `/issue-triage`*
```

---

## Template 4 -- Close (Out of Scope)

Use when: issue requests something that doesn't align with ccboard's design goals (e.g., write operations to ~/.claude, multi-provider support, non-Claude Code integrations).

```markdown
## Closing: Out of Scope

After review, this request falls outside ccboard's current design goals.

### Rationale

{1-2 sentences explaining why -- be specific. Reference design constraints if relevant, e.g., "ccboard is intentionally read-only -- it never writes to ~/.claude to ensure zero risk of data corruption or accidental config changes."}

### Alternatives

{If applicable: what the user can do instead. E.g., "For this use case, the `/api/sessions` endpoint (available when running `ccboard web`) returns the raw session data you can pipe into your own tooling."}

If the use case evolves or the scope changes in a future version, feel free to reopen with updated context.

---
*Triaged via [ccboard](https://github.com/FlorianBruniaux/ccboard) `/issue-triage`*
```

---

## Formatting Rules

**Tone**: Professional, constructive, factual. Help the user move forward. Challenge the issue scope, not the person who filed it.

**Length**: 100-250 words per comment. Long enough to be useful, short enough to respect the reader's time.

**Specificity**: Always name the exact command, file, or behavior in question. Vague comments waste everyone's time.

**No superlatives**: Don't write "great issue", "excellent report", "amazing catch". Just address the substance.

**Priority labels**:
- P0 -- Critical: security vulnerability, data loss, broken core functionality (startup crash, data not loading)
- P1 -- High: significant bug affecting common workflows, actionable this sprint
- P2 -- Medium: valid issue, queue for backlog
- P3 -- Low: nice-to-have, future consideration

**Effort labels**:
- XS: <1 hour
- S: 1-4 hours
- M: 1-2 days
- L: 3-5 days
- XL: >1 week

**ccboard-specific context to include when relevant**:
- Always ask for `ccboard --version` as first diagnostic step for bug reports
- Ask for install method (Homebrew / `cargo install` / install script / build from source) -- behavior differs significantly between methods
- For Web UI 404s: likely issue #44 (`cargo install` does not bundle WASM frontend) -- direct to Homebrew or install script
- Reference relevant crate when known: `ccboard-core/src/parsers/`, `ccboard-tui/src/tabs/`, `ccboard-web/src/`
- Performance constraint: startup must remain <2s for 1000+ sessions (mention when rejecting heavy dependencies)
- Read-only design: ccboard never writes to `~/.claude` -- mention when rejecting write-operation requests
