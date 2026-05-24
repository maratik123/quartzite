# Design-system conformance audit + fixes (incl. vertical-alignment axis)

**Source:** user description (free-text task)
**Date:** 2026-05-24
**Tracked in:** #555

## Problem statement

The user reports that the implementation in `quartzite-style/src/default_style/` and
`quartzite-widgets/src/widgets/` deviates from the visual contract laid out in
`design-system/` (README + `preview/*.html` mocks + `proposals/*.md`). Concretely
quoted by the user: `Button` text is not vertically centred in the committed
golden snapshots, while `design-system/preview/comp-button-anatomy.html` and
`comp-button-states.html` show centred text as the target.

A walk-through of the existing snapshots vs the design-system mocks confirms at
least one mechanical root cause and several lower-leverage conformance gaps.

The user further directed (round 2) that introducing a **vertical-alignment axis
control** — symmetric to the existing horizontal `Alignment` enum — belongs in
this spec's scope, not in a follow-up. The fix surface therefore covers both
the conformance gaps and the paint-API extension that makes the fixes idiomatic.

## Confirmed conformance gaps (round 1)

These are direct observations made during interview round 1 by reading
`quartzite-style/src/default_style/`, `quartzite-renderer/src/vello_painter.rs`,
the committed golden PNGs under `quartzite-style/tests/snapshots/auto/` and
`shared/`, and the design-system preview HTML / proposal docs.

### G1. `Painter::draw_text_in` is horizontal-only — vertical anchor is `rect.top()`

`Alignment` (`quartzite-geometry/src/alignment.rs`) is a four-value
single-axis enum (`Left | Center | Right | Justify`); the variant doc
strings already mention both axes ("Left = left (horizontal) or top
(vertical)"), but `Painter::draw_text_in`
(`quartzite-paint-api/src/painter.rs:147-154`) accepts only one
`alignment: Alignment` parameter that the renderer maps to parley's
horizontal alignment. `VelloPainter::draw_text_in`
(`quartzite-renderer/src/vello_painter.rs:629`) confirms: `py` glyph
origin is `rect.top() * scale` unconditionally; vertical anchor is
fixed.

Net effect: every `draw_text_in(geom, …, Alignment::Center)` call paints
text **vertically anchored at the top of `geom`**. Vertical centring
today requires the caller to construct a smaller rect at
`geom.top() + (geom.size().height() - line_height) / 2`.

This is the **root cause** of the Button-not-centred observation, and
the gap this spec closes at the paint-API level (see Scope item 0).

### G2. `Button` paints text into full `geom` with `Alignment::Center`

`quartzite-style/src/default_style/mod.rs:237-244`:

```rust
painter.draw_text_in(
    geom,                       // ← full widget rect
    &w.text,
    &font,
    &Brush::solid(text_color),
    Alignment::Center,          // ← horizontal only (see G1)
);
```

Golden `button_idle.png` (and every other `button_*` PNG under
`tests/snapshots/auto/`) shows the literal consequence: "OK" sits in the
top-third of the 64×64 canvas. The design-system anatomy card requires
the label centred on both axes inside the button rect.

### G3. `Label` paints text into full `geom`, top-anchored — must vertically centre

`quartzite-style/src/default_style/mod.rs:277`:

```rust
painter.draw_text_in(geom, &w.text, &font, &Brush::solid(text_color), w.alignment);
```

Committed golden `label.png` (`shared/label.png`) shows "hi" at the
top-left of the 64×64 canvas. **Round-2 decision (K3)** is to vertically
centre `Label` text, in line with `Button` and `LineEdit`, rather than
keep the top-anchored behaviour suggested by `comp-label.html`'s mock
frame. The mock is documentation-only; the implemented contract is
"single-line text widgets centre vertically".

Fix: once the paint-API extension lands, call the new two-axis variant
with `Alignment::Center` on the vertical axis and preserve
`w.alignment` on the horizontal axis. Until the API extension lands the
caller-side "compute smaller rect" recipe from PR #554 is the bridging
implementation.

### G4. `LineEdit` was fixed in PR #554 (precedent for the centring recipe)

The commit `ae68826` already applies the vertical-centring recipe to
`Paint<LineEdit>`:

```rust
let line_height = { painter.text_carets(text_arg, &font).line_height() };
let text_top = geom.top() + (geom.size().height() - line_height) / 2;
let text_rect = Rect::new(
    Point::new(geom.left(), text_top),
    Size::new(geom.size().width(), line_height),
);
painter.draw_text_in(text_rect, text_arg, &font, &text_brush, Alignment::Left);
```

After the paint-API extension lands, this caller-side recipe is
**replaced** by passing the vertical-axis alignment directly to the
painter (see Scope item 0). PR #554's recipe is allowed to remain
in-place during the migration; the design subagent decides whether to
delete it in the same PR or in a follow-up.

### G5. `TextEdit` is intentionally top-anchored

`comp-text-edit.html`'s mock CSS includes `vertical-align: top` for
`.qz-text-edit`, and the framework's behaviour (multi-line text laid out
from the top) matches. Golden `text_edit_plain.png` shows "abc" at the
top-left — design-conformant. After the paint-API extension lands,
`TextEdit` calls the two-axis painter with vertical = `Alignment::Left`
(top) explicitly, which preserves its semantics but stops relying on the
implicit-top default.

