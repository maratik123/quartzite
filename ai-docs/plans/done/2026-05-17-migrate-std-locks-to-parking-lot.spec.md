# Migrate `std::sync::{Mutex,RwLock}` to `parking_lot`

**Source:** issue #442
**Date:** 2026-05-17
**Tracked in:** #442

## Scope

Replace every remaining `std::sync::Mutex` and `std::sync::RwLock` use across the
workspace with the corresponding `parking_lot::Mutex` / `parking_lot::RwLock`,
dropping the `.unwrap_or_else(|e| e.into_inner())` poisoned-recovery boilerplate
at every call site. `parking_lot` is already a direct dependency of
`quartzite-core` (optional, gated by the `std` feature) and `quartzite-runtime`,
so this issue extends an established workspace convention rather than
introducing a new primitive. The workspace also already re-exports
`parking_lot::Mutex` through `quartzite_core` (`quartzite-core/src/lib.rs:76`,
re-surfaced in `src/lib.rs:365` as `quartzite::prelude::Mutex`); the prelude
re-export stays as is.

**Production sites — `std::sync::Mutex` (8 files):**

The five enumerated in the issue body, plus three additional hits found by the
verification grep that the issue's per-file list omitted, plus
`quartzite-core/src/id.rs` added during the 2026-05-17 spec-amendment round (the
grouped-import shape `use std::{... sync::Mutex ...}` was missed by both the
issue body and the initial spec-draft grep — see the amended AC regex below).
All eight are bound by the zero-hits acceptance criterion below.

- `quartzite-style/src/registry.rs` (top-level `use std::sync::{Mutex, OnceLock};`)
- `quartzite-renderer/src/wrapped_handler.rs` (inside `#[cfg(test)] mod tests`)
- `quartzite-widgets/src/widgets/text_edit.rs` (inside `#[cfg(test)] mod tests`)
- `quartzite-widgets/src/widgets/button.rs` (inside `#[cfg(test)] mod tests`)
- `quartzite-widgets/src/widgets/line_edit.rs` (inside `#[cfg(test)] mod tests`)
- `quartzite-runtime/src/thread_pool.rs` (inside `#[cfg(test)] mod tests`, line 108)
- `quartzite-runtime/src/event_loop.rs` (inside `#[cfg(test)] mod tests`, line 285)
- `quartzite-core/src/id.rs` (inside `#[cfg(feature = "std")]` + `#[cfg(test)] mod tests`, line 130: `use std::{collections::HashSet, sync::Mutex, thread};`; **2** `Mutex::new(...)` declarations at lines 164, 180 and **4** `.lock().unwrap()` rewrites at lines 169, 173, 185, 189). `quartzite-core` already declares `parking_lot` (optional, gated by `std`), so no `Cargo.toml` change is needed for this file.

**Production sites — `std::sync::RwLock` (1 declarator + 2 consumers):**

- `quartzite-runtime/src/factory.rs` — `static FACTORY: OnceLock<Arc<RwLock<ObjectFactory>>>`
  declarator at `sync::{Arc, OnceLock, RwLock}` (line 4); plus the public
  `pub fn global() -> Option<Arc<RwLock<Self>>>` (line 103) whose return type
  must change to `Arc<parking_lot::RwLock<Self>>`. This is a **public-API
  break** — acceptable per AGENTS.md *API Stability* (pre-`cargo publish`, no
  compat shims). Doc-comment examples on `install` / `global` / `register` /
  surrounding API also need to drop the `.expect("poisoned")` calls.
- `quartzite-runtime/src/snapshot/object.rs` — 2 consumer call sites using
  `.read().unwrap_or_else(|e| e.into_inner())`.
- `quartzite-runtime/src/snapshot/tree.rs` — 1 consumer call site.

**Test sites (7 files):**

