# clippy: narrow significant_drop_in_scrutinee to per-item allows

**Source:** issue #481
**Date:** 2026-05-20
**Tracked in:** #481

## Scope

1. Remove the workspace-level allow `significant_drop_in_scrutinee = "allow"` (and the `# 5 hits …` justifying comment immediately above it) from the root `Cargo.toml`'s `[workspace.lints.clippy]` table.
2. Add per-item `#[allow(clippy::significant_drop_in_scrutinee, reason = "…")]` at each of the 5 lint sites:
   - `quartzite-runtime/src/connection_table.rs:163` — `ConnectionTable::remove`, `if let Some(record) = self.connections.write().remove(&id)` scrutinee.
   - `quartzite-runtime/src/timer_drivers.rs:105` — `ThreadDriver::stop`, `if let Some((thread_handle, join)) = self.handle.lock().take()` scrutinee.
   - `quartzite-runtime/src/timer_drivers.rs:197` — `AppDriver::stop`, same shape as the `:105` site.
   - `quartzite-runtime/src/timer_drivers.rs:407` — `impl Drop for PoolDriver`, `if let Some(handle) = self.inner.handle.lock().take()` scrutinee.
   - `quartzite-runtime/tests/timer.rs:250` — `if let Some(actual) = *observed_thread.lock()` scrutinee in the `AppDriver` integration test.
3. Each `#[allow]` MUST carry a `reason = "…"` string justifying why the drop is deliberately held across the match — the locks/guards in question all guard atomic check-then-mutate sequences (lock-acquire-then-`.take()` / `.remove()` / `.lock()` whose held guard is the whole point of the construct).

## Out of scope

- Refactoring any of the 5 lock-acquire-and-match patterns to release the guard before the body runs. The workspace comment at root `Cargo.toml:47` records the original intent ("deliberately held for atomicity"); the design phase reaffirms it but does not rewrite the locking discipline.
- Touching any other workspace allow in `[workspace.lints.clippy]`. The remaining 5 entries (`must_use_candidate`, `redundant_pub_crate`, `return_self_not_must_use`, `type_complexity`, etc.) are unrelated.
- Changing the workspace lint level for `significant_drop_in_scrutinee` (it inherits from `clippy::nursery = warn` once the explicit `allow` is dropped, escalated to deny by the CI `-D warnings` invocation — exactly the desired posture).

## Deferred

- None.

## Key decisions

| Question | Decision |
|---|---|
| Per-item `#[allow]` vs `#[expect]`? | `#[allow]` per the issue body and per the existing precedent set by PR #443 (`audit-workspace-clippy-allows` — same workspace-allow → per-site narrowing pattern across ~100 sites). `#[expect]` is reserved for cases where the lint firing is the verification signal; here the lint firing on a non-suppressed call site would be a behavioural change we don't want. |
| Reason-string wording? | Design-phase detail. The reason string for each site should name the atomic operation the held guard guards (e.g. for `connection_table.rs:163`: the `connections` write-guard must remain live across both the `record` extraction and the two follow-up `by_receiver` / `by_signal` lookups so concurrent inserts cannot interleave). Workspace-comment rationale ("MutexGuard match-arm patterns … deliberately held for atomicity") is the starting point; per-site specifics belong in the design. |
| Where does the `#[allow]` go on the test site? | Above the `if let Some(actual) = *observed_thread.lock()` statement (statement-level `#[allow]`), not on the test fn — narrower scope. Same approach as PR #443 used for statement-level scrutinee narrows. |

## Technical constraints

- `cargo clippy --workspace --all-targets -- -D warnings` is the binding verification. The `--all-targets` is load-bearing because the test-binary site (`tests/timer.rs:250`) only compiles under `--tests`.
- `[workspace.lints.clippy]` and per-crate `[lints.clippy]` tables are mutually exclusive under Cargo's lint inheritance model. The narrows therefore MUST be in-source `#[allow(..., reason = "…")]` attributes (the same constraint PR #443 documented).
- `reason = "…"` requires Rust 1.81+. The workspace `rust-version` is comfortably past that.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | The line `significant_drop_in_scrutinee = "allow"` and its `# 5 hits …` comment on the immediately-preceding line are removed from root `Cargo.toml`'s `[workspace.lints.clippy]` table. `grep -n 'significant_drop_in_scrutinee' Cargo.toml` returns no hits. |
| AC2 | Each of the 5 sites enumerated in § Scope carries a `#[allow(clippy::significant_drop_in_scrutinee, reason = "…")]` attribute. The `reason =` string is non-empty and names the atomicity / held-guard invariant specific to that site (not a copy-paste placeholder). |
| AC3 | `grep -rn 'significant_drop_in_scrutinee' --include='*.rs' .` returns exactly 5 hits (one per site). |
| AC4 | `cargo clippy --workspace --all-targets -- -D warnings` exits 0. |
| AC5 | `cargo build` exits 0 and `Cargo.lock` is refreshed before commit per AGENTS.md § Workflow. |
| AC6 | `cargo test --workspace` exits 0 — no behavioural change is intended; this is a pure lint-suppression narrow. |
| AC7 | `cargo fmt -- --check` exits 0. Multi-line `#[allow]` attributes follow the existing repo style (PR #443 set the precedent: rustfmt naturally wraps `reason =` strings to one attribute per line). |

## Open questions

- None.