### G6. Numeric tokens currently soft-coded

The mocks declare specific paddings / min-widths / min-heights:

| Widget   | min-width | min-height | padding |
|----------|-----------|------------|---------|
| Button   | 64 px     | 32 px      | 6 px × 16 px |
| LineEdit | 180 px    | 28 px      | 4 px × 6 px |
| TextEdit | 220 px    | 72 px      | 4 px × 6 px |

These tokens do **not** appear in `quartzite-widgets/src/widgets/`
(`size_hint`/`min_size_hint` are not surfaced on the built-in widgets,
and `WidgetBase` carries no padding). Whether to introduce them is a
design decision the design subagent must take — see Key Decision K4.

### G7. Other design-system items already-conformant (per round-1 read)

The round-1 audit found the following items to be already conformant with
the design system; they are **not** part of this task's fix list but are
listed here so that the design subagent does not redundantly investigate:

- **Palette**: light + dark seeds in `quartzite-style-types::Palette`
  match `colors_and_type.css` and `README.md`'s seed table for every
  `ColorRole` slot, including `FocusRing` and `ScrollBar`.
- **State derivations** (`ColorGroup`): `Hover = c.blend(WindowText, 0.06)`,
  `Pressed = c.blend(WindowText, 0.16)` match the formulas in
  `README.md` and the `colors-state-derivations.html` preview.
- **Read-only overlay**: the implemented `WindowText × 0.10` overlay
  matches `proposals/text-edit-read-only-overlay.md` (the
  `Window × 0.5` formula in the mock CSS is a documentation simplification
  superseded by the proposal). Confirmed by round-1 grep against
  `READ_ONLY_OVERLAY_ALPHA` and `READ_ONLY_TEXT_ALPHA` constants.
- **Pressed / checked outline colour**: code resolves outline to
  `HighlightedText` (white) on pressed/checked Button — matches the
  `border-color: #FFFFFF` rule encoded in the `qz-button.pressed` /
  `qz-button.checked` mock CSS.
- **Focus ring**: 2 px overlay reading `ColorRole::FocusRing × Normal`
  applied additively in `Paint<Button>`, `Paint<Label>`, `Paint<LineEdit>`,
  `Paint<ScrollArea>` — matches the visual rule.
- **Disabled**: α × 0.5 alpha modifier applied post role selection in
  every `Paint<W>` impl through the shared `maybe_disabled` helper.
- **Caret + selection** (`LineEdit`, `TextEdit`): conform to
  `proposals/caret-and-selection.md` — 1 px caret, line-box height,
  pixel-snapped, 530 ms blink, reduced-motion override, unfocused-with-
  selection greyed.

