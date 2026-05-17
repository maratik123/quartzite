# Design: Replace `serial_test` with a `parking_lot::Mutex`-based test helper

**Issue:** #440
**Date:** 2026-05-17

## Approach

Replace every `#[serial]` / `#[serial_test::serial]` attribute with an in-body `let _lock = test_lock();` acquisition of a `parking_lot::Mutex<()>`. Per spec amendment (post round-1 GO), the static + helper fn are defined **exactly once** in a new workspace member crate `quartzite-test-helpers/` at `quartzite-test-helpers/src/lib.rs`. Each consumer crate declares `quartzite-test-helpers = { path = ... }` in `[dev-dependencies]` and calls `quartzite_test_helpers::test_lock()` from each migrated test body.

**Why shared crate (option (b) — spec amendment):**

- Copy-pasting the `static TEST_LOCK + fn test_lock()` pair across the 6 test-binary sites (4 prod-crate `--lib` binaries — `quartzite-core`, `quartzite-runtime`, `quartzite-style`, `quartzite-style-dispatch` — plus 2 integration-test binaries — `quartzite-runtime/tests/snapshot.rs` and `tests/signal_to_signal.rs`) would mean 6 copies of the same 8 lines, with the same drift risk that the spec's `ai-docs/learnings.md` 2026-05-17 copy-paste-for-≥3-sites learning is meant to prevent.
- A shared crate centralises the helper in one place — exactly one `static TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());` declaration in the workspace, exactly one `pub fn test_lock() -> parking_lot::MutexGuard<'static, ()>` definition.
- **Per-binary serialisation semantics are preserved.** Each Rust test binary (`--lib` per crate + each `tests/*.rs` integration test) is a separate process with its own address space. Even though the `static TEST_LOCK` is declared in one source file, each downstream test binary that links `quartzite-test-helpers` ends up with **its own instance** of that static — same per-binary serialisation contract as the previous per-crate-static design, plus the same contract `serial_test` itself provided (cross-binary test isolation was never on offer, per spec § Out of scope).
- The helper crate is a workspace leaf with one regular `[dependencies]` entry (`parking_lot = "0.12"`) and no inter-workspace coupling. It does not depend on any other workspace member — no cycle risk in v1.

**Helper crate layout:**

```
quartzite-test-helpers/
├── Cargo.toml         # [package] name = "quartzite-test-helpers"; [dependencies] parking_lot = "0.12"
└── src/
    └── lib.rs         # pub fn test_lock() over static TEST_LOCK; one smoke test in #[cfg(test)] mod tests
```

`src/lib.rs` shape (illustrative, not normative — code agent finalises):

```rust
//! Workspace test-serialisation helper.
//!
//! Each test binary that uses `test_lock()` links its own instance of the
//! [`TEST_LOCK`] static — the lock therefore serialises tests **within** a
//! single test binary, matching the per-binary semantics that
//! `#[serial_test::serial]` previously provided. Cross-binary serialisation
//! is **not** provided (and was not provided by `serial_test` either).

use parking_lot::{Mutex, MutexGuard};

static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Acquire the per-binary test serialisation lock.
///
/// Hold the returned guard for the entire duration of any test that mutates
/// process-global state (e.g. global dispatcher, snapshot registry).
pub fn test_lock() -> MutexGuard<'static, ()> {
    TEST_LOCK.lock()
}
```

`parking_lot` is a **regular** dep (not a dev-dep) of `quartzite-test-helpers` — the helper itself uses `parking_lot::Mutex` at runtime when consumers call `test_lock()`. The helper crate itself does **not** need a `std` feature: `parking_lot` requires `std` unconditionally. `quartzite-test-helpers` will therefore only build / be consumed during `cargo test` and other std-feature builds — never on the `--no-default-features --features libm` production path, because dev-dep edges are not compiled there.

**Consumer-side wiring (5 `Cargo.toml` files):**

| File | Change |
|---|---|
| Workspace root `Cargo.toml` | Add `"quartzite-test-helpers"` to `[workspace] members = [...]`. Replace `serial_test = "3"` in `[dev-dependencies]` with `quartzite-test-helpers = { path = "quartzite-test-helpers" }`. |
| `quartzite-core/Cargo.toml` | Replace `serial_test = "3"` in `[dev-dependencies]` with `quartzite-test-helpers = { path = "../quartzite-test-helpers" }`. Unconditional (dev-deps don't compile on no_std production path; the `#[serial]` sites stay `#[cfg(feature = "std")]`-gated at call sites). |
| `quartzite-runtime/Cargo.toml` | Replace `serial_test = "3"` in `[dev-dependencies]` with `quartzite-test-helpers = { path = "../quartzite-test-helpers" }`. |
| `quartzite-style/Cargo.toml` | Replace `serial_test = "3"` in `[dev-dependencies]` with `quartzite-test-helpers = { path = "../quartzite-test-helpers" }`. |
| `quartzite-style-dispatch/Cargo.toml` | Replace `serial_test = "3"` in `[dev-dependencies]` with `quartzite-test-helpers = { path = "../quartzite-test-helpers" }`. |

