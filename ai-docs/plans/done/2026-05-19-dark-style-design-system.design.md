# Design: Implement dark-style alongside DefaultStyle from design-system

**Issue:** #488
**Date:** 2026-05-19

## Approach

The deliverable is a **palette-only** dark theme that reuses `DefaultStyle` against
a new compile-time `Palette` constant, plus matching snapshot goldens. No new
`Style` impl, no changes to `DefaultStyle::paint`, no changes to
`quartzite-style/tests/support/mod.rs` (snapshot-helper sync group is untouched).

Visual rules (slot RGBA, derivation formulae unchanged, no new invariants) come
straight from `design-system/README.md` § *Dark theme* + `design-system/colors_and_type.css`
`[data-theme="dark"]` block (lines 88–117). The 11 hex values and their
3-decimal linear-float equivalents are pinned in the worked Rust example at
`design-system/README.md` lines 256–269; the design re-uses those numbers
verbatim, with no re-derivation.

### Concrete API form: `pub const DARK_PALETTE: Palette`

The spec offers a choice between `pub const DARK_PALETTE: Palette` and
`pub const fn Palette::dark() -> Self` and defaults to the constant. We adopt
the **constant** for three reasons:

1. **Call-site ergonomics:** `DARK_PALETTE` is one identifier; `Palette::dark()`
   is a call expression. Mirrors `Color::WHITE` / `Color::BLACK` / `Color::SKY_BLUE`
   precedent in `quartzite-paint-api` and the same crate already exposes
   `Palette::new()` as a `const fn` — adding `Palette::dark()` next to it would
   imply a forthcoming `Palette::light()` family that we do not plan to ship
   (light is `Palette::default()`).
2. **Doc surface is smaller:** a constant carries one doc block; an inherent
   `const fn` would need both a method doc and probably a sibling `# Examples`
   on `Palette` that documents the family. YAGNI.
3. **AC1 / AC3 satisfied identically:** both forms compile in a `const _: Palette =
   …;` context. The constant form does so without a method call.

The constant lives in a new module file `quartzite-style-types/src/dark_palette.rs`
and is re-exported from `quartzite-style-types/src/lib.rs`. We additionally
re-export it from `quartzite-style/src/lib.rs` (alongside the existing
`pub use quartzite_style_types::{ColorRole, Palette};` line) so application
code that already depends on `quartzite-style` does not need a fresh dependency
on `quartzite-style-types` just to get the dark seed — same pass-through model
the crate already uses for `Palette` itself.

### Snapshot-test wiring: thin helper, no parameterisation of existing fns

The existing 10 `*_renders` test fns in `quartzite-style/tests/snapshots.rs`
hard-code `&Palette::default()` (lines 49, 63, 77, 90, 104, 119, 133, 147, 161,
174). The spec leaves the wiring choice open. We pick the **thin per-test
helper** path: each dark-variant test is a separate `#[test]` fn (no `rstest`
parameterisation) that mirrors the corresponding light test verbatim except for
the palette argument and the snapshot name. Rationale:

- **Read-as-grep parity:** a reviewer searching for `dark_button_idle` finds
  exactly one fn; the light/dark split mirrors the file layout under
  `tests/snapshots/shared/`.
- **No changes to existing tests:** AC4 / AC9 forbid altering the light tests
  or `DefaultStyle`'s paint logic. A parameterised refactor would touch the
  existing fns; a free function helper does not.
- **No changes to `support/mod.rs`:** the snapshot-helper sync group stays
  untouched (AC6). The new helper lives **inside** `snapshots.rs` as a private
  `fn render_dark<F>(name: &str, build: F)` closure-receiver that owns the
  `harness_or_skip` + `render_widget` + `snapshot_assert` sequence.

A small private helper in `snapshots.rs` removes the per-test boilerplate but
**must not** move into `support/mod.rs` — the snapshot-helper sync group
contract (AGENTS.md Propagation Rule) requires lock-step changes to
`quartzite-widgets/tests/support/mod.rs`, which has no dark-theme use case.
Keeping the helper local to `snapshots.rs` confines the change to the style
crate.

### Rejected alternatives

- **Adding a `DarkStyle` struct in `quartzite-style`.** Rejected per spec § *Key
  decisions* row 1: the dark theme is *palette only*; `DefaultStyle`'s paint
  formulae already produce correct dark output against dark seeds (proved in
  `design-system/README.md` § *Derived state values follow the framework's same
  formulas*).
