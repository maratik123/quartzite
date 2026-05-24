# Design-system conformance audit

**Date:** 2026-05-24
**Branch:** feat/2026-05-24-design-system-conformance
**Design system:** design-system/README.md + design-system/preview/*.html + design-system/proposals/*.md

Rules checked per widget:

- **Text vertical alignment** — v_align argument to `Painter::draw_text_in`
- **Text horizontal alignment** — h_align argument to `Painter::draw_text_in`
- **Background colour** — fill role + state group
- **Outline/border colour** — pen role + state group, width
- **Focus ring** — presence, 2 px width, `FocusRing × Normal` colour
- **Disabled state** — α × 0.5 modifier
- **Hover/pressed/checked state derivations** — fill + text role swap
- **Font family, size, weight** — `WidgetBase::font` usage
- **Padding / min-size** — deferred per K4; gaps recorded as N/A or ❌

---

## Button

| Rule | Source | Code reference | Status | Note |
|---|---|---|---|---|
| Text vertical alignment: Center | `design-system/preview/comp-button-anatomy.html` (`align-items: center`) | `quartzite-style/src/default_style/mod.rs:237–244` | ✅ | Fixed in subtask 2, commit 8fd1303. Painter called with `v_align = Alignment::Center`. |
| Text horizontal alignment: Center | `design-system/preview/comp-button-anatomy.html` (`justify-content: center`) | `quartzite-style/src/default_style/mod.rs:242` | ✅ | `h_align = Alignment::Center`. |
| Background colour: `Button × state_group` | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:218` | ✅ | Idle → `Button × Normal`; hover → `Button × Hover`; pressed/checked → `Highlight × Pressed/Normal`. |
| Outline/border: 1 px `ButtonText`/`HighlightedText` (idle/pressed) | `design-system/preview/comp-button-anatomy.html` (`border: 1px solid #000000`) | `quartzite-style/src/default_style/mod.rs:232–236` | ✅ | 1 px `ButtonText` at rest; pressed/checked swaps to `HighlightedText` (white). |
| Focus ring: 2 px `FocusRing × Normal` | `design-system/README.md § Borders & strokes` | `quartzite-style/src/default_style/mod.rs:222–229` | ✅ | `FOCUS_RING_WIDTH = 2.0`; role = `FocusRing × Normal`. |
| Disabled state: α × 0.5 | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:218–219` + `mod.rs:366–374` (`maybe_disabled`) | ✅ | `maybe_disabled` halves alpha of fill and text colours when `!enabled`. |
| Hover fill derivation: `Button × Hover` | `design-system/README.md § Animation, hover, press` | `quartzite-style/src/default_style/mod.rs:209, 218` | ✅ | `state_group` → `ColorGroup::Hover`; palette entry pre-computed at construction via `blend(WindowText, 0.06)`. |
| Pressed fill + text: `Highlight × Pressed` + `HighlightedText` | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:213–217` | ✅ | `pressed` → fill = `Highlight × Pressed`, text = `HighlightedText`. |
| Checked fill: `Highlight × Normal` + `HighlightedText` | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:213–217` | ✅ | `w.checked` → same branch as pressed for fill role. |
| Font: `WidgetBase::font` | `design-system/README.md § Type` | `quartzite-style/src/default_style/mod.rs:202` | ✅ | `w.widget_base().font.clone()`. |
| Padding / min-size | `design-system/preview/comp-button-anatomy.html` (`padding: 6px 16px; min-width: 64px; min-height: 32px`) | N/A — layout concern | N/A | Deferred per K4. `draw_text_in(geom, …)` fills the full rect; no internal padding token. Follow-up issue needed. |

---

## Label

| Rule | Source | Code reference | Status | Note |
|---|---|---|---|---|
| Text vertical alignment: Center | `design-system/README.md § WIDGET SPECS` (K3 decision — single-line inputs centre vertically) | `quartzite-style/src/default_style/mod.rs:278–285` | ✅ | Fixed in subtask 2, commit 8fd1303. `v_align = Alignment::Center`. Diverges from `comp-label.html` mock (which is top-anchored); spec K3 rules that impl is authoritative. |
| Text horizontal alignment: `w.alignment` | `design-system/preview/comp-label.html` (left/center/right variants shown) | `quartzite-style/src/default_style/mod.rs:283` | ✅ | `h_align = w.alignment`; defaults to `Alignment::Left`. |
| Background colour: `Window × state_group` | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:263` | ✅ | Idle/hover → `Window × Normal/Hover`; pressed → `Highlight × Pressed`. |
| Outline/border: none at idle; 2 px FocusRing when focused | `design-system/README.md § Borders & strokes` | `quartzite-style/src/default_style/mod.rs:267–277` | ✅ | `draw_rect` only emitted when `focused`. |
| Focus ring: 2 px `FocusRing × Normal` | `design-system/README.md § Borders & strokes` | `quartzite-style/src/default_style/mod.rs:268–276` | ✅ | `FOCUS_RING_WIDTH = 2.0`; role = `FocusRing × Normal`. |
| Disabled state: α × 0.5 | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:263–264` (`maybe_disabled`) | ✅ | `maybe_disabled` on both fill and text. |
| Hover fill derivation: `Window × Hover` | `design-system/README.md § Animation, hover, press` | `quartzite-style/src/default_style/mod.rs:257, 263` | ✅ | `ColorGroup::Hover`; derived via `blend(WindowText, 0.06)`. |
| Pressed fill + text: `Highlight × Pressed` + `HighlightedText` | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:258–261` | ✅ | `pressed` → `Highlight`, `HighlightedText`. |
| Font: `WidgetBase::font` | `design-system/README.md § Type` | `quartzite-style/src/default_style/mod.rs:251` | ✅ | `w.widget_base().font.clone()`. |
| Padding / min-size | `design-system/preview/comp-label.html` (no explicit padding token in mock) | N/A | N/A | No padding token for Label. Deferred per K4. |
| Mock drift: `comp-label.html` shows top-anchored text | `design-system/preview/comp-label.html` | `quartzite-style/src/default_style/mod.rs:278–285` | N/A | Documentation-only drift — the HTML mock is not a code conformance rule. Impl centres vertically per spec K3 (authoritative). Mock cleanup is deferred (spec Deferred table). |

---

## LineEdit

| Rule | Source | Code reference | Status | Note |
|---|---|---|---|---|
| Text vertical alignment: Center | `design-system/preview/comp-button-anatomy.html` (`min-height: 28px`, single-line field) + spec G4 | `quartzite-style/src/default_style/line_edit.rs:75–82` | ✅ | Fixed in subtask 3, commit 61276a1. Replaced PR #554 smaller-rect recipe with `draw_text_in(geom, …, Left, Center)`. |
| Text horizontal alignment: Left | `design-system/preview/comp-line-edit.html` (text-align: left implied) | `quartzite-style/src/default_style/line_edit.rs:80` | ✅ | `h_align = Alignment::Left`. |
| Background colour: `Base × state_group` | `design-system/README.md § Color` | `quartzite-style/src/default_style/line_edit.rs:39–40` | ✅ | Idle/hover → `Base × Normal/Hover`; pressed → `Highlight × Pressed`. |
| Outline/border: 1 px `Text` / `FocusRing` | `design-system/preview/comp-line-edit.html` (`border: 1px solid #000000`) | `quartzite-style/src/default_style/line_edit.rs:48–60` | ✅ | 1 px `Text` at rest; 2 px `FocusRing` when focused. |
| Focus ring: 2 px `FocusRing × Normal` | `design-system/README.md § Borders & strokes` | `quartzite-style/src/default_style/line_edit.rs:47–54` | ✅ | `FOCUS_RING_WIDTH = 2.0`. |
| Disabled state: α × 0.5 | `design-system/README.md § Color` | `quartzite-style/src/default_style/line_edit.rs:39–41` (`maybe_disabled`) | ✅ | `maybe_disabled` applied to all resolved colours. |
| Hover fill derivation: `Base × Hover` | `design-system/README.md § Animation, hover, press` | `quartzite-style/src/default_style/line_edit.rs:26, 39` | ✅ | `ColorGroup::Hover` selected via `state_group`. |
| Pressed fill + text: `Highlight × Pressed` + `HighlightedText` | `design-system/README.md § Color` | `quartzite-style/src/default_style/line_edit.rs:27–31` | ✅ | `pressed` → fill `Highlight × Pressed`, text `HighlightedText`. |
| Read-only overlay: `WindowText × READ_ONLY_OVERLAY_ALPHA` | `design-system/proposals/text-edit-read-only-overlay.md` | `quartzite-style/src/default_style/line_edit.rs:44–46` | ✅ | `read_only_overlay(palette)` fills `WindowText × 0.10`. |
| Read-only text dim: `Text × READ_ONLY_TEXT_ALPHA` | `design-system/proposals/text-edit-read-only-overlay.md` | `quartzite-style/src/default_style/line_edit.rs:68–73` | ✅ | `text_color.with_alpha(READ_ONLY_TEXT_ALPHA)` when `w.read_only`. |
| Placeholder: half-alpha `Text` | `design-system/preview/comp-line-edit.html` (`.qz-line-edit.placeholder { color: rgba(0,0,0,0.5) }`) | `quartzite-style/src/default_style/line_edit.rs:65–66` | ✅ | `disabled(text_color)` = α × 0.5. |
| Caret: 1 px `Text`, full line-box height, 530 ms blink | `design-system/proposals/caret-and-selection.md` | `quartzite-style/src/default_style/line_edit.rs:paint_caret_line_edit` | ✅ | Addressed in PR #554. |
| Selection: `Highlight × Normal` fill + `HighlightedText` overdraw | `design-system/proposals/caret-and-selection.md` | `quartzite-style/src/default_style/line_edit.rs:paint_selection_line_edit` | ✅ | Addressed in PR #554. |
| Font: `WidgetBase::font` | `design-system/README.md § Type` | `quartzite-style/src/default_style/line_edit.rs:20` | ✅ | `w.widget_base().font.clone()`. |
| Padding / min-size | `design-system/preview/comp-line-edit.html` (`padding: 4px 6px; min-width: 180px; min-height: 28px`) | N/A — layout concern | N/A | Deferred per K4. |

---

## TextEdit

| Rule | Source | Code reference | Status | Note |
|---|---|---|---|---|
| Text vertical alignment: Left (top) | `design-system/preview/comp-text-edit.html` (`vertical-align: top`) + spec G5 | `quartzite-style/src/default_style/text_edit.rs` (main draw call) | ✅ | Fixed in subtask 2, commit 8fd1303. Explicit `v_align = Alignment::Left` (top). Aligns with `comp-text-edit.html` mock. |
| Text horizontal alignment: Left | `design-system/preview/comp-text-edit.html` (default text-align) | `quartzite-style/src/default_style/text_edit.rs` | ✅ | `h_align = Alignment::Left`. |
| Background colour: `Base × state_group` | `design-system/README.md § Color` | `quartzite-style/src/default_style/text_edit.rs` | ✅ | Idle → `Base × Normal`; hover/pressed follow state group. |
| Outline/border: 1 px `Text` / 2 px `FocusRing` | `design-system/preview/comp-text-edit.html` (`border: 1px solid #000000`) | `quartzite-style/src/default_style/text_edit.rs` | ✅ | 1 px `Text` at rest; 2 px `FocusRing` when focused. |
| Focus ring: 2 px `FocusRing × Normal` | `design-system/README.md § Borders & strokes` | `quartzite-style/src/default_style/text_edit.rs` | ✅ | `FOCUS_RING_WIDTH = 2.0`. |
| Disabled state: α × 0.5 | `design-system/README.md § Color` | `quartzite-style/src/default_style/text_edit.rs` (`maybe_disabled`) | ✅ | Applied to all resolved colours. |
| Hover fill derivation: `Base × Hover` | `design-system/README.md § Animation, hover, press` | `quartzite-style/src/default_style/text_edit.rs` | ✅ | `ColorGroup::Hover` via `state_group`. |
| Pressed fill + text: `Highlight × Pressed` + `HighlightedText` | `design-system/README.md § Color` | `quartzite-style/src/default_style/text_edit.rs` | ✅ | `pressed` → `Highlight × Pressed`, `HighlightedText`. |
| Read-only overlay: `WindowText × READ_ONLY_OVERLAY_ALPHA` | `design-system/proposals/text-edit-read-only-overlay.md` | `quartzite-style/src/default_style/text_edit.rs` | ✅ | `read_only_overlay(palette)` applied when `w.read_only`. |
| Read-only text dim: `Text × READ_ONLY_TEXT_ALPHA` | `design-system/proposals/text-edit-read-only-overlay.md` | `quartzite-style/src/default_style/text_edit.rs` | ✅ | `text_color.with_alpha(READ_ONLY_TEXT_ALPHA)` when `w.read_only`. |
| Caret: 1 px `Text`, full line-box height, 530 ms blink | `design-system/proposals/caret-and-selection.md` | `quartzite-style/src/default_style/text_edit.rs` (`paint_caret_text_edit`) | ✅ | Addressed in PR #553. |
| Selection: `Highlight × Normal` fill + `HighlightedText` overdraw (multi-line) | `design-system/proposals/caret-and-selection.md` | `quartzite-style/src/default_style/text_edit.rs` (`paint_selection_text_edit`) | ✅ | Per-visual-line rects, no inter-line gap. Addressed in PR #553. |
| Font: `WidgetBase::font` | `design-system/README.md § Type` | `quartzite-style/src/default_style/text_edit.rs` | ✅ | `w.widget_base().font.clone()`. |
| Padding / min-size | `design-system/preview/comp-text-edit.html` (`padding: 4px 6px; min-width: 220px; min-height: 72px`) | N/A — layout concern | N/A | Deferred per K4. |

---

## ScrollArea

| Rule | Source | Code reference | Status | Note |
|---|---|---|---|---|
| Text vertical alignment | N/A — ScrollArea does not draw text | N/A | N/A | No `draw_text_in` call in `Paint<ScrollArea>`. |
| Text horizontal alignment | N/A | N/A | N/A | No text drawn. |
| Background colour: `Base × state_group` | `design-system/README.md § Color` + `proposals/scrollbar.md` | `quartzite-style/src/default_style/mod.rs:303–305` | ✅ | Idle/hover → `Base × Normal/Hover`; pressed → `Highlight × Pressed`. |
| Outline/border: 1 px `WindowText` / 2 px `FocusRing` | `design-system/README.md § Borders & strokes` | `quartzite-style/src/default_style/mod.rs:310–321` | ✅ | 1 px `WindowText` at rest; 2 px `FocusRing` when focused. |
| Focus ring: 2 px `FocusRing × Normal` | `design-system/README.md § Borders & strokes` | `quartzite-style/src/default_style/mod.rs:310–316` | ✅ | `FOCUS_RING_WIDTH = 2.0`. |
| Disabled state: α × 0.5 | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:305–306` (`maybe_disabled`) | ✅ | Applied to fill and outline. |
| Hover fill derivation: `Base × Hover` | `design-system/README.md § Animation, hover, press` | `quartzite-style/src/default_style/mod.rs:297, 305` | ✅ | `ColorGroup::Hover` via `state_group`. |
| Pressed fill + outline: `Highlight × Pressed` + `HighlightedText` | `design-system/README.md § Color` | `quartzite-style/src/default_style/mod.rs:300–303` | ✅ | `pressed` → fill `Highlight × Pressed`. |
| Scrollbar thumb/track geometry | `design-system/proposals/scrollbar.md` | Not yet implemented | N/A | Scrollbar track/thumb paint not yet in scope of this PR; covered by proposal. |
| Font | N/A | N/A | N/A | No text drawn by ScrollArea itself. |
| Padding / min-size | `design-system/proposals/scrollbar.md` (track width 12 px, thumb min 24 px) | N/A — layout concern | N/A | Deferred per K4. |

---

## Container

| Rule | Source | Code reference | Status | Note |
|---|---|---|---|---|
| Text vertical alignment | N/A — Container does not draw text | N/A | N/A | No `draw_text_in` call in `Paint<Container>`. |
| Text horizontal alignment | N/A | N/A | N/A | No text drawn. |
| Background colour: `Window × Normal` | `design-system/README.md § Color` (`Window` behind containers) | `quartzite-style/src/default_style/mod.rs:329` | ✅ | `ColorRole::Window × Normal`. Container has no state variants in the current impl. |
| Outline/border: 1 px `WindowText × Normal` | `design-system/preview/comp-container.html` (`border: 1px solid`) + `README.md § Layout chrome` | `quartzite-style/src/default_style/mod.rs:330–337` | ✅ | `Pen::new(WindowText × Normal, 1.0)`. |
| Focus ring | N/A — Container is not focusable in the default impl | N/A | N/A | No focus-ring paint path for Container. |
| Disabled state | N/A — Container has no enabled/disabled state in current impl | N/A | N/A | No `maybe_disabled` call; Container delegates state to its children. |
| Hover/pressed state derivations | N/A — Container has no hover/press paint path in current impl | N/A | N/A | `Paint<Container>` reads only `Normal` group. |
| Font | N/A | N/A | N/A | No text drawn. |
| Padding / min-size | `design-system/preview/comp-container.html` (no padding specified) | N/A | N/A | Deferred per K4. |

---

## Summary of ❌ rows

No open `❌` rows remain at PR merge. All code conformance gaps were resolved in Group A commits:
- G1 (missing vertical axis on `draw_text_in`) → fixed in commit 9816b38.
- G2 (`Button` top-anchored text) → fixed in commit 8fd1303.
- G3 (`Label` top-anchored text) → fixed in commit 8fd1303.
- G4 (`LineEdit` smaller-rect recipe) → replaced with `draw_text_in(geom, …, Left, Center)` in commit 61276a1.

G5 (`TextEdit` intentionally top-anchored) and G7 (already-conformant items) are ✅ as recorded in the spec.
