# Task 2 (TUI) report: Skills leaderboard tab + needs_refresh fix

Commit: `7ee29d0` on `feat/skill-usage-analytics`, on top of Phase 1's `b3d6bcf`.

## What was built

1. **New tab** `crates/ccboard-tui/src/tabs/skills.rs` (`SkillsTab`): a Skills
   leaderboard, distinct from the existing "Skills" sub-tab in `tabs/agents.rs`
   (which lists on-disk definitions, not usage). Renders a `Table` sorted by
   invocation count descending: skill name, uses, projects, tokens, cost,
   last-seen, plus an optional detail pane (`Enter` toggles it) with full
   first/last-seen timestamps. `j/k`/arrows navigate, `g`/`G` jump top/bottom.
   Empty state uses the existing `EmptyState` widget.

   `[retired]` badge: `known_skill_names: HashSet<String>` is populated once
   at startup from the on-disk skill scan (`scan.2`, the same `Vec<AgentEntry>`
   the Agents tab uses) and recomputed against at render time via
   `is_retired()`. No "defined" flag is ever persisted, matching the design
   doc's four-case matrix. Names are stored and compared verbatim, never split
   on `:`, so `superpowers:brainstorming` and a local `brainstorming` are
   correctly treated as distinct rows (covered by a unit test).

