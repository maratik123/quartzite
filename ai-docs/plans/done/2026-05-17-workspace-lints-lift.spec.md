# Lift per-crate doc-lint directives into `[workspace.lints.*]`

**Source:** issue #460
**Date:** 2026-05-17
**Tracked in:** #460

## Scope

1. **Extend root `Cargo.toml`** with workspace-wide doc-lint tables, lifting the 7 directives currently duplicated at the top of every member crate's `lib.rs`:

   ```toml
   [workspace.lints.rust]
   missing_docs = "deny"

   [workspace.lints.rustdoc]
   broken_intra_doc_links = "deny"

   [workspace.lints.clippy]
   # (existing entries unchanged)
   undocumented_unsafe_blocks = "warn"
   # The four below are likely already implied by
   # `pedantic = { level = "warn", priority = -1 }`. Verify during
   # implementation (see *Implementation pre-check* below); if pedantic
   # already covers them at warn-level, drop the explicit entries.
   missing_errors_doc = "warn"
   missing_panics_doc = "warn"
   missing_safety_doc = "warn"
   doc_markdown = "warn"
   ```

2. **Strip the 7 `#![deny(...)] / #![warn(...)]` directives** from every `lib.rs` that currently carries them. Live grep at spec time returns **15 files** (issue body cites 10; the spec's count is the authoritative one):

   - `src/lib.rs` (workspace facade `quartzite`)
   - `quartzite-core/src/lib.rs`
   - `quartzite-event-types/src/lib.rs`
   - `quartzite-events/src/lib.rs`
   - `quartzite-geometry/src/lib.rs`
   - `quartzite-macros/src/lib.rs`
   - `quartzite-paint/src/lib.rs`
   - `quartzite-paint-api/src/lib.rs`
   - `quartzite-renderer/src/lib.rs`
   - `quartzite-runtime/src/lib.rs`
   - `quartzite-style/src/lib.rs`
   - `quartzite-style-dispatch/src/lib.rs`
   - `quartzite-style-types/src/lib.rs`
   - `quartzite-test-helpers/src/lib.rs`
   - `quartzite-widgets/src/lib.rs`

   Every affected `Cargo.toml` already carries `[lints] workspace = true` — verified at spec time. No `Cargo.toml` edits needed beyond the root.

   `quartzite-renderer/src/lib.rs` is missing `#![deny(missing_docs)]` from its current directive block (live grep confirmed) — after the lift, it inherits the workspace-level `missing_docs = "deny"` and becomes consistent with the rest of the workspace. Any new missing-doc warnings the renderer surfaces are in-scope to fix as part of this task (see AC9).

3. **Add a `//!` crate-level doc to every integration test file that currently lacks one.** Workspace lints apply to all targets that opt in via `[lints] workspace = true` — including the `tests/*.rs` integration test crates. Pre-lift `#![deny(missing_docs)]` lived inside `src/lib.rs` and thus only covered library targets; the workspace-level lift broadens enforcement to test targets, surfacing ~20 new errors (`missing documentation for the crate`). These are addressed in Subtask 1 of the implementation: each affected `tests/<name>.rs` gets a single `//! <one-line description>` line at the top of the file. Live grep at amendment time identified the following test files lacking a `//!` (final live count + identity verified at implementation time):

   - `quartzite-core/tests/object_safety.rs`
   - `quartzite-macros/tests/{extend,meta_enum,object,object_impl,via_facade}.rs`
   - `quartzite-renderer/tests/application.rs`
   - `quartzite-runtime/tests/{application,event_loop,factory,object_tree,object_tree_ext,snapshot,timer,timer_single_shot_app}.rs`
   - workspace-root `tests/{single_dep,signal_to_signal}.rs` (additionally any other workspace-root integration test files surfacing under the same gate)

   Doc lines are short, descriptive of what the test file covers, and follow normal `///`/`//!` rules. This is a doc-only addition with no behavioural impact — the test crate count and test count are unchanged (AC10 still holds).

4. **Update cross-reference text** in instruction files (Propagation Rule fires — single PR):
   - `AGENTS.md § Code Style → Linter posture` row — extend to mention `[workspace.lints.rust]` and `[workspace.lints.rustdoc]` alongside `[workspace.lints.clippy]`.
   - `AGENTS.md § Code Style → Documentation` row — rewrite the *"every crate has `#![deny(missing_docs)]`…"* fragment to reflect workspace-level declaration.
   - `ai-docs/code-style.md § Linter posture` — mirror the AGENTS.md update.
   - `ai-docs/code-style.md § Documentation` (line 135) — rewrite *"Every crate must have `#![deny(missing_docs)]` …"* similarly.
   - `ai-docs/code-style.md § Lints that mechanically enforce parts of this convention` (line 405) — update the parenthetical *"Already present in every crate"* to reference the workspace declaration.
   - `ai-docs/doc-convention.md § Lints that mechanically enforce parts of this convention` (line 539) — same update: *"Each crate's `lib.rs` enables:"* becomes *"The workspace declares (each crate opts in via `[lints] workspace = true`):"* or similar wording.

## Out of scope

- Changing the **severity** of any existing lint at its existing enforcement site. Severities of all 7 lifted directives stay at their pre-lift level (`deny` stays `deny`, `warn` stays `warn`).
- Adding clippy lints not already declared per-crate today.
- Removing per-crate `[lints] workspace = true` blocks — they remain the opt-in mechanism.
- Touching `#![no_std]`, `extern crate alloc;`, or any other per-crate prologue line that is genuinely per-crate. Only the 7 lint directives lift.
- Fixing pre-existing lint warnings on master (out-of-scope unless the lift surfaces *new* warnings — see AC9 and Scope item 3).

> **Enforcement-scope broadening (intentional, NOT a severity change).** Pre-lift `#![deny(missing_docs)]` lived in `src/lib.rs` only and thus covered library targets only. Post-lift `[workspace.lints.rust] missing_docs = "deny"` applies to every target with `[lints] workspace = true` — including `tests/*.rs` integration test crates and any `examples/*.rs` declared as workspace members. This broadening is intentional (workspace lints have no per-target gating, and the alternative — per-crate target-conditional suppression — defeats the centralization goal) and is handled by Scope item 3 above (one-time `//!` additions to test files). The same broadening rationale applies to the other lifted lints (`broken_intra_doc_links`, `undocumented_unsafe_blocks`) but in practice they do not fire on test code (no intra-doc links in test files; no unsafe blocks in test files lacking `// SAFETY:`); the implementation verifies live and includes any incidental fixes under AC9.

## Deferred

- (none — issue body is self-contained.)

## Key decisions

| Question | Decision |
|---|---|
| Source of truth for the file count | Live `rg -l '#!\[deny\(missing_docs\)\]' --type rust` at spec time. Returns **15** files (issue body's "10" is stale; the renderer/runtime/macros/events crates were missed in the enumeration). |
| Whether to drop the 4 pedantic-implied clippy lines | Implementation pre-check verifies live (see below). If `cargo clippy --workspace -- -D warnings` fires them via `pedantic` already, drop the explicit lines. Otherwise keep them. The `undocumented_unsafe_blocks` line is **always kept** — it is the load-bearing add (not in pedantic). |
| Whether to also lift `#![no_std]` from `no_std` crates | No — `#![no_std]` is a crate-level attribute, not a lint, and cannot live in `[workspace.lints.*]`. Out of scope. |
| Whether to introduce a new helper crate for shared prologue | No — workspace lints are the idiomatic Cargo-native mechanism (1.74+). No new crate. |
| Commit decomposition | Per issue body's suggestion: (1) add workspace lint tables + verify gates, (2) strip per-crate directives + verify gates, (3) instruction-file cross-reference updates. Three commits on a single feature branch. |
| Renderer's missing `#![deny(missing_docs)]` line | The renderer currently lacks the `#![deny(missing_docs)]` directive in its prologue (live-grep confirmed). After the lift it inherits the workspace deny; any new missing-doc warnings surface as in-scope work (AC9 explicitly covers this). |
| Integration test crates inheriting `missing_docs = "deny"` post-lift | **Amended (2026-05-18) after Subtask 1 live measurement surfaced 20 new `cargo test --workspace` failures.** Workspace lints apply to all targets with `[lints] workspace = true` — including `tests/*.rs` integration test crates that were never under per-crate `#![deny(missing_docs)]` (which lived in `src/lib.rs` only). User-approved decision: extend Subtask 1 to add a one-line `//!` crate-doc to each affected test file (~20 files). Spec § Scope item 3 enumerates them; AC20 verifies. Alternative considered: per-target lint scoping via per-crate `[lints.rust]` blocks or `#![cfg_attr(test, allow(missing_docs))]` — rejected because both defeat the spec's centralization goal. The chosen approach incrementally expands the project's doc-coverage surface (a strict improvement). |

## Technical constraints

- **Workspace lints feature requires Cargo 1.74+.** `workspace.package.rust-version` is currently `1.95` (live-read from root `Cargo.toml` at spec time), so the feature is available unconditionally.
- **Every member crate must carry `[lints] workspace = true`** for inheritance to take effect. Live-verified: all 14 leaf crates plus the root facade already have this block. No `Cargo.toml` edits beyond the root.
- **Gate parity:** the post-lift workspace must pass every gate the pre-lift one passes, with no severity drift on any of the 7 lifted lints. See ACs.

## Implementation pre-check

Before stripping per-crate directives in commit 2, run inside commit 1:

```bash
# After adding the workspace tables, BEFORE removing per-crate directives,
# verify which of the 5 clippy doc-family lints fire via pedantic vs.
# only via the explicit entry. If `pedantic` already covers a lint at
# warn-level, the explicit entry is redundant and SHOULD be dropped to
# keep the workspace lint table minimal.
cargo clippy --workspace --message-format=json -- -D warnings 2>&1 \
  | jq -r 'select(.message?.code?.code?) | .message.code.code' \
  | sort -u \
  | grep -E '^clippy::(missing_errors_doc|missing_panics_doc|missing_safety_doc|doc_markdown|undocumented_unsafe_blocks)$'
```

Alternative coarser check: temporarily set `pedantic = { level = "allow", priority = -1 }` in the workspace clippy table and run `cargo clippy --workspace -- -D warnings` — any of the 5 lints that still fire are independently enabled (i.e., NOT pedantic-implied). Revert the temp change before commit.

`undocumented_unsafe_blocks` is **NOT** in `clippy::pedantic` per upstream lint groups — keep it explicitly. The other four (`missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`, `doc_markdown`) are in `clippy::pedantic`; if the live check confirms they fire via pedantic, drop the explicit entries.

This pre-check is informational only; it does NOT change scope. The redundancy hypothesis is per AGENTS.md *Dependency Versions* AXIOM presence dimension — verify, do not trust memory.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `rg '^#!\[deny\(missing_docs\)\]' --type rust` returns 0 hits. |
| AC2 | `rg '^#!\[deny\(rustdoc::broken_intra_doc_links\)\]' --type rust` returns 0 hits. |
| AC3 | `rg '^#!\[warn\(clippy::(missing_errors_doc\|missing_panics_doc\|missing_safety_doc\|doc_markdown\|undocumented_unsafe_blocks)\)\]' --type rust` returns 0 hits. |
| AC4 | Root `Cargo.toml` contains `[workspace.lints.rust]` with `missing_docs = "deny"`. |
| AC5 | Root `Cargo.toml` contains `[workspace.lints.rustdoc]` with `broken_intra_doc_links = "deny"`. |
| AC6 | Root `Cargo.toml` `[workspace.lints.clippy]` contains `undocumented_unsafe_blocks = "warn"` (load-bearing add, always present), plus any of the 4 doc-family lints (`missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`, `doc_markdown`) that the implementation pre-check showed are NOT pedantic-implied. The implementation commit message records which of the 4 were kept and which were dropped, with the pre-check command output as justification. |
| AC7 | Every member crate's `Cargo.toml` retains its `[lints] workspace = true` block (no regression). |
| AC8 | `cargo build --workspace` clean. |
| AC9 | `cargo clippy --workspace -- -D warnings` clean — no new warnings surface after the lift (including for `quartzite-renderer` which currently lacks `#![deny(missing_docs)]`). If new warnings appear (e.g., missing docs in renderer), they MUST be fixed in commit 2 of the same PR. |
| AC10 | `cargo test --workspace` clean (no test count drift; net delta = 0). |
| AC11 | `cargo fmt -- --check` clean. |
| AC12 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean. |
| AC13 | `cargo build -p quartzite --no-default-features --features libm` clean (derive-free / no_std-friendly path). |
| AC14 | `AGENTS.md § Linter posture` row mentions `[workspace.lints.rust]` AND `[workspace.lints.rustdoc]` (in addition to the existing `[workspace.lints.clippy]` reference). |
| AC15 | `AGENTS.md § Documentation` row no longer asserts *"every crate has `#![deny(missing_docs)]` + `#![warn(clippy::undocumented_unsafe_blocks)]`"* in its current form — replaced with workspace-level wording. |
| AC16 | `ai-docs/code-style.md § Linter posture` mirror updated (Propagation Rule for AGENTS.md ↔ code-style.md pair). |
| AC17 | `ai-docs/code-style.md § Documentation` line 135 and `§ Lints that mechanically enforce…` line 405 updated to reflect workspace-level declaration. |
| AC18 | `ai-docs/doc-convention.md § Lints that mechanically enforce…` line 539 updated similarly. |
| AC19 | Commit decomposition matches the issue's suggested plan: (1) workspace tables added + test-file `//!` additions per AC20 + gates green, (2) per-crate directives stripped + gates green + any additional warnings the strip surfaces fixed, (3) instruction-file cross-references updated. Three commits on a single feature branch. |
| AC20 | Every integration test file in the workspace (everything under `**/tests/*.rs` and any workspace-root `tests/*.rs`) has a `//!` crate-level doc comment after the lift. Equivalently: `cargo test --workspace` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` both pass with `[workspace.lints.rust] missing_docs = "deny"` in effect. The doc lines are short, descriptive of what the test file covers, and added as part of Subtask 1 (so all 6 gates pass at the Subtask 1 commit). |

## Open questions

- (none — the issue body provides a complete plan; the implementation pre-check is the only thing that needs live measurement, and AC6 documents what to do with each outcome.)

