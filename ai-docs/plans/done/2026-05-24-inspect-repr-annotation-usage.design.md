# Design: Inspect repr annotation usage

**Issue:** #558
**Date:** 2026-05-24
**Spec:** [`ai-docs/plans/2026-05-24-inspect-repr-annotation-usage.spec.md`](2026-05-24-inspect-repr-annotation-usage.spec.md)

## Approach

The spec already did the audit (10 `#[repr(...)]` sites classified) and locked the resolution per case in *Key decisions*. The design's job is to schedule the edits + the documentation rule + the verification gate, not to re-examine the audit.

**Five concrete deletions, one doc addition, one workspace verification.** The five `#[repr(i64)]` annotations on `MetaEnum`-derived enums are dead weight — `MetaEnum`'s `IntoValue::into_value` performs `self as i64` at the cast site (`quartzite-macros/src/meta_enum/codegen.rs:89`), so the source enum's repr is irrelevant to wire-format width. The `VAlignment` precedent (no `#[repr]`, identical macro use, all tests green) proves removal is safe. Stripping makes the compiler pick the tightest representation (1 byte for ≤ 256 variants), shrinking containing structs (`WidgetBase`, `ScrollArea`, future style structs) at zero source-cost.

The four `#[bitflags] #[repr(u8)]` and one `FontWeight #[repr(u16)]` annotations are load-bearing (enumflags2 macro contract / OpenType external numeric spec / `u16` width fits 100..900 where `u8` cannot) and must stay. The doc rule in AC11 codifies *why* — so the next contributor doesn't mistakenly strip them under the same banner that triggered this issue.

**Why this approach over alternatives:**

