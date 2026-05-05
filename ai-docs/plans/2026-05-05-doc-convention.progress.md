# Progress: doc-convention — ACTIVE
_Updated: 2026-05-05_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** feat/2026-05-05-doc-convention
**base_commit:** 5ee77d67d7d48cd37143d2bc18f00efbb96b7d84
**Last build:** PASS (workspace `cargo build` green; clippy expected-red — 28 warnings + 2 errors recorded as audit worklist)

**Issue:** #80
**Spec:** ai-docs/plans/2026-05-05-doc-convention.spec.md
**Design:** ai-docs/plans/2026-05-05-doc-convention.design.md

## Next action

**Do this immediately:** Subtask 3 — audit `quartzite-core`. Walk every public item under `quartzite-core/src/{id,meta,object_base,receiver_guard,signal,traits,value}.rs`; bring summary lines to 3rd-person present; add `# Parameters` for every fn with ≥1 arg; add `# Errors`/`# Panics` per clippy worklist; reorder per convention. Also fix pre-existing `clippy::approx_constant` at `quartzite-core/src/value.rs:429` (test case `3.14f64`) — replace with `std::f64::consts::PI` or a non-PI value. Verify with `cargo clippy -p quartzite-core --all-targets -- -D warnings` clean, doc gate clean, and `cargo test -p quartzite-core` green.

## Subtasks

- [x] 1. Write `ai-docs/doc-convention.md`, AGENTS.md pointer, `clippy.toml` (no clippy gate yet)
- [x] 2. Add the five lint attributes to every crate's `lib.rs` (`cargo build` still green; clippy expected red)
- [ ] 3. Audit & fix `quartzite-core` (clippy/doc/test gates clean for the crate)  ← CURRENT
- [ ] **HANDOFF here per design** — `/context-reset` after subtask 3
- [ ] 4. Audit & fix `quartzite-geometry`
- [ ] 5. Audit & fix `quartzite-events` (AC13 — `MouseEvent::new` doctest)
- [ ] 6. Add codegen tests for `quartzite-macros` (string-contains assertions on emitted docs)
- [ ] 7. Update `quartzite-macros` codegen — emit conforming docs at four `quote!` sites
- [ ] **HANDOFF here per design** (recommended) — `/context-reset` after subtask 7
- [ ] 8. Audit & fix `quartzite-runtime` (heaviest `# Errors`/`# Panics` work)
- [ ] 9. Audit & fix `quartzite` facade (`src/lib.rs`)
- [ ] 10. Update `code-review` skill + `review-findings` + `self-review` agents (Propagation Rule)
- [ ] 11. Final workspace verification — `cargo fmt --check`, full clippy/doc/test/no_std

## Key discoveries (don't re-investigate)

- Per design: no `# Parameters` / `# Errors` / `# Panics` sections currently exist anywhere in the workspace; summary tense is mostly correct (~10–15 imperative remnants per crate); `#[doc(hidden)]` macro internals are exempt.
- Trait-impl methods (inside `impl Trait for Type {}` blocks, including derives like `From`, `Display`, `Drop`) are exempt — only inherent and trait-definition methods carry the convention.
- `clippy::doc_markdown` enabled at `warn`-level becomes a hard error via CI's `cargo clippy -- -D warnings`. `clippy.toml` `doc-valid-idents` is seeded (~60 entries) and grows during the audit as new false positives surface.
- Proc-macro `quote!` sites that emit user-facing `pub` items (per design § *Proc-macro emitted-doc audit*): `emit_signal_wrappers`, `emit_connect_auto_wrappers`, `emit_connect_queued_wrappers`, `emit_root_trait_and_impl`. These four are the entire scope of subtask 7.
- Lint insertion order in each `lib.rs`: keep `#![cfg_attr(...)]`/`#![no_std]`/`#![cfg_attr(docsrs,...)]` first, then the new lints, then `#![deny(missing_docs)]`, then `#![doc=...]`.
- Audit order (subtasks 3–9) is bottom-up by dependency: `core → geometry → events → macros (tests then codegen) → runtime → facade`.

## AC Status

| AC | Status |
|----|--------|
| AC1 | PASS (subtask 1 — `ai-docs/doc-convention.md` written) |
| AC2 | PASS (subtask 1 — AGENTS.md Code Style updated) |
| AC3 | NOT_TESTED |
| AC4 | NOT_TESTED |
| AC5 | PASS (subtask 2 — five lints in every `lib.rs`) |
| AC6 | PASS (subtask 1 — `clippy.toml` seeded; will grow during 3–9) |
| AC7 | NOT_TESTED |
| AC8 | NOT_TESTED |
| AC9 | NOT_TESTED |
| AC10 | NOT_TESTED |
| AC11 | NOT_TESTED |
| AC12 | NOT_TESTED |
| AC13 | NOT_TESTED |

## Files touched

- `ai-docs/doc-convention.md` (new) — canonical doc convention reference
- `AGENTS.md` — Documentation Conventions pointer paragraph in *Code Style*
- `clippy.toml` (new) — seed `doc-valid-idents` allowlist (~60 entries)
- `quartzite-core/src/lib.rs` — added 5 lint attrs
- `quartzite-events/src/lib.rs` — added 5 lint attrs
- `quartzite-geometry/src/lib.rs` — added 5 lint attrs
- `quartzite-macros/src/lib.rs` — added 5 lint attrs
- `quartzite-runtime/src/lib.rs` — added 5 lint attrs
- `src/lib.rs` — added 5 lint attrs

## Audit worklist (from subtask 2 baseline clippy run)

`cargo clippy --workspace --all-targets` output: 28 warnings + 2 errors.

- `quartzite-core (lib)`: 2 × `missing_errors_doc`.
- `quartzite-runtime (lib)`: 12 warnings — mix of `missing_errors_doc`, `missing_panics_doc`, `doc_markdown` (2 backtick fixes).
- `quartzite-macros (lib)`: 1 × `doc_markdown` or similar.
- `quartzite-runtime (lib test)`: same 12 (duplicates).
- `quartzite-runtime (test "object_tree")`: 1 × `methods called 'new' usually return Self` — **investigate scope**.
- `quartzite-core (lib test) ERROR`: `clippy::approx_constant` at `quartzite-core/src/value.rs:429` — pre-existing, fix in subtask 3.

Run `cargo clippy --workspace --all-targets 2>&1` again at the start of each crate-audit subtask to refresh the worklist after each fix.
