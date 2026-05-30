# Design: Fix all-features clippy debt across the workspace (quartzite-core + quartzite-runtime)

**Issue:** #586
**Date:** 2026-05-30

## Approach

> **Amendment note (2026-05-30, round 1):** Scope expanded from 28 quartzite-core
> sites to 35 workspace sites (28 quartzite-core + 7 quartzite-runtime). The
> original quartzite-core work (subtasks 1–3) is **already committed** on branch
> `feat/2026-05-30-all-features-clippy-debt`:
> - `fd3169a` — 24 `use_self` rewrites in quartzite-core
> - `2e3f2a2` — `BTreeMap::default()` / `const fn` / `single_char` in
>   quartzite-core (+ dual-gated `BTreeMap` import in `snapshot.rs` `mod tests`)
> - `32a88b8` — CI `Clippy` step added to the `Feature matrix` job (note: as
>   committed this step carried `--all-targets`; **round-2 subtask 5 amends it
>   to drop `--all-targets`** — see below)
>
> Subtasks 1–3 below are therefore **DONE** (kept intact for the record).
> The round-1 amendment adds the new quartzite-runtime cleanup as **subtask 4**.
> The 7 runtime sites were masked in the original `--all-features` reproduction
> because `cargo clippy ... -D warnings` aborts at the first crate that fails to
> compile clean: quartzite-core failed first, so clippy never reached
> quartzite-runtime. Cleaning quartzite-core unblocked the gate and surfaced
> them (user-approved expansion).
>
> **Amendment note (2026-05-30, round 2): the CI clippy gate sheds `--all-targets`.**
> Once all 35 clippy-lint sites were fixed, a SECOND masked layer surfaced: 51
> pre-existing `missing '///' documentation` **HARD ERRORS** (`compile_error!`
> emitted by the `#[object_impl]` proc-macro — NOT clippy lints) fire under
> `cargo clippy --workspace --all-targets --all-features -- -D warnings` AND under
> the equivalent `cargo build` (confirming they are not clippy-specific). Root
> cause: `--all-features` enables BOTH the macro's `undocumented-allow` and
> `undocumented-deny` features and `undocumented-deny` wins → undocumented
> `#[object_impl]` test/example/bench fixtures become hard errors under
> `--all-targets`. The user **DEFERRED** this whole class to follow-up issue
> **#587** and **rescoped** AC1 + the new CI clippy step to drop `--all-targets`.
> This adds **subtask 5**: a one-line amend of the already-committed Clippy step
> in `ci.yml` (`32a88b8`), removing `--all-targets` so the step mirrors the
> sibling `Build` / `Test` steps and the all-features leg stays green. Empirical
> facts (verified live, 2026-05-30, branch has all 35 sites fixed):
> - `cargo clippy --all-features --workspace -- -D warnings` (**no** `--all-targets`):
>   **EXIT 0** — the reframed AC1 / new gate command.
> - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: 51
>   macro-doc hard errors (deferred #587).
> - `cargo build --workspace --all-targets --all-features`: same 51 (not
>   clippy-specific).

Three-part mechanical cleanup, fully grounded against the live tree
(quartzite-core verified on `master`; quartzite-runtime verified on the branch,
2026-05-30):

1. **Resolve 28 `--all-features` clippy errors** in serde-gated `quartzite-core`
   code by applying clippy's per-site suggested rewrite verbatim. All 28 are
   confirmed reproducible via
   `cargo clippy --workspace --all-targets --all-features -- -D warnings`. The
   live breakdown matches the spec exactly:

   | Count | Lint | Sites (verified) |
   |------:|------|------------------|
   | 24 | `clippy::use_self` ("unnecessary structure name repetition") | `snapshot/tree.rs:29` (1), `value.rs` lines 571, 599–608, 610, 621–629, 635, 638 (23) |
   | 2 | `clippy::default_trait_access` (`BTreeMap::default()`) | `snapshot.rs:186`, `snapshot.rs:203` |
   | 1 | `clippy::missing_const_for_fn` (`const fn`) | `snapshot/tree.rs:98` (`validate_version`) |
   | 1 | `clippy::single_char_pattern` (`char` pattern) | `snapshot.rs:244` (inside a `#[cfg(test)]` assertion) |

   Per-site rewrites (each is clippy's literal `help: try:` suggestion):
   - **use_self:** replace the repeated type name with `Self`. In
     `value.rs` this lands in three constructs — `Deserialize for WeakObjectRef`
     (line 571: `WeakObjectRef(...)` → `Self(...)`), `From<ValueProxy> for Value`
     (lines 599–610: RHS `Value::X` → `Self::X`), and `Serialize for Value`
     (lines 621–638: match-arm `Value::X` → `Self::X`). In `snapshot/tree.rs:29`
     the field type `Vec<ObjectNode>` inside `struct ObjectNode` → `Vec<Self>`.
     Note the `Value::serialize` arms keep the **string literal** `"Value"`
     argument to `serialize_*_variant` unchanged — only the *type-path* `Value::`
     is rewritten to `Self::`; the serde variant name string is data, not a path.
   - **default_trait_access:** `properties: Default::default()` →
     `properties: BTreeMap::default()` at both `snapshot.rs:186` and `:203`.
     **`BTreeMap` is NOT in scope in `snapshot.rs`** — it is imported only in
     `snapshot/object.rs`; `snapshot.rs` merely re-exports
     `ObjectSnapshot`/`ObjectNode`/`TreeSnapshot` via `pub use`. Both sites are
     inside `#[cfg(test)] mod tests` (which has `use super::*;`), and `super::*`
     does not bring `BTreeMap` in. Applying clippy's verbatim `BTreeMap::default()`
     therefore fails with `error[E0433]: cannot find type BTreeMap in this scope`
     (×2). The fix MUST first add a `BTreeMap` import to the `mod tests` block,
     using the crate's established dual `alloc`/`std` idiom (verified live against
     `snapshot/object.rs:2–5` and `value.rs:3,5`):

     ```rust
     #[cfg(not(feature = "std"))]
     use alloc::collections::BTreeMap;
     #[cfg(feature = "std")]
     use std::collections::BTreeMap;
     ```

     Placed at the top of `mod tests` (after `use super::*;`). The dual gate is
     chosen first for idiom-consistency — it matches the crate's established
     sibling import idiom in `snapshot/object.rs:2–5` and `value.rs:3,5` — and
     second for forward-safety: `mod tests` is gated only by `#[cfg(test)]` (no
     feature gate), so it compiles under both the `std` and `no_std + alloc` test
     configurations, and the dual gate keeps a future or local
     `cargo test --no-default-features --features serde` path building. (No
     current CI matrix leg combines `serde`, which gates `mod snapshot`, with
     no-std, so a bare `use std::collections::BTreeMap;` would not break CI
     today — the dual gate is the consistency-and-forward-safety choice, not a
     fix for a present CI break.)
   - **missing_const_for_fn:** `pub fn validate_version` →
     `pub const fn validate_version` (purely additive; body is a comparison plus
     `Result` construction, all const-stable; no caller change).
   - **single_char_pattern:** `s.contains("1")` → `s.contains('1')` at
     `snapshot.rs:244` (a test-only assertion).

2. **Add a regression-guarding clippy step to the `features` job** in
   `.github/workflows/ci.yml`, shaped to avoid the deferred #587 collision. The
   `features:` job (verified at line 464, `name: Feature matrix`) is an
   ubuntu-only matrix over four feature sets
   (`--no-default-features --features {libm,std,derive}` + `--all-features`) with
   `Build` (line 506) and `Test` (line 509) steps. The work split across two
   commits:
   - **Committed in `32a88b8` (subtask 3):** `components: clippy` added to the
     `Install Rust toolchain` step's `with:` block (line 493) — verified absent
     before, needed by the new step (the default-features `clippy` job at line
     147 already installs it; the `features` job did not). **Keep this** — still
     required. A `Clippy` step was also added after `Test`, but with the **wrong
     shape** (`cargo clippy ${{ matrix.features }} --workspace --all-targets --
     -D warnings`, line 513).
   - **Round-2 amend (subtask 5):** drop `--all-targets` from that committed
     step so it reads
     `cargo clippy ${{ matrix.features }} --workspace -- -D warnings` —
     mirroring the sibling `Build` / `Test` steps (which never carry
     `--all-targets`). The step still inherits `RUSTC_WRAPPER: sccache` from the
     job `env`.

   **Why the gate omits `--all-targets` (the deny collision).** With
   `--all-targets`, the `--all-features` leg compiles the test/example/bench
   fixtures, which trip the 51 `#[object_impl]` macro-doc **hard errors**
   (`undocumented-deny` wins over `undocumented-allow` when `--all-features`
   enables both) — a pre-existing class distinct from any clippy lint, deferred
   to #587. Dropping `--all-targets` scopes the gate to the **production
   serde-gated lib** clippy debt across all four feature legs, which is exactly
   the 35-site debt this task cleans, and keeps the all-features leg green.

   **Deferred coverage gap (tracked in #587).** Without `--all-targets`, the
   serde-gated `#[cfg(test)]` test-module sites are **not** linted on the
   all-features leg (clippy only sees the lib target). Extending the step to
   `--all-targets` — which would lint those sites too — is blocked on resolving
   the #587 macro-doc-deny class first, so both are deferred together to #587.

3. **Resolve 7 `--all-features` clippy errors** in serde-gated
   `quartzite-runtime` code (the amendment scope), applying clippy's per-site
   suggested rewrite verbatim. All 7 verified live on the branch (2026-05-30) at
   the exact line/col below:

   | File:Line:Col | Lint | Verbatim fix |
   |---|---|---|
   | `snapshot/object.rs:144:22` | `clippy::use_self` | `Box::new(Self { … })` (inside `impl Sample { fn new_boxed }`) |
   | `snapshot/object.rs:314:25` | `clippy::default_trait_access` | `properties: BTreeMap::default()` |
   | `snapshot/object.rs:330:25` | single-item `into_iter` | `std::iter::once(("count".into(), Value::Bool(true))).collect()` |
   | `snapshot/object.rs:347:22` | `clippy::use_self` | `Box::new(Self { … })` (inside `impl BrokenObject { fn new_boxed }`) |
   | `snapshot/tree.rs:53:10` | `clippy::or_fun_call` | `ok_or_else(\|\| SerializeError::ObjectNotInTree { id: id.raw() })` |
   | `snapshot/tree.rs:244:22` | `clippy::use_self` | `Box::new(Self { … })` (inside `impl TreeSample { fn new_boxed }`) |
   | `snapshot/tree.rs:443:33` | `clippy::default_trait_access` | `properties: BTreeMap::default()` |

   Per-site notes (all verified against live source):
   - **use_self (×3):** `object.rs:144` / `:347` / `tree.rs:244` are all
     `Box::new(<StructName> { … })` inside `fn new_boxed()` of `impl
     <StructName>` test-fixture blocks — `Self` resolves to the constructed
     struct. Type-path `<StructName>` → `Self`; field initialisers unchanged.
   - **default_trait_access (×2):** `object.rs:314` and `tree.rs:443` are both
     `properties: Default::default()` inside `#[cfg(test)] mod tests`. **`BTreeMap`
     is NOT in scope** in either file: `object.rs` imports no collection at top
     level, and `tree.rs:2` imports only `std::collections::HashMap`; both
     production `BTreeMap` usages are fully-qualified `std::collections::BTreeMap::new()`.
     Applying clippy's verbatim `BTreeMap::default()` therefore needs a
     `BTreeMap` import added to each `mod tests` block, else it fails
     `error[E0433]: cannot find type BTreeMap in this scope`.

     **Import idiom — plain `use std::collections::BTreeMap;` (NOT the dual
     `alloc`/`std` gate used in quartzite-core).** Verified: `quartzite-runtime`
     is **std-only** — its `Cargo.toml` declares **no** `no_std` / `alloc` /
     `libm` feature, `src/lib.rs` has **no** `#![no_std]`, and there is **no**
     `extern crate alloc` or `alloc::` usage anywhere in the crate. The crate's
     own established collection idiom is `use std::collections::HashMap;`
     (`tree.rs:2`) and fully-qualified `std::collections::BTreeMap::new()`
     (`object.rs:35`, `tree.rs:487`). The dual gate would be wrong here — there
     is no `no_std` build of this crate to keep compiling. Add, in each `mod
     tests` block (placed with the existing `use std::…` imports — e.g.
     `object.rs:115` `use std::assert_matches;`):

     ```rust
     use std::collections::BTreeMap;
     ```

     `object.rs`'s `mod tests` (line 114) already imports `value::Value` (line
     128), so the `std::iter::once(…)` rewrite below needs no extra import; only
     the `BTreeMap` line is added.
   - **single-item `into_iter` (`object.rs:330`):** the live expression is
     `properties: [("count".into(), Value::Bool(true))].into_iter().collect()`,
     collecting one `(String, Value)` tuple into the `properties` BTreeMap field
     of `ObjectSnapshot`. Clippy's `std::iter::once(("count".into(),
     Value::Bool(true)))` yields the same single item; `.collect()` is retained,
     so it still builds the BTreeMap. `Value` is already in scope (line 128). No
     import needed (path is fully qualified, `std::iter::once`).
   - **or_fun_call (`tree.rs:53`):** the live expression is `.ok_or(SerializeError::ObjectNotInTree
     { id: id.raw() })??` inside `fn capture_node(tree: &ObjectTree, id:
     ObjectId)`. `id: ObjectId` is the param, `id.raw()` is the function call
     clippy flags (already used at `tree.rs:62`), and `SerializeError` is
     imported via `quartzite_core::snapshot::SerializeError` (top-level use,
     `tree.rs:4`). The `or_fun_call` fix defers the call into a closure:
     `.ok_or_else(|| SerializeError::ObjectNotInTree { id: id.raw() })??` — the
     trailing `??` is unchanged. Purely lazy-eval; no behavioural change.

   **No behavioural change (AC3).** All 7 are mechanical: `use_self` is a
   type-path rename, `default_trait_access` is `Default::default()` →
   `BTreeMap::default()` (same value — empty `BTreeMap`), `into_iter` →
   `iter::once` produces an identical single-element iterator, and `ok_or` →
   `ok_or_else` only defers an already-pure call. `cargo test --all-features`
   output is unchanged (AC3 backstop).

**Why run clippy across all four matrix legs (not just `--all-features`):** the
spec's stated default (Key decisions row 4) is symmetry with `Build` / `Test`,
unless that adds material CI time. Using `${{ matrix.features }}` adds clippy to
three already-cheap legs that share the leg's compile cache with `Build`/`Test`;
sccache amortises the cost. This is the lower-surprise choice and matches the
existing step idiom exactly. AC6 only requires the `--all-features` leg to be
covered — running all four legs is a strict superset and satisfies AC6.

**Rejected alternatives:**
- *Blanket `#[allow(clippy::...)]` at the sites* — forbidden by the spec (AC5)
  and AGENTS.md (no blanket allow without justification). Rejected.
