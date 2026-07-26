# Task 0 report: incremental watcher invocation stats

## What changed and why

`crates/ccboard-core/src/store.rs` now maintains invocation stats (agent/command/skill
counts) as a two-layer structure instead of a single opaque aggregate:

1. **`invocation_by_path: RwLock<HashMap<PathBuf, InvocationStats>>`**, a new field on
   `DataStore`, one `InvocationStats` entry per session file, initialized empty in `new()`.
2. **`compute_invocations()`**, rewritten to scan each session file individually
   (`InvocationParser::scan_session` per path instead of the batch
   `InvocationParser::scan_sessions`), populate `invocation_by_path` file by file, swap it
   into the map under a short write lock, then call the new `fold_invocations()` to
   rebuild the aggregate. Behavior is otherwise unchanged: same per-file parser, same
   skip-and-log-on-error handling for malformed files.
3. **`fold_invocations()`** (new, private), merges every entry in `invocation_by_path`
   into a fresh `InvocationStats` via the existing `InvocationStats::merge()`, then
   recomputes `agent_token_stats` from `session.tool_token_usage["Task"]`. That step has
   to run on the merged aggregate, not per-file, because it distributes tokens across
   whichever agent types appear in the final merged `agents` map. This logic was already
   present in `compute_invocations()`, just moved verbatim into the fold step. Finishes
   with a short write-lock swap of `invocation_stats`.
4. **`refresh_invocations_for(path: &Path)`** (new, public), rescans exactly one file,
   replaces its entry in `invocation_by_path` (short write lock), and calls
   `fold_invocations()`. This is the incremental path: touching one session no longer
   requires rescanning every other session's file to get an accurate aggregate.
5. **`update_session()`**, the file-watcher entry point, now calls
   `self.refresh_invocations_for(path).await` right after inserting the updated
   `SessionMetadata`, so every watcher-triggered session update also refreshes invocation
   stats incrementally.

This is pure plumbing: no skill-name parsing, leaderboard, or CLI code was added. Scope
was kept to `store.rs` plus its embedded test module, per the brief's 2-file budget.
`session_index.rs` was read but not modified. `CACHE_VERSION` was not touched.

### Locking (ADR-001 compliance)

Both write-lock sites (the `invocation_by_path` map swap in `compute_invocations()` and
the single-entry `insert` in `refresh_invocations_for()`) are scoped to a block containing
only the swap or insert itself, never held across an `.await`, and released before
`fold_invocations()` runs (which takes its own short-lived read lock on
`invocation_by_path` and write lock on `invocation_stats`).

## TDD

Test written first (red), then the implementation (green). Per the brief:
`test_watcher_recompute_skills` in `crates/ccboard-core/src/store.rs` (`#[cfg(test)] mod tests`):

1. Write a temp JSONL file with a single `user` line (no tool_use).
2. `store.update_session(&session_path).await`.
3. Assert `store.invocation_stats().total_invocations() == 0`.
4. Append an `assistant` line with a `Skill` tool_use (`"skill":"pdf-generator"`).
5. `store.update_session(&session_path).await` again, same file, same DataStore, no
   restart.
6. Assert `stats.skills.get("pdf-generator") == Some(&1)` and `total_invocations() == 1`.

Red step (before implementation):

```
thread 'store::tests::test_watcher_recompute_skills' panicked at crates/ccboard-core/src/store.rs:2100:9:
assertion `left == right` failed: Incremental refresh must pick up the new skill invocation without a full rescan
  left: None
 right: Some(1)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 327 filtered out; finished in 0.01s
```

Green step (after implementation):

```
$ cargo test -p ccboard-core test_watcher_recompute_skills -- --nocapture
cargo test: 1 passed, 371 filtered out (6 suites, 0.02s)
```

## Exit gate commands

```
$ cargo fmt --all && cargo clippy --all-targets
cargo clippy: No issues found

$ cargo test --all
cargo test: 506 passed, 13 ignored (18 suites, 27.78s)
```

Zero clippy warnings, zero test failures, 506 passed (up from 505 baseline, the one new
test). The 13 ignored tests are pre-existing (integration tests gated behind
`#[cfg(feature = "integration")]` needing a real `~/.claude` directory, unrelated to this
change).

## Manual verification note

