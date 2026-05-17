# Design: Tighten clippy — pedantic + nursery + size-aware lints

**Issue:** #423
**Date:** 2026-05-17

## Approach

Single-PR, big-bang tightening per the spec. Three mechanical layers stacked in order:

1. **Lint policy** — root `Cargo.toml` grows a `[workspace.lints.clippy]` table (the four group/lint enables) and a sibling workspace-root `clippy.toml` materialises the two 512 KiB defaults. Each member crate (13 leaves + the root `quartzite` package) gets `[lints] workspace = true`. This is the AC1/AC3/AC4 chassis.
2. **Allow-list curation** — driven by a real first-run audit of `cargo clippy --workspace --all-targets`, not by rubber-stamping the issue's *Expected allow-list*. The audit run during this design phase (with the four groups live and **zero** allows) produced **666 hits across 51 distinct lints** (URL-anchor count from the `index.html#<lint>` help lines — the authoritative per-hit signal). The allow-list is curated against that table; AC2 + AC8 are reached by either fixing the hit or adding a one-line-justified `clippy::* = "allow"`.
3. **Code fixes** — the lints we choose to honour rather than allow get fixed in three batches (mechanical-syntax, mechanical-types/docs, judgement-required). Fixes are commit-grouped so review can scan diffs by intent. Cascade rule: any fn whose body shape changes under a fix must re-evaluate its `_Simple._` / `#[inline]` marker per AGENTS.md *Code Style → `#[inline]` and the `_Simple._` doc tag* (strip + cascade re-test).

### First-run audit summary (authoritative)

The provisional clippy run was: add the four enables to root `Cargo.toml`, add `[lints] workspace = true` to every member + root, **no allow entries, no source fixes**, then `cargo clippy --workspace --all-targets`. Zero `error:` output (warnings only — would only fail under `-D warnings`). `large_stack_frames` and `large_stack_arrays` produced **0 hits** against current code with the 512 KiB defaults, so the size-aware enables are forward-protection only at v1 — no fix work, no allow needed.

The 51 distinct lints break down into three disposition buckets:

**Bucket A — workspace-wide allow (high-noise / project-style mismatch).** These contribute ~450 of the 666 hits and collapse into a small allow-list:

| Lint | Hits | Group | Justification (one-line, to be copied verbatim into the allow comment) |
|---|---|---|---|
| `must_use_candidate` | 170 | pedantic | Project does not treat omitted-return-value as a bug; opting in per-fn is noise relative to signal |
| `redundant_pub_crate` | 61 | nursery | Project convention: keep explicit `pub(crate)` even when the module is private — read-locally cheaper than re-deriving visibility |
| `missing_const_for_fn` | 58 | nursery | Nursery lint with churn; `const fn` proliferation is a design call we don't want lint-driven |
| `float_cmp` | 49 | pedantic | Every hit is `assert_eq!(…, <exact f32/f64 literal>)` inside unit tests on representable values; allow workspace-wide rather than per-assert |
| `items_after_statements` | 14 | pedantic | Project pattern: small nested helper fn placed after local setup is more readable than hoisting to module scope |
| `return_self_not_must_use` | 12 | pedantic | Same family as `must_use_candidate`; same justification (`must_use` is opt-in here) |
| `cast_possible_wrap` | 11 | pedantic | Geometry crate intentionally crosses i32↔u32 with design-bounded ranges; per-cast `try_from` would obscure intent |
| `option_if_let_else` | 8 | nursery | `map_or_else` often hurts readability when the closures aren't trivial; case-by-case judgement during fix batch keeps the worth-fixing ones |
| `needless_pass_by_value` | 8 | pedantic | Allowed at workspace level **but** with the macro `codegen(ir: IR)` family fixed in code per Subtask 5 (see below). Allow is for genuine-move sites elsewhere. |
| `cast_possible_truncation` | 6 | pedantic | Geometry & rendering deliberately truncate within known bounds; per-call `try_from` adds noise without catching real bugs |
| `cast_precision_loss` | 5 | pedantic | i32 → f32 widening in geometry is intentional (sub-pixel coords); precision loss is the API contract |
| `significant_drop_in_scrutinee` | 5 | nursery | Project pattern: `match guard { … }` where `guard` is a `MutexGuard`; tightening here harms readability and the lock scope is already deliberately small |
| `significant_drop_tightening` | 22 | nursery | Same pattern as above — `MutexGuard`s are deliberately held to keep critical sections atomic; tightening would split the section |
| `type_complexity` | 5 | pedantic | Complex generic types in `connect.rs` and `factory.rs` are intentional API surface (signal/slot family); extraction would couple unrelated callers |
| `struct_excessive_bools` | 2 | pedantic | Flags struct with 3+ bools; existing call sites are config-style — bools are clearer than an enum at v1 |
| `doc_link_code` | 2 | pedantic | Stylistic; project doc convention currently does not normalise `[`code`]` link form, no value adds from forcing it |

