# Phase 2 report: Web skills leaderboard page + Axum route

Commit: on `feat/skill-usage-analytics`, on top of Phase 1's `b3d6bcf` and Phase 2 TUI's `7ee29d0`.

## What was built and why

### Axum route (`crates/ccboard-web/src/router.rs`)

`GET /api/skills/leaderboard`, registered alongside the existing `/api/skills`
route. `skills_leaderboard_handler` calls the existing
`DataStore::skill_leaderboard()` (Phase 1, zero-arg, SQLite-backed, already
sorted by `total_tokens` descending) and annotates each entry with a `retired:
bool` computed at request time, never persisted.

`on_disk_skill_names()` builds the "defined" set from `~/.claude/skills/*/SKILL.md`
bare directory names. This intentionally reuses the exact source the existing
`skills_handler`/`scan_skills_recursive` already scan. I initially built a more
elaborate resolver that also walked `~/.claude/plugins/cache/**/skills/*/SKILL.md`
to catch namespaced plugin skills like `superpowers:brainstorming` (verified on
my own `~/.claude` that these live outside `~/.claude/skills` entirely, so a
naive comparison would falsely retire every plugin skill). I dropped that after
reading the TUI implementer's `task-2-tui-report.md`: their `SkillsTab` computes
`[retired]` from the same bare `~/.claude/skills` scan `scan_all_blocking` already
uses for the Agents tab, with no plugin-namespace resolution either. Shipping my
more thorough version would have caused a cross-surface parity bug (a plugin
skill shown retired in Web but not TUI), which is worse than the two surfaces
sharing one known, documented blind spot. Coordinated via direct message before
and after the decision; the doc comment on `on_disk_skill_names()` records the
blind spot for whoever eventually builds a real plugin-catalogue resolver.

Response shape: `{"entries": [{skill, invocations, projects, total_tokens,
estimated_cost_usd, first_seen, last_seen, retired}], "total": N}`, snake_case
field names matching `SkillLeaderboardEntry` verbatim (same convention as the
existing `plugins.rs` page's `PluginUsage`/`PluginAnalytics` DTOs, which also
skip `rename_all = "camelCase"`).

### Leptos page (`crates/ccboard-web/src/pages/skills.rs`)

`Skills` component: a single sortable-by-nature table (server already sorts by
tokens desc), columns skill/invocations/projects/tokens/est. cost/first
seen/last seen, `[retired]` badge (reusing the existing `.badge.badge-warning`
class, with a `title` tooltip) next to the skill name when flagged. Loading via
`LocalResource` + `Suspense`, matching the `PluginsPage`/`Costs` pattern
exactly. Empty state and error state follow the same markup convention as
`plugins.rs`. No sparklines, no graphs, no drilldown, no SSE, per the brief's
explicit scope boundary.

Table CSS added to `static/style.css` (`.skills-table*`), modeled directly on
the pre-existing `.sessions-table` block rather than introducing new visual
language.

### Navigation

One-line registrations: `pages/mod.rs` (`mod skills; pub use skills::Skills;`),
`app.rs` (`<Route path=path!("/skills") .../>`), `sidebar.rs` (new nav item
with an inline SVG icon, placed before "Tasks", following the existing
per-link markup pattern exactly).

## Coordination

Messaged `phase2-tui-implementer` before starting to ask how they were
resolving the on-disk skill catalogue for `[retired]`, since the design doc
requires this logic to agree across TUI/Web. Their report landed before a
reply came back; read it directly, confirmed the shared source, simplified my
already-written plugin-cache-walking version down to match, and sent a
closing-the-loop message documenting the decision.

## Verification

```
cargo build --all                                                    # 2 crates rebuilt, clean
cargo fmt --all                                                       # clean
cargo clippy -p ccboard-web --all-targets --features ssr              # 0 warnings
cargo clippy -p ccboard-web --no-default-features --features hydrate \
  --target wasm32-unknown-unknown                                     # 0 warnings
cargo test --all                                                      # 524 passed, 13 ignored, 18 suites
```

`trunk build` (the real frontend build pipeline) was attempted twice and both
times the process was killed by signal 9 (OOM) inside this sandboxed
environment, independent of `--release` or `CARGO_BUILD_JOBS`. As a substitute
verification of the actual WASM/hydrate compilation path (the part trunk
delegates to `cargo build --target wasm32-unknown-unknown` before running
wasm-bindgen/wasm-opt), I ran `cargo build`/`cargo clippy -p ccboard-web
--no-default-features --features hydrate --target wasm32-unknown-unknown`
directly (the same flag combination `Trunk.toml` configures), both clean. This
confirms the Leptos component code and all frontend-side types compile
correctly for WASM; it does not confirm the final wasm-bindgen/asset-embedding
step, which I could not exercise in this environment. Did not manually launch
`trunk serve` + browser-test for the same reason (no successful `trunk build`
to serve from).

## Files touched

- `crates/ccboard-web/src/router.rs` (new route + handler + on-disk skill scan helper)
- `crates/ccboard-web/src/pages/skills.rs` (new)
- `crates/ccboard-web/src/pages/mod.rs`
- `crates/ccboard-web/src/app.rs`
- `crates/ccboard-web/src/components/sidebar.rs`
- `crates/ccboard-web/static/style.css`

Did not touch `ccboard-tui/*` or `ccboard-core/src/analytics/plugin_usage.rs`
(TUI implementer's scope), per the brief's file boundaries.