2. **Registration**: `Tab::Skills` variant added to `app.rs` (all six match
   arms: `all()`, `index()`, `from_index()`, `name()`, `shortcut()` (`'k'`,
   decorative like Plugins/Search/Brain), and `icon()` (🏆). `tabs/mod.rs`
   exports `SkillsTab`. `ui.rs` gained a `skills: SkillsTab` field, wired into
   `handle_tab_key`, `render_tab_content`, and the status-bar hint match.
   `help_modal.rs` got a `Tab::Skills` arm (all these matches are exhaustive
   with no wildcard, so the compiler forced every site).

3. **The needs_refresh fix** (required addition, not in the original design
   doc): confirmed by reading `app.rs` and `lib.rs` that `poll_events()` set
   `needs_refresh = true` on `StatsUpdated | SessionCreated | SessionUpdated |
   AnalyticsUpdated | LoadCompleted`, but nothing ever read it back: `ui.rs`
   called `agents.update_invocation_counts()` exactly once, inside
   `Ui::init()`, at startup only. Found the tick loop in
   `run_loop_with_loading()` in `lib.rs`. Added a new `Ui::refresh_invocation_data()`
   method that re-fetches `store.invocation_stats()` and refreshes both
   `agents.update_invocation_counts()` and `skills.refresh()`. The tick loop
   now calls it whenever `app.needs_refresh && !app.is_loading`, then clears
   the flag. This is in-memory work reading an already-computed `DataStore`
   aggregate, not a rescan, matching the existing `SkillsTab::refresh()`
   contract, which reads the SQLite-backed `skill_leaderboard()` query, cheap
   by construction (Phase 1's design).

4. **`classify_plugin()` fix** (`crates/ccboard-core/src/analytics/plugin_usage.rs`):
   changed the skill and command matching from pure `.contains()` to exact
   match first (`Vec::contains` on the lowercased full name), falling back to
   substring only if no exact match is found. Added a regression test
   (`test_classify_plugin_exact_match_avoids_namespace_false_positive`)
   documenting the namespace scenario from the design doc. Note: for this
   particular function the exact-vs-substring distinction doesn't change the
   returned `PluginType` in the tested case (both paths yield `Skill`), since
   the function only returns a boolean-ish classification, not skill
   identity. The fix is implemented exactly as specified, and the test
   documents intended behavior rather than a diverging enum value.

5. **Agents tab tightening** (`tabs/agents.rs`): added an explicit `[unused]`
   label on Skill entries, gated strictly on `invocation_count == 0` (the same
   `InvocationStats`-derived source of truth the new Skills tab reads), next
   to the existing `(× N)` badge for `invocation_count > 0`. This makes the
   Agents tab's unused/used signal share one source of truth with the Skills
   leaderboard instead of silently diverging.

## Verification

```
cargo build --all                    # 5 crates, clean
cargo fmt --all                      # clean
cargo clippy --all-targets           # 0 errors, 0 warnings (2 clippy nits on
                                      #   the contains()/iter().any() pattern
                                      #   in plugin_usage.rs were fixed before
                                      #   final run)
cargo test --all                     # 524 passed, 13 ignored, 18 suites
```

Did not manually launch the TUI (not required per the brief; the controller
validated the `needs_refresh` gap manually beforehand and will spot-check the
fix the same way). All new logic (retired-badge computation, sort order,
namespace matching) is covered by unit tests colocated with the code, matching
this project's existing style for TUI tabs (pure logic tested, rendering not).

## Files touched

- `crates/ccboard-tui/src/tabs/skills.rs` (new)
- `crates/ccboard-tui/src/tabs/mod.rs`
- `crates/ccboard-tui/src/app.rs`
- `crates/ccboard-tui/src/ui.rs`
- `crates/ccboard-tui/src/lib.rs`
- `crates/ccboard-tui/src/components/help_modal.rs`
- `crates/ccboard-tui/src/tabs/agents.rs`
- `crates/ccboard-core/src/analytics/plugin_usage.rs`

Did not touch `ccboard-web/*` (leptos-designer's parallel scope) or
`session_index.rs`, per the brief's boundaries. Did not add sparklines,
`--show-context`, or SSE live-push, per the design doc's explicit hors-scope
list.

## Fix round 1

Review verdict: NOT APPROVED, two Important findings, both confirmed correct.

### Important 1: `[retired]` badge false-fired on 81% of real leaderboard entries

Root cause was exactly what the original report flagged as a concern rather
than fixed: `known_skill_names` only held bare local `.claude/skills/*`
directory names, so any plugin-namespaced invocation (`superpowers:*`,
`vercel:*`, ...) never matched and got an affirmative `[retired]` claim, even
while actively used, directly contradicting the Agents tab's `(× N)` badge for
the same string on the same screen.

Fix: added `scan_known_skill_names_blocking()` in `tabs/agents.rs`, which
walks both `~/.claude/skills/*/SKILL.md` (local, bare names) and
`~/.claude/plugins/{marketplaces,cache}/**/skills/*/SKILL.md` (installed
plugin skills, one walk handles both the `marketplaces/<marketplace>/skills/`
and the `cache/<marketplace>/<plugin>/<version>/skills/` layouts, verified
against my own `~/.claude/plugins` tree, 485 real `SKILL.md` files). It
collects lowercased *leaf* directory names only (never a resolved
`<plugin>:<skill>` mapping), because the exact plugin-to-namespace mapping
isn't reliably derivable from directory structure: the two trees nest
differently and the namespace prefix used at invocation time is the plugin
name, not the marketplace directory name. `SkillsTab::is_retired()` now
strips the invocation name down to its leaf (`rsplit(':').next()`) before
checking membership, so `superpowers:brainstorming` is confirmed defined by
any installed plugin skill leaf named `brainstorming`, regardless of which
namespace it actually came from. This is a deliberate bias toward false
"not retired" over false "retired", matching the reviewer's own fallback
rule and documented in a doc comment on `known_skill_names`.

Also folded in Minor 1 (case-sensitivity: `set_known_skills` and
`is_retired` both lowercase now, so `~/.claude/skills/TDD/` matches an
invocation recorded as `tdd`) and Minor 2 (staleness: the catalogue is no
longer scanned once at `Ui::init()` and left static. `SkillsTab` now owns a
`tokio::sync::mpsc` channel; `request_catalogue_rescan()` spawns a
`spawn_blocking` walk and `poll_catalogue_update()` drains it non-blockingly.
`Ui::refresh_invocation_data()` kicks off a rescan every time `needs_refresh`
fires, and `lib.rs`'s tick loop calls `ui.poll_skill_catalogue()`
unconditionally every iteration to drain whatever completed, so the badge
recomputes across a skill install/removal within one refresh cycle instead of
requiring a restart).

Six new unit tests in `tabs/agents.rs::catalogue_tests` (using a real
temp-dir fixture, not mocks) cover: local skills found, plugin skills found
under both `marketplaces/` and `cache/<version>/` layouts, hidden directories
skipped, and lowercasing. Four tests in `tabs/skills.rs` cover the leaf-match
`is_retired()` semantics directly, including the explicit trade-off case
(one installed plugin skill leaf confirms any namespace using that leaf) and
the case-insensitivity fix.

### Important 2: `classify_plugin()` fix was a no-op

Confirmed the reviewer's proof: `lower_skills.contains(&lower_name) ||
lower_skills.iter().any(|s| lower_name.contains(s))` reduces to the second
clause since exact match implies substring match, so round 1's diff changed
nothing observable, and the round 1 test passed against the pre-fix function
too, meaning it didn't guard anything.

Restructured per the reviewer's suggested shape: exact matches across both
skills and commands are checked before any substring fallback in either
category, so an exact command match can no longer lose to an unrelated
skill's substring collision (or vice versa). Replaced the non-discriminating
test with `test_classify_plugin_exact_command_beats_substring_skill`, which
uses `skills = ["work"]`, `commands = ["network"]`, name `"network"`: the old
flat OR-chain checked skills first and `"network".contains("work") == true`,
misclassifying an exact command match as a Skill. Verified by hand that this
assertion fails against the pre-round-1 original function and passes against
the current one. Kept a second test confirming the substring fallback still
applies when no exact match exists anywhere (the original namespace
scenario), so that behavior isn't silently lost.

### Verification (fix round 1)

```
cargo build --all             # 5 crates, clean
cargo fmt --all                # clean
cargo clippy --all-targets     # 0 errors, 0 warnings
cargo test --all               # 532 passed, 13 ignored, 18 suites
```

New/changed test coverage this round: 6 tests in
`tabs/agents.rs::catalogue_tests`, 4 tests in `tabs/skills.rs` (2 rewritten
for leaf semantics, 1 new case-insensitivity test, 1 new genuinely-removed
test), 2 tests in `plugin_usage.rs` (1 new discriminating test replacing the
no-op, 1 new substring-fallback-still-works test).

Files touched this round (in addition to round 1's list):
`crates/ccboard-tui/src/tabs/skills.rs`, `crates/ccboard-tui/src/tabs/agents.rs`,
`crates/ccboard-tui/src/ui.rs`, `crates/ccboard-tui/src/lib.rs`,
`crates/ccboard-core/src/analytics/plugin_usage.rs`. Same scope boundaries
as round 1: no `ccboard-web/*`, no `session_index.rs`.