- *Restrict the new clippy step to only the `--all-features` leg* (e.g. an `if`
  guard on `matrix.features`) — satisfies AC6 but breaks step symmetry and adds
  YAML conditional complexity for no benefit. Rejected in favour of the
  `${{ matrix.features }}` form.
- *Add a separate standalone `--all-features` clippy job* — duplicates toolchain
  install + sccache setup the `features` leg already pays for; the spec
  explicitly chose to reuse the existing leg. Rejected.
- *Workspace-level `use_self` config or crate-wide rewrite* — out of scope; the
  task is the 28 named sites only, not a workspace lint-policy change.
- *Keep `--all-targets` on the new clippy step (the originally-committed `32a88b8`
  shape)* — fails the all-features leg with the 51 `#[object_impl]` macro-doc
  hard errors (deny collision), which is a distinct pre-existing class deferred
  to #587. Rejected: round-2 subtask 5 drops `--all-targets` so the gate stays
  green and scoped to production lib clippy debt.
- *Fix the 51 macro-doc hard errors here so `--all-targets --all-features` goes
  green* — out of scope; the user deferred the whole macro-doc-deny class to
  #587. This task scopes to the 35 clippy-lint sites + a CI gate shaped to stay
  green. Rejected.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | **DONE (`fd3169a`).** Apply the 24 `use_self` rewrites (`Self`/`Self::`) at the verified sites | `quartzite-core/src/value.rs`, `quartzite-core/src/snapshot/tree.rs` | — |