Allow table delta vs the spec's *Expected allow-list* hypothesis (verbatim from spec/Key decisions): the hypothesis is broadly correct. Divergences:

- The hypothesis did not enumerate `cast_possible_truncation`, `cast_precision_loss`, `cast_possible_wrap`, `significant_drop_tightening`, `significant_drop_in_scrutinee`, `option_if_let_else`, `doc_link_code`, `struct_excessive_bools`, `items_after_statements`, `return_self_not_must_use`, `redundant_pub_crate`, or `missing_const_for_fn`. The first-run audit added all of them with the justifications above.
- The hypothesis allowed `needless_pass_by_value` outright; design narrows this to "workspace-allow + fix the macro `codegen(ir: IR)` family in code" because those four call sites are genuinely better with `&IR` (no clone needed, smaller monomorphisations).

**Bucket B — mechanical-fix lints (subtask 3, batch A — ~110 hits).** Pure mechanical refactors clippy can almost auto-apply:

| Lint | Hits | Shape of fix |
|---|---|---|
| `use_self` | 81 | Replace `StructName` with `Self` inside `impl StructName { … }` and inside `impl Trait for StructName` return positions |
| `default_constructed_unit_structs` | 10 | `Foo {}` / `Foo::default()` for unit structs → `Foo` |
| `ignored_unit_patterns` | 13 | `Ok(_) => …` where the inner type is `()` → `Ok(()) => …` |
| `manual_let_else` | 7 | `if let X = y { … } else { … }` (where the else returns/diverges) → `let X = y else { … };` |
| `redundant_closure_for_method_calls` | 6 | `|x| x.foo()` → `Foo::foo` (where lifetimes allow) |
| `semicolon_if_nothing_returned` | 2 | Add trailing `;` on unit-returning final statement |
| `uninlined_format_args` | 3 | `format!("{}", x)` → `format!("{x}")` |
| `needless_continue` | 1 | Drop trailing `continue;` |
| `needless_for_each` | 2 | `.for_each(|…| body)` → `for … { body }` |
| `needless_return` | 1 | Drop trailing `return expr;` |
| `collapsible_if` | 1 | Merge nested `if let` / `if` |
| `single_match_else` | 1 | `match x { A => …, _ => … }` → `if let A = x { … } else { … }` |
| `manual_string_new` | 1 | `String::from("")` → `String::new()` |
| `manual_midpoint` | 1 | `(a + b) / 2` → `a.midpoint(b)` |
| `explicit_iter_loop` | 1 | `for x in c.iter() {` → `for x in &c {` |
| `explicit_into_iter_loop` | 2 | `for x in c.into_iter() {` → `for x in c {` |

**Bucket C — mechanical-fix lints (subtask 4, batch B — ~95 hits).** Same mechanicality but a different conceptual axis (widening casts + doc backticks + smaller groups):

| Lint | Hits | Shape of fix |
|---|---|---|
| `cast_lossless` | 53 | `x as i64` (where `x: i32`) → `i64::from(x)` (widening only — no truncating casts here) |
| `doc_markdown` | 22 | Add backticks around bare type / fn / variant identifiers in `///` / `//!` doc comments |
| `no_effect_underscore_binding` | 8 | `let _x = expr;` → `let _ = expr;` (intent: drop result) or rename if the binding is later used |
| `ref_as_ptr` | 3 | `&foo as *const _` → `std::ptr::from_ref(&foo)` (or `from_mut`) |
| `borrow_as_ptr` | 3 | Same family as `ref_as_ptr` |
| `default_trait_access` | 3 | `Default::default()` → `Foo::default()` at the call site |
| `match_same_arms` | 3 | Collapse identical arms with `|`-pattern |
| `manual_assert` | 2 | `if cond { panic!("...") }` → `assert!(!cond, "...")` |
| `elidable_lifetime_names` | 2 | Drop named lifetime where rustc elision suffices |
| `derive_partial_eq_without_eq` | 2 | Add `Eq` to `#[derive(PartialEq, ...)]` where all fields impl `Eq` |
| `too_long_first_doc_paragraph` | 2 | Split overlong first paragraph in `//!` / `///` |
| `needless_type_cast` | 2 | Drop redundant `as Type` |
| `branches_sharing_code` | 2 | Hoist shared expressions out of if/else arms |
| `struct_field_names` | 1 | Rename field that repeats its struct name |
| `redundant_clone` | 1 | Drop redundant `.clone()` |
| `map_unwrap_or` | 1 | `.map(…).unwrap_or(…)` → `.map_or(…, …)` |
| `or_fun_call` | 1 | `.unwrap_or(expensive())` → `.unwrap_or_else(\|\| expensive())` |
| `unused_self` | 1 | Convert `&self` method to associated fn (or take `&self` use) |
| `too_many_lines` | 1 | Refactor `quartzite-renderer` 118-line fn into helper(s) — judgement call; defer to allow if extraction is artificial |

