# Replace `serial_test` with a `parking_lot::Mutex`-based test helper

**Source:** issue #440
**Date:** 2026-05-17
**Tracked in:** #440

## Scope

1. Create a new workspace member crate `quartzite-test-helpers/` with its own
   `Cargo.toml` declaring `parking_lot = "0.12"` as a regular `[dependencies]`
   entry (the helper uses `parking_lot::Mutex` at runtime; consumers depend on
   the helper crate, not on `parking_lot` directly via this path). The crate's
   `src/lib.rs` exports `pub fn test_lock() -> parking_lot::MutexGuard<'static, ()>`
   over `static TEST_LOCK: parking_lot::Mutex<()>`. Add
   `"quartzite-test-helpers"` to the root `Cargo.toml`'s `[workspace] members =
   [...]` array.
2. Remove `serial_test` from every `Cargo.toml` in the workspace:
   - Workspace root `Cargo.toml` (`[dev-dependencies]`).
   - `quartzite-runtime/Cargo.toml` (`[dev-dependencies]`).
   - `quartzite-core/Cargo.toml` (`[dev-dependencies]`).
   - `quartzite-style/Cargo.toml` (`[dev-dependencies]`).
   - `quartzite-style-dispatch/Cargo.toml` (`[dev-dependencies]`).
3. Add `quartzite-test-helpers = { path = "../quartzite-test-helpers" }` (or
   the root-relative equivalent for the workspace-root integration tests) to
   the `[dev-dependencies]` of every crate listed in item 2 (5 `Cargo.toml`
   files), so every test binary that currently uses `#[serial]` can call
   `quartzite_test_helpers::test_lock()`.
4. Replace every `#[serial]` / `#[serial_test::serial]` attribute (41 total
   across 9 source files) with an in-body `let _lock =
   quartzite_test_helpers::test_lock();` acquisition (or a localised `use
   quartzite_test_helpers::test_lock;` + `let _lock = test_lock();` shape —
   design picks).
5. Remove every `use serial_test::...` import.
6. Single shared serialisation group, defined once in
   `quartzite-test-helpers`. Each test binary is a separate process that links
   its own instance of the `static TEST_LOCK` symbol — so per-binary
   serialisation semantics are preserved while the helper code lives in
   exactly one place. **No copy-paste of the static + helper fn across
   crates.**
7. `cargo update` to remove `serial_test`, `scc`, and `sdd` from `Cargo.lock`.

### Call sites to migrate (41 attributes / 9 files)

| File | `#[serial]` count |
|------|-------------------|
| `quartzite-core/src/signal.rs` | 9 |
| `quartzite-core/src/connect.rs` | 1 |
| `quartzite-runtime/src/snapshot/object.rs` | 4 |
| `quartzite-runtime/src/snapshot/tree.rs` | 3 |
| `quartzite-runtime/tests/snapshot.rs` | 7 |
| `quartzite-style/src/registry.rs` | 5 (1 `use` + 4 attrs — recount during impl) |
| `quartzite-style/src/default_style_tests.rs` | 1 |
| `quartzite-style-dispatch/src/dispatch.rs` | 12 |
| `tests/signal_to_signal.rs` (root facade integration test) | 1 |

Mechanical recount during implementation. The number 41 in #440 is informative, not load-bearing — ACs key off exact-match greps.

## Out of scope

- Splitting the single bare-default group into multiple named groups (no named groups exist in the codebase today; v1 preserves single-group semantics).
- Cross-binary test isolation (`serial_test` does not provide this either; nothing changes).
- Migrating production code from `std::sync::Mutex` to `parking_lot::Mutex` — `#442` already standardised the workspace preference and landed independently. This issue only adopts that preference for the new test helper.
- Resolving #427 directly. This issue is the *implementation track* for option (f) of #427's proposed paths. Verifying that the post-merge Miri run is green is a release-gate observation, not a code change in this PR.

## Deferred

- A future PR introducing named serialisation groups (if/when tests legitimately need distinct lock domains) | because v1 has no demonstrated need | yes — file a fresh issue if/when a named-group requirement materialises.

## Key decisions

| Question | Decision |
|----------|----------|
| Mutex implementation for the helper? | `parking_lot::Mutex` — workspace standard per AGENTS.md / `ai-docs/code-style.md` *Library safety idioms*, no poisoning, no `lock().ok()?` ceremony. `#442` already landed this preference. |
| Number of locks? | One per test binary (`static TEST_LOCK`). All 41 attribute sites use bare `#[serial]` (no named groups), so a single lock preserves observable behaviour. |
| Helper return type? | `parking_lot::MutexGuard<'static, ()>` — RAII drop releases the lock at end of scope (mirrors `#[serial]` "until test returns" semantics). |
| Poison handling? | Not applicable — `parking_lot::Mutex` cannot poison. This matches `serial_test`'s resilience (a panicking test does not wedge the lock). |
| `parking_lot` availability in each affected crate? | Not directly relevant under the shared-crate placement — only `quartzite-test-helpers/Cargo.toml` declares `parking_lot = "0.12"` as a regular dep. Consumer crates depend on `quartzite-test-helpers` via `[dev-dependencies]`; they do **not** need their own `parking_lot` dev-dep entry for the test-lock use case. `quartzite-core`'s `#[serial]` sites remain `#[cfg(feature = "std")]`-gated; the `quartzite-test-helpers` dev-dep declaration in `quartzite-core/Cargo.toml` is unconditional (dev-deps only compile during `cargo test`, never on the `--no-default-features` production path). |
| Helper placement (per-crate static / shared crate / `tests/support`)? | **Resolved at spec level — option (b) shared crate `quartzite-test-helpers`.** Per user instruction during spec amendment: copy-pasting the static + helper fn across 4+ crates is rejected (see [`ai-docs/learnings.md`](../learnings.md) 2026-05-17 entry on copy-paste avoidance for ≥ 3 sites). The static `TEST_LOCK` is defined once in the shared crate; each test binary that depends on it links its own instance (per-binary semantics preserved). Design phase only decides per-binary linkage details (whether the consuming sites use a localised `use` import or the fully-qualified path) and smoke-test placement. |
| Helper function name? | `test_lock()` per the skeleton in #440 — matches the existing `serial_test::serial` mental model. |
| Migration order across crates? | Design phase decides task decomposition. Single PR is permissible because the change is mechanical, but design agent may split per-crate for review-clarity. |