| 2 | **DONE (`2e3f2a2`).** **First** add the dual-gated `BTreeMap` import to `snapshot.rs`'s `mod tests` (after `use super::*;`), **then** apply the 2 `BTreeMap::default()` rewrites at `snapshot.rs:186`/`:203`, the `const fn` conversion of `validate_version`, and the `'1'` char-pattern rewrite; locally gate with `cargo build --all-features -p quartzite-core --tests` before moving on | `quartzite-core/src/snapshot.rs`, `quartzite-core/src/snapshot/tree.rs` | — |
| 3 | **DONE (`32a88b8`).** Add `components: clippy` to the `features` job toolchain install + a `Clippy` step running `cargo clippy ${{ matrix.features }} --workspace --all-targets -- -D warnings`; run `actionlint .github/workflows/ci.yml` before staging. (Step shape later amended by subtask 5 — `--all-targets` dropped; `components: clippy` kept.) | `.github/workflows/ci.yml` | — |
| 4 | **DONE (round-1 amendment).** Apply the 7 quartzite-runtime rewrites: (a) **first** add `use std::collections::BTreeMap;` to the `mod tests` block of `object.rs` (~line 114) and `tree.rs` (~line 214); (b) apply the 2 `BTreeMap::default()` rewrites (`object.rs:314`, `tree.rs:443`), the 3 `use_self` rewrites (`object.rs:144`/`:347`, `tree.rs:244`), the single-item `into_iter`→`std::iter::once(…)` rewrite (`object.rs:330`), and the `ok_or`→`ok_or_else(\|\| …)` rewrite (`tree.rs:53`); (c) locally gate with `cargo build --all-features -p quartzite-runtime --tests` before moving on | `quartzite-runtime/src/snapshot/object.rs`, `quartzite-runtime/src/snapshot/tree.rs` | — |
| 5 | **NEW (round-2 amendment).** One-line amend of the already-committed Clippy step (line 513) in the `features` job: **remove `--all-targets`** so it reads `cargo clippy ${{ matrix.features }} --workspace -- -D warnings`, mirroring the sibling `Build` / `Test` steps and avoiding the deferred #587 macro-doc-deny collision on the `--all-features` leg. **Keep `components: clippy`** (still required). Run `actionlint .github/workflows/ci.yml` before staging (AC7). | `.github/workflows/ci.yml` | 3 |