**Bucket D — judgement-required (subtask 5, ~25 hits).** Need per-call inspection; either fix-with-judgement or extend the allow-list:

| Lint | Hits | Notes |
|---|---|---|
| `needless_pass_by_value` (macro `codegen(ir: IR)` family) | 4 of the 8 | Fix in code: change to `&IR` (smaller monomorphisations, no clone) |
| `similar_names` | 5 | `bare` vs `base` in extend codegen; rename `bare` → `bare_generics_idents` or similar |
| `option_if_let_else` (selective) | 0–8 fixed, rest allowed | Per-site judgement; allow line stays for the rest |

### Rejected alternatives

- **Phased rollout (warn first, deny later).** Rejected because pre-publish + no downstream consumers, per spec *Key decisions* and AGENTS.md *API Stability*.
- **Per-crate allow-list (different allow set per crate).** Rejected per spec *Out of scope*; v1 ships workspace-level uniformity, per-crate refinement is a follow-up.
- **Enabling `clippy::restriction`.** Out of scope per spec.
- **Tightening size-aware thresholds below 512 KiB defaults.** Out of scope per spec; deferred follow-up.
- **Auto-apply via `cargo clippy --fix`.** Rejected because the mix of fix vs allow is itself a design decision per lint; bulk auto-apply would convert allowable-by-comment cases into mechanical changes that obscure intent. Manual per-lint application keeps the audit trail.
- **Splitting into multiple PRs.** Considered — the M ≤ 7 task count fits a single PR, and per-PR atomicity of the allow-list (you need the allow-list and the fixes landed together to keep `-D warnings` green) makes a phased split actively harmful.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `[workspace.lints.clippy]` table (4 enables, `priority = -1` on the two groups), create workspace-root `clippy.toml` with the two `*-size-threshold = 524288` entries, add `[lints] workspace = true` to all 13 leaf crate `Cargo.toml` files **and** the root `quartzite` package. No allow entries yet; no source fixes. (Implements AC1, AC3, AC4. Tree fails `cargo clippy --workspace -- -D warnings` at this point — that's expected; subtasks 2–6 close it.) | `Cargo.toml`, `clippy.toml`, all 13 `quartzite-*/Cargo.toml` | — |
| 2 | Append the Bucket A allow-list (16 `clippy::* = "allow"` entries) to `[workspace.lints.clippy]`, each preceded by a `#`-comment with the justification from the audit table above. Re-run `cargo clippy --workspace --all-targets` and confirm hit count drops from ~666 → ~210 (i.e., only Bucket B+C+D hits remain). | `Cargo.toml` | 1 |
| 3 | Apply Bucket B mechanical fixes (batch A — ~110 hits). Touch every file flagged by `use_self`, `default_constructed_unit_structs`, `ignored_unit_patterns`, `manual_let_else`, `redundant_closure_for_method_calls`, `semicolon_if_nothing_returned`, `uninlined_format_args`, `needless_continue`, `needless_for_each`, `needless_return`, `collapsible_if`, `single_match_else`, `manual_string_new`, `manual_midpoint`, `explicit_iter_loop`, `explicit_into_iter_loop`. After this subtask, run a localised clippy check to confirm Bucket B is empty. | Files identified by audit; concentrated in `quartzite-renderer/src/vello_painter.rs`, `quartzite-core/src/{value,signal,meta,object_base}.rs`, `quartzite-runtime/src/object_tree.rs`, `quartzite-paint-api/src/{color,font}.rs`, `quartzite-geometry/src/{rect,size,point}.rs`, `quartzite-style/src/*`, `quartzite-widgets/src/layout/grid_layout.rs`, `quartzite-events/src/{mouse,keyboard}.rs`, `quartzite-style-types/src/color_role.rs` | 2 |
| 4 | Apply Bucket C mechanical fixes (batch B — ~95 hits): `cast_lossless` widening casts to `From::from`, `doc_markdown` backtick additions, `no_effect_underscore_binding`, `ref_as_ptr` / `borrow_as_ptr` → `ptr::from_ref` / `ptr::from_mut`, `default_trait_access`, `match_same_arms` arm collapse, `manual_assert`, `elidable_lifetime_names`, `derive_partial_eq_without_eq` (add `Eq`), `too_long_first_doc_paragraph` split, `needless_type_cast` removal, `branches_sharing_code` hoist, `struct_field_names` rename, `redundant_clone`, `map_unwrap_or`, `or_fun_call`, `unused_self`, and `too_many_lines` (refactor or allow). Doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) must still pass after the doc-touching subset. | Files identified by audit; concentrated in `quartzite-core/src/value.rs` (`cast_lossless` cluster on `From<…> for Value`), `quartzite-paint-api/src/color.rs`, `quartzite-renderer/src/vello_painter.rs`, plus `///`-comment edits across all crates | 3 |
| 5 | Apply Bucket D judgement fixes: change `pub(crate) fn codegen(ir: <IR>) -> TokenStream` to `&<IR>` in `quartzite-macros/src/{extend,meta_enum,object,object_impl}/codegen.rs` (4 sites); rename `bare` → `bare_generics_idents` (or similar) in `quartzite-macros/src/extend/codegen.rs` to silence `similar_names`; selectively rewrite `option_if_let_else` hits where `map_or` / `map_or_else` improves readability, leave the rest under the workspace allow. **Cascade re-test:** for every fn touched in subtasks 3, 4, 5 — if the marker is `#[inline]` (concrete fn / concrete-impl trait method) or `_Simple._` (generic / trait declaration / generic-impl trait method) and the body shape changed, re-evaluate against the `_Simple._` definition (recursively-simple: no branches/loops, ≤ 1 non-simple call). Strip the marker if no longer simple; if the strip cascades into a caller becoming non-simple, walk the chain. | `quartzite-macros/src/extend/codegen.rs`, `quartzite-macros/src/meta_enum/codegen.rs`, `quartzite-macros/src/object/codegen.rs`, `quartzite-macros/src/object_impl/codegen.rs`, audit-identified `option_if_let_else` sites | 4 |
| 6 | Close the allow-list: any AC8-class hit that surfaced and can't be cleanly fixed gets added to the allow-list with a justification comment (and enumerated in the PR-body draft for reviewer judgement); confirm `cargo clippy --workspace -- -D warnings` is green; run the full AGENTS.md gate set: `cargo build`, `cargo test --workspace`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`. (Implements AC2 final form, AC5, AC6, AC8, AC9.) | `Cargo.toml` (final allow tweaks if any) | 5 |
| 7 | Update `AGENTS.md` *Code Style → Linter posture* row to mention `[workspace.lints.clippy]` as the location of workspace-wide policy and `clippy.toml` as the location of size thresholds. Mirror the same change into `ai-docs/code-style.md § Linter posture`. Per AGENTS.md *Propagation Rule*: `AGENTS.md` ↔ `ai-docs/code-style.md` is implicit in the existing posture row — both must be updated in the same PR. No other rule files need touching (the Linter posture row only appears in these two). (Implements AC7.) | `AGENTS.md`, `ai-docs/code-style.md` | 6 |