## Technical constraints

- **AGENTS.md *Library safety idioms*** — `parking_lot::Mutex` / `parking_lot::RwLock` are the workspace default; the helper adopts this.
- **AGENTS.md *Rust Test Conventions*** — unit tests stay co-located in `#[cfg(test)] mod tests`; integration tests stay in `tests/`. The helper preserves this layout (no test relocation).
- **AGENTS.md *Dependency Versions*** — when removing `serial_test = "3"` and adding `parking_lot` where missing, follow the existing `parking_lot = "0.12"` style (caret on `0.x`). No `~`, no patch-level pin.
- **`quartzite-core` `std` feature gating** — every `#[serial]` site in `quartzite-core` is already inside `#[cfg(feature = "std")]`. The helper and its uses inherit the same gate; `no_std`-only builds remain `parking_lot`-free.
- **`Cargo.lock`** — must be refreshed via `cargo update` after Cargo.toml edits so `serial_test`, `scc`, and `sdd` are removed from the lockfile and the build (per AGENTS.md *Workflow* "run `cargo build` before committing so `Cargo.lock` is refreshed").
- **Workflow gate** — `actionlint` not in play (no workflow files touched). Standard `cargo build` / `cargo clippy --workspace -- -D warnings` / `cargo fmt -- --check` / doc gate / `cargo test --workspace` all apply.
- **`pub(crate)` visibility for the helper** — keep `test_lock` `pub(crate)` (unit-test helper) or test-binary-local (integration-test helper) so it never leaks into the public API surface.
- **`_unchecked` AXIOM** — `test_lock` does not take user input and never panics on contention; no naming concern.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `grep -r 'serial_test' --include='Cargo.toml' .` returns zero matches. |
| AC2 | `rg '#\[serial' --type rust \| wc -l` returns `0`. |
| AC3 | `rg 'use serial_test\|serial_test::' --type rust \| wc -l` returns `0`. |
| AC4 | `cargo build` and `cargo build -p quartzite --no-default-features --features libm` both succeed clean. |
| AC5 | `cargo test --workspace` passes — pre-change test count (1446+) is preserved within ±1 (the helper itself may carry one or two doctest examples). No new test failures. |
| AC6 | `cargo fmt -- --check` clean. |
| AC7 | `cargo clippy --workspace -- -D warnings` clean. |
| AC8 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean. |
| AC9 | `Cargo.lock` no longer lists `serial_test`, `scc`, or `sdd`: `grep -E '^name = "(serial_test\|scc\|sdd)"' Cargo.lock` returns no matches. |
| AC10 | `cargo tree --invert sdd` returns empty (no remaining reverse deps on `sdd`). |
| AC11 | The new `quartzite-test-helpers` crate exists at workspace root, is registered in the root `Cargo.toml` `[workspace] members` array, declares `parking_lot = "0.12"` in `[dependencies]`, and exports `pub fn test_lock() -> parking_lot::MutexGuard<'static, ()>` with at least a one-line `///` doc comment plus a top-of-module note explaining the single-per-binary serialisation guarantee. |
| AC12 | Every crate that previously declared `serial_test = "3"` in `[dev-dependencies]` now declares `quartzite-test-helpers = { path = ... }` in `[dev-dependencies]` (5 `Cargo.toml` files: root + `quartzite-core` + `quartzite-runtime` + `quartzite-style` + `quartzite-style-dispatch`). |
| AC13 | Every test previously carrying `#[serial]` acquires `test_lock()` (via `quartzite_test_helpers::test_lock()` or a `use` import) as the first statement in its body (before any other lock, dispatcher install, or registry mutation) so the serialisation contract is preserved end-to-end. |
| AC14 | No file outside `quartzite-test-helpers/src/lib.rs` defines a `static TEST_LOCK: parking_lot::Mutex<()>` (or equivalent) — verifies the no-copy-paste contract from spec Scope item 6. `rg -U 'static\s+TEST_LOCK\s*:' --type rust` returns exactly one hit, inside `quartzite-test-helpers/`. |
| AC15 | Post-merge: the first scheduled / on-demand Miri Tree Borrows run is green — the `sdd/tag.rs:49` integer-to-pointer warning no longer fires (verifies #427 resolution path (f)). This AC is observed, not asserted in CI of *this* PR. |

## Open questions

_None — design-affecting ambiguities (helper placement, per-crate split vs shared module, exact location of the `static` in unit-test modules) are deferred to the design phase per Key Decisions._
