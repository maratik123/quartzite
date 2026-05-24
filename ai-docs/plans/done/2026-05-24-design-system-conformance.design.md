# Design: Design-system conformance audit + vertical-alignment paint-API axis

**Spec:** `ai-docs/plans/2026-05-24-design-system-conformance.spec.md`
**Issue:** #555
**Date:** 2026-05-24

## Approach

The spec ships three artefacts on a single feature branch (per K2):

0. **Paint-API extension** — `Painter::draw_text_in` gains a second alignment parameter for the vertical axis.
1. **Audit doc** `ai-docs/design-conformance-audit.md` (one section per paintable widget, one row per design-system rule).
2. **Conformance fixes** on every `❌` row the audit surfaces (at minimum: `Button` vertical centring per AC2, `Label` vertical centring per AC3) + regenerated goldens.

### K5 — chosen API shape: split `Alignment` into `HAlignment` + `VAlignment` *(Design Amendment 2 — PR #556 reviewer request)*

The original K5 resolution (reuse `Alignment` for both axes) was superseded: the reviewer noted that a single shared enum gives confusing semantics (`Alignment::Left` means "top" on the vertical axis) and required a runtime `debug_assert!` to guard `Justify` rather than rejecting it at the type level.

**`HAlignment`** — rename of the existing `Alignment` enum in `quartzite-geometry/src/alignment.rs`:

```rust
pub enum HAlignment { Left, Center, Right, Justify }
```

**`VAlignment`** — new enum in `quartzite-geometry/src/v_alignment.rs`:

```rust
#[derive(MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum VAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}
```

The derive set mirrors `HAlignment`'s (`MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default`) — `MetaEnum` is included for symmetry with `HAlignment` (the derive is already used workspace-wide for both enums; dropping it from `VAlignment` would be asymmetric). Neither enum derives `Hash`. `#[default] Top` is the natural top-anchor default for the vertical axis (matches TextEdit's G5 top-anchor and the spec's K3 vertical-centre wins only when an explicit `VAlignment::Center` is passed).

`VAlignment` has no `Justify` variant — `Justify` is meaningless on the vertical axis and the type system now enforces this at compile time, eliminating the need for any `debug_assert!`.

The updated signature:

```rust
fn draw_text_in(
    &mut self,
    rect: Rect,
    text: &str,
    font: &Font,
    brush: &Brush,
    h_align: HAlignment,
    v_align: VAlignment,
);
```

**Migration scope:** rename `Alignment` → `HAlignment` everywhere (clean break per AGENTS.md § *API Stability*); add `VAlignment`; update `Label.alignment: Alignment` → `Label.alignment: HAlignment`; remove the `Alignment::Justify`-on-vertical `debug_assert!` and the associated `#[should_panic]` test (both are obsolete — the type system now catches it).

### `draw_text_in` trait rustdoc — parameter-order note

The trait rustdoc for `Painter::draw_text_in` MUST include the parameter-order contract as **free-form prose in the doc comment body** (NOT as a non-canonical `# Parameter order` section — only the canonical section headings `# Parameters`, `# Returns`, `# Errors`, `# Panics`, `# Safety`, `# Examples`, `# See also` are permitted per `ai-docs/doc-convention.md`). The prose appears before the `# Examples` block:

> **Parameter order:** `h_align` always precedes `v_align`; call sites must rely on positional order. `HAlignment` governs the horizontal axis; `VAlignment` governs the vertical axis — they are distinct types so the compiler catches transposed arguments.