These items remain **out of scope** unless a future audit pass surfaces a
gap.

## Scope

The deliverable is split into three artefacts:

0. **A paint-API extension exposing a vertical-alignment axis.** The
   public `Painter::draw_text_in` surface gains the ability to specify
   alignment on **both** horizontal and vertical axes, symmetric to
   the role the existing `Alignment` enum plays on the horizontal axis.
   The shape of the API change — e.g. a separate `VAlignment` enum vs.
   re-using `Alignment` (whose variant doc strings already document both
   axes) vs. accepting an `(Alignment, Alignment)` tuple vs. adding a
   sibling method — is a **design-phase decision** (see Key Decision K5).
   The design subagent picks one shape, updates every `Painter` impl
   (`VelloPainter`, `quartzite-paint-util`'s test painter,
   `quartzite-style-dispatch` dispatch shim, `quartzite-style`'s style
   thunks, `third_party_paint.rs` test painter, the snapshot-helper
   painter, the `Painter` trait's `# Examples` block in
   `quartzite-paint-api`), and lands the change in the same PR.
   Per AGENTS.md § *API Stability* (pre-crates.io clean-break axiom)
   the old single-axis form is replaced cleanly; no alias / no wrapper
   layer remains. Callers in `quartzite-style/src/default_style/` are
   updated accordingly.

1. **An audit document** at `ai-docs/design-conformance-audit.md`
   enumerating, for every paintable widget × every design-system rule, the
   conformance status as of the spec's date. Format: one section per
   widget, one row per rule, columns
   `Rule | Source | Code reference | Status (✅ / ❌ / N/A) | Note`. The audit
   is checked into the repo as a durable record so that future
   conformance passes can diff against it.

2. **A code fix** addressing every `❌` row the audit produces, on a
   single feature branch, in one PR. The fix landing-criterion is that
   every golden snapshot under `quartzite-style/tests/snapshots/`
   regenerates to the conformant target render, all symbolic AC tests
   in `quartzite-style/src/default_style_tests.rs` continue to pass (or
   are updated to assert the new behaviour where the assertion target
   moved), and `cargo build` / `cargo clippy --workspace --all-targets
   -- -D warnings` / `cargo test` / `cargo doc` (with the workspace doc
   flags) are all green.

The fix is bounded by the issues the **audit step** surfaces — not by the
round-1 G-list above (the audit is allowed to add or drop rows). The
G-list is the spec author's starting hypothesis.

## Out of scope

- Adding new widgets or removing existing ones.
- Changing `ColorRole` / `Palette` / `ColorGroup` semantics (those are
  governed by separate specs — `palette-state-groups.proposal.md`
  remains the source of truth on the colour-axis side).
- Caret / selection / scrollbar conformance — already covered by the
  proposals under `design-system/proposals/` and addressed in the
  recently-merged PRs #553 and #554.
- Brand assets (`design-system/assets/quartzite-*.svg`,
  `Quartzite Designer *.html`) — these are documentation artefacts, not
  code.
- Per-widget cursor shape (`proposals/cursor-shapes.md`) — separate
  open issue #404.
- Popup / tooltip conformance (`proposals/popups-and-tooltips.md`) — no
  Popup or Tooltip widget exists yet, so there is nothing to conform.
- **Extending the painter API beyond text vertical alignment.** Scope item
  0 covers vertical-axis alignment for `draw_text_in` only; analogous
  extensions for `draw_image` placement, `draw_path` anchor semantics,
  or widget-level child-content alignment are not in scope.

## Deferred

