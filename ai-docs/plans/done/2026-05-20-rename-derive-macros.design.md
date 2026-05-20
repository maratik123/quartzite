# Design: Rename Derive macros to Object

**Issue:** #489
**Date:** 2026-05-20

## Approach

The proc-macro `quartzite_macros::Object` is already defined as `Object`
(`#[proc_macro_derive(Object, ...)]` in `quartzite-macros/src/lib.rs:103`).
Every occurrence of `DeriveObject` / `ObjectDerive` in the workspace is an
**import-site rename alias** — either an `as`-clause on a `use` / `pub use`,
or a downstream identifier emitted by that alias. No proc-macro renaming is
required; the work is a mechanical, single-pass alias removal across two
shapes:

1. **Prelude re-export shape** — `pub use quartzite_macros::{..., Object as DeriveObject, ...}`
   in `src/lib.rs:374` collapses to `pub use quartzite_macros::{..., Object, ...}`.
   The prelude **also** re-exports the `quartzite_core::Object` **trait** at
   `src/lib.rs:348`. The two coexist because Rust resolves names by namespace:
   derive proc-macros live in the **macro namespace**, traits live in the
   **type namespace**. `use quartzite::prelude::*;` followed by
   `#[derive(Object)] impl Object for Foo { ... }` is well-formed (precedent:
   `std::fmt::Debug` trait + `#[derive(Debug)]`). The obsolete comment at
   `src/lib.rs:371` ("`Object` is re-exported as `DeriveObject` to avoid
   shadowing…") is deleted in the same edit.

2. **Call-site / import shape** — `use … Object as DeriveObject;` (or
   `Object as ObjectDerive` in `quartzite-macros/tests/object_impl.rs`)
   plus the matching `#[derive(Extend, DeriveObject)]` / `#[derive(Extend, ObjectDerive)]`
   sites. Every such file gets the `as`-clause dropped from the `use` line
   and every `#[derive(…, DeriveObject)]` / `#[derive(…, ObjectDerive)]`
   token rewritten to `#[derive(…, Object)]`.

The doc-cross-reference `[`DeriveObject`](macros::Object)` at `src/lib.rs:179`
is the link's **display text** — the rustdoc target (`macros::Object`)
already resolves correctly. The fix changes only the display text to
`Object`, yielding `[`Object`](macros::Object)`.

The lib-crate prose comment at `src/lib.rs:13` and the doctest at
`src/lib.rs:25` both reference `DeriveObject` and are rewritten to `Object`.

The single live-doc reference outside the spec — `ai-docs/plans-summary.md:22`
— is updated from `#[derive(Extend, DeriveObject)]` to
`#[derive(Extend, Object)]`. Historic `ai-docs/plans/done/**` references and
the append-only `ai-docs/learnings.md` entry stay untouched per spec
*Out of scope* and AGENTS.md *Learning Log* Boundary rule 1.

**Rejected alternatives:**

- *Leave a `pub use Object as DeriveObject` aliasing line for backwards
  compatibility* — rejected by spec § *Key decisions* and AGENTS.md
  § *API Stability* (pre-`cargo publish`, clean breaks; no compat shims).
- *Rename the proc-macro definition itself* — rejected; the macro is already
  named `Object`. No change to `quartzite-macros/src/lib.rs:103` is required
  or appropriate.
- *Split the rename into two PRs (one per alias variant)* — rejected; both
  aliases point at the same macro and a single grep sweep covers both
  variants. Splitting would double review/CI cost with no isolation benefit.

## Decomposition

The work decomposes into three atomic edit groups by file fan-out. Each
group is independently compilable and testable; later groups depend on
earlier ones only for clean `git grep` AC verification, not for compile
correctness.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Drop the `Object as DeriveObject` alias from the prelude re-export and rewrite the surrounding lib-crate doc references. Concretely: `src/lib.rs:374` re-export line (drop `as DeriveObject`), `src/lib.rs:371` comment (delete the "re-exported as `DeriveObject` to avoid shadowing" line), `src/lib.rs:179` doc-link display text (`DeriveObject` → `Object`), `src/lib.rs:13` lib-crate doc comment (`DeriveObject` → `Object`), `src/lib.rs:25` doctest `#[derive]` invocation (`DeriveObject` → `Object`). | `src/lib.rs` | — |
| 2 | Rewrite every external test / bench / example call site under the workspace root (`use`-line `as`-clause dropped, `#[derive]` token updated). Files: `benches/macro_object.rs` (lines 5 + 10), `tests/single_dep.rs` (line 7), `tests/signal_to_signal.rs` (lines 13 + 27), `examples/combined.rs` (lines 1 doc-comment + 15 + 80), `examples/hello_object.rs` (line 5), `examples/object_tree.rs` (line 5), `examples/signals_slots.rs` (line 5). | `benches/macro_object.rs`, `tests/single_dep.rs`, `tests/signal_to_signal.rs`, `examples/combined.rs`, `examples/hello_object.rs`, `examples/object_tree.rs`, `examples/signals_slots.rs` | 1 |
| 3 | Rewrite `quartzite-macros/` integration tests + update the single live ai-docs reference. Files: `quartzite-macros/tests/via_facade.rs` (lines 6 + 8, `DeriveObject` shape), `quartzite-macros/tests/object_impl.rs` (lines 4 + 8 + 34 + 67, `ObjectDerive` shape), `ai-docs/plans-summary.md:22` (replace `DeriveObject` → `Object`). Then run the full AC battery: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`, and the two AC grep sweeps (AC1 + AC2 + AC10). | `quartzite-macros/tests/via_facade.rs`, `quartzite-macros/tests/object_impl.rs`, `ai-docs/plans-summary.md` | 2 |

Subtask count `M = 3`. Within the AGENTS.md "≤ 7 tasks per design" budget;
no splitting needed.

## Handoff plan

`M = 3` (one group, terminal):

- **Group A:** subtasks 1–3 — terminal group (3 subtasks; within the `1..=3`
  range). On entry into Group A, spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  No subsequent group, so no further handoff; Group A completes Step 8 in
  its own `/context-reset` subagent.

## Risks

- **Risk:** Rust resolves `use quartzite::prelude::*;` to bring both the
  `quartzite_core::Object` trait and the `quartzite_macros::Object` derive
  macro into scope, and a downstream user writes `Object::method(...)`
  expecting trait-namespace resolution. **Mitigation:** No mitigation needed
  — the macro lives in the macro namespace, accessed only via the `#[derive(...)]`
  attribute. `Object::method(...)` unambiguously resolves to the trait.
  The 9 call sites in this repo (every `use quartzite::prelude::*;` site +
  the doctest at `src/lib.rs:25`) exercise the exact `#[derive(Object)]` +
  trait-impl interaction; if Rust's namespace separation were broken, all
  three CI phases (build/test/clippy) would fail loudly. The `std::fmt::Debug`
  precedent (trait + same-named derive in the same crate, both glob-imported
  via `std::prelude::*`) has been in stable Rust since 1.0 — no semver risk.

- **Risk:** A grep sweep misses a `DeriveObject` / `ObjectDerive` reference
  hidden in a non-`.rs` / non-`.toml` file (e.g., a comment in a `.yml`
  workflow or in a feature-gated module). **Mitigation:** AC1 + AC2 grep
  sweeps are run as part of subtask 3; any miss surfaces immediately.
  Spec scope is explicit that `ai-docs/plans/done/**` and
  `ai-docs/learnings.md` are exempt.

- **Risk:** The doctest at `src/lib.rs:25` is gated on `feature = "derive"`
  (via `#[cfg_attr(feature = "derive", doc = r#"# Quickstart…"#)]` at
  `src/lib.rs:18`); a default-features-only run might pass even if the
  rename broke the `--no-default-features --features std` matrix.
  **Mitigation:** AC9 explicitly requires
  `cargo build -p quartzite --no-default-features --features libm`,
  and the spec's *Technical constraints* row pins all feature
  combinations CI exercises. The pre-existing learnings.md entry at line 5
  (the original cause of the alias gymnastics: a `no_run` doctest failing
  under `--no-default-features --features std`) is itself the canonical
  reminder; if the doctest changes shape, both feature paths get re-tested.

- **Risk:** `clippy::similar_names` or another pedantic lint fires when the
  derive name `Object` is in the same scope as the trait name `Object` at
  a `use quartzite::prelude::*;` site. **Mitigation:** They are the same
  spelling (not similar), so `similar_names` does not trigger. The two
  resolve via separate namespaces; clippy does not lint namespace coexistence.
  AC7 (`cargo clippy --workspace --all-targets -- -D warnings`) catches any
  surprise lint as a hard CI failure.

- **Risk:** Rustdoc display-text update at `src/lib.rs:179` accidentally
  breaks the intra-doc link by reformatting the target path. **Mitigation:**
  The edit changes only the **display label** inside the surrounding
  ``[`…`](macros::Object)`` markdown — the link target `macros::Object`
  is untouched. AC8 (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc
  --no-deps --workspace --all-features`) catches a broken intra-doc link as
  a hard CI failure.

## Test Design

The task is **structural rename**, not new logic. There are no new code
paths and therefore no new tests required. Instead, the existing test /
example / doctest surface acts as the regression-detection harness.
Per AGENTS.md § *Workflow* "TDD + lint-changed-files" — when the change
introduces no new behaviour, the existing suite is the test plan.

**Existing harness coverage (each file is *both* a target of the rename
AND a check that the rename did not break compilation):**

- **Location:** `src/lib.rs:25` (Quickstart doctest, gated on `feature = "derive"`)
  - **Entry point:** `rustdoc`-compiled doctest using `#[derive(Extend, Object)]`
    against `Counter`, exercising `prelude` glob + derive macro + trait coexistence
  - **Scenarios:** the trait-vs-derive namespace coexistence (the highest-risk
    surface in this rename) is exercised by every `Object::write_property`,
    `Object::read_property`, `Object::id` call inside the body
  - **Fixtures:** the doctest is self-contained

- **Location:** `tests/single_dep.rs`
  - **Entry point:** integration test verifying the prelude-only happy path
    (`use quartzite::prelude::*;` brings the derive macro + the trait + the
    base types). Re-validated by AC6.
  - **Scenarios:** the most common downstream usage pattern; happy path

- **Location:** `tests/signal_to_signal.rs`
  - **Entry point:** integration test using two `#[derive(Extend, Object)]`
    structs (`Emitter`, `Relay`) under `quartzite::prelude::*`
  - **Scenarios:** double-derive site under glob-import — catches any
    accidental shadowing or macro/trait namespace confusion

- **Location:** `quartzite-macros/tests/via_facade.rs`
  - **Entry point:** integration test verifying that macros accessed via the
    `quartzite::macros` facade emit `::quartzite::core` paths
  - **Scenarios:** the `quartzite::macros::Object` (no-alias) path

- **Location:** `quartzite-macros/tests/object_impl.rs`
  - **Entry point:** integration test covering sole-mode + multi-block
    `#[object_impl]` types
  - **Scenarios:** three separate `#[derive(Extend, Object)]` sites
    (lines 8, 34, 67); covers the previously-inconsistent `ObjectDerive`
    spelling

- **Location:** every file under `examples/*.rs` (`combined.rs`,
  `hello_object.rs`, `object_tree.rs`, `signals_slots.rs`)
  - **Entry point:** each example compiles as a binary target via
    `cargo build --workspace --all-targets`; AC5 + AC7 enforce this
  - **Scenarios:** all four examples use `quartzite::prelude::*;` —
    re-validates the prelude-import shape

- **Location:** `benches/macro_object.rs`
  - **Entry point:** Criterion benchmark binary; compiles under `cargo build --workspace
    --all-targets` and lints under `cargo clippy --workspace --all-targets`
  - **Scenarios:** bench target uses `quartzite::macros::Object` directly
    (not via prelude) — catches any breakage in the non-prelude path

**Acceptance-criteria gates (AC1–AC10) — all listed in the spec — act as
the project-level test plan; subtask 3 runs them in order.**

## Open questions

(none — the spec is concrete: scope, out-of-scope, key decisions, technical
constraints, and acceptance criteria are all enumerated. The AGENTS.md
§ *API Stability* AXIOM resolves the "leave a compat alias?" question,
and the Rust trait/macro namespace separation resolves the "trait-vs-derive
coexistence" question via the documented `std::fmt::Debug` precedent.)