**Per-file call-site migration (9 files, 42 attributes — live recount):**

Per-file live count (`rg -c '#\[serial' --type rust`):

| File | Live count | Test binary |
|---|---|---|
| `quartzite-core/src/signal.rs` | 9 | `quartzite-core` `--lib` |
| `quartzite-core/src/connect.rs` | 1 | `quartzite-core` `--lib` |
| `quartzite-runtime/src/snapshot/object.rs` | 4 | `quartzite-runtime` `--lib` |
| `quartzite-runtime/src/snapshot/tree.rs` | 3 | `quartzite-runtime` `--lib` |
| `quartzite-runtime/tests/snapshot.rs` | 7 | `quartzite-runtime` integration `snapshot` |
| `quartzite-style/src/registry.rs` | 4 | `quartzite-style` `--lib` |
| `quartzite-style/src/default_style_tests.rs` | 1 | `quartzite-style` `--lib` (attached via `#[path]` from `default_style.rs`) |
| `quartzite-style-dispatch/src/dispatch.rs` | 12 | `quartzite-style-dispatch` `--lib` |
| `tests/signal_to_signal.rs` (workspace root) | 1 | workspace-root integration `signal_to_signal` |

**Total: 42 attributes across 9 files, 6 test binaries.** The spec said 41 (informative); ACs key off exact-match greps requiring zero residual `#[serial]` after migration, so the exact pre-change count is not load-bearing.

**Per-call-site shape:**

```rust
// Before
use serial_test::serial;
#[test]
#[serial]
fn some_test() { /* body */ }

// After
#[test]
fn some_test() {
    let _lock = quartzite_test_helpers::test_lock();
    /* body */
}
```

The `use serial_test::...` import is removed; the helper is invoked via the fully-qualified path (`quartzite_test_helpers::test_lock()`) **or** a localised `use quartzite_test_helpers::test_lock;` at the top of the test module — implementer's choice per-file. Both shapes satisfy AC13. The `let _lock = …` binding (not `let _ =`) is mandatory — see Risks.

**`quartzite-core` `std`-gating note.** Every `#[serial]` site in `quartzite-core` lives inside `#[cfg(feature = "std")]` already. The `quartzite-test-helpers` dev-dep in `quartzite-core/Cargo.toml` is unconditional — dev-deps are never compiled on the `--no-default-features --features libm` production path, so no extra gating is needed on the dep declaration. The call sites themselves retain their existing `#[cfg(feature = "std")]` gates.

**Rejected alternatives (post-amendment):**