| What | Why | Separate issue needed? |
|---|---|---|
| Padding / min-size tokens (G6) | Currently widget geometry is fully caller-driven; introducing built-in `min_size_hint` defaults is a layout-system concern, not a paint-rule concern. | Yes — follow-up spec. The conformance audit will surface this as a row but the spec carries no implementation requirement. |
| Refresh `design-system/preview/*.html` mocks where the mock simplification differs from the implemented rule (e.g. read-only overlay using `Window × 0.5` vs implemented `WindowText × 0.10`; `comp-label.html`'s top-anchored Label mock vs the K3 vertical-centring rule). | The mocks are documentation-only; the proposal / spec is authoritative. A drift-cleanup pass keeps the two sources aligned but is not required for code-conformance. | Yes — documentation follow-up. |

## Key decisions

| Question | Decision |
|---|---|
| K1. Two-step deliverable (audit doc + code fix) vs one-step (fix only)? | **Audit + fix** (round-1 Q1 answer) — write `ai-docs/design-conformance-audit.md` enumerating every gap, then fix every `❌` row in the same PR. Durable record for future passes. |
| K2. Single PR vs per-widget PR series? | **Single PR** (round-1 Q2 answer) — all conformance fixes + audit doc + paint-API extension + golden regen on one feature branch. The fixes share a single recipe and a single regeneration of the golden snapshot suite; splitting per widget multiplies the regeneration noise. |
| K3. `Label` vertical centring — vertically centre like Button, or top-anchor per design-system mock? | **Vertically centre** (round-1 Q3 answer) — apply the vertical-axis centring to `Paint<Label>`. Diverges from the `comp-label.html` mock; the mock is documentation-only and is queued for a documentation-drift cleanup (see Deferred). |
| K4. Add `min_size_hint` / padding defaults to built-in widgets (G6)? | **No** — deferred to a follow-up spec (see Deferred row). The fix scope here is paint-rule conformance, not layout-system conformance. |
| K5. Vertical-alignment API shape (paint-API extension — Scope item 0). | **Design subagent decides** the concrete shape among (a) reuse `Alignment` with a second parameter, (b) introduce `VAlignment` as a parallel enum, (c) accept a struct `Anchor2D { h, v }`, (d) add a sibling method `draw_text_in_2d` / equivalent. Constraint: the new API MUST be expressive enough for every existing `draw_text_in` call site plus the vertical-centring demanded by K3 and AC2 / AC3. AGENTS.md § *API Stability* permits a clean rename / replacement (pre-crates.io clean-break axiom). |
| K6. Audit-doc format | One section per widget, one row per rule, columns `Rule | Source | Code reference | Status (✅ / ❌ / N/A) | Note`. (See Scope item 1.) |
| K7. Snapshot regeneration policy | Every golden PNG that the conformance fix changes is regenerated in the same commit as the code change. The PR diff inspects the regenerated PNGs visually to verify the conformance target was hit, not just that the test passed. |
| K8. PR #554's caller-side smaller-rect recipe — keep or remove during migration? | **Remove in the same PR** (preferred) — once the painter accepts a vertical-axis alignment, the recipe becomes redundant and adds drift risk. The design subagent may elect to leave it for one PR cycle if the diff size forces it; the spec is permissive but biased toward removal. |

## Technical constraints

- **`Alignment` enum identity is preserved.** The design subagent MAY
  extend or re-use it (Key Decision K5), but renames / removals of
  existing variants are out of scope (this is a paint-API change, not
  an enum-redesign change).
- **Snapshot tests use a 64×64 canvas** (`CANVAS` constant in
  `quartzite-style/tests/snapshots.rs`). The conformance fixes must
  produce visually-correct output on this canvas size; the canvas
  size itself is not changed by this spec.
- **`WidgetBase::font` is the source of font for paint.** Every paint
  impl reads `w.widget_base().font.clone()` — the conformance fix
  uses this same source (not a per-style override).
- **Snapshot helpers are mirrored** between
  `quartzite-widgets/tests/support/mod.rs` and
  `quartzite-style/tests/support/mod.rs` (Snapshot-helper sync group in
  AGENTS.md § Propagation Rule). The fix must keep them in sync if
  either is touched.
- **Every `Painter` impl must update in lockstep.** Concrete
  implementations enumerated in the round-1 grep:
  `quartzite-renderer/src/vello_painter.rs`,
  `quartzite-paint-util/src/lib.rs` (no-op test painter),
  `quartzite-paint-util/tests/panic_safety.rs` (panic-safety test impl),
  `quartzite-style-dispatch/src/dispatch.rs` (dispatch shim),
  `quartzite-style/src/paint_widget.rs` (paint-widget thunk),
  `quartzite-style/src/registry.rs` (registry dispatch),
  `quartzite-style/src/style.rs` (style dispatch),
  `quartzite-style/tests/third_party_paint.rs` (third-party paint test
  fake), plus every snapshot-helper `Painter` impl. The design subagent
  enumerates the live set with `ast-index implementations "Painter"`
  before finalising and double-checks no impl is missed.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `ai-docs/design-conformance-audit.md` exists, lists every paintable widget × every applicable design-system rule, with columns `Rule | Source | Code reference | Status | Note`. Every `❌` row has an associated code change in the same PR. |
| AC2 | `Button` text is vertically centred in every committed golden under `quartzite-style/tests/snapshots/auto/button_*.png` and `shared/button_*.png` / `shared/dark_button_*.png`. Verifiable by visual inspection of the regenerated PNGs and by a new symbolic AC test that asserts the painter receives a vertical-axis-centring argument for the `Button` paint call. |
| AC3 | `Label` text is vertically centred (preserving the horizontal `w.alignment`) in every committed golden under `shared/label*.png` / `shared/dark_label*.png` and any `auto/label_*.png` produced by snapshot generation. Verifiable by visual inspection of the regenerated PNGs and by a new symbolic AC test mirroring AC8's recipe-assertion for `Paint<Label>`. |
| AC3b | For every other widget the audit (AC1) flags `❌`, the corresponding paint impl is updated and the corresponding golden PNGs regenerate to the conformant target. (Provisionally: at least `Button` per AC2 and `Label` per AC3; possibly nothing else if the audit confirms G5 / G7 stand.) |
| AC4 | `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, and `cargo build -p quartzite --no-default-features --features libm` all pass on the feature branch before merge. |
| AC5 | Every regenerated golden PNG passes the existing snapshot-equality test on Linux + Vulkan + macOS harness rows (where the harness is present); no platform-skipped golden is unintentionally introduced. |
| AC6 | The PR body links the audit doc and lists, per widget, the conformance rows the fix closed; lists the paint-API extension shape (K5 decision) and every `Painter` impl touched. |
| AC7 | The paint-API extension (Scope item 0) is a clean break per AGENTS.md § *API Stability*: every `Painter` impl in the workspace updates to the new signature in the same commit; no alias / no `pub use … as …` redirect / no parallel old method remains. All call sites of the previous single-axis `draw_text_in` variant are migrated. |
| AC8 | A new symbolic test (or extension to `quartzite-style/src/default_style_tests.rs`) asserts the vertical-centring contract for **both** `Button` and `Label` — independently of the golden PNG — so that a future renderer change cannot silently regress the centring without also breaking the golden. The test asserts on the recorded painter-call argument shape (vertical-axis alignment value) rather than on rect arithmetic at the call site, so it survives the PR #554 caller-side-recipe migration (K8). |
| AC9 | The `Painter` trait doc comments and the `# Examples` block in `quartzite-paint-api/src/painter.rs` document the new vertical-alignment parameter / type and its semantics, and pass `RUSTDOCFLAGS="-D warnings -D missing-docs"`. |

## Open questions

None unresolved after round-3 incorporation of the vertical-alignment-axis
scope addition. The concrete API shape (K5) is intentionally a design-phase
choice with constraints rather than a spec-level question — AGENTS.md
§ *spec-writer Optimization target* notes that questions resolvable by the
`design` Subagent via convention are not design-affecting at spec level.
Items deferred above (`min_size_hint` tokens, mock drift cleanup) remain
explicit follow-ups, not gaps in the current spec.