Subtasks 1–4 are independent (no `Depends on`); subtask 4 does not depend on
1–3 for compilation, but it can only be *clippy-verified* end-to-end (AC1) after
1–2 land, since the workspace gate aborts at the first dirty crate — and those
landed in `fd3169a`/`2e3f2a2`. Subtask 5 `Depends on` 3 (it amends the step that
subtask 3 introduced). The full-gate verification (AC1–AC5, AC7) is performed
once after subtask 5 lands, in subtask 5's terminal-group `/context-reset`
subagent, not as a separate decomposition row.

**Subtask 2 intra-step ordering is load-bearing.** The `BTreeMap` import edit
MUST precede (or land together with) the two `BTreeMap::default()` rewrites — if
the rewrites land first, the file fails to compile with `error[E0433]: cannot
find type BTreeMap in this scope` (×2). The per-subtask local gate
`cargo build --all-features -p quartzite-core --tests` exists precisely because
AC1's clippy gate stops at the E0433 type-resolution error and would NOT surface
this distinctly as a clippy lint — clippy cannot lint a file that does not
type-check, so the missing import would masquerade as an opaque clippy failure.
The local build gate catches it directly at subtask granularity.

**Subtask 4 intra-step ordering is load-bearing (same lesson).** The two
`use std::collections::BTreeMap;` imports MUST precede (or land together with)
the two `BTreeMap::default()` rewrites (`object.rs:314`, `tree.rs:443`) — else
the files fail with `error[E0433]: cannot find type BTreeMap in this scope`
(×2). The per-subtask local gate `cargo build --all-features -p
quartzite-runtime --tests` exists for the same reason as subtask 2's: AC1's
clippy gate would stop at E0433 and mask the missing import as an opaque clippy
failure rather than a distinct type-resolution error. Note the import idiom
here is the **plain** `use std::collections::BTreeMap;`, NOT quartzite-core's
dual `alloc`/`std` gate — quartzite-runtime is std-only (see Approach part 3).

