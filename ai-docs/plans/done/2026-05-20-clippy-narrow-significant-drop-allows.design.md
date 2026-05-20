# Design: clippy — narrow `significant_drop_in_scrutinee` to per-item allows

**Issue:** #481
**Date:** 2026-05-20

## Approach

**Chosen solution.** Delete the workspace-level `significant_drop_in_scrutinee = "allow"` line (and its `# 5 hits …` justifying comment immediately above it) from root `Cargo.toml`'s `[workspace.lints.clippy]` table, then add a per-site `#[allow(clippy::significant_drop_in_scrutinee, reason = "…")]` attribute at each of the 5 enumerated lint sites. The lint then inherits `clippy::nursery = warn` and is escalated to deny by CI's `-D warnings`, with the 5 deliberate cases individually documented.

This follows the precedent set by PR #443 (`audit-workspace-clippy-allows`), which narrowed several workspace-wide clippy allows to per-site attributes across ~100 sites. The reason-string discipline (one non-placeholder sentence per site) and rustfmt-friendly multi-line attribute layout are inherited from that PR.

**Site classification.** All 5 sites fall into the same pattern family: a `parking_lot::{Mutex,RwLock}` guard whose `.take()` / `.remove()` / direct dereference is the scrutinee of an `if let` (or `match`) expression, and whose held lifetime is load-bearing for the atomicity of the surrounding read-modify or check-then-act sequence. The reason strings therefore name the *specific atomic operation* the held guard guards at each site — not a generic "lock held intentionally" copy-paste. Reason strings are drafted below as design-phase detail (per spec § Key decisions row 2).

**Attribute placement.**
- 4 of 5 sites (`connection_table.rs:163`, `timer_drivers.rs:105`, `:197`, `:407`) are `if let` expressions at the top of a single-statement function or `Drop::drop` body — statement-level `#[allow]` directly above the `if let` line is the narrowest scope that suppresses the lint.
- The 5th site (`tests/timer.rs:250`) is also an `if let` statement near the end of an integration test fn; statement-level placement per spec § Key decisions row 3 (not on the test fn — narrower scope).

**Rejected alternatives.**

