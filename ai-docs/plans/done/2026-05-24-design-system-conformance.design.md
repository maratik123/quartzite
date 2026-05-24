# Design: Design-system conformance audit + vertical-alignment paint-API axis

**Spec:** `ai-docs/plans/2026-05-24-design-system-conformance.spec.md`
**Issue:** #555
**Date:** 2026-05-24

## Approach

The spec ships three artefacts on a single feature branch (per K2):

0. **Paint-API extension** — `Painter::draw_text_in` gains a second alignment parameter for the vertical axis.
1. **Audit doc** `ai-docs/design-conformance-audit.md` (one section per paintable widget, one row per design-system rule).
2. **Conformance fixes** on every `❌` row the audit surfaces (at minimum: `Button` vertical centring per AC2, `Label` vertical centring per AC3) + regenerated goldens.

### K5 — chosen API shape: reuse `Alignment` with two parameters

The current signature:

```rust
fn draw_text_in(&mut self, rect: Rect, text: &str, font: &Font, brush: &Brush, alignment: Alignment);
```

becomes:

```rust
fn draw_text_in(
    &mut self,
    rect: Rect,
    text: &str,
    font: &Font,
    brush: &Brush,
    h_align: Alignment,
    v_align: Alignment,
);
```

**Why (a) over alternatives:**
- `Alignment`'s variant doc strings already document both axes (`Left = left (horizontal) or top (vertical)`). Reusing it consumes the existing contract.
- `VAlignment` parallel enum doubles the derive surface for no semantic gain — `Top/Center/Bottom/Justify` is the same enum modulo a rename; the question of `Justify`-on-vertical exists in either form.
- `Anchor2D { h, v }` adds a wrapping type that callers must construct at every call site; buys nothing the tuple of two enums does not buy.
- Sibling method `draw_text_in_2d` violates AC7's clean-break requirement (parallel API stays alive) and AGENTS.md § *API Stability*'s pre-publish clean-break axiom.

**`Alignment::Justify` on the vertical axis:** no meaningful mapping in parley's layout model. Treatment: `debug_assert!(!matches!(v_align, Alignment::Justify), "draw_text_in: Alignment::Justify is invalid on the vertical axis")` at the trait contract level. Every concrete `Painter` impl falls back to `Alignment::Left` (top) when `Justify` is encountered in release builds. Documented in the trait rustdoc and in `Alignment`'s variant doc comment.

**Exact wording added to `Alignment::Justify` variant doc-comment** (in `quartzite-geometry/src/alignment.rs`):

> Invalid on the vertical axis when passed to `Painter::draw_text_in`'s `v_align`; debug-asserts in debug builds, falls back to `Left` (top) in release.

### `draw_text_in` trait rustdoc — parameter-order note

The trait rustdoc for `Painter::draw_text_in` MUST include the parameter-order contract as **free-form prose in the doc comment body** (NOT as a non-canonical `# Parameter order` section — only the canonical section headings `# Parameters`, `# Returns`, `# Errors`, `# Panics`, `# Safety`, `# Examples`, `# See also` are permitted per `ai-docs/doc-convention.md`). The prose appears before the `# Examples` block:

> **Parameter order:** `h_align` always precedes `v_align`. Both are `Alignment` — call sites must rely on positional order; treat any `draw_text_in(..., v, h)` ordering as a defect.