## Handoff plan

Per `.claude/agents/design.md` § Rules → handoff-grouping, the `## Handoff plan`
is mandatory for every design with `M ≥ 1`. Here `M = 5` after the round-2
amendment, split into **two** groups of `3 + 2`. There is no group-COUNT cap;
the rule is that non-terminal groups MUST be exactly 3 consecutive subtasks and
only the terminal group may be smaller (within `1..=3`). Group A is the full
3-subtask non-terminal chunk; Group B is the 2-subtask terminal chunk. The
boundaries also align with implementation state — Group A is the
already-committed quartzite-core chunk; Group B holds the already-committed
quartzite-runtime amendment (subtask 4) plus the new one-line CI-gate amend
(subtask 5). Subtasks 4 and 5 are independent (subtask 5 `Depends on` 3, not 4),
so co-grouping them is sound.

- **Group A:** subtasks 1–3 — non-terminal chunk (3 subtasks; equal to the
  3-consecutive-subtask cap). **Already committed** (`fd3169a`, `2e3f2a2`,
  `32a88b8`) — no further implementation work; retained as a group of record.
  Entry into Group A spawns `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Handoff after Group A:** spawn `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
  Parent `/task` resumes in Group B with fresh context. (Because Group A is
  already committed, the practical entry point for resumed work is Group B.)
