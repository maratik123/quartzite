# Audit all workspace clippy `allow`s from #423

**Source:** issue #477
**Date:** 2026-05-18
**Tracked in:** #477

## Scope

1. Audit every workspace-level `#[allow]` entry under `[workspace.lints.clippy]` in root `Cargo.toml` that was introduced by PR #423 (`tighten-clippy-pedantic-nursery`).
2. The 15 lints in scope, grouped per the issue body's 11 audits:
   1. `cast_possible_wrap`, `cast_possible_truncation`, `cast_precision_loss` (cast family — 1 audit)
   2. `must_use_candidate`, `return_self_not_must_use` (must-use family — 1 audit)
   3. `significant_drop_in_scrutinee`, `significant_drop_tightening` (significant-drop family — 1 audit)
   4. `redundant_pub_crate`
   5. `missing_const_for_fn`
   6. `float_cmp`
   7. `items_after_statements`
   8. `option_if_let_else`
   9. `needless_pass_by_value`
   10. `type_complexity`
   11. `struct_excessive_bools`
3. For each audit, choose one outcome per the issue's decision framework:
   | Outcome | Action |
   |---|---|
   | **Keep** | Verify scope claim; update `Cargo.toml` comment with the surveyed hit count + per-crate scope; comment must name doctests if applicable. |
   | **Narrow** | Move the allow to per-crate `[lints.clippy]` (≥ 4 concentrated sites per crate) or per-fn / per-item `#[allow(clippy::lint_name, reason = "…")]` (≤ 3 sites in a crate); remove the workspace allow; full `--all-targets` clippy stays green. |
   | **Fix** | Rewrite the protected sites with the rewriting strategy named in the audit questions for that lint; remove the workspace allow; full `--all-targets` clippy stays green. |