This prose is reproduced verbatim in the rustdoc for `RecordingPainter::draw_text_in` and `NullPainter::draw_text_in` so the contract is visible at every reading entry point. (Design Amendment 1: original design prescribed `# Parameter order` — corrected to canonical free-form prose per self-review finding #1. Design Amendment 2: types changed from `Alignment×2` to `HAlignment + VAlignment`.)

### `VelloPainter` vertical-axis implementation recipe

`VelloPainter::draw_text_in` currently sets glyph `py = rect.top() * scale` unconditionally. After the API change:

1. Build the layout exactly as today (parley's `layout.align(parley_h_align, …)` continues to handle the horizontal axis; `h_align: HAlignment` maps to parley's `Alignment` the same way `Alignment` did before — `HAlignment` is the same enum, just renamed).
2. Compute the total laid-out text height: `layout.height()`.
3. Translate `py` depending on `v_align: VAlignment`:
   - `VAlignment::Top` → `py = rect.top() * scale` (unchanged)
   - `VAlignment::Center` → `py = rect.top() * scale + (rect.height() * scale - layout.height()) / 2.0`
   - `VAlignment::Bottom` → `py = rect.top() * scale + (rect.height() * scale - layout.height())`
4. No `Justify` arm is needed — `VAlignment` has no `Justify` variant; the compiler enforces this.

Pixel-snap `py` via `py.round()` after the v_align translation (before `emit_layout_glyphs`) to avoid sub-pixel baseline drift per the workspace's "integer-pixel geometry" rule.

### Caller-side updates in `default_style/`

- **`Button`** (`default_style/mod.rs:237`) → `h_align = HAlignment::Center, v_align = VAlignment::Center`
- **`Label`** (`default_style/mod.rs:277`) → `h_align = w.alignment, v_align = VAlignment::Center` (K3; `w.alignment` is now `HAlignment`)
- **`LineEdit`** (`default_style/line_edit.rs:84 + 205`) → replace the PR #554 smaller-rect recipe with `painter.draw_text_in(geom, …, HAlignment::Left, VAlignment::Center)` (K8 — remove in same PR). The `text_carets` `line_height` query remains for `paint_selection_line_edit` / `paint_caret_line_edit` (those helpers compute their own vertical positions for bands and carets — not dependent on the main-text-draw's smaller rect).
- **`TextEdit`** (`default_style/text_edit.rs:56 + 298`) → `h_align = HAlignment::Left, v_align = VAlignment::Top` (explicit top-anchor, preserves G5).

### Every `Painter` impl that must update in lockstep

| File | Notes |
|---|---|
| `quartzite-geometry/src/alignment.rs` | Rename enum `Alignment` → `HAlignment`; update module doc, `# Examples`, and `#[cfg(test)]`; remove `Justify`-on-vertical doc paragraph (type system now enforces this) |
| `quartzite-geometry/src/v_alignment.rs` | **New file** — `VAlignment { Top, Center, Bottom }` with derives, `#[cfg(test)]`, `# Examples` |
| `quartzite-geometry/src/lib.rs` | Add `mod v_alignment; pub use v_alignment::VAlignment;`; rename re-export `Alignment` → `HAlignment` |
| `quartzite-paint/src/lib.rs` | Update re-export: `HAlignment` + `VAlignment`; update module doc and doctest |
| `quartzite-paint-api/src/painter.rs` | Trait signature + `# Examples` block + in-crate `RecordingPainter`; remove Justify `debug_assert!` and `#[should_panic]` test |
| `quartzite-renderer/src/vello_painter.rs` | Real impl — applies VAlignment arithmetic; remove Justify `debug_assert!`; v_align match is a 3-arm exhaustive match on VAlignment |
| `quartzite-paint-util/src/lib.rs` | `RecordingPainter` in utility crate |
| `quartzite-paint-util/tests/panic_safety.rs` | Test painter |
| `quartzite-style-dispatch/src/dispatch.rs` | Dispatch shim |
| `quartzite-style-dispatch/src/lib.rs` | Rustdoc example comment |
| `quartzite-style/src/paint_widget.rs` | `NullPainter` |
| `quartzite-style/src/style.rs` | `NullPainter` + exerciser test call site |
| `quartzite-style/src/registry.rs` | `NullPainter` + exerciser test call site |
| `quartzite-style/src/default_style/mod.rs` | `Paint<Button>` + `Paint<Label>` call sites; import `VAlignment` |
| `quartzite-style/src/default_style/line_edit.rs` | `Paint<LineEdit>` text + selection-overdraw call sites (K8 recipe removal) |
| `quartzite-style/src/default_style/text_edit.rs` | `Paint<TextEdit>` text + selection-overdraw call sites |
| `quartzite-style/src/default_style_tests.rs` | `PaintEvent::DrawTextIn` gains `v_align: VAlignment`; all AC test assertions updated |
| `quartzite-style/tests/third_party_paint.rs` | Third-party-style sanity test painter |
| `quartzite-widgets/src/lib.rs` | Re-export `HAlignment` instead of `Alignment` |
| `quartzite-widgets/src/widgets/label.rs` | `alignment: Alignment` → `alignment: HAlignment` |
| `quartzite-widgets/tests/snapshots.rs` | `draw_text_in_center` test call site |
| `quartzite-widgets/tests/re_exports.rs` | `Alignment` → `HAlignment` in type-id assertion |
| `src/lib.rs` | Update prelude `pub use quartzite_geometry::Alignment;` → `HAlignment` (add `VAlignment` for symmetry); update `mod paint` doc-comment intra-doc link `Alignment` → `HAlignment`. |

Design subagent enumerates the live set with `ast-index implementations "Painter"` before finalising to confirm no impl is missed.

### Audit-doc shape (AC1, K6)

File: `ai-docs/design-conformance-audit.md`. Per-widget H2 (`## Button`), one row per rule, columns `Rule | Source | Code reference | Status | Note`:
- **Source** = `design-system/README.md#anchor`, `design-system/preview/<file>.html`, or `design-system/proposals/<file>.md`
- **Code reference** = path + line range
- **Status** = ✅ / ❌ / N/A
- **Note** = one sentence; `❌` rows reference the decomposition task that closes them

Widgets: `Button`, `Label`, `LineEdit`, `TextEdit`, `ScrollArea`, `Container`.

### Snapshot regeneration (K7)

Every golden the fix changes regenerates in the same commit as the code change:
- `button_*.png` (shared/ + dark variants) — Button vertical centring (AC2)
- `label*.png` (shared/ + dark variants) — Label vertical centring (AC3)
- `line_edit_*.png` / `text_edit_*.png` — exercises new signature; bytes expected identical (LineEdit already centred, TextEdit top-anchored)

Regen: `scripts/update-snapshots.sh --crate style`. Promote `auto/` output to `shared/` per `support/mod.rs` bootstrap workflow. Visual inspection of representative states (idle, hovered, pressed, focused) in the PR diff before merge.

### Rejected alternatives

- **One PR per widget** (per K2) — golden regeneration is a single bulk operation.
- **Reuse `Alignment` for both axes** — superseded by Design Amendment 2 (PR #556 reviewer request). The shared enum gave confusing semantics and required a runtime guard for `Justify`; `HAlignment + VAlignment` are distinct types, fixing both.
- **`Anchor2D { h, v }` struct** (per K5 original) — adds a wrapping type that callers must construct at every call site.
- **Sibling method `draw_text_in_2d`** (per K5 original) — violates AC7's clean-break requirement.
- **Keep LineEdit smaller-rect recipe one PR cycle** (K8 permissive path) — removed immediately; recipe is dead weight from day one of the new API.

## Decomposition

*(Design Amendment 2: task 0 added; tasks 1–6 updated for `HAlignment`/`VAlignment` types.)*

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 0 | Rename `Alignment` → `HAlignment` throughout `quartzite-geometry`: rename the enum in `alignment.rs` and update its module doc, `# Examples`, and test names. Add `VAlignment { Top, Center, Bottom }` in new `v_alignment.rs` with full derives + `# Examples` + `#[cfg(test)]`. Update `quartzite-geometry/src/lib.rs` exports. Update `quartzite-paint/src/lib.rs` to re-export both. Update `quartzite-widgets/src/lib.rs` re-export. Update `Label.alignment: Alignment` → `Label.alignment: HAlignment`. Update `quartzite-widgets/tests/re_exports.rs`. Update root `quartzite` crate `src/lib.rs`: prelude `pub use quartzite_geometry::Alignment;` → `HAlignment` (add `VAlignment` for symmetry); `mod paint` doc-comment intra-doc link `Alignment` → `HAlignment`. | `quartzite-geometry/src/alignment.rs`, `quartzite-geometry/src/v_alignment.rs` (new), `quartzite-geometry/src/lib.rs`, `quartzite-paint/src/lib.rs`, `quartzite-widgets/src/lib.rs`, `quartzite-widgets/src/widgets/label.rs`, `quartzite-widgets/tests/re_exports.rs`, `src/lib.rs` | — |
| 1 | Update `Painter::draw_text_in` trait signature to `(h_align: HAlignment, v_align: VAlignment)`. Update trait rustdoc + `# Examples` block (AC9); update parameter-order note to reference `HAlignment`/`VAlignment`. Remove the now-obsolete `Alignment::Justify`-on-vertical `debug_assert!` from `RecordingPainter` and from `VelloPainter`, and delete the `draw_text_in_justify_on_vertical_axis_panics_in_debug` `#[should_panic]` test (type system now enforces this). Update every concrete impl in lockstep (all rows in the table except `default_style/` call sites). Update exerciser test call sites in `style.rs` and `registry.rs` (`VAlignment::Top` for v_align). | `quartzite-paint-api/src/painter.rs`, `quartzite-renderer/src/vello_painter.rs`, `quartzite-paint-util/src/lib.rs`, `quartzite-paint-util/tests/panic_safety.rs`, `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-style-dispatch/src/lib.rs`, `quartzite-style/src/paint_widget.rs`, `quartzite-style/src/style.rs`, `quartzite-style/src/registry.rs`, `quartzite-style/tests/third_party_paint.rs`, `quartzite-widgets/tests/snapshots.rs` | 0 |
| 2 | Update `default_style/mod.rs` Button + Label call sites: Button = `(HAlignment::Center, VAlignment::Center)`, Label = `(w.alignment, VAlignment::Center)`. Update `default_style/text_edit.rs` main + overdraw call sites: `(HAlignment::Left, VAlignment::Top)`. | `quartzite-style/src/default_style/mod.rs`, `quartzite-style/src/default_style/text_edit.rs` | 1 |
| 3 | Update `default_style/line_edit.rs`: replace the K8 smaller-rect recipe with `painter.draw_text_in(geom, …, HAlignment::Left, VAlignment::Center)`; update overdraw call similarly. Preserve the `text_carets` `line_height` query. | `quartzite-style/src/default_style/line_edit.rs` | 1 |
| 4 | Update `PaintEvent::DrawTextIn` in `default_style_tests.rs`: change `h_align: Alignment` → `h_align: HAlignment` AND `v_align: Alignment` → `v_align: VAlignment` in the `PaintEvent::DrawTextIn` variant definition and the corresponding `draw_text_in` signature impl in the test painter; update all existing AC assertions to use the renamed types. Update the `button_and_label_use_vertical_centre` test to use `VAlignment::Center`. | `quartzite-style/src/default_style_tests.rs` | 2, 3 |
| 5 | Audit doc `ai-docs/design-conformance-audit.md` — no changes needed to rows (conformance results are unchanged); update any `Alignment` type-name references to `HAlignment`/`VAlignment`. | `ai-docs/design-conformance-audit.md` | 4 |
| 6 | Regenerate goldens (`scripts/update-snapshots.sh --crate style`; promote `auto/` → `shared/`). Goldens should be byte-identical (the rendering arithmetic is unchanged; only type names changed). Run full gate. | `quartzite-style/tests/snapshots/shared/*.png` | 4, 5 |

## Handoff plan

**M = 7**, three groups (Design Amendment 2: task 0 added; non-terminal groups MUST be exactly 3 per the every-group-handoff contract, so a 0–3 / 4–6 split is not valid — re-grouped as 0–2 / 3–5 / 6):

- **Group A (subtasks 0–2):** enum split (`HAlignment` rename + new `VAlignment`) + paint-API trait/impl lockstep + Button/Label/TextEdit caller updates in `default_style/mod.rs` + `default_style/text_edit.rs`. Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at start of Group A.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B (subtasks 3–5):** LineEdit K8 recipe removal + symbolic test updates in `default_style_tests.rs` + audit-doc type-name updates. Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at start of Group B.
- **Handoff after Group B:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group C with fresh context.
- **Group C (subtask 6):** regenerated goldens + full green gate. Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at start of Group C. Terminal group (1 subtask; within the 1..=3 range).

## Risks

| Risk | Mitigation |
|---|---|
| Goldens fail equality on a future-PR's `shared/` set | Commit every regenerated PNG alongside its triggering code change; reviewers visually inspect representative states in the PR diff |
| `Justify` on v_axis silently mis-renders post-merge | **Eliminated by Design Amendment 2.** `VAlignment` has no `Justify` variant; the compiler rejects `VAlignment::Justify` at the type level. No runtime guard or fallback is required. |
| `VelloPainter` v_align arithmetic produces sub-pixel y, shifting the rasterised baseline | Pixel-snap `py` via `py.round()` after v_align translation, before `emit_layout_glyphs` |
| Documentation drift: `comp-label.html` shows top-anchored Label but impl centres (K3) | Audit task 5 calls out the drift in the `Label` section's `Note` column; deferred cleanup tracked in spec's Deferred table |
| PR #554 smaller-rect recipe removal breaks `paint_selection_line_edit` / `paint_caret_line_edit` | Those helpers each call `text_carets` independently — verified not dependent on main-text-draw's smaller rect |
| `Painter` signature change is a public-API change | Permitted by AGENTS.md § *API Stability* pre-crates.io clean-break axiom; explicitly surfaced for design-review |

## Test design

### Task 0 — enum split
- `quartzite-geometry/src/alignment.rs` `#[cfg(test)]` tests: rename any test referencing the type to use `HAlignment`; keep the same variant-coverage shape (`Left / Center / Right / Justify`).
- `quartzite-geometry/src/v_alignment.rs` `#[cfg(test)]` tests (new): construct each variant (`Top / Center / Bottom`), assert `Debug` / `Eq` / `Default` round-trip per the derive set used by `HAlignment` (`MetaEnum, Copy, Clone, Debug, PartialEq, Eq, Default`). Neither enum derives `Hash`, so no `Hash` assertion. Assert `VAlignment::default() == VAlignment::Top`.
- `quartzite-widgets/tests/re_exports.rs`: assert `HAlignment` re-export `TypeId` equality; add an assertion for `VAlignment` re-export.

### Task 1 — trait + all non-default-style impls
- Existing object-safety / dispatch tests in `quartzite-paint-api/src/painter.rs` `#[cfg(test)]` updated to pass `(HAlignment, VAlignment)` values.
- `RecordingPainter` extended to record `v_align: VAlignment`; dispatch tests assert both axes propagate through the trait object.
- **Rendering-level vertical-centring test (new):** in `quartzite-widgets/tests/snapshots.rs` alongside the existing `draw_text_in_center` test, add a test that:
  1. Constructs a known-height rect (e.g. `Rect::new(0, 0, 200, 64)`).
  2. Renders a single-line text via `VelloPainter::draw_text_in(rect, "Sample", …, HAlignment::Center, VAlignment::Center)`.
  3. Locates the rendered glyph cluster's y-extent on the rasterised canvas (scan pixel rows for non-transparent text-coloured pixels; record min_y / max_y).
  4. Computes `glyph_y_midpoint = (min_y + max_y) / 2`.
  5. Asserts `glyph_y_midpoint` lies within ±2 px of the canvas vertical midpoint (`rect.height() / 2`).
  6. A parallel sub-case with `v_align = VAlignment::Top` asserts `min_y` lies within ±2 px of `rect.top()`; another with `v_align = VAlignment::Bottom` asserts `max_y` lies within ±2 px of `rect.bottom()`. The three sub-cases together exercise every concrete v_align branch in `VelloPainter`.
- Tolerance ±2 px accommodates one-pixel rounding from `py.round()` plus parley's intrinsic ascent/descent variation around the layout's baseline.
- **No `#[should_panic]` test for `Justify` on the vertical axis** — the type system rejects `VAlignment::Justify` at compile time; the previously-planned `draw_text_in_justify_on_vertical_axis_panics_in_debug` test is obsolete and is deleted.

### Task 2 — Button / Label / TextEdit callers
- Extend `PaintEvent::DrawTextIn` assertions in the existing `button_records_fill_outline_and_centred_text` and `label_records_fill_and_text_with_label_alignment` tests to assert `v_align == VAlignment::Center`.
- TextEdit tests assert `v_align == VAlignment::Top`.
- Edge: Label with `alignment = HAlignment::Right` records `v_align = VAlignment::Center, h_align = HAlignment::Right`.

### Task 3 — LineEdit K8 recipe removal
- Existing LineEdit AC tests at `default_style_tests.rs:1180+` extended to assert `DrawTextIn { rect: geom, h_align: HAlignment::Left, v_align: VAlignment::Center }` (full-rect, not smaller rect).
- Selection-overdraw call at ~line 205 also asserts `(HAlignment::Left, VAlignment::Center)`.

### Task 4 — AC8 symbolic test
- New test `button_and_label_use_vertical_centre`: asserts `DrawTextIn { v_align: VAlignment::Center, .. }` for both `Paint<Button>` and `Paint<Label>`. Asserts on the recorded argument, not rect arithmetic — survives K8 migration.

### Tasks 5 & 6
- Task 5: audit doc reviewed manually against AC1; no executable test.
- Task 6: existing snapshot equality tests pass with the regenerated `shared/` PNGs.

## Open questions

None. K5 shape resolved by Design Amendment 2 (split into `HAlignment` + `VAlignment` — distinct types per axis, replacing the original reuse-`Alignment`-twice resolution); `Justify`-on-vertical resolved at the type level (`VAlignment` has no `Justify` variant, the compiler rejects it — no `debug_assert!` or release fallback needed); K8 resolved (remove smaller-rect recipe in this PR); K7 honoured (regen every golden in the same commit).