- **Group B:** subtasks 4–5 — terminal group (2 subtasks; within the `1..=3`
  range). Subtask 4 is the **already-committed** (round-1 amendment)
  quartzite-runtime cleanup; subtask 5 is the remaining new one-line `ci.yml`
  amend (drop `--all-targets`). Entry into Group B spawns `/context-reset` per
  `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry); the
  group completes Step 8 in its own `/context-reset` subagent. The full-gate
  verification (the reframed AC1 + AC2–AC5 + AC7) runs at the end of Group B.

## Risks

- **`use_self` rewrite touches the serde variant-index string literal by
  mistake** (`serialize_*_variant("Value", …)`): mitigation — the string
  `"Value"` is a serde wire-format name and MUST stay; only the *type path*
  `Value::` is rewritten to `Self::`. AC3 (`cargo test --all-features` unchanged)
  is the backstop — any string drift breaks round-trip/bincode-index tests.
- **`BTreeMap` import resolution at `snapshot.rs:186`/`:203`** (build-break):
  `BTreeMap` is NOT in scope in `snapshot.rs` (it lives in `snapshot/object.rs`;
  `snapshot.rs` only `pub use`s the snapshot types). Applying clippy's verbatim
  `BTreeMap::default()` without first adding an import fails with
  `error[E0433]: cannot find type BTreeMap in this scope` (×2). Mitigation —
  subtask 2 adds the dual-gated `alloc`/`std` `BTreeMap` import to `mod tests`
  **before** the rewrites, and runs the per-subtask local gate
  `cargo build --all-features -p quartzite-core --tests`. The dual gate (not a
  bare `use std::...`) is chosen for idiom-consistency with the crate's sibling
  imports (`snapshot/object.rs:2–5`, `value.rs:3,5`) and forward-safety: `mod
  tests` compiles under both `std` and `no_std + alloc` test builds, so the dual
  gate keeps a future or local `cargo test --no-default-features --features serde`
  path building. (No current CI leg combines `serde` with no-std, so a std-only
  import would not break CI today — this is a consistency-and-forward-safety
  choice, not a present-break fix.) AC1's clippy gate does NOT distinctly
  surface this — it stops at the E0433 type-resolution error — so the local build
  gate is the real backstop.
- **`const fn` conversion regresses on a non-const operation:** mitigation —
  `validate_version`'s body is a `u64` comparison plus `Result` construction
  against an associated const; all const-stable on the project MSRV (Rust 1.96).
  `cargo build --all-features` (AC-adjacent) confirms compilation.
- **(runtime) Wrong `BTreeMap` import idiom in `quartzite-runtime`** (over-engineering
  / mismatch): the dual `alloc`/`std` gate used in quartzite-core would be
  *wrong* here. quartzite-runtime is std-only — no `no_std`/`alloc`/`libm`
  feature in its `Cargo.toml`, no `#![no_std]`, no `alloc::` usage; its own idiom
  is plain `use std::collections::HashMap;` (`tree.rs:2`). Mitigation — subtask 4
  adds the **plain** `use std::collections::BTreeMap;` to each `mod tests`,
  matching the crate's sibling import idiom. A dual gate would add a dead
  `#[cfg(not(feature = "std"))]` arm (the `alloc` crate is not even available
  here) and is a defect.