- **Putting `DARK_PALETTE` in `quartzite-style` instead of
  `quartzite-style-types`.** Rejected per spec § *Key decisions* row 6:
  `Palette` lives in `quartzite-style-types`, so the dark constant belongs in
  the same leaf crate. Re-exporting from `quartzite-style` is purely a
  convenience.
- **`Palette::dark() -> Self` inherent `const fn`.** Rejected for the
  ergonomics + doc-surface reasons listed above. Kept for future revisitation
  if a palette-family API (`Palette::high_contrast()`, etc.) emerges.
- **Parameterising the existing snapshot tests with `rstest`.** Rejected
  because it would force edits to the existing 10 light tests, violating the
  "no changes outside the new dark surface" reading of AC4 / AC9.
- **Putting dark goldens under `shared/dark/` subdir.** Rejected per spec §
  *Key decisions* row 7 and § *Open questions* row 3: `support/mod.rs` would
  need a new lookup branch, breaking the snapshot-helper sync-group contract.
- **`Color::from_hex` constructor.** Rejected: the spec § *Technical
  constraints* explicitly forbids it; numbers come from
  `design-system/README.md` already-computed 3-decimal linear floats.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add `DARK_PALETTE` const + module file. Create `quartzite-style-types/src/dark_palette.rs` containing `pub const DARK_PALETTE: Palette = Palette::new().with_role(…).…;` chain — 11 `with_role` links, one per `ColorRole`, using the 3-decimal linear floats from `design-system/README.md` lines 258–268 (e.g. `Color::new(0.169, 0.169, 0.169, 1.0)` for `#2B2B2B`; `Color::WHITE` for `HighlightedText`). Add `mod dark_palette;` + `pub use dark_palette::DARK_PALETTE;` to `quartzite-style-types/src/lib.rs`. Doc-comment per `ai-docs/doc-convention.md`: one-line summary + `# Examples` block exercising one role lookup, intra-doc links to `Palette` / `ColorRole`. | `quartzite-style-types/src/dark_palette.rs` (new), `quartzite-style-types/src/lib.rs` | — |
| 2 | Add unit tests for the dark palette invariants. Append two `#[test]` fns to the existing `#[cfg(test)] mod tests` in `quartzite-style-types/src/palette.rs` (NOT in `dark_palette.rs` — they exercise `DARK_PALETTE` against the same `Palette` invariants the existing `default_*` tests cover, and co-locating them keeps the two invariant tests of the file paired): `dark_palette_has_non_transparent_color_for_every_role` and `dark_palette_highlight_differs_from_highlighted_text`, modeled on the existing `default_has_non_transparent_color_for_every_role` (lines 152–161) and `default_highlight_differs_from_highlighted_text` (lines 163–172). The new tests need `use crate::DARK_PALETTE;` explicitly because `use super::*` only imports from the `palette` module, not the sibling `dark_palette` module. The compile-time AC3 assertion `const _: Palette = DARK_PALETTE;` lives inside `#[cfg(test)] mod tests` in `quartzite-style-types/src/dark_palette.rs` (amended per PR #490 reviewer; AC3 remains satisfied because `pub const DARK_PALETTE` itself is const-evaluated at the declaration site). | `quartzite-style-types/src/palette.rs`, `quartzite-style-types/src/dark_palette.rs` | 1 |
| 3 | Re-export `DARK_PALETTE` from `quartzite-style`. Add `pub use quartzite_style_types::DARK_PALETTE;` to the existing `pub use quartzite_style_types::{ColorRole, Palette};` line at `quartzite-style/src/lib.rs:42` (combine into a single brace group). Verifies that downstream code can pick up the dark seed via `use quartzite_style::DARK_PALETTE;`. | `quartzite-style/src/lib.rs` | 1 |
| 4 | Regenerate dark goldens. Add the 10 dark `#[test]` fns to `quartzite-style/tests/snapshots.rs` (see Test Design § *Dark snapshot tests* below); run `QUARTZITE_REGENERATE_SNAPSHOTS=1 scripts/update-snapshots.sh --crate style` to produce the PNGs under `quartzite-style/tests/snapshots/auto/dark_*.png` (or `<backend>/` if `WGPU_BACKEND` is set); manually `mv` them to `quartzite-style/tests/snapshots/shared/` per the bootstrap-shared-default convention in `support/mod.rs` lines 36–38. Visual-inspect each PNG against the design-system preview (cf. `design-system/preview/dark-*.html` per `README.md` line 224) — dark `Window` `#2B2B2B`, dark `Button` `#3C3C3C` (or its hover-blend `#345067` when hovered/pressed/checked), dark `Text` `#E8E8E8`. | `quartzite-style/tests/snapshots.rs`, `quartzite-style/tests/snapshots/shared/dark_*.png` (10 new) | 3 |
| 5 | Run the full local gate. `cargo build`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt -- --check`; `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`; `cargo build -p quartzite --no-default-features --features libm`. All must exit 0 (AC7 / AC8). Confirm no diff in `design-system/colors_and_type.css`, `design-system/README.md`, or `quartzite-style/tests/support/mod.rs` (AC6 / AC9). | (no source edits — gate run) | 4 |

## Handoff plan

`M = 5` (two groups, 3 + 2):

- **Group A:** subtasks 1–3 — palette artefact + unit tests + re-export (initial implementation chunk). Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at the start of this group so the implementer enters with a clean buffer carrying only the spec + design + AGENTS.md slice.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent /task resumes in Group B with fresh context.
- **Group B:** subtasks 4–5 — golden regeneration + full local gate. Terminal group (2 subtasks; within the 1..=3 range).

## Risks

- **Risk: 3-decimal linear floats round trip through wgpu and produce a different
  byte than the equivalent HTML mock pixel** (e.g. `Color::new(0.169, 0.169,
  0.169, 1.0)` could surface as `#2B2B2B` or `#2C2C2C` after sRGB encoding
  depending on backend gamma).
  - **Mitigation:** the golden is the cross-backend `shared/` fallback per
    `support/mod.rs` lines 36–38; we accept whatever wgpu produces as
    *the* dark golden and check against it. The `nv-flip` mean tolerance of
    `0.05` (`FLIP_TOLERANCE`, `support/mod.rs:58`) already absorbs sub-LSB
    rounding. The mocks in `design-system/preview/dark-*.html` are visual
    references for review-time eyeballing, not pixel-equal references.

