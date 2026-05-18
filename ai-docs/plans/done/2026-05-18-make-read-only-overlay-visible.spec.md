# Make the `read_only` overlay visible on `Palette::default`

**Source:** issue #458
**Date:** 2026-05-18
**Tracked in:** #458

## Scope

1. Replace the `read_only` overlay computation in `DefaultStyle`'s
   `Paint<TextEdit>` and `Paint<LineEdit>` impls so the field renders
   visibly distinct from a writable field on `Palette::default` —
   today `disabled(Window)` over `Base` composites identically to
   `Base` on the seeded palette (both are `Color::WHITE`).
2. Introduce a private `read_only_overlay(&Palette) -> Color` helper
   in `quartzite-style/src/default_style.rs` that returns
   `palette.color(ColorRole::WindowText).with_alpha(READ_ONLY_OVERLAY_ALPHA)`.
   Both `TextEdit` and `LineEdit` paint paths call it.
3. When `read_only`, draw the field's text at
   `palette.color(ColorRole::Text).with_alpha(READ_ONLY_TEXT_ALPHA)`
   instead of the full-alpha role colour. Applies to both widgets'
   non-placeholder text branch. (`LineEdit`'s placeholder branch
   keeps its existing `disabled(Text)` brush.)
4. Promote the two new alpha constants to module-level
   `const SCREAMING_SNAKE_CASE` per
   [`ai-docs/code-style.md` → Magic numbers](../code-style.md#magic-numbers):
   - `const READ_ONLY_OVERLAY_ALPHA: f32 = 0.10;`
   - `const READ_ONLY_TEXT_ALPHA: f32 = 0.65;`
5. Update the existing unit tests in
   `quartzite-style/src/default_style_tests.rs` that assert the
   read-only overlay's brush colour:
   - `text_edit_read_only_inserts_overlay_fill` (currently asserts
     `super::disabled(palette.color(ColorRole::Window))` at line ~304).
   - `line_edit_read_only_inserts_overlay` (asserts the same at
     line ~949).
   - `line_edit_read_only_with_placeholder_overlays_and_renders_placeholder`
     (asserts the same at line ~979).
   - Add at least one new assertion per widget covering the dimmed
     text brush (`Text.with_alpha(READ_ONLY_TEXT_ALPHA)`).
   - The total event counts (3 / 4) stay the same — no new
     painter events are emitted.
6. Regenerate the golden snapshot
   `quartzite-style/tests/snapshots/shared/text_edit_read_only.png`
   to capture the new render. The writable golden
   `text_edit_plain.png` is unaffected (the writable branch does
   not change).

## Out of scope

- Public API of `DefaultStyle`, `Paint`, `Style`, `Palette`, or
  `ColorRole` — none of those signatures change.
- The `disabled()` helper itself — kept as-is for the existing
  disabled-state and placeholder paths.
- The `Paint<Button>` / `Paint<Label>` / `Paint<ScrollArea>` /
  `Paint<Container>` impls — only `TextEdit` and `LineEdit`
  participate in this fix.
- A dark `Palette` preset — the dark palette work referenced in
  the issue's `colors_and_type.css` discussion is a separate
  effort. This task only ensures the default (light) palette
  renders the read-only state visibly, and that the chosen
  overlay derivation will re-derive correctly when a dark palette
  later ships.
- The `disabled` (non-read-only) state painting — unchanged.

## Deferred

- Adding `line_edit_read_only.png` as a new golden snapshot — the
  issue body marks it "if present"; the file does not exist today
  and the unit tests in `default_style_tests.rs` already cover the
  `LineEdit` read-only paint path. New golden is not required.
  Deferred for a follow-up issue if visual-regression coverage of
  the `LineEdit` read-only state becomes desirable. | rationale:
  unit tests already pin the brush colour; image gold is
  redundant for this fix's verification. | separate issue
  needed? yes, if pursued.

## Key decisions

| Question | Decision |
|---|---|
| Which colour role drives the overlay tint? | `ColorRole::WindowText` — the foreground role, chosen because it is guaranteed to carry contrast against `Window` and `Base` (by palette-design convention). Avoids the `Palette::default` failure mode where `Window == Base == WHITE`. |
| Overlay alpha? | `0.10` — yields a perceptible but subtle tint (light: `#FFFFFF` → `#E6E6E6`; dark proposed: `#1E1E1E` → `≈#323232`). Issue body fixes this value. |
| Read-only text alpha? | `0.65` — clearly dimmed against the tinted field without losing legibility. Issue body fixes this value. |
| Helper extraction shape? | Module-level `fn read_only_overlay(palette: &Palette) -> Color` (private, `#[inline]`, doc-commented), called from both `Paint<TextEdit>` and `Paint<LineEdit>`. Mirrors the `disabled` / `maybe_disabled` / `brush` helper pattern already in the file. |
| Magic-number policy for the two alphas? | Extract to module-level `const READ_ONLY_OVERLAY_ALPHA` and `const READ_ONLY_TEXT_ALPHA` per `ai-docs/code-style.md` § Magic numbers — both carry semantic meaning beyond their literal value. |
| Where to draw the read-only / writable text-brush branch in `LineEdit`? | Inside the existing `if … else …` that picks between placeholder and text. Add an `else if w.read_only` arm so the order is: empty-with-placeholder → read-only-text → writable-text. Mirror the same conditional shape in `Paint<TextEdit>` even though `TextEdit` has no placeholder. |
| Snapshot golden update strategy? | Regenerate only `text_edit_read_only.png`; do not add a new `line_edit_read_only.png`. (See *Deferred*.) `text_edit_plain.png` is left untouched. |
| Snapshot regeneration mechanism? | Follow the project's existing snapshot-update workflow (whichever env-var / cargo command the snapshot tests already document); no new tooling. Design agent confirms the exact incantation from `quartzite-style/tests/support/mod.rs`. |

## Technical constraints

- File touched: `quartzite-style/src/default_style.rs` (production code);
  `quartzite-style/src/default_style_tests.rs` (unit tests);
  `quartzite-style/tests/snapshots/shared/text_edit_read_only.png` (golden).
- The helper `read_only_overlay` and the two `const` declarations
  live in `default_style.rs` (module-private). No re-exports.
- Both helpers stay private — no `pub`. No new items appear in
  `quartzite-style`'s public surface.
- The fix MUST re-derive correctly for any future palette
  (including the dark palette in `design-system/`): every colour
  comes through `palette.color(ColorRole::*)`, never a hard-coded
  literal.
- `Color::with_alpha` is `const fn` — the constants can be
  `const`-initialised if needed in the future, but the helper
  itself returns a runtime-computed `Color` because `palette` is
  not known at compile time.
- Lint compliance: workspace `-D warnings`, `missing_docs = deny`,
  `clippy::undocumented_unsafe_blocks` — the new private helper
  needs a one-line `///` doc (private items don't trip
  `missing_docs` but the codebase convention is to document
  every helper).
- The design-system pointer skill (`design-system/SKILL.md` →
  `design-system/README.md`) applies because this changes the
  `DefaultStyle` paint path. Design agent should consult the
  design-system visual rules before finalising the alpha values
  if any conflict surfaces.
- `cargo build`, `cargo test --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo fmt -- --check`,
  `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`
  all pass.

## Acceptance Criteria

| #   | Criterion |
|-----|-----------|
| AC1 | On `Palette::default`, `DefaultStyle::draw_widget` of a `read_only` `TextEdit` produces a `FillRect` whose brush colour differs from `Color::WHITE` (the writable-field background). A unit-level assertion verifies the second `FillRect` brush equals `palette.color(ColorRole::WindowText).with_alpha(READ_ONLY_OVERLAY_ALPHA)`. |
| AC2 | On `Palette::default`, `DefaultStyle::draw_widget` of a `read_only` `LineEdit` produces a second `FillRect` whose brush equals `palette.color(ColorRole::WindowText).with_alpha(READ_ONLY_OVERLAY_ALPHA)` — same assertion shape as AC1, ported to `LineEdit`. |
| AC3 | Both `Paint<TextEdit>` and `Paint<LineEdit>` draw the read-only text with brush `palette.color(ColorRole::Text).with_alpha(READ_ONLY_TEXT_ALPHA)`. A unit assertion per widget covers this. The non-read-only (writable) text brush keeps full alpha. |
| AC4 | `LineEdit`'s placeholder branch is unchanged — when `text.is_empty() && !placeholder.is_empty()`, the brush is still `disabled(palette.color(ColorRole::Text))` regardless of `read_only`. Existing tests `line_edit_placeholder_drawn_when_text_empty` and `line_edit_read_only_with_placeholder_overlays_and_renders_placeholder` continue to pass after their overlay-brush expectation is updated to the new value. |
| AC5 | The painter event counts stay the same: 3 events for writable `TextEdit` / `LineEdit`, 4 events for `read_only`. No new draw calls are introduced; only brush values change. |
| AC6 | `quartzite-style/tests/snapshots/shared/text_edit_read_only.png` is regenerated and committed; the test pixel-matches the new golden. `text_edit_plain.png` is unchanged byte-for-byte. |
| AC7 | The two new alphas are declared as module-level `const READ_ONLY_OVERLAY_ALPHA: f32 = 0.10;` and `const READ_ONLY_TEXT_ALPHA: f32 = 0.65;`; no `0.10` or `0.65` literal appears inline in `default_style.rs`. |
| AC8 | A private helper `fn read_only_overlay(palette: &Palette) -> Color` exists in `default_style.rs`, is called from exactly the two read-only branches (`Paint<TextEdit>` and `Paint<LineEdit>`), and has a `///` doc comment. |
| AC9 | The fix re-derives for any palette: a unit test using a hand-built palette where `WindowText` is e.g. `Color::new(0.0, 0.5, 1.0, 1.0)` shows the overlay brush equals that colour at alpha `0.10`. |
| AC10 | `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, and the rustdoc gate from AGENTS.md (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) all pass. |

## Open questions

- None blocking. The exact snapshot-regeneration command is
  derivable by the design agent from `quartzite-style/tests/support/mod.rs`
  and need not be pinned at spec time.