- **(runtime) `BTreeMap` import resolution** (build-break, `object.rs:314` /
  `tree.rs:443`): `BTreeMap` is not in scope in either `mod tests`. Mitigation —
  subtask 4 adds the import **before** the rewrites and runs `cargo build
  --all-features -p quartzite-runtime --tests`; AC1's clippy gate would otherwise
  stop at E0433 and mask the missing import as an opaque clippy failure.
- **(runtime) `into_iter`→`iter::once` collects into a different container**
  (behaviour drift): mitigation — the rewrite keeps the trailing `.collect()` and
  the single tuple `("count".into(), Value::Bool(true))`, so it still builds the
  same one-entry `BTreeMap`. `std::iter::once` yields exactly one item; AC3
  (`cargo test --all-features` unchanged) is the backstop.
- **(runtime) `ok_or`→`ok_or_else` changes eval semantics** (behaviour drift):
  mitigation — `id.raw()` is a pure accessor (already called identically at
  `tree.rs:62`); deferring it into a closure only makes the error-value
  construction lazy and never changes the produced error. The trailing `??` is
  retained verbatim. AC3 backstops.
- **No-op or false-positive after rewrite (clippy still fires):** mitigation —
  AC1 re-runs the full `--all-features` gate to 0 errors; any residual site is
  caught immediately.
- **CI clippy gate `--all-targets` deny collision (root of round-2 amendment):**
  fixing all 35 clippy-lint sites unmasked a SECOND pre-existing class — 51
  `missing '///' documentation` **hard errors** (`compile_error!` from the
  `#[object_impl]` proc-macro, NOT clippy lints) that fire under
  `--workspace --all-targets --all-features` because `--all-features` enables both
  the macro's `undocumented-allow` and `undocumented-deny` features and the deny
  wins. `cargo build --workspace --all-targets --all-features` fails identically,
  confirming the errors are not clippy-specific. Mitigation — the new CI clippy
  step (and the reframed AC1) **omit `--all-targets`**, mirroring the sibling
  `Build` / `Test` steps; `cargo clippy --all-features --workspace -- -D warnings`
  is verified EXIT 0 (2026-05-30, all 35 sites fixed). The whole macro-doc-deny
  class **and** the future extension of the CI step to `--all-targets` are
  **deferred to #587** (already filed). Accepted coverage gap: serde-gated
  `#[cfg(test)]` test-module sites are not linted on the all-features leg under
  the no-`--all-targets` shape — tracked in #587. This is a deliberate scoping
  decision, not an oversight.
- **`actionlint` skipped before staging the workflow** (AGENTS.md AXIOM): the
  workflow file is in `git status` as modified → `actionlint
  .github/workflows/ci.yml` is a hard gate before `git add`, encoded as part of
  subtasks 3 and 5. AC7 re-verifies (subtask 5 re-touches the workflow, so it
  MUST re-run `actionlint`).
- **New clippy step adds material CI time across 3 extra legs:** mitigation —
  the legs already compile under `Build`/`Test`; sccache (`RUSTC_WRAPPER`,
  job-level `env`) shares the compile cache, so clippy reuses artifacts. If
  measured time is material, fall back to restricting the step to the
  `--all-features` leg (still satisfies AC6).