1. **Keep the workspace allow.** Rejected — the issue (#481) explicitly tracks the audit-narrowing follow-up of PR #443, and a 5-hit workspace allow loses signal: a future 6th site (e.g. a new lock-acquire-and-match introduced during refactoring) would be silently absorbed instead of forcing the author to justify it per-site.
2. **Refactor the 5 sites to release the guard before the body runs** (e.g. `let extracted = self.handle.lock().take(); if let Some(...) = extracted { ... }`). Rejected per spec § Out of scope — the workspace comment at `Cargo.toml:47` records the intent ("deliberately held for atomicity"), and the design phase reaffirms it. Some of the sites *require* the held guard for correctness (the `ConnectionTable::remove` case crosses three guards and a `record` extraction; releasing the outer `connections` write-guard mid-flow would open a race window).
3. **`#[expect]` instead of `#[allow]`.** Rejected per spec § Key decisions row 1 — `#[expect]` is reserved for cases where lint-firing is the verification signal; here lint-firing on a non-suppressed site would be a behavioural change we don't want.
4. **Per-crate `[lints.clippy]` table in `quartzite-runtime/Cargo.toml`.** Rejected — would still allow the lint across the *entire* `quartzite-runtime` crate (vs. the 5 named sites), losing the per-site documentation. Also blocked by the spec § Technical constraints note: per-crate `[lints.clippy]` tables are mutually exclusive with the workspace-inherited `[lints] workspace = true` block.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `#[allow(clippy::significant_drop_in_scrutinee, reason = "…")]` at the 4 production sites in `quartzite-runtime/src/` (one site in `connection_table.rs`, three in `timer_drivers.rs`). Each site receives a reason string drafted from the per-site rationale table below. | `quartzite-runtime/src/connection_table.rs`, `quartzite-runtime/src/timer_drivers.rs` | — |
| 2 | Add `#[allow(clippy::significant_drop_in_scrutinee, reason = "…")]` at the test-binary site `quartzite-runtime/tests/timer.rs:250`. | `quartzite-runtime/tests/timer.rs` | — |
| 3 | Remove the workspace allow `significant_drop_in_scrutinee = "allow"` and its `# 5 hits …` justifying comment from root `Cargo.toml`'s `[workspace.lints.clippy]` table. Run `cargo build` to refresh `Cargo.lock` per AGENTS.md § Workflow. | `Cargo.toml`, `Cargo.lock` | 1, 2 |
| 4 | Verification gates: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build`, `cargo test --workspace`, `cargo fmt -- --check`, `grep -rn 'significant_drop_in_scrutinee' --include='*.rs' .` returns exactly 5 hits, `grep -n 'significant_drop_in_scrutinee' Cargo.toml` returns 0 hits. | — (verification only) | 3 |

### Per-site reason-string drafts (design-phase detail; finalised at implementation)

| Site | `reason = "…"` draft |
|---|---|
| `connection_table.rs:163` (`ConnectionTable::remove`) | `"connections write-guard must remain live across record extraction and the two by_receiver / by_signal lookups so a concurrent insert cannot observe a half-removed entry"` |
| `timer_drivers.rs:105` (`ThreadDriver::stop`) | `"handle mutex guard held across .take() and the unpark+join shutdown sequence so a concurrent start cannot install a fresh handle mid-shutdown"` |
| `timer_drivers.rs:197` (`AppDriver::stop`) | `"handle mutex guard held across .take() and the unpark+join shutdown sequence so a concurrent start cannot install a fresh handle mid-shutdown"` (same shape as :105) |
| `timer_drivers.rs:407` (`impl Drop for PoolDriver`) | `"handle mutex guard held across .take() and the join() drop-finalisation so the pool thread cannot be racingly re-spawned during teardown"` |
| `tests/timer.rs:250` (`AppDriver` integration test) | `"observed_thread mutex guard held across the deref-Some match so the assertion observes the same value the if-let bound, not a value mutated by a still-running tick"` |

These are *drafts* — the implementer may tighten wording, but must preserve the per-site specificity (no copy-paste placeholders, no generic "intentional lock-hold" boilerplate). The `:105` / `:197` shape-pair is the one acceptable repetition because the two `stop` methods *are* literal shape twins; the words still accurately describe each site.

## Handoff plan

`M = 4`. Two groups, 3 + 1.

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` enters Group A with fresh context.
- **Group A:** subtasks 1–3 — full implementation chunk (per-site `#[allow]` attributes on production + test sites, then workspace-allow removal with `Cargo.lock` refresh). 3 subtasks; matches the non-terminal cap of exactly 3.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtask 4 — terminal group (1 subtask; within the 1..=3 range). Verification-only: runs the AC1–AC7 gate commands and reports.

## Risks

- **Reason-string copy-paste drift (AC2 says non-empty + site-specific).** Mitigation: per-site draft table above gives the implementer 5 distinct strings to start from. Self-review should check that no two reason strings on the production sites are byte-identical except the `:105` / `:197` shape-twin pair (which is acceptable). The `design-review` and `self-review` agents both treat copy-paste reason strings as a `major` finding under PR #443 precedent.
- **`reason =` MSRV.** `reason = "…"` requires Rust 1.81+; workspace `rust-version` is comfortably past that (spec § Technical constraints). No mitigation needed; named here for completeness.
- **Multi-line attribute formatting drift from rustfmt.** Long reason strings may force rustfmt to wrap. PR #443 set the precedent (rustfmt wraps to one-attribute-per-line naturally). Mitigation: `cargo fmt -- --check` is part of the AC7 gate; if rustfmt rewrites the attribute layout, accept its output rather than fighting it.
- **A new `significant_drop_in_scrutinee` site is introduced concurrently** (e.g. by another PR merged between design and implementation). Mitigation: the AC3 grep "returns exactly 5 hits" gate would catch it; the implementer should re-grep at implementation time and, if 6+ hits surface, surface as a Spec Amendment per `.claude/skills/task/SKILL.md` Step 7 rather than silently allowlisting the 6th.
- **`cargo clippy --workspace --all-targets` not exercising the lint due to feature-gating.** All 5 sites compile under the default feature set + `--tests`; no feature gates required. The spec § Technical constraints already names `--all-targets` as load-bearing because the `tests/timer.rs:250` site only compiles under `--tests`.
- **Workspace-comment regression.** Removing the `# 5 hits …` comment loses the rollup count. Acceptable: the per-site reason strings supersede the rollup; future readers `grep` the codebase if they need the count.

## Test Design

No new tests. This is a pure lint-suppression narrowing — the spec § Acceptance Criteria AC6 (`cargo test --workspace` exits 0) is the regression gate confirming no behavioural change.

- **Verification command set** (subtask 4):
  - `cargo clippy --workspace --all-targets -- -D warnings` — AC4
  - `cargo build` — AC5 + refreshes `Cargo.lock`
  - `cargo test --workspace` — AC6 (no regression)
  - `cargo fmt -- --check` — AC7
  - `grep -n 'significant_drop_in_scrutinee' Cargo.toml` — AC1 (expect 0 hits)
  - `grep -rn 'significant_drop_in_scrutinee' --include='*.rs' .` — AC3 (expect exactly 5 hits)

## Open questions

- None. The spec § Key decisions table resolved the `#[allow]` vs `#[expect]` question, the attribute-placement question (statement-level on the test site), and the reason-string-specificity question (design-phase per-site detail). The reason-string drafts in this design satisfy that last commitment; the implementer may tighten wording without re-entering design.
