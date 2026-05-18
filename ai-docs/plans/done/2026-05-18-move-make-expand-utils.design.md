# Design: Move `make_expand!` macro to `util.rs`

**Issue:** #457
**Date:** 2026-05-18

## Approach

Move the `macro_rules! make_expand` definition from `quartzite-macros/src/lib.rs` (lines 14–23) into the existing `quartzite-macros/src/util.rs` file. Replace the current root-level `pub(crate) use make_expand;` (which exports a macro defined in the same scope) with a root-level `pub(crate) use util::make_expand;` re-export so that the path `crate::make_expand` continues to resolve at the three call sites (`extend/mod.rs:4`, `meta_enum/mod.rs:4`, `object/mod.rs:4`) without any edit to those files.

**Why this shape (Option A from the spec):**

- `macro_rules!` macros declared inside a module are by default private to that module. Rust 2018+ lets `use` statements bring macros into scope just like other items, so `pub(crate) use util::make_expand;` at the crate root re-exports the macro at `crate::make_expand` with the same `pub(crate)` visibility it has today.
- This keeps the macro's *definition* co-located with the other internal helpers in `util.rs` (which already hosts `crate_root`, `as_trait_name`, `inline_if_concrete`, `emit_compile_error`, etc. — the macro fits the same "internal codegen helpers" theme), while the *re-export* stays at the crate root so the existing `crate::make_expand!()` path is preserved.
- `lib.rs` already contains `mod util;` (line 26), so no module declaration changes are needed.

**Rejected alternatives:**

- **Option B (spec): module-level `pub(crate) use make_expand;` inside `util` + nothing at root.** This would expose the macro as `crate::util::make_expand`, not `crate::make_expand`. The three call sites would have to change to `crate::util::make_expand!()`, violating AC3 / spec scope item 2 ("call sites stay unchanged"). Rejected.
- **`#[macro_export]` on the definition.** Would publish the macro at the crate root externally (effectively `pub`), violating the spec's "Visibility — Remain `pub(crate)`. No external consumers." decision. Rejected.
- **Leave the definition at the root and just move call-site infrastructure.** Defeats the purpose of the move. Rejected.
- **Rename `util.rs` to `utils.rs` to match the issue title literally.** Explicitly out of scope per the spec. Rejected.

## Decomposition