## Test Design

This is a lint-only cleanup plus a CI workflow edit; no new test code is
written. Verification is the acceptance-criteria gate, run once after subtask 5
lands (subtasks 1–4 already verified at commit time):

- **AC1 (primary, REFRAMED in round 2):** `cargo clippy --all-features --workspace -- -D warnings`
  (**NO `--all-targets`**) exits 0. Entry point: all 35 previously-failing sites
  (28 quartzite-core + 7 quartzite-runtime). This is the gate that was previously
  masked — with quartzite-core clean (subtasks 1–2), it now reaches and lints
  quartzite-runtime. Verified live EXIT 0 (2026-05-30, branch has all 35 sites
  fixed). **The literal `--all-features --all-targets` combination is OUT of
  scope (deferred #587):** it still fails with 51 `#[object_impl]` macro-doc hard
  errors, which `cargo build --workspace --all-targets --all-features` reproduces
  identically (not clippy-specific).
- **AC2:** `cargo clippy --workspace --all-targets -- -D warnings` (default
  features) still exits 0 — guards against the `use_self`/`const fn` edits
  introducing a default-gate regression.
- **AC3:** `cargo test --all-features` passes with unchanged output — the
  behavioural backstop for the serde rewrites (variant index / round-trip).
- **AC4:** `cargo fmt -- --check` passes.
- **AC5:** `grep -rn '#\[allow(clippy::' quartzite-core/src/{snapshot.rs,snapshot/tree.rs,value.rs} quartzite-runtime/src/snapshot/{object.rs,tree.rs}`
  shows no new allow attribute at any of the 35 sites.
- **AC6 (REFRAMED in round 2):** the `features`-job Clippy step reads
  `cargo clippy ${{ matrix.features }} --workspace -- -D warnings` (**no
  `--all-targets`**), guarding the serde-gated PRODUCTION lib clippy debt across
  all 4 feature legs and staying green on the all-features leg. Verified
  structurally by reading the amended `ci.yml` step (subtask 5). Coverage gap
  (serde-gated `#[cfg(test)]` sites unlinted on the all-features leg) is
  deferred to #587 per AC6's note.
- **AC7:** `actionlint .github/workflows/ci.yml` passes after the workflow edit.
  Subtask 5 re-touches the workflow (drops `--all-targets`), so it **MUST**
  re-run `actionlint` before `git add` (AGENTS.md actionlint AXIOM); subtask 4
  did not touch the workflow.

The only `#[cfg(test)]` module changes are additive `BTreeMap` imports required
to make clippy's `BTreeMap::default()` rewrites resolve: the dual-gated
`alloc`/`std` import in `quartzite-core/src/snapshot.rs` (subtask 2, committed)
and the **plain** `use std::collections::BTreeMap;` in `quartzite-runtime`'s
`object.rs` + `tree.rs` test modules (subtask 4). No new fixtures, no new test
functions, no new integration tests. The existing serde round-trip / restore
tests in both crates (exercised by AC3) are the sufficient behavioural net for
the source rewrites; AC6's coverage assertion is satisfied structurally by the
new CI step, not by a unit test. The per-subtask local gates
`cargo build --all-features -p quartzite-core --tests` (subtask 2) and
`cargo build --all-features -p quartzite-runtime --tests` (subtask 4) are the
immediate checks that each test-module import resolves before the AC suite runs.

## Open questions

- (none — spec is fully specified; all line numbers, the lint breakdown, the
  `features`-job shape, and the absent `components: clippy` were verified against
  the live tree during this design. For the amendment, all 7 quartzite-runtime
  sites were re-verified live on the branch (2026-05-30): line/col, the `impl`
  blocks for the 3 `use_self` sites, both `mod tests` blocks lacking a `BTreeMap`
  import, the std-only crate config justifying the plain import, the
  `into_iter`/`collect` expression at `object.rs:330`, and the `id: ObjectId` /
  `id.raw()` / `SerializeError::ObjectNotInTree` scope at `tree.rs:53`.)