4. Single PR carrying all 11 decisions and their fix-ups, as the issue title says ("…audit all workspace allows from #423"). Closes #443–#453.
5. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo fmt -- --check` all pass on the final commit.

## Out of scope

- New `[workspace.lints.clippy]` allow entries unrelated to PR #423 (none exist today; the workspace allow list is exactly the 15 lints above).
- The pedantic/nursery group-level `warn` setting itself; only the per-lint `allow` overrides are under audit.
- `large_stack_frames` / `large_stack_arrays` / `undocumented_unsafe_blocks` (all set `deny`, not `allow`) and the `[workspace.lints.rust]` / `[workspace.lints.rustdoc]` entries.
- The `Library safety idioms` carve-out for `.unwrap_or_else(|e| e.into_inner())` — that lives in `[workspace.lints.rust]` only conceptually; no clippy allow exists for it.

## Deferred

| What | Why | Separate issue needed? |
|---|---|---|
| Auditing project-level group settings (`pedantic = warn`, `nursery = warn`) for whether they should escalate to `deny` | Different decision class (group-level lint posture) than per-lint allow audit | No — out-of-scope per #423's framing; revisit if needed in a separate spec |
| `bitflags!` / `enumflags2` collapse of `WidgetBase` bool flags into `BitFlags<WidgetState>` | Issue #316 already tracks the design call; out of `struct_excessive_bools` audit which only decides Keep/Narrow/Fix on the allow | Not new — #316 covers it |

## Key decisions

| Question | Decision |
|---|---|
| Single PR vs split-per-lint? | Single PR. Issue title and body explicitly batch all 11 audits into one `/task` pass. |
| What scoping primitive to use when narrowing? | Per-fn / per-item `#[allow(clippy::lint, reason = "…")]` when ≤ 3 sites in a crate; per-crate `[lints.clippy]` allow when ≥ 4 concentrated sites; matches AGENTS.md *Code Style — Linter posture* "no blanket `#[allow]` without justification". Design phase decides the exact threshold per lint. |
| Reason-string requirement on every narrowed `#[allow]`? | Yes. AGENTS.md *Code Style — Linter posture* mandates justification on any `#[allow]`; reason strings carry it inline. |
| `significant_drop_*` survey baseline | Run **after** the #442 (parking_lot workspace migration) merge — already landed 2026-05-17, so this PR audits the current `parking_lot::Mutex` / `parking_lot::RwLock` codebase. |
| `float_cmp` and doctest hits | Discovery during round-1 survey: `float_cmp` triggers in `///` doctest blocks (`assert_eq!(x, 3.5)` inside `/// ```` doctests), NOT only inside `#[cfg(test)]` modules. The issue's "every hit is inside unit tests" claim is technically correct only if doctests count. Design phase chooses one of: (a) keep workspace allow with comment updated to name doctests; (b) narrow to `#[cfg(test)]` and add `# #[allow(clippy::float_cmp)]` inside the doctest blocks; (c) rewrite doctest assertions to use ε-tolerance. Spec records the discovery; the choice is design's. |
| Public API renames during fix-up | Pre-publish — AGENTS.md *API Stability* axiom applies; clean break if a rename makes the lint disappear cleanly. |
| Per-PR PR body content | Per-lint hit count + per-file distribution table; per-lint decision (Keep / Narrow / Fix) with one-line rationale; AC checklist with all six items checked. |

## Technical constraints

- Workspace lint policy lives in root `Cargo.toml` `[workspace.lints.clippy]` (matches AGENTS.md *Code Style — Linter posture*). Per-crate overrides land in each crate's `Cargo.toml [lints.clippy]` block (each crate already declares `[lints] workspace = true`).
- Clippy gate command is `cargo clippy --workspace --all-targets -- -D warnings` (matches AGENTS.md *Build & Test*); `--all-targets` ensures benches/tests/examples all participate.
- Doc gate is `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` and must remain green if `float_cmp` resolution touches `///` doctest blocks.
- No-default-features build (`cargo build -p quartzite --no-default-features --features libm`) must still compile — relevant if narrowing puts per-crate `[lints.clippy]` blocks anywhere on the `libm` derive-free path.
- `#[cfg(test)]` blocks may carry a module-scoped `#![allow(clippy::float_cmp)]` if narrowing chooses option (b); doctest blocks carry inline `# #[allow(...)]` (the `# ` prefix hides the line from rendered docs).
- Per AGENTS.md *Corrections Log* — Boundary rule 2 exception (in-flow learning capture during `/task` Steps 8–12): if survey reveals a recurring pattern worth recording (e.g. doctest sites being a third lint-surface beyond unit + integration tests), capture it as a learning entry in the same turn as the fix commit.

### Round-1 survey data (full `--all-targets`)

Hit counts after flipping every audited allow → `warn` and running `cargo clippy --workspace --all-targets`:

| Lint | Hits | Top crates / files |
|---|---|---|
| `must_use_candidate` | 170 | geometry/rect.rs (26), core/meta.rs (20), paint-api/font.rs (9), geometry/size.rs (8), paint-api/color.rs (7), events/mouse.rs (7), core/object_base.rs (7), runtime/object_ref.rs (6), geometry/point.rs (6), geometry/margins.rs (6), events/keyboard.rs (6), … |
| `redundant_pub_crate` | 60 | macros/util.rs (13), macros/object_impl/codegen.rs (6), macros/extend/parse.rs (5), macros/object_impl/parse.rs (4), runtime/global_tree.rs (3), macros/object/parse.rs (3), macros/object_impl/accumulator.rs (3), core/signal.rs (3), … (31 / 60 in `quartzite-macros/`) |
| `missing_const_for_fn` | 59 | core/meta.rs (6), macros/tests/object_impl.rs (5), events/keyboard.rs (5), core/object_base.rs (5), renderer/vello_painter.rs (4), events/mouse.rs (4), renderer/application_builder.rs (3), paint-api/font.rs (3), … |
| `float_cmp` | 51 | paint-api/color.rs (23), style/default_style_tests.rs (14), renderer/render_harness.rs (3), paint-api/font.rs (3), paint-api/pen.rs (2), geometry/size.rs (2), geometry/point.rs (2), paint-api/path.rs (1). **All in `#[cfg(test)]` modules or `///` doctest blocks** — none in plain production paths. |
| `items_after_statements` | 22 | style/style.rs (10), style/registry.rs (4), core/signal.rs (2), … |
| `significant_drop_tightening` | 22 | widgets/tests/support_internals.rs (10), runtime/object_tree.rs (4), runtime/loop_registry.rs (2), core/connect.rs (2), runtime/tests/object_tree_ext.rs (1), runtime/event_loop.rs (1) |
| `return_self_not_must_use` | 12 | geometry/rect.rs (6), paint-api/font.rs (3), … |
| `cast_possible_wrap` | 11 | widgets/layout/grid_layout.rs (6), widgets/tests/snapshots.rs (3), style/tests/snapshots.rs (2) |
| `option_if_let_else` | 8 | macros/util.rs (2), macros/extend/codegen.rs (2), runtime/object_tree.rs (1), macros/object_impl/codegen.rs (1), macros/object_impl/accumulator.rs (1), core/value.rs (1) |
| `cast_possible_truncation` | 6 | renderer/event_convert.rs (2), widgets/tests/snapshots.rs (1), renderer/wrapped_handler.rs (1), geometry/lib.rs (1), core/value.rs (1) |
| `cast_precision_loss` | 5 | geometry/size.rs (2), geometry/point.rs (2), renderer/vello_painter.rs (1) |
| `significant_drop_in_scrutinee` | 5 | runtime/timer_drivers.rs (3), runtime/tests/timer.rs (1), runtime/connection_table.rs (1) |
| `type_complexity` | 5 | runtime/object_tree.rs (3), renderer/wrapped_handler.rs (2). **Note:** issue body cited `connect.rs` and `factory.rs` but the surveyed sites are `object_tree.rs` / `wrapped_handler.rs` — the comment in `Cargo.toml` for `type_complexity` is stale and must be rewritten regardless of outcome. |
| `needless_pass_by_value` | 3 | core/connect.rs (2), renderer/wrapped_handler.rs (1) |
| `struct_excessive_bools` | 2 | widgets/widget_base.rs (1 — `WidgetBase`, per #316), macros/object/parse.rs (1) |

Total: 441 warnings across the 15 audited lints. The survey was reproduced with: copy `Cargo.toml`, replace every audited `= "allow"` with `= "warn"`, run `cargo clippy --workspace --all-targets 2>&1`, parse warning URLs `rust-clippy/…/index.html#<lint_name>`.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | All 11 survey recipes from the issue body have been executed and per-lint hit counts + per-file distribution are recorded in the PR body. |
| AC2 | The `significant_drop_*` audit was performed against the post-#442 (parking_lot) codebase. |
| AC3 | A decision (Keep / Narrow / Fix) is documented for each of the 11 audits, with a one-line rationale tied to the surveyed data. |
| AC4 | For every **Narrow** or **Fix** decision: the workspace-level allow line is removed from root `Cargo.toml [workspace.lints.clippy]`, and `cargo clippy --workspace --all-targets -- -D warnings` exits 0. |
| AC5 | For every **Keep** decision: the comment immediately above the allow line in `Cargo.toml` is updated to record the surveyed hit count + scope summary; doctests are explicitly mentioned for `float_cmp` if Keep is chosen there. |
| AC6 | Stale `Cargo.toml` comments are corrected — at minimum the `type_complexity` comment which still cites `connect.rs`/`factory.rs` though surveys hit `object_tree.rs` / `wrapped_handler.rs`. |
| AC7 | Every narrowed `#[allow(clippy::…)]` carries a `reason = "…"` string per AGENTS.md *Code Style — Linter posture*. |
| AC8 | `cargo test --workspace` passes on the final commit. |
| AC9 | `cargo fmt -- --check` passes on the final commit. |
| AC10 | `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` passes on the final commit (relevant if `float_cmp` resolution touches doctest blocks). |
| AC11 | `cargo build -p quartzite --no-default-features --features libm` passes on the final commit (verifies no-default-features path is not broken by per-crate `[lints.clippy]` additions). |
| AC12 | PR closes #443, #444, #445, #446, #447, #448, #449, #450, #451, #452, #453 via the PR body's `Closes …` line. |

## Open questions

- **Doctest-`float_cmp` narrowing strategy.** Design phase chooses among: (a) Keep with comment update; (b) `#[cfg(test)]` narrowing + per-doctest `# #[allow]`; (c) ε-tolerance rewrite of doctest assertions. Defensible default exists for each — design agent picks based on hit-rewrite cost vs reader value. Not asked because all three options are within the design space and the issue explicitly allows that judgement.
- **`significant_drop_tightening` test-code concentration.** 10 / 22 hits live in `quartzite-widgets/tests/support_internals.rs` (a test helper), where MutexGuard atomicity is rarely load-bearing. Design phase may split the test-side hits (Fix via `drop(guard)`) from the prod-side hits (Keep — the documented atomicity rationale). Not blocking spec finalisation.
- **`needless_pass_by_value` in `connect.rs`.** Both hits are on signal/slot infrastructure functions where the value parameter is part of a stable cross-crate API surface. Per-fn `#[allow(reason = "…")]` is the likely Fix-equivalent; design phase decides whether changing the signature is preferable.