- `quartzite-renderer/tests/support/mod.rs` (`Mutex`, single-line `use std::sync::{Arc, Mutex};` at line 15)
- `quartzite-renderer/tests/multi_window.rs` (`Mutex`, single-line `use std::sync::{Arc, Mutex};` at line 21)
- `quartzite-widgets/tests/support_internals.rs` (`Mutex`, single-line `use std::sync::Mutex;` at line 24) — also has a
  `std::sync::MutexGuard<'static, ()>` field type at line 41 that must follow
  the import migration to `parking_lot::MutexGuard`.
- `quartzite-runtime/tests/snapshot.rs` (`RwLock` — no `sync::` import; only a
  consumer-side `.write().unwrap_or_else(|e| e.into_inner())` at line 194 against the
  `factory.rs`-returned `Arc<RwLock<…>>`; AC3 covers it).
- `quartzite-runtime/tests/event_loop.rs` (`Mutex`, multi-line `use std::{ sync::{Arc, Mutex}, … }` at lines 1–2 — **not enumerated in the issue body but in scope via the ACs**)
- `quartzite-runtime/tests/per_thread_loops.rs` (`Mutex`, multi-line `use std::{ sync::{ Arc, Mutex, atomic::{...}, mpsc, }, … }` at lines 2–10 — **added during the 2026-05-17 spec-amendment round 2; missed by the round-1 single-line AC1/AC2 regex pair; AC1/AC2 now use `rg -U`**; 2 `.lock().unwrap()` rewrites at lines 39, 48 and 1 `Mutex::new(...)` declaration at line 33)
- `quartzite-runtime/tests/timer.rs` (`Mutex`, multi-line `use std::{ sync::{ Arc, Mutex, atomic::{...}, mpsc, }, … }` at lines 22–28 — **added during the 2026-05-17 spec-amendment round 2; missed by the round-1 single-line AC1/AC2 regex pair; AC1/AC2 now use `rg -U`**; **7** `.lock().expect("…")` rewrites at lines 77, 91, 193, 204, 213, 223, 247, plus a surviving `.unwrap()` on the inner `Option<ThreadId>` payload at line 213 (NOT on the lock — stays post-migration), and **3** `Mutex::new(...)` declarations at lines 69, 188, 216. The `.expect("…")` shape is the parking_lot-compiler-incompatible form: parking_lot's `lock()` returns the guard directly with no `Result`, so the `.expect("…")` extractor disappears.)

**`Cargo.toml` dependency additions:**

Each crate listed above that does not already declare `parking_lot` adds
`parking_lot = "0.12"` to `[dependencies]` (or `[dev-dependencies]` for
test-only consumers). Per AGENTS.md *Dependency Versions*, write `0.12`
verbatim (matches the existing pattern in `quartzite-runtime/Cargo.toml:19` and
`quartzite-core/Cargo.toml:34`; the `^0.12` cargo default semantics are
intended). After the edit, run `cargo update` followed by `cargo build` to
refresh `Cargo.lock`.

Verify with `grep -l parking_lot */Cargo.toml` before editing — the crates that
need the dep added are:
- `quartzite-style/Cargo.toml`
- `quartzite-renderer/Cargo.toml`
- `quartzite-widgets/Cargo.toml`

`quartzite-core` and `quartzite-runtime` already declare it. (Tests sitting
inside `quartzite-renderer` and `quartzite-widgets` are dev-side; the regular
`[dependencies]` entry covers both prod and dev uses within the same crate.)

**Delete poison-specific test surface in `quartzite-style/src/registry.rs`:**

- Delete the `poison_for_test()` `pub(crate) fn` (lines ~144–159) — parking_lot
  has no poison concept, so the helper has no semantics to test.
- Delete the `try_style_recovers_from_poisoned_mutex` test that calls it
  (around line 277).
- Delete or rewrite the module-level doc comment claiming "Lock-poisoning is
  recovered via …" (line 7) and the `try_style` doc paragraph "The Mutex poison
  flag is intentionally tolerated" (lines 101–103).

**Instruction-file mirror (Propagation Rule):**

- `AGENTS.md` § *Library safety idioms* — rewrite the mutex bullet to name
  `parking_lot::Mutex` / `parking_lot::RwLock` as the workspace default. The
  `.unwrap_or_else(|e| e.into_inner())` recovery idiom is retained only as a
  fallback note for rare FFI-imposed `std::sync` retainees, no longer the
  workspace-default pattern. The `OnceLock` / `Arc` / `Weak` / `AtomicBool`
  bullet stays unchanged (`OnceLock` is explicitly out of scope).
- `ai-docs/code-style.md` § *Library safety idioms* (lines 73–87) — mirror
  edit, paired with the AGENTS.md change in the same commit per the Propagation
  Rule's `AGENTS.md` ↔ `ai-docs/code-style.md` pair.

**Workspace clippy allow removal (root `Cargo.toml`):**

After every `.unwrap_or_else(|e| e.into_inner())` site is gone, delete from
`[workspace.lints.clippy]`:

- the `redundant_closure_for_method_calls = "allow"` line (line 66 of root
  `Cargo.toml`), **and**
- its justifying comment block above it (lines 65, the
  "AGENTS.md *Library safety idioms* designates …" sentence).

`cargo clippy --workspace -- -D warnings` must stay green after the removal.

The adjacent `significant_drop_in_scrutinee` / `significant_drop_tightening`
allows (lines 55–58) **stay** — `parking_lot::MutexGuard` has the same
significant-drop semantics as `std::sync::MutexGuard`.

## Out of scope

- Migrating `std::sync::Once` / `std::sync::OnceLock` (called out by the issue
  body — these are not mutexes and have no parking_lot equivalent that improves
  on std).
- Migrating `std::sync::Arc` / `std::sync::Weak` / `std::sync::atomic::*` /
  `std::sync::mpsc` — out of scope; only `Mutex`/`RwLock`/`MutexGuard`/
  `RwLockReadGuard`/`RwLockWriteGuard` move.
- Replacing the `quartzite_core::Mutex` re-export at
  `quartzite-core/src/lib.rs:76` — it already re-exports `parking_lot::Mutex`.
- Performance benchmarking. The motivation is consistency, correctness
  (no poisoning), and idiom simplification; any perf delta is a side-effect.

## Deferred

- Cross-references in `ai-docs/learnings.md` / past spec docs to "poisoned
  mutex recovery" wording — these stay verbatim as historical record per
  AGENTS.md Corrections-Log Boundary rule 1 (append-only). | why: historical
  artefact, not actionable | separate issue needed? no — no follow-up issue.

## Key decisions

| Question | Decision |
|---|---|
| Compat shims / `pub use` aliases / `#[deprecated]` wrappers for the `factory.rs` public-API change? | None. AGENTS.md *API Stability*: pre-publish, clean breaks. Update doc-comment examples and let downstream code (there is none) follow. |
| `Cargo.toml` version constraint for `parking_lot`? | `"0.12"` verbatim, matching the existing entries in `quartzite-core` / `quartzite-runtime` and AGENTS.md *Dependency Versions* (`0.x` for `0.x.y`). |
| Where does the dep go for crates whose only `parking_lot` use is `#[cfg(test)] mod tests` inside a `src/` file? | `[dependencies]`, not `[dev-dependencies]` — inline test modules compile as part of the crate's normal target. (`tests/` integration files would be `[dev-dependencies]`; this matters for `quartzite-renderer`, which has both inline `#[cfg(test)]` Mutex uses and a `tests/` Mutex use, and `quartzite-widgets`, same pattern.) |
| Preserve the `poison_for_test` helper as a no-op for API stability? | No. Delete it together with `try_style_recovers_from_poisoned_mutex` — parking_lot's `Mutex` has no poison state, so the helper's contract is meaningless. Pre-publish; nothing depends on it externally. |
| Adjust `significant_drop_in_scrutinee` / `significant_drop_tightening` workspace clippy allows? | No. `parking_lot::MutexGuard` carries the same significant-drop semantics as `std::sync::MutexGuard`; the justifying comments at lines 55–58 of root `Cargo.toml` remain accurate. |
| `--no-default-features --features libm` build impact? | None. `parking_lot` requires `std`, but the three new dep-adding crates (`quartzite-style`, `quartzite-renderer`, `quartzite-widgets`) are already std-only (`quartzite-style/Cargo.toml:9` description says `(std)`). `quartzite-core` keeps its `dep:parking_lot` gating behind the `std` feature unchanged. |
| Commit decomposition? | Adopt the 6-commit sequence from the issue body. Step 1 (instruction files) lands first so subsequent code commits can cite the revised idiom. Step 5 (clippy allow removal) lands only after every `.unwrap_or_else(|e| e.into_inner())` is gone — this is the gating verification. |

## Technical constraints

- **AGENTS.md *Library safety idioms* update is bound by the Propagation Rule.**
  The `AGENTS.md` ↔ `ai-docs/code-style.md` pair is explicitly listed in the
  rule's table; both edits must land in the same commit.
- **AGENTS.md *Dependency Versions* AXIOM** is satisfied by the verification
  greps already run during spec drafting (`grep -n 'parking_lot' */Cargo.toml`
  confirms the four crates already declaring it; the three that need it added
  are listed above).
- **Per-file rewrite shape** (mechanical, no judgement calls):
  - `use std::sync::{Mutex, …};` → `use parking_lot::Mutex;` + retain other
    std::sync imports (`Arc`, `OnceLock`, etc.) in a separate `use` statement
    if they remain in scope.
  - `lock().unwrap_or_else(|e| e.into_inner())` → `lock()` (returns
    `MutexGuard` directly).
  - `read().unwrap_or_else(|e| e.into_inner())` → `read()`.
  - `write().unwrap_or_else(|e| e.into_inner())` → `write()`.
  - `.lock().unwrap()` / `.read().unwrap()` / `.write().unwrap()` in test code
    → `.lock()` / `.read()` / `.write()`.
  - `.lock().expect("…")` / `.read().expect("…")` / `.write().expect("…")` in
    test code → `.lock()` / `.read()` / `.write()` (parking_lot's lock methods
    return guards directly, so the `.expect(...)` extractor disappears; the
    `timer.rs` integration test uses this shape exclusively).
  - `MutexGuard<'static, T>` field types → `parking_lot::MutexGuard<'static, T>`
    (parking_lot's `MutexGuard` is invariant in the same way; the
    `support_internals.rs` field at line 41 is the only one).
- **`static` initialisers** continue to work without `OnceLock` wrapping —
  `parking_lot::Mutex::new` and `parking_lot::RwLock::new` are `const fn`.
- **Doc-comment examples in `factory.rs`** (lines 17–22, 30–37, 47–53, 75–79,
  93–101, 115–121, etc.) must drop `.expect("poisoned")` since the lock methods
  no longer return `Result`. Some examples use `.unwrap()` on the lock — drop
  the `.unwrap()` too.
- **CI gate**: the existing `cargo build` / `cargo test --workspace` /
  `cargo fmt -- --check` / `cargo clippy --workspace -- -D warnings` /
  doc gate / `--no-default-features --features libm` build are all already
  required in `AGENTS.md § Build & Test`; no new gate added by this issue.
  Miri's first post-merge master run is regression coverage; parking_lot's
  internals already pass under TB per issue #427's sister scope.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `rg -U 'sync::(Mutex\|RwLock)\b' --type rust` returns zero hits. (Catches direct-form imports `use std::sync::Mutex`, FQN usages `std::sync::Mutex::new`, and the deep-import shape `use std::{… sync::Mutex …}`. Does **not** false-positive on `sync::Arc<parking_lot::Mutex<…>>` because that contains `sync::Arc`, not `sync::Mutex`. The `-U` (multi-line) flag is harmless here — AC1's pattern has no `.` or `[^}]*` that would benefit from spanning newlines — but is kept for symmetry with AC2 and to defend against future single-line regressions in rg defaults.) |
| AC2 | `rg -U 'sync::\{[^}]*\b(Mutex\|RwLock)\b' --type rust` returns zero hits. (Catches grouped-import shapes including the **multi-line** form used by `factory.rs`, `per_thread_loops.rs`, `timer.rs`, and the inline `mod tests` of `thread_pool.rs` / `event_loop.rs` (src + tests). The `-U` flag is **load-bearing**: without it, line-based `[^}]*` cannot span the newline between `sync::{` and `Arc, Mutex,` on the next line, and the multi-line imports slip through — verified against round-2 design-review's `per_thread_loops.rs` finding. The `[^}]*` content class still confines the match to inside a brace pair, so unrelated `sync::Arc<…>` constructs in the same file cannot bridge to a separate `Mutex<…>` further down — verified against `quartzite-runtime/src/object_tree.rs` which has `std::sync::Arc<parking_lot::Mutex<…>>` and does **not** match.) |
| AC3 | `rg 'unwrap_or_else\(\|e\| e.into_inner\(\)\)' --type rust` returns zero hits. |
| AC4 | Every crate that previously used `std::sync::{Mutex,RwLock}` declares `parking_lot = "0.12"` in its `Cargo.toml`. The three crates needing the addition are `quartzite-style`, `quartzite-renderer`, `quartzite-widgets`. `quartzite-core` (already declares it, optional `std` gate) and `quartzite-runtime` (already declares it) are unchanged. |
| AC5 | `cargo build`, `cargo test --workspace`, `cargo fmt -- --check`, and `cargo clippy --workspace -- -D warnings` all pass on the final HEAD. |
| AC6 | Root `Cargo.toml` `[workspace.lints.clippy]` no longer contains the `redundant_closure_for_method_calls = "allow"` line or its justifying comment (lines 65–66 of the pre-change file). |
| AC7 | Doc gate passes: `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. |
| AC8 | `cargo build -p quartzite --no-default-features --features libm` passes (no_std / derive-free path unbroken). |
| AC9 | `AGENTS.md § Library safety idioms` bullet names `parking_lot::Mutex` / `parking_lot::RwLock` as the workspace default; std fallback wording retained only as a footnote for rare FFI-imposed retainees. |
| AC10 | `ai-docs/code-style.md § Library safety idioms` (lines 73–87 pre-change) mirrors AC9, paired in the same commit per the Propagation Rule. |
| AC11 | The `poison_for_test` helper, the `try_style_recovers_from_poisoned_mutex` test, and the poison-tolerance doc-comment sentences in `quartzite-style/src/registry.rs` (lines 7, 101–103 pre-change) are deleted. |
| AC12 | `quartzite-runtime/src/factory.rs` doc-comment examples on `install` / `global` / `register` no longer contain `.expect("poisoned")` or `.unwrap()` on lock calls; the `pub fn global()` return type is `Option<Arc<parking_lot::RwLock<Self>>>`. |
| AC13 | The 6-commit decomposition from the issue body is preserved on the feature branch; commit 1 is the instruction-file pair, commit 5 is the clippy-allow removal (must land green), commit 6 is the final-audit grep + `cargo update`. |
| AC14 | First post-merge `master` Miri run is green — regression coverage for parking_lot guards under Tree Borrows on the migrated call sites (parking_lot's internals already pass TB per issue #427's sister scope). |

## Open questions

None — all design-affecting questions are resolved by AGENTS.md defaults
(API stability, dependency-version format, propagation rule) or by the issue's
own unambiguous AC list. The discrepancy between the issue's per-file scope
enumeration and its grep-based ACs is resolved silently in favour of the ACs
(zero hits binds the scope union).
