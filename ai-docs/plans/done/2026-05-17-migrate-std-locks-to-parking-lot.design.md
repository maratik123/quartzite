# Design: Migrate `std::sync::{Mutex,RwLock}` to `parking_lot`

**Issue:** #442
**Date:** 2026-05-17

## Approach

The migration is mechanical: every remaining `std::sync::Mutex` and
`std::sync::RwLock` site moves to its `parking_lot` equivalent, the
`unwrap_or_else(|e| e.into_inner())` poisoned-recovery boilerplate disappears
(parking_lot locks return `MutexGuard` / `RwLock{Read,Write}Guard` directly),
and the workspace clippy allow for `redundant_closure_for_method_calls` —
introduced solely to silence the boilerplate — is deleted together with its
justifying comment block. The motivation is consistency, idiom simplification,
and dropping a poisoning concept the codebase actively tolerates rather than
relies on.

The six-commit decomposition from issue #442 / spec §Key decisions row AC13 is
adopted unchanged: instruction-file pair first (so subsequent code commits cite
the new idiom), then the prod-Mutex sweep, then the RwLock declarator + the
public-API break on `ObjectFactory::global`, then test-side Mutex / RwLock,
then the clippy-allow removal (gating step that proves zero residual
`unwrap_or_else(|e| e.into_inner())` sites), then a final-audit grep +
`cargo update`.

Three deliberate sub-choices:

1. **No compat shim for the `factory.rs` public-API break.** AGENTS.md *API
   Stability* AXIOM authorises clean breaks pre-`cargo publish`. The four
   callers of `ObjectFactory::global()` already in scope
   (`snapshot/object.rs`, `snapshot/tree.rs`, `tests/snapshot.rs`,
   `factory.rs` itself) update directly; there is no external downstream.
   *Rejected:* a `pub use std::sync::RwLock as PoisonableRwLock` alias plus a
   second method — explicitly disallowed by AGENTS.md.
2. **`parking_lot` lands in `[dependencies]`, not `[dev-dependencies]`, for
   crates whose only current `Mutex` use is in `#[cfg(test)] mod tests`.**
   Inline `#[cfg(test)]` test modules compile as part of the crate's normal
   target (not the `dev` target), so a `[dev-dependencies]` entry would
   surface as `unresolved import` at `cargo test`. `quartzite-renderer` and
   `quartzite-widgets` additionally have `tests/` integration files using
   `Mutex`, but the regular `[dependencies]` entry already covers both prod
   and dev within the same crate.
   *Rejected:* a per-crate split between `[dependencies]` and
   `[dev-dependencies]` — over-fitted for zero observable benefit, and would
   require two `cargo update` rounds.
3. **Delete the `poison_for_test()` helper and the
   `try_style_recovers_from_poisoned_mutex` test wholesale instead of
   preserving them as no-ops.** `parking_lot::Mutex` has no poison concept;
   the test's contract (`PoisonError` is recoverable) is meaningless against
   the new primitive, and a no-op stub would be dead code immediately.
   *Rejected:* a `#[cfg_attr(deprecated)] fn poison_for_test() {}` stub —
   AGENTS.md API-Stability bans compat shims, and the helper is `pub(crate)`
   anyway.

**Per-call-site rewrite shapes** (mechanical, all five shapes observed in
live code; the third was added by 2026-05-17 spec-amendment round 2 once
`timer.rs` entered scope):

| Pre-migration shape | Post-migration shape |
|---|---|
| `lock().unwrap_or_else(\|e\| e.into_inner())` | `lock()` |
| `read().unwrap_or_else(\|e\| e.into_inner())` / `write().unwrap_or_else(\|e\| e.into_inner())` | `read()` / `write()` |
| `.lock().unwrap()` / `.read().unwrap()` / `.write().unwrap()` (test code) | `.lock()` / `.read()` / `.write()` |
| `.lock().expect("…")` / `.read().expect("…")` / `.write().expect("…")` (test code, **`timer.rs`** is the sole user — 7 sites) | `.lock()` / `.read()` / `.write()` (parking_lot returns the guard directly; the `.expect(...)` extractor disappears) |
| `std::sync::MutexGuard<'static, T>` field type | `parking_lot::MutexGuard<'static, T>` (`support_internals.rs` line 41 — sole site) |