| # | Task                                                                                                                                                                                                                                                                                          | Files                                       | Depends on |
|---|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------|------------|
| 1 | In `quartzite-macros/src/util.rs`, append the `macro_rules! make_expand { ... }` block (verbatim from `lib.rs` lines 14–23) below the existing items and above the `#[cfg(test)] mod tests` block. The macro body remains unchanged (`pub(crate) fn expand(...) { match parse::parse(...) ... }`). | `quartzite-macros/src/util.rs`              | —          |
| 2 | In `quartzite-macros/src/lib.rs`, delete the `macro_rules! make_expand { ... }` block (lines 14–23) and replace the existing `pub(crate) use make_expand;` (line 24) with `pub(crate) use util::make_expand;`. The new re-export must appear after `mod util;` (line 26) so the `util` module path resolves — move both lines so the order is `mod util;` then `pub(crate) use util::make_expand;`. | `quartzite-macros/src/lib.rs`               | 1          |
| 3 | Run the verification gate: `cargo build`, `cargo test -p quartzite-macros`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`. Also `grep -n 'macro_rules! make_expand' quartzite-macros/src/lib.rs` MUST return no matches (AC1) and `grep -n 'macro_rules! make_expand' quartzite-macros/src/util.rs` MUST return exactly one (AC2). The three call sites — verify by `grep -rn 'crate::make_expand!' quartzite-macros/src/` — must still appear and still resolve (compile + test pass = AC3, AC4, AC5). | (verification only — no file edits)         | 2          |

(3 atomic tasks, well under the 7-task split threshold.)

## Handoff plan

`M = 3` — one group, terminal.

- **Group A:** subtasks 1–3 — terminal group (3 subtasks; at the cap, within the 1..=3 range). No handoff between groups; the single group completes Step 8 in its own `/context-reset` subagent per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).

## Risks

- **Macro path resolution silently breaks at the call sites.** The whole task hinges on `pub(crate) use util::make_expand;` re-publishing the macro at `crate::make_expand`. Mitigation: subtask 3's `cargo build` covers this — if the re-export shape is wrong, `extend/mod.rs:4`, `meta_enum/mod.rs:4`, and `object/mod.rs:4` will fail to expand the macro and the build will error with `cannot find macro 'make_expand' in this scope`. The test suite (which exercises every derive) doubles as a runtime check that the generated `expand` fn still exists in each module.
- **`use` ordering vs `mod` declaration.** `pub(crate) use util::make_expand;` MUST appear *after* `mod util;` so the path resolves. Subtask 2's ordering note covers this; the build will fail loudly with `unresolved import 'util'` if violated.
- **Clippy lint creep.** The macro currently lives at the crate root where rustfmt may treat it differently from items inside `util.rs` (which already contains the larger `tests` block and helper fns). Mitigation: `cargo fmt` (not `cargo fmt -- --check`) is acceptable to apply before commit; clippy is run with `-- -D warnings` per AGENTS.md.
- **Visibility regression.** The macro must remain `pub(crate)` (decision in spec). Option A preserves this — `pub(crate) use util::make_expand;` re-exports with `pub(crate)` visibility. The alternative `#[macro_export]` would silently widen visibility to `pub`; explicitly rejected above.
- **No behaviour change to the macro body.** The `expand` fn it generates is byte-identical pre- and post-move. Existing tests in `extend/`, `meta_enum/`, `object/`, and the trybuild compile-fail/pass suites cover the generated code paths transitively; no new tests are required by this move.

## Test Design

This task is a pure code-motion refactor — no new logic. The acceptance gate is the existing test suite plus the documented `grep` invariants.

- **Location:** existing `quartzite-macros/tests/` integration tests + `cargo test -p quartzite-macros` exercises every macro the moved `make_expand!` instantiates. No new test file.
- **Entry points:** `derive_extend`, `derive_object`, `object_part`, `object_impl`, `derive_meta_enum` — every public proc-macro in `lib.rs` calls a module-local `expand` fn produced by `make_expand!()`, so any regression in macro resolution or generated-fn signature surfaces as a compile error or test failure in the existing suite.
- **Scenarios:**
  - **Happy path** — `cargo build` and `cargo test -p quartzite-macros` pass; trybuild compile-pass cases in `quartzite-macros/tests/` continue to expand.
  - **Compile-fail surface** — the existing trybuild compile-fail cases still produce the same error messages (the `expand` fn's `Err(e) => e.to_compile_error()` branch is byte-identical).
  - **No new edge cases** — the move adds zero new control flow.
- **Fixtures / helpers:** none new. Existing `quartzite-macros/tests/` fixtures cover the surface.
- **Static invariants (grep, run during subtask 3):**
  - `grep -n 'macro_rules! make_expand' quartzite-macros/src/lib.rs` → no matches (AC1).
  - `grep -n 'macro_rules! make_expand' quartzite-macros/src/util.rs` → exactly one match (AC2).
  - `grep -rn 'crate::make_expand!' quartzite-macros/src/` → exactly three matches, one per call site (AC3, unchanged).

The `util.rs` `#[cfg(test)] mod tests` block already exists and covers the helper fns; the moved `make_expand!` macro does not need its own unit test because (a) it is exercised end-to-end by every proc-macro test, and (b) `macro_rules!` macros that contain only a single fn definition with no input parameters have no meaningful unit-test surface beyond "does it expand and compile" — covered by `cargo build`.

## Open questions

(none — spec's open questions section is empty and the design choice between Option A vs Option B is settled above by the AC3 constraint)