## Handoff plan

M = 7. Group sizes: 3, 3, 1 (terminal). All non-terminal groups are exactly 3 subtasks; terminal group is 1 (within `1..=3`).

- **Entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` enters Group A with fresh context.
- **Group A:** subtasks 1–3 — establish lint policy + allow-list + mechanical batch A (the high-leverage chunk that drops the warning count from ~666 to ~115 hits).
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–6 — mechanical batch B + judgement fixes + close the allow-list and run the full verification gate (closes AC5, AC6, AC8, AC9).
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group C with fresh context.
- **Group C:** subtask 7 — terminal group (1 subtask; within the 1..=3 range). Update `AGENTS.md` + `ai-docs/code-style.md` Linter-posture rows. The single group completes Step 8 in its own `/context-reset` subagent.

## Risks

- **Risk: the audit numbers shift between design-phase run and implementation run.** Toolchain updates, new code merged to master between design and implementation, or fixes in one subtask creating/removing hits in another can shift counts. **Mitigation:** the design lists *categories* (which lints are mechanical-fix vs allow) rather than committing to exact hit counts; the implementation agent re-runs `cargo clippy --workspace --all-targets` after each subtask and adjusts. The Bucket B/C/D **lint-list contract** is normative; the per-lint **hit counts** in this design are observational.
- **Risk: a lint we put in Bucket B turns out to require source rewrites the audit didn't anticipate (e.g., `use_self` blocked by a trait-impl elision rule).** **Mitigation:** subtask 5 (judgement bucket) is the catch-all; if a Bucket B lint hits a wall, the call site goes into the Bucket D allow-list with a comment and surfaces in the AC8 PR-body enumeration.
- **Risk: `_Simple._` cascade re-test is missed for a fn that subtask 3 or 4 reshapes.** **Mitigation:** subtask 5 owns the cascade re-test explicitly; the file list it touches is a superset of any file touched by subtasks 3/4 that hosts a marked-simple fn. Cross-check via `rg '#\[inline\]|_Simple\._' quartzite-*/src` ∩ files-touched-by-subtasks-3-4.
- **Risk: `[workspace.lints]` doesn't flow through `cargo clippy --workspace -- -D warnings` as expected.** **Mitigation:** Cargo 1.74+ contract is well-documented; workspace MSRV is 1.95 (verified in root `Cargo.toml` `[workspace.package]`); subtask 1 ends with a smoke check that `cargo clippy --workspace` reports lints from the new policy (it should — the design-phase audit already proved this).
- **Risk: the `no_effect_underscore_binding` fixes break a deliberate "hold this guard / temp" pattern.** **Mitigation:** subtask 4 inspects each hit; `_x` bindings are sometimes deliberate (RAII guard alive until end of scope). If so, rename `_x` → `_keep_alive_x` (still starts with `_` but the name signals intent) — clippy allows underscore-prefixed names whose suffix has a non-empty meaning.
- **Risk: `cast_lossless` "fix" inadvertently widens a value the API meant to truncate.** **Mitigation:** the audit only flags widening casts (the lint is by design widening-only); `cast_possible_truncation` is in the allow-list separately, so this risk class is bounded by the lint's own semantics.
- **Risk: subtask 7 docs update bleeds wording into other AGENTS.md rows.** **Mitigation:** the change is constrained to the single bullet for "Linter posture" in AGENTS.md and the single section in `ai-docs/code-style.md`. Both edits are wording-only — they do not change the existing `-D warnings` requirement or the no-blanket-`#[allow]` rule.
- **Risk: pedantic-fix changes a fn body and triggers an `actionlint` re-run requirement.** **Mitigation:** N/A — no `.github/workflows/*.yml` is touched (spec *Technical constraints*).