**Compile-time enforcement.** Switching the `use` import from
`std::sync::Mutex` to `parking_lot::Mutex` in a file containing
`.lock().unwrap()` / `.lock().expect("…")` / `.lock().unwrap_or_else(...)`
causes a compile error: `parking_lot::Mutex::lock()` returns
`MutexGuard<'_, T>` directly (not `LockResult<MutexGuard<'_, T>>`), so
calling `.unwrap()` / `.expect()` / `.unwrap_or_else()` on a non-`Result`
fails type-checking. **The compiler is the real gate** that forces every
call-site rewrite at every migrated import — AC3 + AC5 ratify post-hoc that
no shape survived. This is why partial-task incremental commits stay green:
either every site in a file is rewritten in the same commit, or the file
does not compile.

The 6-task decomposition below maps 1:1 onto the planned 6 commits; each task
is also its own group boundary for the 2-of-3 commit split (see §Handoff plan)
so handoffs happen at natural commit boundaries.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Instruction-file pair: rewrite `AGENTS.md` § *Library safety idioms* mutex bullet to name `parking_lot::Mutex` / `parking_lot::RwLock` as the workspace default; demote `.unwrap_or_else(\|e\| e.into_inner())` to a footnote for rare FFI-imposed `std::sync` retainees. Mirror the same change in `ai-docs/code-style.md` § *Library safety idioms* (lines 73–87). Land both edits in the **same commit** per the Propagation Rule's AGENTS.md ↔ ai-docs/code-style.md pairing. The `OnceLock` / `Arc` / `Weak` / `AtomicBool` bullet stays untouched. | `AGENTS.md`, `ai-docs/code-style.md` | — |
| 2 | Prod `Mutex` sweep: rewrite `quartzite-style/src/registry.rs` (`use std::sync::{Mutex, OnceLock}` → split into `use parking_lot::Mutex;` + `use std::sync::OnceLock;`; drop both `unwrap_or_else` calls at lines 95 and 115; drop the third inside `clear_for_test` at line 139; **delete** `poison_for_test()` (lines 144–159), **delete** `try_style_recovers_from_poisoned_mutex` (lines 275–288 plus the surrounding blank line), and rewrite the module doc comment lines 5–9 and the `try_style` doc paragraph lines 100–103 to drop poison-tolerance wording). Add `parking_lot = "0.12"` to `quartzite-style/Cargo.toml [dependencies]`. Run `cargo update && cargo build && cargo test -p quartzite-style`. | `quartzite-style/src/registry.rs`, `quartzite-style/Cargo.toml`, `Cargo.lock` | 1 |
| 3 | RwLock declarator + public-API break: `quartzite-runtime/src/factory.rs` — rewrite line 4 import to `use std::sync::{Arc, OnceLock};` + new `use parking_lot::RwLock;`; static declarator stays `OnceLock<Arc<RwLock<ObjectFactory>>>` (the concrete `RwLock` type now resolves to parking_lot); change `pub fn global()` return type from `Option<Arc<std::sync::RwLock<Self>>>` to `Option<Arc<parking_lot::RwLock<Self>>>` (the import makes `RwLock` resolve to parking_lot, so the existing return-type literal stays valid but its meaning changes — public-API break). Update the four doc-comment blocks: line 89–91 `Callers must lock` paragraph (drop `.expect("poisoned")`), line 95–101 `# Examples` block for `global` (drop `.expect("poisoned")`), and any other `.unwrap()` / `.expect("poisoned")` on lock calls in `factory.rs` doc comments. RwLock-consumer call sites in `quartzite-runtime/src/snapshot/object.rs` line 87 (`.read().unwrap_or_else(\|e\| e.into_inner())` → `.read()`) and lines 253–256 (`arc.write().unwrap_or_else(\|e\| e.into_inner()).register(...)` → `arc.write().register(...)`); `quartzite-runtime/src/snapshot/tree.rs` lines 351–353 (same shape as object.rs line 253). `quartzite-runtime` already declares `parking_lot = "0.12"` so no Cargo.toml change in this task. Run `cargo build && cargo test -p quartzite-runtime`. | `quartzite-runtime/src/factory.rs`, `quartzite-runtime/src/snapshot/object.rs`, `quartzite-runtime/src/snapshot/tree.rs` | 1 |
| 4 | Test-side Mutex sweep — prod-crate inline `#[cfg(test)] mod tests`: rewrite `use std::sync::{Arc, Mutex};` → `use std::sync::Arc;` + `use parking_lot::Mutex;` in `quartzite-widgets/src/widgets/button.rs` (line 157), `quartzite-widgets/src/widgets/line_edit.rs` (line 136), `quartzite-widgets/src/widgets/text_edit.rs` (line 128), `quartzite-renderer/src/wrapped_handler.rs` (line 223), `quartzite-runtime/src/thread_pool.rs` (line 108, multi-line `use std::{..., sync::{Arc, Mutex}, ...}` — split the `sync` brace), `quartzite-runtime/src/event_loop.rs` (line 285, same multi-line shape as `thread_pool.rs`). Also fold in `quartzite-core/src/id.rs` (deep-import shape `use std::{collections::HashSet, sync::Mutex, thread};` at line 130, inside `#[cfg(feature = "std")]` + `#[cfg(test)] mod tests`): rewrite to `use std::{collections::HashSet, thread};` + `use parking_lot::Mutex;`; rewrite the **2** `Mutex::new(HashSet::new())` declarations at lines 164 and 180 (type literal `Mutex<HashSet<u64>>` stays unchanged — `Mutex` now resolves to parking_lot); drop the **4** `.lock().unwrap()` calls at lines 169, 173, 185, 189 to `.lock()` (live count `rg -c '\.lock\(\)\.unwrap\(\)' quartzite-core/src/id.rs` = 4 — corrected from the round-1 amendment's "six"). **`quartzite-core` already declares `parking_lot` (optional, gated by `std`) — no `Cargo.toml` change for `quartzite-core`.** Drop every `.lock().unwrap()` call to `.lock()` across all seven files in this task (mechanical sweep; precise per-file count from `rg -c '\.lock\(\)\.unwrap\(\)' --type rust …`: button=10, line_edit=10, text_edit=8, wrapped_handler=12, thread_pool=4, event_loop_src=4, id=4 = **52 sites total**). Add `parking_lot = "0.12"` to `quartzite-widgets/Cargo.toml [dependencies]` and `quartzite-renderer/Cargo.toml [dependencies]` (no Cargo.toml change for `quartzite-core` / `quartzite-runtime`). Run `cargo update && cargo build && cargo test --workspace`. | `quartzite-core/src/id.rs`, `quartzite-widgets/src/widgets/button.rs`, `quartzite-widgets/src/widgets/line_edit.rs`, `quartzite-widgets/src/widgets/text_edit.rs`, `quartzite-renderer/src/wrapped_handler.rs`, `quartzite-runtime/src/thread_pool.rs`, `quartzite-runtime/src/event_loop.rs`, `quartzite-widgets/Cargo.toml`, `quartzite-renderer/Cargo.toml`, `Cargo.lock` | 1, 3 |
| 5 | Test-side Mutex / RwLock sweep — integration test files under `tests/` (now **7 files** per 2026-05-17 spec-amendment round 2, which added `per_thread_loops.rs` + `timer.rs`): rewrite `use std::sync::{Arc, Mutex};` → `use std::sync::Arc;` + `use parking_lot::Mutex;` in `quartzite-renderer/tests/multi_window.rs` (line 21, 5 `.lock().unwrap()` sites), `quartzite-renderer/tests/support/mod.rs` (line 15, 6 `.lock().unwrap()` sites), `quartzite-runtime/tests/event_loop.rs` (lines 1–5 multi-line `use`, 4 `.lock().unwrap()` sites). Rewrite `use std::sync::Mutex;` → `use parking_lot::Mutex;` and `_lock: std::sync::MutexGuard<'static, ()>` → `_lock: parking_lot::MutexGuard<'static, ()>` in `quartzite-widgets/tests/support_internals.rs` (lines 24, 41). Drop the `.lock().unwrap_or_else(\|e\| e.into_inner())` at line 49 → `.lock()` (1 site). In `quartzite-runtime/tests/snapshot.rs` (line 185 area), drop the `.write().unwrap_or_else(\|e\| e.into_inner()).register(...)` → `.write().register(...)` at lines 194–196. **New in round-2 amendment — multi-line deep-import shapes**: `quartzite-runtime/tests/per_thread_loops.rs` (lines 2–10 multi-line `use std::{ sync::{ Arc, Mutex, atomic::{...}, mpsc, }, … }` — split the `sync` brace into `use std::{ sync::{ Arc, atomic::{...}, mpsc, }, … };` + `use parking_lot::Mutex;`; 1 `Mutex::new(...)` declaration at line 33; 2 `.lock().unwrap()` rewrites at lines 39, 48); `quartzite-runtime/tests/timer.rs` (lines 22–29 multi-line `use std::{ sync::{ Arc, Mutex, atomic::{...} }, … }` — same split; 3 `Mutex::new(...)` declarations at lines 69, 188, 216; **7 `.lock().expect("…")` rewrites** at lines 77, 91, 193, 204, 213, 223, 247 → `.lock()`; note line 213's tail `.unwrap()` is on the inner `Option<ThreadId>`, NOT on the lock — it stays as `.lock().unwrap()` after the `.expect("el_id lock")` extractor disappears). `quartzite-runtime` already declares `parking_lot = "0.12"` (no Cargo.toml change for `event_loop.rs` / `per_thread_loops.rs` / `timer.rs` / `snapshot.rs`); the `parking_lot` deps for `quartzite-widgets` and `quartzite-renderer` were added in Task 4 (single `[dependencies]` entry covers prod and integration tests). Run `cargo build && cargo test --workspace`. | `quartzite-renderer/tests/multi_window.rs`, `quartzite-renderer/tests/support/mod.rs`, `quartzite-runtime/tests/event_loop.rs`, `quartzite-runtime/tests/per_thread_loops.rs`, `quartzite-runtime/tests/timer.rs`, `quartzite-widgets/tests/support_internals.rs`, `quartzite-runtime/tests/snapshot.rs` | 1, 3, 4 |
| 6 | Workspace clippy-allow removal + final-audit grep + `cargo update`: delete line 66 (`redundant_closure_for_method_calls = "allow"`) and line 65 (the justifying `# AGENTS.md *Library safety idioms* designates …` comment) from root `Cargo.toml [workspace.lints.clippy]`. Confirm via spec AC3: `rg 'unwrap_or_else\(\|e\| e\.into_inner\(\)\)' --type rust` returning zero hits; if any site remains, fix it before deleting the allow. Verify the full AC sweep using the spec's tightened two-pattern regex — **both patterns invoked with `rg -U` (multi-line) per spec AC1/AC2 wording finalised in 2026-05-17 spec-amendment round 2**: spec AC1: `rg -U 'sync::(Mutex\|RwLock)\b' --type rust` zero hits (catches direct-form `use std::sync::Mutex`, FQN `std::sync::Mutex::new`, and deep-import `use std::{… sync::Mutex …}` shapes); spec AC2: `rg -U 'sync::\{[^}]*\b(Mutex\|RwLock)\b' --type rust` zero hits — the `-U` flag is **load-bearing**: without it, line-based `[^}]*` cannot span the newline between `sync::{` and `Arc, Mutex,` on the next line, and the multi-line imports of `factory.rs` / `per_thread_loops.rs` / `timer.rs` / `thread_pool.rs` / `event_loop.rs` slip through (verified against round-2 finding). Run `cargo update`, `cargo build`, `cargo test --workspace`, `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`. | `Cargo.toml` (root), `Cargo.lock` | 1, 2, 3, 4, 5 |

## Handoff plan

`M = 6` subtasks → two groups of 3 + 3 (both `<=` 3 cap, terminal within
`1..=3`).

- **Entry into Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § *Compaction recovery (re-entry)* —
  every group starts under a fresh context per AGENTS.md every-group handoff
  contract, including the first.
- **Group A:** subtasks 1, 2, 3 — instruction-file pair + prod Mutex sweep
  in `quartzite-style/src/registry.rs` + RwLock declarator / consumer sweep
  in `quartzite-runtime/{src/factory.rs, src/snapshot/object.rs,
  src/snapshot/tree.rs}`. Commits 1, 2, 3 of the 6-commit branch land here.
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § *Compaction recovery (re-entry)*.
  Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4, 5, 6 — test-side prod-crate inline `#[cfg(test)]`
  sweep (including the `quartzite-core/src/id.rs` deep-import shape folded in
  via 2026-05-17 spec-amendment round 1) + test-side `tests/`-integration
  sweep (now **7 files** after round-2 amendment added
  `quartzite-runtime/tests/per_thread_loops.rs` and
  `quartzite-runtime/tests/timer.rs` — both multi-line deep-import shapes
  that the round-1 single-line AC2 regex missed) + clippy-allow removal +
  final-audit grep with the now-mandatory `rg -U` prefix + `cargo update`.
  Commits 4, 5, 6 land here. Terminal group (3 subtasks; within the 1..=3
  range).

## Risks

- **Public-API break on `ObjectFactory::global()` (return type changes from
  `Option<Arc<std::sync::RwLock<Self>>>` to `Option<Arc<parking_lot::RwLock<Self>>>`):**
  mitigated by AGENTS.md *API Stability* AXIOM — pre-publish, clean breaks,
  no compat shim. The four in-tree callers (`snapshot/object.rs`,
  `snapshot/tree.rs`, `tests/snapshot.rs`, `factory.rs` doc examples) update
  in the same task (Task 3). `gh search` for external `cargo` consumers will
  return none (project not on crates.io).
- **Subtle behaviour difference: parking_lot is unfair vs std-fair locks
  under contention:** mitigated by — (a) none of the migrated sites depend on
  ordering, (b) `quartzite-runtime` already runs `parking_lot::RwLock` in
  `connection_table.rs` and `loop_registry.rs` without issue, (c) the
  workspace clippy allow for `significant_drop_*` (lines 55–58 of root
  Cargo.toml) stays — `parking_lot::MutexGuard` has the same significant-drop
  semantics. No new bench / load test is required (spec §Out of scope).
- **Doc examples in `factory.rs` referenced from rustdoc + `cargo doc` gate:**
  mitigated by Task 3 explicitly enumerating each doc block that touches
  `.lock()` / `.read()` / `.write()` / `.expect("poisoned")` / `.unwrap()` on
  lock calls. Task 6's doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs"
  cargo doc --no-deps --workspace --all-features`) catches any
  doc-comment site missed in Task 3.
- **`unwrap_or_else(|e| e.into_inner())` survives in a `.rs` file outside
  the enumerated scope:** mitigated by Task 6's AC3 grep (`rg
  'unwrap_or_else\(\|e\| e.into_inner\(\)\)' --type rust` → zero hits) running
  **before** the clippy-allow deletion, so the gate either passes or stays
  red without the cosmetic-cleanup commit landing.
- **A `std::sync::{Mutex,RwLock}` multi-line grouped-import shape escapes
  AC2 because the `[^}]*` content class is line-based:** mitigated by spec
  AC1/AC2 now requiring `rg -U` (multi-line) at every audit. The round-2
  amendment finding — `per_thread_loops.rs` and `timer.rs` slipping through
  the round-1 single-line regex — is encoded directly into AC1/AC2 wording
  and into Task 6's final-audit grep. Task 6 runs the exact `rg -U …`
  invocation, so a regression here would have to defeat both the AC and the
  task gate simultaneously.
- **A `.lock().unwrap()` / `.lock().expect("…")` / `.lock().unwrap_or_else(...)`
  site survives a partial import migration:** mitigated by compile-time
  enforcement — switching the `use` import to `parking_lot::Mutex` makes
  every surviving `.unwrap()` / `.expect()` / `.unwrap_or_else()` on a
  non-`Result` a type error. The file does not compile until every site is
  rewritten, so AC5 (`cargo build`, `cargo test --workspace`) catches the
  case at the commit boundary, not at AC3 audit time. AC3 and AC5 are
  post-hoc ratification of what `rustc` already enforces.
- **`--no-default-features --features libm` regression on quartzite root
  crate:** mitigated by — (a) `quartzite-core` is the only crate whose
  `parking_lot` dep is feature-gated (`std` feature), and the gate is
  unchanged by this issue; (b) `quartzite-style` / `quartzite-renderer` /
  `quartzite-widgets` are already `std`-only crates (spec §Key decisions);
  (c) Task 6 runs the `cargo build -p quartzite --no-default-features
  --features libm` gate explicitly.
- **Miri (post-merge `master` run): parking_lot internals under Tree
  Borrows:** mitigated by — (a) the project already exercises
  `parking_lot::Mutex` / `parking_lot::RwLock` via `quartzite-core` and
  `quartzite-runtime` and Miri stays green per issue #427; (b) the migration
  adds no new unsafe code (parking_lot's internals are the only `unsafe` and
  are upstream-audited). Acceptable residual risk.
- **`MutexGuard<'static, ()>` field in `quartzite-widgets/tests/support_internals.rs`
  line 41:** `parking_lot::MutexGuard` has a different module path than
  `std::sync::MutexGuard`; the rewrite must change both the type literal AND
  the field's import path. Mitigated by Task 5 calling out both the
  `_lock: std::sync::MutexGuard<'static, ()>` field type rewrite and the
  import migration to `parking_lot::MutexGuard` explicitly.
- **`cargo update` pulls a major bump on an unrelated dep between Tasks 2/4
  and Task 6's final `cargo update`:** mitigated by — running `cargo update`
  in Task 2 and Task 4 already refreshes `Cargo.lock`; Task 6's `cargo update`
  is a no-op confirmation step rather than a fresh resolution. If a major
  bump does slip in, it would surface as a CI build failure on the affected
  task's commit, before reaching Task 6.

## Test Design

The migration is mechanical; no new test is added. The existing test surface
verifies behaviour-preservation per spec AC5 (`cargo test --workspace` green).
Specific scenarios already covered by the existing suite, broken out for
auditability:

- **`quartzite-style/src/registry.rs` `#[cfg(test)] mod tests`:** the three
  remaining tests `try_style_returns_none_before_set`,
  `try_style_returns_some_after_set`, `set_style_replaces_previous` exercise
  the lock + guard happy-path post-migration. The deleted
  `try_style_recovers_from_poisoned_mutex` is intentionally removed
  (spec AC11) — `parking_lot::Mutex` has no `PoisonError`, so the test's
  contract is meaningless. **No replacement test.**
- **`quartzite-runtime/src/factory.rs` `#[cfg(test)] mod tests`:** existing
  `registered_class_creates_instance`, `unregistered_class_returns_none`,
  and others exercise `install` / `global` / `register` post-migration —
  the return type change on `global()` is type-checked by the compiler;
  the runtime contract (`Option<Arc<RwLock<Self>>>`) is unchanged at the
  lock-method-call level. **No replacement test.**
- **`quartzite-runtime/tests/snapshot.rs`:** the integration test's
  `install_factory()` helper at line 187 now uses
  `arc.write().register(...)` instead of
  `arc.write().unwrap_or_else(|e| e.into_inner()).register(...)`; the
  shared-process fallback path (when `ObjectFactory::install` already
  errored) still exercises the lock under contention via `OnceLock`'s
  `get_or_init`. **No replacement test.**
- **`quartzite-widgets/tests/support_internals.rs`:** `ENV_LOCK`-serialised
  env-mutation tests continue to exercise the lock guard's `'static`-bound
  field semantics; `parking_lot::MutexGuard<'static, ()>` has identical
  variance + `Drop` semantics to `std::sync::MutexGuard<'static, ()>`.
  **No replacement test.**
- **`quartzite-renderer/{src/wrapped_handler.rs,tests/multi_window.rs,
  tests/support/mod.rs}`:** all migrated `Arc<Mutex<Vec<RootEvent>>>`-style
  shared-state collectors continue to exercise their `.lock()` paths
  through the existing window-event tests; no semantic change. **No
  replacement test.**

**Post-merge regression coverage:** spec AC14 — first post-merge `master` Miri
run under Tree Borrows acts as regression coverage for parking_lot guard usage
on migrated call sites. Action on failure: file a fresh bug per `/bugfix`.

**Coverage of the new `quartzite-core/src/id.rs` site (added during the
2026-05-17 spec-amendment round 1):** the four existing
`#[cfg(feature = "std")]` concurrent tests —
`object_id_new_returns_distinct_concurrent`,
`connection_id_new_returns_distinct_concurrent` (each running 64 threads via
`thread::scope` and consolidating IDs into the `Mutex<HashSet<u64>>`), plus the
two scalar `*_new_returns_distinct_sequential` / `*_later_allocation_is_greater`
tests — exercise the `.lock()` + `HashSet::insert` / `HashSet::len` paths
post-migration. **No replacement test.**

**Coverage of the new `quartzite-runtime/tests/per_thread_loops.rs` and
`quartzite-runtime/tests/timer.rs` sites (added during 2026-05-17
spec-amendment round 2):** the existing top-level `#[test]` functions in
each file — `per_thread_loops.rs` exercises the `Arc<Mutex<Option<ThreadId>>>`
shared-thread-id assertion (closure posted via `QueuedDispatcher` records
the executing thread, main thread reads it back); `timer.rs` exercises the
three `Mutex`-protected accumulators (`counts: Arc<Mutex<Vec<usize>>>`,
`el_thread_id: Arc<Mutex<Option<ThreadId>>>`, `observed_thread`) across the
fire-count, single-shot, and thread-affinity scenarios — exercise every
migrated `.lock()` site post-migration. The `.expect("…")` extractors
disappear with the shape rewrite; the underlying `HashSet::insert` /
`Vec::push` / `Option` payload reads are semantically unchanged. **No
replacement test.** Note that `timer.rs` line 213's
`el_thread_id.lock().expect("el_id lock").unwrap()` retains its trailing
`.unwrap()` post-migration — that `.unwrap()` is on the inner
`Option<ThreadId>` value, not on the lock result, and survives as
`el_thread_id.lock().unwrap()` (lock → guard → deref → `Option::unwrap()`).

## Open questions

None — every design-affecting question is resolved by AGENTS.md defaults
(API Stability AXIOM, Dependency-Versions AXIOM, Propagation Rule) or by
the spec's unambiguous AC list (zero-hits binds the scope union).