This was validated exclusively through the automated unit test above. I did not launch
the TUI or web frontend to click through a live file-watcher scenario, so I'm not
claiming that interactive check was performed. The unit test is a faithful proxy for the
watcher path: `update_session()` is the exact function `ccboard-core`'s `notify`-based
file watcher calls on a debounced file-change event (see `CLAUDE.md`'s "Concurrency
Model" section, 500ms debounce, then dispatch to `DataStore::update_session`). The test
exercises that same function twice against the same file on the same live `DataStore`
without any process restart, which is the property Phase 0 needed to prove: no full
rescan of `invocation_by_path`, no restart, aggregate reflects the second scan.

## Commit

Branch: `feat/skill-usage-analytics` (current branch, as instructed). Committed
`crates/ccboard-core/src/store.rs` only. `.serena/project.yml` was already modified in
the working tree before this task started (unrelated, pre-existing change) and was left
out of this commit.

## Fix round 1: lost-update race between compute_invocations() and refresh_invocations_for()

### Finding

Task review flagged an Important issue: `compute_invocations()` replaced the entire
`invocation_by_path` map unconditionally (`*guard = per_file;`) with a snapshot built by
scanning `self.sessions` sequentially, a slow operation. `refresh_invocations_for()` only
inserted a single entry and had no way to detect a concurrent full-scan write clobbering
it afterward. The two are not mutually exclusive at the call site: `crates/ccboard/src/main.rs`
spawns `compute_invocations()` as a background `tokio::spawn` task in `run_web()` (line
507-510) and starts the file watcher immediately after (line 512-521), and the same
pattern repeats in `run_both()` (`compute_invocations()` at line 566, watcher at line
572-581). If the watcher fired `refresh_invocations_for()` for a changed file while the
background full scan was still mid-flight, the full scan's later completion would
silently overwrite `invocation_by_path` with its own stale snapshot for that path,
discarding the fresher incremental entry, exactly the startup window on large
`~/.claude` directories where this feature matters most.

### Fix

Added a monotonic `invocation_epoch: AtomicU64` field on `DataStore`. Every
`invocation_by_path` entry is now tagged `(epoch, InvocationStats)` instead of bare
`InvocationStats`:

- `refresh_invocations_for()` reserves a fresh epoch with `fetch_add` before taking the
  write lock, then inserts `(epoch, stats)`.
- `compute_invocations()` snapshots `start_epoch = invocation_epoch.load()` *before* it
  starts its slow sequential per-file scan. Once the scan finishes, instead of a blind
  `*guard = per_file` replace, it calls a new private `merge_invocation_snapshot()`
  which, for each scanned path, only overwrites the map entry if the entry's current
  epoch is not newer than `start_epoch`. An entry with a newer epoch means
  `refresh_invocations_for()` wrote it after the full scan began, so the full scan's
  now-stale read is discarded instead of clobbering it. `merge_invocation_snapshot()`
  also drops entries for sessions no longer present in the live session set, preserving
  the original full-scan removal semantics, but never drops an entry with a newer epoch
  than `start_epoch`, for the same reason.
- The merge itself is synchronous HashMap work with no I/O or `.await`, so the write
  lock scope is unchanged in kind (still a single short critical section), even though
  it's now O(n) merge instead of O(1) swap. This is the necessary trade-off for
  correctness here: ADR-001 requires the lock not be held across an `.await`, which
  still holds, not that the critical section be O(1).
- `merge_invocation_snapshot()` was split out of `compute_invocations()` as a private
  method taking `start_epoch` as an explicit parameter (rather than reading the atomic
  internally) specifically so a test can drive the exact race deterministically: the
  real `AtomicU64`'s live value can't be pinned to a "stale" snapshot from the outside
  once `compute_invocations()` is mid-call, so the race can't be reproduced through the
  public API alone with a stable interleaving point.

Also fixed the reviewer's minor/cosmetic note: removed the redundant
`aggregated.sessions_analyzed = 0;` in `fold_invocations()` (dead code immediately after
`InvocationStats::new()`, which already zero-initializes that field).

### New test

`test_compute_invocations_does_not_clobber_concurrent_refresh` in
`crates/ccboard-core/src/store.rs`:

1. Seeds `invocation_by_path` directly with a fake path at epoch 1, carrying a skill
   invocation, standing in for what `refresh_invocations_for()` would have written
   concurrently.
2. Calls `merge_invocation_snapshot(0, stale_snapshot, &live_paths)` directly, standing
   in for `compute_invocations()`'s write-back of a `start_epoch = 0` snapshot for the
   same path (an empty `InvocationStats`, as if scanned before the concurrent skill
   invocation happened).
3. Asserts the map entry after the merge still has epoch 1 and the skill invocation
   count intact. Before the fix (calling the old unconditional `*guard = per_file`
   logic in place of `merge_invocation_snapshot`), this assertion fails: the stale
   snapshot silently overwrites the fresher entry.

### Commands run

```
$ cargo fmt --all && cargo clippy --all-targets
cargo clippy: No issues found

$ cargo test -p ccboard-core --lib compute_invocations_does_not_clobber -- --nocapture
cargo test: 1 passed, 328 filtered out (1 suite, 0.01s)

$ cargo test -p ccboard-core test_watcher_recompute_skills -- --nocapture
cargo test: 1 passed, 372 filtered out (6 suites, 0.01s)

$ cargo test --all
cargo test: 507 passed, 13 ignored (18 suites, 30.10s)
```

507 passed (up from 506 in the round-0 report, the one new regression test), zero
clippy warnings, zero failures. The 13 ignored tests remain the same pre-existing
integration tests noted above, unrelated to this change.

### Files touched

`crates/ccboard-core/src/store.rs` only. No changes to `session_index.rs`, no
`CACHE_VERSION` bump, no skill-parsing/leaderboard/CLI code, consistent with the
original Phase 0 scope boundaries.