- **Per-crate / per-file static (option (a) — round-1 design).** Rejected by spec amendment. Copy-paste across 6 sites violates the 2026-05-17 copy-paste-for-≥3-sites learning; the per-binary linkage guarantee (each test binary links its own instance of a `static` declared in a shared crate) makes the shared-crate placement strictly dominant.
- **`tests/support/mod.rs` for each integration-test binary.** Rejected. A one-line helper does not earn a support module, and the shared-crate path already covers integration tests via `[dev-dependencies]` on the parent crate's `Cargo.toml` (the integration test binary inherits the dev-dep edge).
- **`std::sync::Mutex<()>` instead of `parking_lot::Mutex`.** Violates AGENTS.md *Library safety idioms* (`parking_lot::Mutex` is the workspace default). Rejected.
- **Single PR vs per-crate PRs.** Spec § Key decisions permits a single PR. The change is mechanical; review-clarity gain from per-crate PRs is marginal. Single PR chosen.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create the `quartzite-test-helpers/` crate. Add `quartzite-test-helpers/Cargo.toml` (`[package] name = "quartzite-test-helpers"` with workspace-inherited `version` / `edition` / `rust-version` / `authors` / `license` / `repository`; `[dependencies] parking_lot = "0.12"`; standard `[lints] workspace = true`). Add `quartzite-test-helpers/src/lib.rs` with the `#![deny(missing_docs)]` crate attribute (per AGENTS.md *Documentation*), top-of-module doc explaining per-binary serialisation semantics, `static TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());`, `pub fn test_lock() -> parking_lot::MutexGuard<'static, ()>` with one-line `///` doc, and a `#[cfg(test)] mod tests` smoke test (`test_lock_acquires_and_releases` — see Test Design). Register the crate in the workspace root `Cargo.toml` `[workspace] members = [...]` array. Run `cargo build -p quartzite-test-helpers` and `cargo test -p quartzite-test-helpers` to validate. **No consumer-crate edits yet.** | `quartzite-test-helpers/Cargo.toml` (new), `quartzite-test-helpers/src/lib.rs` (new), `Cargo.toml` (workspace `[workspace] members` only) | — |
| 2 | Swap dev-deps: in each of the 5 consumer `Cargo.toml` files, replace `serial_test = "3"` with `quartzite-test-helpers = { path = ... }` in `[dev-dependencies]`. Path is `"quartzite-test-helpers"` for the workspace root, `"../quartzite-test-helpers"` for each per-crate `Cargo.toml`. Run `cargo update` so `serial_test`, `scc`, `sdd` drop from `Cargo.lock`. **Do not migrate call sites yet** — at this point `cargo build` succeeds (no `serial_test` dep) but `cargo test --workspace` **fails** (the `#[serial]` attrs no longer resolve). This is expected and resolved by Tasks 3–5. The dep swap is a single atomic step; mid-task half-state is acceptable because the next subtask in the same group migrates the first batch of call sites. | `Cargo.toml`, `quartzite-core/Cargo.toml`, `quartzite-runtime/Cargo.toml`, `quartzite-style/Cargo.toml`, `quartzite-style-dispatch/Cargo.toml`, `Cargo.lock` | 1 |
| 3 | Migrate `quartzite-core` call sites: remove every `use serial_test::serial;` / `use serial_test::*;` import in `signal.rs` and `connect.rs`; remove every `#[serial]` / `#[serial_test::serial]` attr (9 in `signal.rs` + 1 in `connect.rs` = 10 sites); prepend `let _lock = quartzite_test_helpers::test_lock();` as the first statement of each affected test body. Run `cargo test -p quartzite-core --features std` to validate. | `quartzite-core/src/signal.rs`, `quartzite-core/src/connect.rs` | 2 |
| 4 | Migrate `quartzite-runtime` call sites (unit + integration): unit-test sites in `snapshot/object.rs` (4) + `snapshot/tree.rs` (3); integration-test sites in `tests/snapshot.rs` (7). Same shape as Task 3: drop `use serial_test::*;`, drop `#[serial]` attrs, prepend `let _lock = quartzite_test_helpers::test_lock();` to each test body. Run `cargo test -p quartzite-runtime` to validate. | `quartzite-runtime/src/snapshot/object.rs`, `quartzite-runtime/src/snapshot/tree.rs`, `quartzite-runtime/tests/snapshot.rs` | 2 |
| 5 | Migrate `quartzite-style` + `quartzite-style-dispatch` + workspace-root integration sites: `registry.rs` (4) + `default_style_tests.rs` (1) + `dispatch.rs` (12) + `tests/signal_to_signal.rs` (1) = 18 sites. Same shape as Tasks 3 and 4. Run `cargo test -p quartzite-style && cargo test -p quartzite-style-dispatch && cargo test --test signal_to_signal` to validate. | `quartzite-style/src/registry.rs`, `quartzite-style/src/default_style_tests.rs`, `quartzite-style-dispatch/src/dispatch.rs`, `tests/signal_to_signal.rs` | 2 |
| 6 | Final gate: run the full local validation chain — `cargo build`, `cargo build -p quartzite --no-default-features --features libm`, `cargo test --workspace`, `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. Verify ACs 1, 2, 3, 9, 10, 14 with the exact `rg`/`grep` commands they specify; in particular AC14 (`rg -U 'static\s+TEST_LOCK\s*:' --type rust` returns exactly 1 hit, inside `quartzite-test-helpers/`). No file edits at this step unless the gates flag fixable nits — if so, fix in place and rerun. | (validation only — no file edits unless gates demand) | 3, 4, 5 |

Six atomic tasks. The helper file lives at `quartzite-test-helpers/src/lib.rs` and nowhere else; **no consumer crate adds its own `static TEST_LOCK`** (enforced mechanically by AC14).

**Dependency shape note (round-2 review note #1):** Tasks 3, 4, 5 are mutually independent — each lists `Depends on: 2` and depends only on Task 2. Task 6 blocks on the conjunction of 3 + 4 + 5. Within Group B the per-subagent `/context-reset` handoff serialises subtask execution by construction; the independence note is informational for any future re-grouping that wants to fan out the per-consumer migrations.

**Workspace `default-members` note (round-2 review note #3):** The workspace root `Cargo.toml` has no `[workspace] default-members = […]` array today; `quartzite-test-helpers` joins `[workspace] members` only — no `default-members` change is required. If a future PR introduces a `default-members` array, the helper crate's inclusion / exclusion is a separate decision (most likely excluded, since it's a test-only support crate).

## Handoff plan

`M = 6` (two groups, 3 + 3):

- **Group A:** subtasks 1–3 — create the helper crate, swap dev-deps, migrate the first consumer (`quartzite-core`). Group A's entry is performed by spawning `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry), per the every-group handoff contract. Group A validates the end-to-end pattern (new crate compiles, `cargo update` cleanly removes `serial_test`/`scc`/`sdd`, the lowest-blast-radius consumer migrates) before fan-out.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — terminal group (3 subtasks; within the `1..=3` range). Migrates the remaining 5 binaries (`quartzite-runtime` unit + `quartzite-runtime` integration, `quartzite-style` unit, `quartzite-style-dispatch` unit, workspace-root integration) and finalises by running the full workspace validation gate.

