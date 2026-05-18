# Design: Make the `read_only` overlay visible on `Palette::default`

**Issue:** #458
**Spec:** [`ai-docs/plans/2026-05-18-make-read-only-overlay-visible.spec.md`](2026-05-18-make-read-only-overlay-visible.spec.md)
**Date:** 2026-05-18

## Approach

The change is a localised, behavioural fix to `quartzite-style/src/default_style.rs`. The current `disabled(Window)` overlay composites identically to `Base` on `Palette::default` (both seed to `Color::WHITE`), making read-only fields visually indistinguishable from writable ones. The spec, the design-system proposal at `design-system/proposals/text-edit-read-only-overlay.md`, and the design-system README's *Transparency & blur* / *Dark theme* sections all converge on a single solution: switch the overlay's source role from the **background** (`Window`) to the **foreground** (`WindowText`), at a small alpha (`0.10`), so the overlay derives from a role guaranteed by palette-design convention to contrast with `Base`. Read-only text is dimmed via `Text.with_alpha(0.65)` so it reads as "inert" against the tinted field.

**Why this approach (chosen):**
- Re-derives correctly for every palette — both light (`#FFFFFF` over `#FFFFFF` → `#E6E6E6`) and the dark seeds in `design-system/colors_and_type.css` (`#E8E8E8` α 0.10 over `#1E1E1E` → ≈`#323232`). No literal colour leaks into `default_style.rs`.
- Zero public-API surface change. `DefaultStyle`, `Paint`, `Style`, `Palette`, `ColorRole` signatures are untouched.
- Same painter-event count: 3 (writable) / 4 (read-only). No new fill/draw calls — only brush *values* change. AC5 explicitly pins this invariant.
- Mirrors the existing `disabled` / `maybe_disabled` / `brush` helper pattern in the file — a new module-private `read_only_overlay(&Palette) -> Color` helper called from both paint impls keeps the two read-only branches in lock-step.
- Magic-number policy honoured: the two alphas (`0.10`, `0.65`) carry semantic meaning beyond their literal value, so they go to module-level `const READ_ONLY_OVERLAY_ALPHA` / `const READ_ONLY_TEXT_ALPHA` per `ai-docs/code-style.md` § *Magic numbers*.

**Rejected alternatives:**
1. **Keep `Window` as the source role but use `with_alpha(0.10)` directly.** Rejected because the spec's *Key decisions* row explicitly identifies the failure mode: `Window == Base == WHITE` on `Palette::default`, so any alpha applied to `Window` over `Base` still composites to `Base`. The role choice — not the alpha value — is what makes the overlay visible.
2. **Reuse `disabled()` with a different alpha factor.** Rejected because `disabled()` is semantically the *disabled-state* α-half operator (used by `maybe_disabled`, the placeholder branch in `Paint<LineEdit>`, and `disabled_button_halves_fill_and_text_alpha`). Overloading it with a parameter would either bend its contract or require renaming. A separate `read_only_overlay` helper keeps the two state axes (disabled vs. read-only) independent.
3. **Inline both literals and skip the helper.** Rejected because the same two lines would appear in both `Paint<TextEdit>` and `Paint<LineEdit>`, violating the helper-extraction shape decision in the spec. AC8 requires the helper to exist and be called from exactly the two read-only branches.
4. **Add a new `line_edit_read_only.png` golden.** Rejected per the spec's *Deferred* section — unit tests in `default_style_tests.rs` already pin the `LineEdit` read-only brush colour. Visual-regression coverage of that state is a follow-up issue if it becomes desirable.
5. **Switch text dimming from `with_alpha(0.65)` to `disabled(Text)`.** Rejected because `disabled` is `× 0.5`, indistinguishable from the disabled-state text. `0.65` reads as "dimmed but readable", `0.50` as "disabled". The spec fixes `0.65`.