This prose is reproduced verbatim in the rustdoc for `RecordingPainter::draw_text_in` and `NullPainter::draw_text_in` so the contract is visible at every reading entry point. (Design Amendment: original design prescribed `# Parameter order` — corrected to canonical free-form prose per self-review finding #1.)

### `VelloPainter` vertical-axis implementation recipe

`VelloPainter::draw_text_in` currently sets glyph `py = rect.top() * scale` unconditionally. After the API change:

1. Build the layout exactly as today (parley's `layout.align(parley_h_align, …)` continues to handle the horizontal axis).
2. Compute the total laid-out text height: `layout.height()`.
3. Translate `py` depending on `v_align`:
   - `Alignment::Left` (top) → `py = rect.top() * scale` (unchanged)
   - `Alignment::Center` → `py = rect.top() * scale + (rect.height() * scale - layout.height()) / 2.0`
   - `Alignment::Right` (bottom) → `py = rect.top() * scale + (rect.height() * scale - layout.height())`
   - `Alignment::Justify` → debug_assert, then fall through to `Left`

Pixel-snap `py` via `py.round()` after the v_align translation (before `emit_layout_glyphs`) to avoid sub-pixel baseline drift per the workspace's "integer-pixel geometry" rule.

### Caller-side updates in `default_style/`

- **`Button`** (`default_style/mod.rs:237`) → `h_align = Alignment::Center, v_align = Alignment::Center`
- **`Label`** (`default_style/mod.rs:277`) → `h_align = w.alignment, v_align = Alignment::Center` (K3)
- **`LineEdit`** (`default_style/line_edit.rs:84 + 205`) → replace the PR #554 smaller-rect recipe with `painter.draw_text_in(geom, …, Alignment::Left, Alignment::Center)` (K8 — remove in same PR). The `text_carets` `line_height` query remains for `paint_selection_line_edit` / `paint_caret_line_edit` (those helpers compute their own vertical positions for bands and carets — not dependent on the main-text-draw's smaller rect).
- **`TextEdit`** (`default_style/text_edit.rs:56 + 298`) → `h_align = Alignment::Left, v_align = Alignment::Left` (explicit top-anchor, preserves G5).

### Every `Painter` impl that must update in lockstep

| File | Notes |
|---|---|
| `quartzite-paint-api/src/painter.rs` | Trait signature + `# Examples` block + in-crate `RecordingPainter` |
| `quartzite-renderer/src/vello_painter.rs` | Real impl — applies v_align arithmetic |
| `quartzite-paint-util/src/lib.rs` | `RecordingPainter` in utility crate |
| `quartzite-paint-util/tests/panic_safety.rs` | Test painter |
| `quartzite-style-dispatch/src/dispatch.rs` | Dispatch shim |
| `quartzite-style-dispatch/src/lib.rs` | Rustdoc example comment |
| `quartzite-style/src/paint_widget.rs` | `NullPainter` |
| `quartzite-style/src/style.rs` | `NullPainter` + exerciser test call site |
| `quartzite-style/src/registry.rs` | `NullPainter` + exerciser test call site |
| `quartzite-style/src/default_style/mod.rs` | `Paint<Button>` + `Paint<Label>` call sites |
| `quartzite-style/src/default_style/line_edit.rs` | `Paint<LineEdit>` text + selection-overdraw call sites (K8 recipe removal) |
| `quartzite-style/src/default_style/text_edit.rs` | `Paint<TextEdit>` text + selection-overdraw call sites |
| `quartzite-style/src/default_style_tests.rs` | `PaintEvent::DrawTextIn` gains `v_align` field; all AC test assertions extended |
| `quartzite-style/tests/third_party_paint.rs` | Third-party-style sanity test painter |
| `quartzite-widgets/tests/snapshots.rs` | `draw_text_in_center` test call site |
| `quartzite-geometry/src/alignment.rs` | `Alignment` variant doc comment — `Justify`-on-vertical paragraph |

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
- **`VAlignment` / `Anchor2D` / sibling method** (per K5) — see K5 rationale above.
- **Keep LineEdit smaller-rect recipe one PR cycle** (K8 permissive path) — removed immediately; recipe is dead weight from day one of the new API.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Extend `Painter::draw_text_in` trait signature with `h_align, v_align: Alignment`. Update trait rustdoc + `# Examples` block (AC9). Document `Justify`-on-vertical = invalid (debug_assert + Left fallback). Update every concrete impl in lockstep (all rows in the table above except `default_style/` call sites and `default_style_tests.rs`). Update existing exerciser test call sites in `style.rs` and `registry.rs` (`Alignment::Left` for v_align). Extend `Alignment` variant doc in `alignment.rs`. | `quartzite-paint-api/src/painter.rs`, `quartzite-renderer/src/vello_painter.rs`, `quartzite-paint-util/src/lib.rs`, `quartzite-paint-util/tests/panic_safety.rs`, `quartzite-style-dispatch/src/dispatch.rs`, `quartzite-style-dispatch/src/lib.rs`, `quartzite-style/src/paint_widget.rs`, `quartzite-style/src/style.rs`, `quartzite-style/src/registry.rs`, `quartzite-style/tests/third_party_paint.rs`, `quartzite-widgets/tests/snapshots.rs`, `quartzite-geometry/src/alignment.rs` | — |
| 2 | Update `default_style/mod.rs` Button + Label call sites: Button = `(Center, Center)`, Label = `(w.alignment, Center)`. Update `default_style/text_edit.rs` main + overdraw call sites: `(Left, Left)`. | `quartzite-style/src/default_style/mod.rs`, `quartzite-style/src/default_style/text_edit.rs` | 1 |
| 3 | Update `default_style/line_edit.rs`: replace the K8 smaller-rect recipe at lines ~75–84 with `painter.draw_text_in(geom, …, Alignment::Left, Alignment::Center)`; update overdraw call at ~line 205 similarly. Preserve the `text_carets` `line_height` query used by `paint_selection_line_edit` / `paint_caret_line_edit`. | `quartzite-style/src/default_style/line_edit.rs` | 1 |
| 4 | Extend `PaintEvent::DrawTextIn` in `default_style_tests.rs` with `v_align: Alignment`; update `RecordingPainter::draw_text_in` to record it. Update all existing AC assertions to include `v_align` checks. Add new test `button_and_label_use_vertical_centre` (AC8). | `quartzite-style/src/default_style_tests.rs` | 2, 3 |
| 5 | Write `ai-docs/design-conformance-audit.md` (AC1, K6): one H2 per widget, one row per design-system rule, ✅/❌/N/A status reflecting the post-fix state, `❌` rows reference the task # that closes them. **Contingency:** if the audit surfaces an unanticipated `❌` row (a divergence not covered by tasks 2/3, e.g. a `ScrollArea` or `Container` rule the spec did not enumerate), append an additional fix task (`5a`) before task 6 closing the row in the same PR; AC3b ("every `❌` row gets a same-PR fix") is therefore satisfied within the same PR cycle. The audit doc's final state still records every row as ✅ / N/A at PR-merge time. | `ai-docs/design-conformance-audit.md` | 2, 3, 4 |
| 6 | Regenerate goldens (`scripts/update-snapshots.sh --crate style`; promote `auto/` → `shared/`). Verify visually: `button_*` + `label*` PNGs centred; `line_edit_*` + `text_edit_*` unchanged. Run full gate: `cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`. | `quartzite-style/tests/snapshots/shared/*.png`, `quartzite-style/tests/snapshots/auto/*.png` | 4, 5 |

## Handoff plan

**M = 6**, two groups:

- **Group A (subtasks 1–3):** paint-API extension lockstep + all `default_style/` callers. Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at start of Group A.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B (subtasks 4–6):** symbolic tests, audit doc, regenerated goldens + green gate. Terminal group (3 subtasks; within the 1..=3 range).

## Risks

| Risk | Mitigation |
|---|---|
| Goldens fail equality on a future-PR's `shared/` set | Commit every regenerated PNG alongside its triggering code change; reviewers visually inspect representative states in the PR diff |
| `Alignment::Justify` on v_axis silently mis-renders post-merge | `debug_assert!` + `Left` fallback in release; documented in trait rustdoc + `Alignment` variant doc |
| `VelloPainter` v_align arithmetic produces sub-pixel y, shifting the rasterised baseline | Pixel-snap `py` via `py.round()` after v_align translation, before `emit_layout_glyphs` |
| Documentation drift: `comp-label.html` shows top-anchored Label but impl centres (K3) | Audit task 5 calls out the drift in the `Label` section's `Note` column; deferred cleanup tracked in spec's Deferred table |
| PR #554 smaller-rect recipe removal breaks `paint_selection_line_edit` / `paint_caret_line_edit` | Those helpers each call `text_carets` independently — verified not dependent on main-text-draw's smaller rect |
| `Painter` signature change is a public-API change | Permitted by AGENTS.md § *API Stability* pre-crates.io clean-break axiom; explicitly surfaced for design-review |

## Test design

### Task 1 — trait + all non-default-style impls
- Existing object-safety / dispatch tests in `quartzite-paint-api/src/painter.rs` `#[cfg(test)]` updated to pass two `Alignment` values.
- `debug_assert!` fires on `v_align = Alignment::Justify` — tested with `#[should_panic]` gated on `#[cfg(debug_assertions)]`.
- `RecordingPainter` extended to record `v_align`; dispatch tests assert both axes propagate through the trait object.
- **Rendering-level vertical-centring test (new):** in `quartzite-widgets/tests/snapshots.rs` alongside the existing `draw_text_in_center` test, add a test that:
  1. Constructs a known-height rect (e.g. `Rect::new(0, 0, 200, 64)`).
  2. Renders a single-line text via `VelloPainter::draw_text_in(rect, "Sample", …, Alignment::Center, Alignment::Center)`.
  3. Locates the rendered glyph cluster's y-extent on the rasterised canvas (scan pixel rows for non-transparent text-coloured pixels; record min_y / max_y).
  4. Computes `glyph_y_midpoint = (min_y + max_y) / 2`.
  5. Asserts `glyph_y_midpoint` lies within ±2 px of the canvas vertical midpoint (`rect.height() / 2`).
  6. A parallel sub-case with `v_align = Alignment::Left` (top) asserts `min_y` lies within ±2 px of `rect.top()`; another with `v_align = Alignment::Right` (bottom) asserts `max_y` lies within ±2 px of `rect.bottom()`. The three sub-cases together exercise every concrete v_align branch in `VelloPainter`.
- Tolerance ±2 px accommodates one-pixel rounding from `py.round()` plus parley's intrinsic ascent/descent variation around the layout's baseline.

### Task 2 — Button / Label / TextEdit callers
- Extend `PaintEvent::DrawTextIn` assertions in the existing `button_records_fill_outline_and_centred_text` and `label_records_fill_and_text_with_label_alignment` tests to assert `v_align == Alignment::Center`.
- TextEdit tests assert `v_align == Alignment::Left`.
- Edge: Label with `alignment = Alignment::Right` records `v_align = Center, h_align = Right`.

### Task 3 — LineEdit K8 recipe removal
- Existing LineEdit AC tests at `default_style_tests.rs:1180+` extended to assert `DrawTextIn { rect: geom, h_align: Left, v_align: Center }` (full-rect, not smaller rect).
- Selection-overdraw call at ~line 205 also asserts `(Left, Center)`.

### Task 4 — AC8 symbolic test
- New test `button_and_label_use_vertical_centre`: asserts `DrawTextIn { v_align: Alignment::Center, .. }` for both `Paint<Button>` and `Paint<Label>`. Asserts on the recorded argument, not rect arithmetic — survives K8 migration.

### Tasks 5 & 6
- Task 5: audit doc reviewed manually against AC1; no executable test.
- Task 6: existing snapshot equality tests pass with the regenerated `shared/` PNGs.

## Open questions

None. K5 shape resolved (reuse `Alignment` with two parameters); `Justify`-on-vertical resolved (debug_assert + Left fallback); K8 resolved (remove smaller-rect recipe in this PR); K7 honoured (regen every golden in the same commit).