## Risks

- **Shared-crate per-binary linkage — silent regression risk.** A reader unfamiliar with Rust's per-binary-static linkage rule might assume the single `static TEST_LOCK` in `quartzite-test-helpers/src/lib.rs` provides cross-binary serialisation. It does not: each test binary that depends on `quartzite-test-helpers` links its own copy of the static (this is exactly the per-process-static argument). *Mitigation:* the helper crate's top-of-module doc-comment explicitly states "Each test binary that uses `test_lock()` links its own instance of the `TEST_LOCK` static" (per AC11). Code-review checklist: confirm no consumer crate attempts to share `TEST_LOCK` across binary boundaries via process-global resources (none of the migrated tests do this today).
- **Circular dev-dep risk (future).** If `quartzite-test-helpers` ever depends on a workspace member that itself needs `test_lock()`, Cargo will reject the build with a dev-cycle error. *v1 status:* `quartzite-test-helpers` is a leaf with a single external dep (`parking_lot`); no workspace-member edge exists. *Mitigation:* keep `quartzite-test-helpers` strictly leaf-shaped. If a future use case requires the helper to call into a workspace member, file a follow-up issue to split (e.g., `quartzite-test-helpers-core` + `quartzite-test-helpers`); do not add the edge ad-hoc.
- **`let _lock = …;` vs `let _ = …;` drop-order trap.** `let _ = x;` **immediately drops** `x`; the helper guard must be bound to a named variable (`_lock` underscore-prefix preserves "unused" lint-silencing while binding for scope). *Mitigation:* AC13 requires "acquires `test_lock()` as the first statement"; code-review additionally checks the binding is `let _lock = …;` (or `let _guard = …;`), never `let _ = …;`. This trap is the same one that bit `Mutex<()>` users in stdlib's own examples.
- **Bound-but-shadowed variable.** A test body that does `let _lock = test_lock(); ... let _lock = something_else;` drops the guard at the second binding. *Mitigation:* code-review check that the only shadowing of `_lock` in any test body is intentional (none expected in the migrated tests; the lock-name is unique to this helper). Renaming to `_test_lock` workspace-wide is a one-line follow-up if review flags concern.
- **`quartzite-core` `std`-gating mismatch.** Every `#[serial]` site in `quartzite-core` is already `#[cfg(feature = "std")]`. The call-site gates are preserved (the `let _lock = …` line lives inside the gated test body, not at module scope). The `quartzite-test-helpers` dev-dep in `quartzite-core/Cargo.toml` is unconditional — dev-deps are never compiled on the `--no-default-features --features libm` production path. *Mitigation:* AC4 already gates on the no-default-features build; Task 3 validates `cargo test -p quartzite-core --features std`.
- **`parking_lot` / `quartzite-test-helpers` dep-presence claim drift.** Per AGENTS.md *Dependency Versions* AXIOM (presence-of-dep dimension), the design's claims about the current dep landscape were verified live: `parking_lot = "0.12"` is workspace-standard (live `0.12.5` is `^0.12`-compatible — no update); `quartzite-test-helpers` does NOT exist anywhere in the workspace (verified `ls -la quartzite-test-helpers` → "does not exist"); the 5 consumer `Cargo.toml` files all currently declare `serial_test = "3"` in `[dev-dependencies]` (verified via `grep -rn 'serial_test' --include='Cargo.toml' .` → 5 lines). Implementation must re-verify at start (the dep landscape may shift between design and implementation).
- **`Cargo.lock` not regenerated.** Forgetting `cargo update` after dep edits would leave `serial_test`, `scc`, `sdd` in `Cargo.lock` and break AC9/AC10. *Mitigation:* Task 2 explicitly chains `cargo update` after dev-dep swap; AGENTS.md *Workflow* "run `cargo build` before committing so `Cargo.lock` is refreshed" applies to Task 6's final gate as a backstop.
- **Test execution time regression.** `serial_test` uses an `scc::HashMap` keyed on group name even for the default unnamed group. Replacing with a single `parking_lot::Mutex<()>` per binary is **simpler and faster** for the unnamed-group case (no map lookup, no group registration). No regression expected; full `cargo test --workspace` in Task 6 catches any surprise.
- **`#[serial]` site missed (false negative on grep).** AC2/AC3 are exact-match greps; the migration is mechanical. *Mitigation:* per-task subsection counts (Tasks 3/4/5 enumerate per-file counts: 9+1=10 in Task 3; 4+3+7=14 in Task 4; 4+1+12+1=18 in Task 5; total 42). Self-review reruns `rg '#\[serial' --type rust | wc -l` after each task and confirms the running total decreases by the expected amount (42 → 32 → 18 → 0).