- **Risk: shape of the `Palette::new().with_role(...)` chain breaks the const
  evaluator on older toolchains.**
  - **Mitigation:** the chain is identical in structure to existing
    `Palette::default()` body (`palette.rs` lines 65–76) plus the
    `default-fn` task chain shape; the workspace `rust-version` already
    supports `const fn` with `&mut self`-style mutation through `with_role`.
    No new const-eval feature is required. The `const _: Palette = DARK_PALETTE;`
    assertion in subtask 2 catches a regression at compile time.

- **Risk: snapshot dimensions or render-order break the `nv-flip` tolerance
  between local and CI.**
  - **Mitigation:** uses the same `harness_or_skip` (`support/mod.rs:82`) +
    `snapshot_assert` (`support/mod.rs:204`) plumbing as the existing 10 light
    tests, which already pass cross-backend via the `shared/` fallback. No
    new harness path is introduced. If a per-backend dark golden ends up
    diverging from `shared/`, the standard remediation (commit a
    `<backend>/dark_<name>.png` override) applies — but we do NOT pre-emptively
    commit per-backend variants in this PR.

- **Risk: doc-test for `DARK_PALETTE` becomes a maintenance hazard if a slot
  RGBA shifts.**
  - **Mitigation:** keep the `# Examples` block minimal — one role lookup
    (e.g. `assert_eq!(DARK_PALETTE.color(ColorRole::Highlight),
    Color::new(0.118, 0.564, 1.000, 1.0));`). One concrete value, not all 11.
    A slot change still updates one place.

- **Risk: a reviewer asks for the `Palette::dark()` form after the constant
  is merged.**
  - **Mitigation:** the spec § *Open questions* explicitly authorises either
    form. If the reviewer prevails, the constant body becomes the body of
    `pub const fn Palette::dark() -> Self { … }` (literally one wrapping
    change) and `DARK_PALETTE` becomes `pub const DARK_PALETTE: Palette =
    Palette::dark();` (or is removed) — a future PR, not blocking.

- **Risk: Propagation Rule misfire — design touches `quartzite-style/src/lib.rs`
  re-export line, which is *not* in the sync group, but the helper choice
  arguably could.**
  - **Mitigation:** the test-helper change is confined to `snapshots.rs` (not
    `support/mod.rs`), so the snapshot-helper sync group is genuinely untouched
    and the Propagation Rule does not fire. AC6 / AC9 + this design section
    document the boundary explicitly so a reviewer can verify by grep.

## Test Design

### Subtask 2 — palette invariants

- **Location:** `quartzite-style-types/src/palette.rs` `#[cfg(test)] mod tests`
  (appended to the existing module that already covers `default_*` invariants).