**Design-system consultation (per AGENTS.md trigger for `DefaultStyle` paint-path changes):**
- `design-system/README.md` § *Transparency & blur* describes the old `Window.with_alpha(0.5)` overlay as the only transparency use in `DefaultStyle`. The change replaces *that exact rule* with `WindowText.with_alpha(0.10)`; the *Dark theme* table will need a follow-up update to the "Read-only" derivation row (out of scope here; the README is descriptive of the source, and the source is what changes).
- `design-system/proposals/text-edit-read-only-overlay.md` is the proposal this spec implements; the diff section there matches the spec's intent exactly.
- No conflict surfaced with the design-system visual rules (flat fill, no gradients, no shadows, 1 px outline) — the change is an overlay-brush colour swap only.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Update unit tests in `default_style_tests.rs` to assert the new overlay brush (`WindowText.with_alpha(READ_ONLY_OVERLAY_ALPHA)`) and the new dimmed text brush (`Text.with_alpha(READ_ONLY_TEXT_ALPHA)`). Tests will FAIL after this commit until task 2 lands — this is TDD per AGENTS.md *Workflow* ("Tests before prod code"). | `quartzite-style/src/default_style_tests.rs` | — |
| 2 | Implement the production change in `default_style.rs`: add the two `const` alphas, add the `read_only_overlay` helper, swap the overlay brush in both `Paint<TextEdit>` and `Paint<LineEdit>`, and add the read-only text-brush branch in both impls. Tests from task 1 now pass. | `quartzite-style/src/default_style.rs` | 1 |
| 3 | Regenerate the `text_edit_read_only.png` golden via `scripts/update-snapshots.sh --crate style`, then `mv` the per-backend PNG to `tests/snapshots/shared/`. Verify `text_edit_plain.png` is byte-unchanged. Run the snapshot test to confirm it pixel-matches the new golden under the configured FLIP tolerance (`0.05`). | `quartzite-style/tests/snapshots/shared/text_edit_read_only.png` | 2 |
| 4 | Run the full gate suite (`cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`). Fix any incidental warnings surfaced by the new helper's doc / lint posture. | (verification only — no edits expected) | 3 |

## Handoff plan

`M = 4` subtasks. With the cap of 3 subtasks per group and a 1..=3 terminal group, the split is 3 + 1.

- **Group A:** subtasks 1–3 — tests-first, production change, snapshot regeneration. Three subtasks, the non-terminal cap. Group A's `/context-reset` subagent runs subtasks 1–3 per `.claude/skills/context-reset/SKILL.md` § *Compaction recovery (re-entry)*.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § *Compaction recovery (re-entry)*. Parent `/task` resumes in Group B with fresh context.
- **Group B:** subtask 4 — full-gate verification. Terminal group (1 subtask; within the 1..=3 range). Group B's `/context-reset` subagent completes Step 8.

## Risks

- **Risk: snapshot regeneration on a non-default backend produces drift.** Mitigation: `scripts/update-snapshots.sh` writes to `tests/snapshots/<backend>/` by default; subtask 3 explicitly `mv`s the per-backend PNG into `shared/` per the script's bootstrap instructions and the `tests/support/mod.rs` § *Per-backend dir + shared fallback* contract. Only one backend's pixels are committed; the FLIP tolerance (`0.05`) absorbs cross-backend rounding drift on subsequent CI runs.
- **Risk: the snapshot test environment lacks a GPU adapter.** Mitigation: `harness_or_skip` returns early with an `eprintln!` notice when no adapter is available — the test passes without comparison. Regeneration requires a working GPU; the implementing engineer must run it on a host with one. If `SKIP_RENDER_SNAPSHOT=1` is in env, the test will skip; the engineer must unset it for regeneration.
- **Risk: an existing assertion outside the three named tests references the old `disabled(palette.color(ColorRole::Window))` brush.** Mitigation: spec § 5 enumerates the three call sites (lines ~304, ~949, ~979); a `grep -n "disabled(palette.color(ColorRole::Window))" quartzite-style/src/` confirms no other occurrences before subtask 1 commits. The placeholder branch's `disabled(palette.color(ColorRole::Text))` (line ~906) is intentionally NOT touched — AC4 pins it.
- **Risk: doc lints (`missing_docs = deny`, `rustdoc::broken_intra_doc_links = deny`) reject the new helper / consts.** Mitigation: every new item gets a one-line `///` per `ai-docs/code-style.md` § *Documentation*. The helper is module-private (`fn`, no `pub`), so `missing_docs` does not technically apply — but the codebase convention is to document every helper anyway. The two `const` items are also private; same convention applies.
- **Risk: `cargo clippy --all-targets` flags the new helper as `clippy::missing_inline_in_public_items` or `clippy::trivially_copy_pass_by_ref`.** Mitigation: the helper is private (no public-items lint), takes `&Palette` (the existing `brush`/`disabled` helpers in the same file pass `&Palette` / take a `Color` by value — same convention). Adding `#[inline]` per the spec's *Key decisions* and `ai-docs/code-style.md` § *`#[inline]` and the `_Simple._` doc tag* — the helper body is a single non-branching expression, so it is "recursively simple" and gets `#[inline]` (concrete fn, not a trait method).
- **Risk: AGENTS.md *Code Style* § *File size* hard cap (1000 lines) breached by the test file (currently 1016 lines).** Mitigation: the test file is already over the 800-line soft cap but exempt from the hard cap because its body is entirely `#[cfg(test)]` (excluded per *File size*). Adding ~10-20 more assertions does not change that exemption. No refactor required.
- **Risk: design-system docs drift.** Mitigation: the design-system *Transparency & blur* paragraph and the *Dark theme* table's "Read-only" row describe the OLD overlay rule. A follow-up doc-only PR will be needed; this is out of scope per the spec (the spec touches code + goldens, not `design-system/`). Recorded here as a known follow-up, not a blocker.
- **Risk: a future palette where `WindowText == Window` would re-introduce the same invisible-overlay failure mode.** Mitigation: both invariants documented at the top of `design-system/README.md` § *Dark theme* require `Highlight ≠ HighlightedText` but not `WindowText ≠ Window`. AC9 already requires a unit test pinning a custom `WindowText` colour through; that test catches the helper's *derivation* but not the *contrast invariant*. Recorded here as a residual risk that is out of scope for this fix — the palette contract is owned elsewhere. The fix is strictly better than the status quo (`Window` source) on every realistic palette including both seeds in `design-system/colors_and_type.css`.