- *Stripping vs. narrowing to `#[repr(u8)]`*: Stripping wins. AGENTS.md *API Stability* AXIOM removes the ABI argument (pre-publish, no downstream clients). Compiler-chosen layout is at least as tight as `u8` for ≤ 256 variants and stays narrow under future variant additions up to 256 without source edits.
- *Single mega-edit vs. per-file tasks*: Per-file tasks isolate the verification surface — if `cargo test` regresses after one strip, the failure is co-located with one file. Reuses the spec's existing per-row inventory directly as the task list.
- *Rewriting doc-tests to match a default-repr enum*: No — `as i64` on a default-repr unit enum with non-negative discriminants compiles and produces the same value (AC10's claim, verified by reading the macro codegen). Doc-tests stay byte-identical.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Strip three `#[repr(i64)]` annotations from the widget enums file. Remove lines 15, 38, 63 (each is the single `#[repr(i64)]` line above the `pub enum` declaration for `FocusPolicy`, `SizePolicy`, `CursorShape`). Leave variants, explicit `= N` discriminants, derives, and doc-comments untouched. Verify `cargo build -p quartzite-widgets` and `cargo test -p quartzite-widgets` locally before marking complete. | `quartzite-widgets/src/enums.rs` | — |
| 2 | Strip the `#[repr(i64)]` on `ScrollPolicy` (line 18). Same recipe as task 1. Verify `cargo build -p quartzite-widgets` and `cargo test -p quartzite-widgets`. | `quartzite-widgets/src/widgets/scroll_area.rs` | — |
| 3 | Strip the `#[repr(i64)]` on `HAlignment` (line 22). Same recipe as task 1. Verify `cargo build -p quartzite-geometry` and `cargo test -p quartzite-geometry` (this also runs `discriminants_match_legacy_widget_alignment` — AC5's named regression guard). | `quartzite-geometry/src/h_alignment.rs` | — |
| 4 | Add a new `## Enum repr` section to `ai-docs/code-style.md` after `## Error types` (before `## Tracing`). The section names two reasons `#[repr(...)]` is required on an enum (`enumflags2::bitflags` macro contract requires `#[repr(uN)]`; an external numeric spec carried in discriminants — e.g. OpenType `usWeightClass`'s `u16` width for `FontWeight`) and forbids decorative use. Add a matching one-line bullet to AGENTS.md `## Code Style` referencing the new anchor `[ai-docs/code-style.md → Enum repr](ai-docs/code-style.md#enum-repr)`. Insertion point: insert the new bullet immediately between the `Error types` (line 99) and `Tracing` (line 100) bullets in the `## Code Style` section. Do not use alphabetical ordering as a rationale — the section is ordered thematically, not alphabetically. Propagation Rule fires: `AGENTS.md` ↔ `ai-docs/code-style.md` are paired here. No other instruction file references the term `#[repr` — verified during design investigation via the search step below. | `ai-docs/code-style.md`, `AGENTS.md` | — |
| 5 | Workspace-level verification gate. Run, in order: `cargo fmt -- --check`; `cargo build`; `cargo test`; `cargo clippy --workspace --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`; `cargo build -p quartzite --no-default-features --features libm`; `rg '#\[repr\(' --type rust -n` (expect exactly the five lines listed in AC9: `FontWeight u16`, `WidgetState u8`, `MouseButton u8`, `KeyModifier u8`, `PropertyFlag u8`). Each command's success closes one AC (AC4 / AC5 / AC6 / AC7 / AC8 / AC9). AC10 is implicitly closed by `cargo test` passing the unmodified doc-tests in tasks 1–3's files. AC11 is closed by reading the new `## Enum repr` section back. | (none — verification only) | 1, 2, 3, 4 |

## Handoff plan

M = 5. Two groups: 3 + 2.

- **Handoff into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` enters Group A in a fresh subagent.
- **Group A:** subtasks 1–3 — the three `#[repr(i64)]` strips. Independent of each other (no `Depends on`) and independent of task 4; can be done in any internal order within the group. Group size 3, within the 1..=3 cap.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — terminal group (2 subtasks; within the 1..=3 range). Task 4 (doc + AGENTS.md bullet) precedes task 5 (workspace verification gate) because task 5's `cargo doc` + `wc -c` consume task 4's output.

## Risks

- **`enumflags2::bitflags` group accidentally stripped.** Mitigation: spec's AC2 lists the four kept enums by name; task 4's doc rule codifies the contract; reviewer checks task 5's `rg '#\[repr\('` output against the AC9 expected-five list before APPROVE. If a stray strip lands, `cargo build` fails immediately with an `enumflags2`-macro error (compile-time tripwire).
- **`FontWeight #[repr(u16)]` accidentally stripped.** Mitigation: same AC9 grep gate; also, every `FontWeight::Variant as u16` cast in `quartzite-renderer/src/vello_painter.rs:432` and in `font.rs` unit tests would silently change result type — `cargo build` would fail on the renderer (`f32::from(font.weight() as u16)` works only when the cast target matches the repr width or fits). The unit-test cast `assert_eq!(FontWeight::Black as u16, 900)` would also fail to compile if discriminant `900` no longer fits the chosen repr.
- **`HAlignment` `discriminants_match_legacy_widget_alignment` regression.** Spec asserts no regression because default-repr unit enums with non-negative variants cast losslessly to `i64`. Mitigation: task 3 runs that exact test locally before marking complete; task 5 re-runs it at the workspace level.
- **AGENTS.md crosses the 40,000-char hard cap.** AGENTS.md is currently 36,792 chars (35k–40k warning band). A one-line `Enum repr` bullet adds ~250 chars → ~37k, still under cap. Mitigation: keep the bullet to one sentence with a single anchor link, mirroring the existing one-line bullets (`Error types`, `Tracing`). If the bullet is drafted longer than ~300 chars during task 4, trim before commit. No risk of crossing 40k from this PR alone, but the cap should be re-checked after this commit lands.
- **`MetaEnum` macro's `self as i64` cast width on default-repr enums.** Default-repr unit enums with discriminants `0..=N` (N small) get a 1-byte representation; `(1u8) as i64` widens with zero-extension and matches every explicit `= 0`, `= 1`, ... discriminant in the affected enums. Confirmed at design time via `quartzite-macros/src/meta_enum/codegen.rs:89` (the cast site). Risk surface is zero.
- **`Cargo.lock` refresh.** No dependency changes are made; `cargo build` in task 5 is still required (workspace policy — refresh `Cargo.lock` before commit even when only source changed). No mitigation needed beyond running the gate.

## Test Design

No new tests are added by this design. AC1 / AC2 / AC3 / AC11 are static-property assertions or rule additions — they have no behavioural surface to test. AC4–AC10 are gated by existing tests + standard workspace gates, executed by task 5.

Existing tests that act as regression guards (must keep passing):

- `quartzite-widgets/src/enums.rs` — doc-tests `FocusPolicy::NoFocus as i64`, `SizePolicy::Fixed as i64`, `CursorShape::Arrow as i64`. Unit tests `focus_policy_default_is_no_focus`, `size_policy_default_is_fixed`, `cursor_shape_default_is_arrow`. Closes AC5 + AC10 for the three widget enums.
- `quartzite-widgets/src/widgets/scroll_area.rs` — doc-test `assert_eq!(ScrollPolicy::default(), ScrollPolicy::AsNeeded)`. No `as i64` doc-test for `ScrollPolicy`; AC10 makes no claim about it.
- `quartzite-geometry/src/h_alignment.rs` — doc-test `HAlignment::Left as i64`, unit tests `default_is_left`, `discriminants_match_legacy_widget_alignment` (named-by-spec), `into_value_round_trip`. Closes AC5 + AC10 for `HAlignment`.
- `quartzite-paint-api/src/font.rs` — unit test `font_weight_numeric_values` (the nine `FontWeight::Variant as u16` casts at lines 298–306). Acts as the AC3 + AC9 tripwire for `FontWeight` — if its `#[repr(u16)]` were stripped, discriminant `900` would not fit and the cast would not compile.
- `quartzite-widgets/tests/re_exports.rs` — `widgets_h_alignment_is_geometry_h_alignment`, `widgets_font_weight_is_paint_api_font_weight`, etc. Indirect — these depend only on `TypeId` identity, which is unaffected by repr changes, but they exercise the touched modules.

Task 5's `cargo test` invocation runs all of the above; no per-task fixture or helper is needed.

## Open questions

- (none — spec resolved all of them in *Key decisions*)