- **Entry point:** the new `DARK_PALETTE` constant.
- **Scenarios:**
  - `dark_palette_has_non_transparent_color_for_every_role` — loops
    `ColorRole::ALL`, asserts each `DARK_PALETTE.color(*role) !=
    Color::TRANSPARENT`. Same loop as the existing `default_*` analogue (lines
    152–161).
  - `dark_palette_highlight_differs_from_highlighted_text` — single
    `assert_ne!` on `(Highlight, HighlightedText)`. Same shape as `default_*`
    analogue (lines 163–172). Note: the dark seeds give Highlight `#1E90FF`
    and HighlightedText `#FFFFFF` — distinct.
- **Compile-time check:** `const _: Palette = DARK_PALETTE;` placed inside a
  `#[cfg(test)] mod tests` block in `quartzite-style-types/src/dark_palette.rs`
  (amended from "top-level non-test" per PR #490 reviewer request). AC3 is still
  satisfied: `pub const DARK_PALETTE: Palette = …` is itself evaluated in a
  `const` context at the declaration site, so the guard is belt-and-suspenders;
  moving it under `#[cfg(test)]` does not weaken AC3.
- **Import note:** the dark palette unit tests live in `palette.rs`'s
  `#[cfg(test)] mod tests`; that module's `use super::*` only imports from
  `palette`, so each new test fn needs `use crate::DARK_PALETTE;` explicitly.
- **Fixtures:** none. The tests are pure value lookups.

### Subtask 4 — dark snapshot tests

- **Location:** `quartzite-style/tests/snapshots.rs` (10 new `#[test]` fns
  appended after the existing `scroll_area_chrome_renders` at line 167).
- **Entry point:** `DefaultStyle.draw_widget(&w as &dyn AsWidget, painter,
  &DARK_PALETTE)` for each widget configuration. Wrapped in a private
  closure-receiving helper `fn render_dark<F: FnOnce(&mut dyn Painter)>(name:
  &str, render: F)` that owns the `harness_or_skip` + `render_widget` +
  `snapshot_assert` sequence (the snapshot name is e.g. `"dark_button_idle"`).
- **Scenarios — one per existing light golden** (mirroring `snapshots.rs`
  line-by-line; the *exact* widget setup is copy-pasted from the corresponding
  light test, only the palette and snapshot name change):
  | # | Test fn | Widget setup | Snapshot name |
  |---|---|---|---|
  | 4.1 | `dark_button_idle_renders` | `Button::new("OK".into())`, geometry only | `dark_button_idle` |
  | 4.2 | `dark_button_hovered_renders` | `set_hovered(true)` | `dark_button_hovered` |
  | 4.3 | `dark_button_pressed_renders` | `set_pressed(true)` | `dark_button_pressed` |
  | 4.4 | `dark_button_checked_renders` | `checked = true` | `dark_button_checked` |
  | 4.5 | `dark_button_focused_renders` | `set_focused(true)` | `dark_button_focused` |
  | 4.6 | `dark_button_disabled_renders` | `set_enabled(false)` | `dark_button_disabled` |
  | 4.7 | `dark_label_renders` | `Label::new("hi".into())` | `dark_label` |
  | 4.8 | `dark_scroll_area_chrome_renders` | `ScrollArea::new()` | `dark_scroll_area_chrome` |
  | 4.9 | `dark_text_edit_plain_renders` | `TextEdit::new()`, `plain_text = "abc"` | `dark_text_edit_plain` |
  | 4.10 | `dark_text_edit_read_only_renders` | `read_only = true` | `dark_text_edit_read_only` |
- **Fixtures / helpers:** the file-local `render_dark` helper described above;
  no shared-helper changes. Each test still uses the existing `canvas_rect()`
  (`snapshots.rs:37`) and `CANVAS = 64` constant.
- **Miri:** the `#![cfg(not(miri))]` at `snapshots.rs:19` already skips the
  entire file under Miri. New dark tests inherit this skip — no per-test
  `#[cfg_attr(miri, ignore = "…")]` needed.

### Doc-test for subtask 1

- `DARK_PALETTE` doc-block carries a `# Examples` doctest that exercises one
  role lookup (e.g. asserting `DARK_PALETTE.color(ColorRole::Highlight) ==
  Color::new(0.118, 0.564, 1.000, 1.0)`). Compiles in the standard `cargo test
  --doc` pass; covered by AC8.

## Open questions

- None. The two open-question items in the spec (constant-vs-`const fn`
  form, flat prefix vs subdir) are resolved in *Approach* (constant; flat
  prefix per § *Key decisions* row 7). The third open-question item
  (multi-palette-at-once) is out of scope per the spec § *Out of scope*.
