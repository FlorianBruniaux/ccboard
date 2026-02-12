# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records documenting key architectural decisions made during ccboard development.

## What is an ADR?

An ADR captures an important architectural decision along with its context and consequences. It creates a historical record of why certain choices were made.

## ADR Index

| ID | Title | Status | Date | Description |
|----|-------|--------|------|-------------|
| [ADR-001](./ADR-001-single-provider-focus.md) | Single Provider Focus (Claude Code Only) | Accepted | 2026-02-12 | Decision to remain Claude Code-only instead of supporting multiple AI providers |
| [ADR-002](./ADR-002-gemini-cli-support-evaluation.md) | Gemini CLI Support Evaluation | Evaluation | 2026-02-12 | Evaluation of Gemini CLI v0.28.2 support following feature discovery - challenges ADR-001 with revised data |

## ADR Template

When creating a new ADR, use this structure:

```markdown
# ADR-XXX: [Title]

**Status**: [Proposed/Accepted/Deprecated/Superseded]
**Date**: YYYY-MM-DD
**Decision Makers**: [Who made this decision]
**Context**: [Brief context]

## Context
[Describe the context and problem statement]

## Decision
[The decision that was made]

## Rationale
[Why this decision was made]

## Consequences
### Positive
[Benefits of this decision]

### Negative
[Drawbacks and risks]

## Alternatives Considered
[Other options that were evaluated]

## Success Metrics
[How to measure if this was the right decision]

## References
[Links to relevant documents, issues, or discussions]
```

## Status Definitions

- **Proposed**: Under consideration, not yet decided
- **Accepted**: Decision made and implemented
- **Deprecated**: No longer relevant due to context changes
- **Superseded**: Replaced by a newer ADR (link to new ADR)

## Contributing

When making significant architectural decisions:
1. Create a new ADR with next sequential number
2. Use the template above
3. Update this index
4. Reference the ADR in relevant code/documentation