## Test Design

The change is structural (test-infrastructure swap). No new behavioural tests are added — the existing 1446+ test suite **is** the test plan. Per the spec's AC5 ("pre-change test count is preserved within ±1"), the only test-count drift is from the helper crate's own smoke test (see below).

- **Existing migrated tests** (42 sites). Each test must continue to pass exactly as before. Test plan = `cargo test --workspace` in Task 6.
- **Helper smoke test (one test, in `quartzite-test-helpers`).** Add **one** `#[cfg(test)] mod tests { #[test] fn test_lock_acquires_and_releases() { … } }` block to `quartzite-test-helpers/src/lib.rs`. Per the spec amendment, this is the natural placement (single home for the helper code, single home for its smoke test). AC5's ±1 tolerance covers this one test.
  - **Location:** `quartzite-test-helpers/src/lib.rs` `#[cfg(test)] mod tests`.
  - **Entry point:** the new `test_lock()` function.
  - **Scenarios:**
    1. **Happy path — acquire-then-drop-then-reacquire.** Single-threaded RAII proof: `let g1 = test_lock(); drop(g1); let g2 = test_lock();` succeeds without deadlock.
    2. **Compile-time return-type check.** `let _: parking_lot::MutexGuard<'static, ()> = test_lock();` enforces the public signature stays stable across refactors.
  - **Fixtures / helpers needed:** none.
- **Doc-comment example.** Per AC11, the `pub fn test_lock()` carries a one-line `///` doc plus a top-of-module note explaining the per-binary serialisation guarantee. The doc example showing `let _lock = test_lock();` is fenced as ` ```no_run ` (round-2 review note #2 — load-bearing choice, not alternative). `no_run` documents the binding shape (caller sees the exact `let _lock = …;` form) and lets the compiler type-check it, while preventing the doctest from contending the static at `cargo test` time; ` ```ignore ` would skip type-checking and is therefore rejected. The doc-gate command (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) catches missing docs but does not execute test-module doctests — Task 6 validates.
- **Negative-case probe (not added; rationale).** A "panicking test does not wedge the lock" probe is not added — `parking_lot::Mutex` cannot poison (spec § Key decisions), and this is a property of `parking_lot::Mutex` itself, not of the helper. Adding a `#[should_panic]` test that re-acquires after a panic would test `parking_lot`, not the helper.

## Open questions

None. Both design-affecting decisions are resolved by the spec amendment:

- **Helper placement:** option (b) — single shared crate `quartzite-test-helpers/` at workspace root. One `quartzite-test-helpers/src/lib.rs` containing the `static TEST_LOCK` + `pub fn test_lock()`. No `src/test_lock.rs` files in consumer crates. No inline `static TEST_LOCK` declarations in integration-test binaries.
- **Per-crate split vs shared crate:** shared crate. Per-binary serialisation semantics preserved by Rust's per-binary-static linkage rule (each test binary links its own instance of the static).