## Test Design

Test file: `quartzite-style/src/default_style_tests.rs` (sibling module attached via `#[cfg(test)] #[path = "default_style_tests.rs"] mod tests;` in `default_style.rs`; `super::` resolves to `crate::default_style`).

### Tests to UPDATE (assertion-only changes; structure preserved)

#### `text_edit_read_only_inserts_overlay_fill` (line ~290)
- **Change:** replace `let expected_overlay = super::disabled(palette.color(ColorRole::Window));` with the new helper-derived value: `let expected_overlay = palette.color(ColorRole::WindowText).with_alpha(super::READ_ONLY_OVERLAY_ALPHA);` (asserting against the helper's output without calling the helper directly, since AC8 requires the helper to be exercised by the production code path, not the test).
- Event count assertion `painter.events.len() == 4` is unchanged.
- The `matches!(&painter.events[1], PaintEvent::FillRect { brush, .. } if brush_color(brush) == expected_overlay)` shape is unchanged — only `expected_overlay`'s RHS changes.

#### `line_edit_read_only_inserts_overlay` (line ~929)
- **Change:** same swap on line ~949 — replace `super::disabled(palette.color(ColorRole::Window))` with `palette.color(ColorRole::WindowText).with_alpha(super::READ_ONLY_OVERLAY_ALPHA)`.
- Event count `== 4` unchanged. Helper functions (`line_edit_read_only_palette`) unchanged — they still pin `Window` for the OLD assertion's predictability, but the NEW assertion derives from `WindowText` (which `Palette::default` seeds to `BLACK` and the test palette inherits without override). Optionally tighten the test palette to pin `WindowText` to a non-default colour for parity with AC9 — see new test below.

#### `line_edit_read_only_with_placeholder_overlays_and_renders_placeholder` (line ~963)
- **Change:** same swap on line ~979 (the overlay assertion).
- The placeholder text assertion on line ~983 (`brush_color(brush) == super::disabled(palette.color(ColorRole::Text))`) is **unchanged** — AC4 pins the placeholder branch to `disabled(Text)` regardless of `read_only`.
- Event count `== 4` unchanged.

### Tests to ADD

#### `text_edit_read_only_dims_text` (new — AC3 for `TextEdit`)
- **Location:** alongside `text_edit_read_only_inserts_overlay_fill` in `default_style_tests.rs`.
- **Entry point:** `DefaultStyle.draw_widget(&edit, &mut painter, &palette)` with `edit.read_only = true` and `edit.plain_text = "abc"`.
- **Scenario:** assert that the last event is `PaintEvent::DrawTextIn` whose brush is `palette.color(ColorRole::Text).with_alpha(super::READ_ONLY_TEXT_ALPHA)`. Event count `== 4`.
- **Fixture:** uses `Palette::default()`; no new helper palette needed.

#### `text_edit_writable_keeps_full_alpha_text` (new — AC3 negative path for `TextEdit`)
- Verifies the writable branch is unchanged: with `edit.read_only = false`, the text brush is `palette.color(ColorRole::Text)` at full alpha (i.e., `.a() == 1.0`).
- Guards against accidental application of the dimming to the writable branch (a regression class easy to introduce by mis-placing the conditional).

#### `line_edit_read_only_dims_text` (new — AC3 for `LineEdit`)
- **Location:** alongside `line_edit_read_only_inserts_overlay` in `default_style_tests.rs`.
- **Entry point:** `DefaultStyle.draw_widget(&e, &mut painter, &palette)` with `e.read_only = true` and `e.text = "abc"` (non-empty, so the placeholder branch does NOT fire).
- **Scenario:** assert `events[3]` is `DrawTextIn { brush, .. }` with `brush_color(brush) == palette.color(ColorRole::Text).with_alpha(super::READ_ONLY_TEXT_ALPHA)`.
- **Fixture:** `line_edit_read_only_palette()`, possibly extended to pin `Text` to a non-default colour for stronger pinning.

#### `line_edit_read_only_empty_text_dims_text` (new — covers the `read_only` branch when `text.is_empty() && placeholder.is_empty()`)
- Mirrors `line_edit_read_only_inserts_overlay` (which asserts the overlay) but additionally asserts the text brush at `events[3]` is the dimmed `Text`.
- Differentiates from `line_edit_read_only_with_placeholder_overlays_and_renders_placeholder` (which exercises the placeholder branch with `disabled(Text)`).

#### `line_edit_writable_keeps_full_alpha_text` (new — AC3 negative path for `LineEdit`)
- Writable + non-empty text: text brush is `palette.color(ColorRole::Text)` at full alpha. Mirrors `text_edit_writable_keeps_full_alpha_text` on the `LineEdit` widget.

#### `read_only_overlay_derives_from_custom_window_text` (new — AC9: re-derives for any palette)
- **Location:** new section near the bottom of `default_style_tests.rs`.
- **Entry point:** build a `Palette` with `ColorRole::WindowText` overridden to `Color::new(0.0, 0.5, 1.0, 1.0)`; render a `read_only` `TextEdit`; assert the overlay brush is exactly `Color::new(0.0, 0.5, 1.0, super::READ_ONLY_OVERLAY_ALPHA)`.
- **Scenario:** confirms the helper does **not** hard-code any literal colour and re-derives correctly through the `Palette::color(ColorRole::WindowText)` lookup. Catches a regression where a future change inadvertently swaps the role.
- **Fixture:** `Palette::default().with_role(ColorRole::WindowText, Color::new(0.0, 0.5, 1.0, 1.0))`.

### Tests that MUST remain unchanged (AC4 / regression guards)

- `text_edit_records_fill_outline_and_text` — writable `TextEdit`, 3 events, `Base` fill. Unchanged.
- `line_edit_records_fill_outline_and_empty_text` — empty writable `LineEdit`, 3 events, full-α `Text` brush. Unchanged.
- `line_edit_records_text_when_non_empty` — non-empty writable `LineEdit`, full-α `Text` brush. Unchanged.
- `line_edit_placeholder_drawn_when_text_empty` — writable + placeholder, `disabled(Text)` brush. Unchanged.
- `line_edit_non_empty_text_ignores_placeholder` — writable + text + placeholder, full-α `Text`. Unchanged.

### Snapshot test (integration)

- File: `quartzite-style/tests/snapshots.rs`, test `text_edit_read_only_renders` (line ~106).
- **No code change** to the test body. The change is to the golden PNG.
- Regeneration command: `scripts/update-snapshots.sh --crate style` (auto-detects backend per `uname -s` → `vulkan` on Linux). Then `mv quartzite-style/tests/snapshots/<backend>/text_edit_read_only.png quartzite-style/tests/snapshots/shared/text_edit_read_only.png`.
- Verification: re-run `cargo test -p quartzite-style --test snapshots text_edit_read_only_renders` without `QUARTZITE_REGENERATE_SNAPSHOTS=1` set; expect pass with `mean FLIP <= 0.05`.
- `text_edit_plain.png` must remain byte-identical — verify with `git diff --stat quartzite-style/tests/snapshots/shared/text_edit_plain.png` reporting no change after regeneration.

### Fixture helpers (no changes required)

- `RecordingPainter` (line ~57) — existing, no change.
- `brush_color` (line ~157) — existing, no change.
- `first_fill` / `first_draw_text_in` / `first_draw_rect` (lines ~131–~150) — existing, no change.
- `line_edit_palette` / `line_edit_read_only_palette` — existing; new tests may reuse them as-is.

## Open questions

None blocking. The exact snapshot-regeneration incantation is fixed by `scripts/update-snapshots.sh` and `tests/support/mod.rs` § *Per-backend dir + shared fallback*. The follow-up doc update to `design-system/README.md` (§ *Transparency & blur*, § *Dark theme* "Read-only" row) is out of scope per the spec's *Out of scope* list and tracked implicitly via the design-system proposal at `design-system/proposals/text-edit-read-only-overlay.md` becoming a "shipped" rather than "proposed" rule.