## Test Design

The work is policy + mechanical refactor. No new behaviour; no new logic. The "test plan" is therefore the full AGENTS.md gate set (run in subtask 6, repeated in subtask 7 if AGENTS.md / code-style edits trigger any doc-link regeneration):

- **Build:** `cargo build` — refreshes `Cargo.lock`.
- **Test:** `cargo test --workspace` — every existing unit + integration test still passes. No test file is touched except where a clippy fix lands inside `#[cfg(test)]` (e.g., `use_self` inside test impls).
- **Lint (the gate this PR defends):** `cargo clippy --workspace -- -D warnings` — must be clean against the post-PR tree. This is AC5.
- **Format:** `cargo fmt -- --check` — pedantic fixes can introduce 100-col overruns (e.g., `i64::from(x)` is longer than `x as i64`); `cargo fmt` resolves; the `--check` form is the gate.
- **Doc gate:** `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` — the `doc_markdown` fixes in subtask 4 only add backticks; they don't change link targets, so `rustdoc::broken_intra_doc_links` stays clean. Sanity-checked by this gate.
- **No-default-features path:** `cargo build -p quartzite --no-default-features --features libm` — AC9; verifies the derive-free / `no_std` path didn't regress under the new lint table. Lints are diagnostic-only; this build is a compile-only gate, so the only way it can break is if a Bucket B/C/D fix accidentally introduced a `std`-dependent construct in a `no_std`-feature-gated module.
- **Cascade re-test (subtask 5):** for each `#[inline]` / `_Simple._` marker on a fn whose body shape changed, re-derive whether the fn is still "recursively-simple: no branches/loops, ≤ 1 non-simple call". Strip the marker if it isn't; cascade to callers per AGENTS.md *Code Style → `#[inline]` and the `_Simple._` doc tag*.

No new `#[cfg(test)] mod tests` blocks needed — none of the subtasks introduces new logic-bearing code.

## Open questions

- _(none — the spec is self-contained, the audit is done, allow-list is curated against real data, and AC8 explicitly carries the escape hatch for any un-fixable / un-justifiable case that surfaces during implementation.)_
