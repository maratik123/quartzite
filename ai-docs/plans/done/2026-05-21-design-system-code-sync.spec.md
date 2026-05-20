# Sync code to design-system colour-value updates

**Source:** user description (free-text)
**Date:** 2026-05-21
**Tracked in:** #514

## Context

This task is the narrow follow-up to the design-system refresh (`design-system/` branch, most recently merged 2026-05-20 in `bfaeb7d v3` and `4bab9a1 rm garbage`). The user's round-1 clarification narrowed the scope: **issue #402** (palette state groups + `FocusRing` role) will be implemented in its own separate task session and is explicitly out of scope here. This spec covers **colour-value drift only** — any hex value currently in the Rust workspace that disagrees with the corresponding hex in `design-system/`.

A side-by-side comparison of every hex token in the design-system folder against every `Color::rgb`-style constant in the Rust workspace was performed:

- `design-system/colors_and_type.css` — referenced by the issue body and by `quartzite-style-types/src/dark_palette.rs:6` doc-comment, but the file **no longer exists** in `design-system/` (verified via `find design-system -name '*.css'` → only `preview/card-base.css` and `ui_kits/widgets/kit.css` remain). The colour seeds the file used to host now live in `design-system/README.md` § *Dark theme* and in the swatch HTML demos under `design-system/preview/`.
- `design-system/README.md` § *Color* and § *Dark theme* — describe the existing **11-role** palette plus the forthcoming 12th (`FocusRing`, issue #402). Every value listed for the 11 existing roles (`#FFFFFF` / `#000000` / `#0080FF` / `#0000FF` for light; `#2B2B2B` / `#E8E8E8` / `#3C3C3C` / `#1E1E1E` / `#1E90FF` / `#FFFFFF` / `#5BB0FF` / `#C58AFF` / `#FF6B6B` for dark) matches the corresponding Rust constant or seed exactly.
- `design-system/palette-state-groups.proposal.md` and `design-system/palette-state-groups-swatches.html` — every hex they introduce that does NOT exist in Rust today (`#F0F0F0`, `#D6D6D6`, `#0078F0`, `#006CD6`, `#464646`, `#585858`, `#2A95FE`, `#3E9EFB`) is a **derived state-cell value gated on issue #402's `ColorGroup` axis**. Per the round-1 answer these are out of scope and intentionally excluded from this spec.
- `design-system/proposals/text-edit-read-only-overlay.md` — proposes `WindowText.with_alpha(0.10)` overlay + `Text.with_alpha(0.65)` for read-only text. `quartzite-style/src/default_style.rs:15` already carries `READ_ONLY_OVERLAY_ALPHA: f32 = 0.10` and `:20` carries `READ_ONLY_TEXT_ALPHA: f32 = 0.65`. No drift; already implemented.

### Per-slot comparison table (current-state audit)

**Light palette (`Palette::new()` in `quartzite-style-types/src/palette.rs:65-76`):**

| Role | design-system hex | Rust constant / value | Match? |
|---|---|---|---|
| `Window` | `#FFFFFF` | `Color::WHITE` (1.0, 1.0, 1.0, 1.0) | ✓ |
| `WindowText` | `#000000` | `Color::BLACK` (0.0, 0.0, 0.0, 1.0) | ✓ |
| `Button` | `#FFFFFF` | `Color::WHITE` (1.0, 1.0, 1.0, 1.0) | ✓ |
| `ButtonText` | `#000000` | `Color::BLACK` | ✓ |
| `Base` | `#FFFFFF` | `Color::WHITE` | ✓ |
| `Text` | `#000000` | `Color::BLACK` | ✓ |
| `Highlight` | `#0080FF` | `Color::SKY_BLUE` (0.0, 0.5, 1.0, 1.0) | ✓ |
| `HighlightedText` | `#FFFFFF` | `Color::WHITE` | ✓ |
| `Link` | `#0000FF` | `Color::BLUE` (0.0, 0.0, 1.0, 1.0) | ✓ |
| `LinkVisited` | `#0000FF` | `Color::BLUE` | ✓ |
| `BrightText` | `#FFFFFF` | `Color::WHITE` | ✓ |

**Dark palette (`DARK_PALETTE` in `quartzite-style-types/src/dark_palette.rs:31-42`):**

| Role | design-system hex | Rust constant | Match? |
|---|---|---|---|
| `Window` | `#2B2B2B` Mine Shaft | `Color::MINE_SHAFT` (0.169, 0.169, 0.169, 1.0) | ✓ |
| `WindowText` | `#E8E8E8` Mercury | `Color::MERCURY` (0.910, 0.910, 0.910, 1.0) | ✓ |
| `Button` | `#3C3C3C` Eclipse | `Color::ECLIPSE` (0.235, 0.235, 0.235, 1.0) | ✓ |
| `ButtonText` | `#E8E8E8` Mercury | `Color::MERCURY` | ✓ |
| `Base` | `#1E1E1E` Nero | `Color::NERO` (0.118, 0.118, 0.118, 1.0) | ✓ |
| `Text` | `#E8E8E8` Mercury | `Color::MERCURY` | ✓ |
| `Highlight` | `#1E90FF` Dodger Blue | `Color::DODGER_BLUE` (0.118, 0.564, 1.000, 1.0) | ✓ |
| `HighlightedText` | `#FFFFFF` | `Color::WHITE` | ✓ |
| `Link` | `#5BB0FF` Light Dodger Blue | `Color::LIGHT_DODGER_BLUE` (0.357, 0.690, 1.000, 1.0) | ✓ |
| `LinkVisited` | `#C58AFF` Charoite | `Color::CHAROITE` (0.773, 0.541, 1.000, 1.0) | ✓ |
| `BrightText` | `#FF6B6B` Pastel Red | `Color::PASTEL_RED` (1.000, 0.420, 0.420, 1.0) | ✓ |

### Result

**Zero colour-value mismatches.** Every existing `ColorRole` slot in both shipped palettes already carries the value the design-system narrates for it. The only artefact found that needs touching is a **stale path in a doc comment** in `quartzite-style-types/src/dark_palette.rs:6`, which cites `design-system/colors_and_type.css` — a file that has been removed from the design-system folder.

The stale-doc-comment finding is a documentation-only fix (single line, no behavioural change, no public API change) that is in spirit a "sync code to design-system" item and is included here. The user may choose to defer it to a separate trivial-typo PR if preferred, but landing it alongside the audit costs nothing and prevents the broken reference from surfacing again in the next `rustdoc` reader's confusion.

## Scope

1. Document the audit result: zero colour-value drift across both palettes (verified per the table above).
2. Update the stale doc-comment in `quartzite-style-types/src/dark_palette.rs:6` to reference an extant design-system source. Replace `design-system/colors_and_type.css [data-theme="dark"]` with `design-system/README.md § Dark theme` (the surviving canonical source). One-line edit; no `#[doc]` semantics change.

## Out of scope

- Issue #402 (palette state groups, `FocusRing` role, derived `Hover` / `Pressed` cells, `ColorGroup` axis, `Palette::color(role, group)` signature change, `with_role_all_groups`, `DefaultStyle::draw_button` refactor to use the matrix, snapshot regeneration of `button_hovered.png` / `button_pressed.png` / dark siblings, doctests for the new signatures). Per the round-1 answer the user will implement #402 in its own task session.
- Adding new `Color` constants to `quartzite-paint-api/src/color.rs`. The eight dark-theme constants (`MINE_SHAFT`, `MERCURY`, `ECLIPSE`, `NERO`, `DODGER_BLUE`, `LIGHT_DODGER_BLUE`, `CHAROITE`, `PASTEL_RED`) are already present (lines 67–89). No new constants are required by any present design-system value.
- Adding documentary state-cell hex constants (`HOVER_WHITE` = `#F0F0F0`, `PRESSED_SKY_BLUE` = `#006CD6`, etc.) to `Color`. These belong to the #402 derivation machinery; the design-system explicitly notes (`palette-state-groups.proposal.md` § *Naming convention*) that they are documentation-only and computed by the `Palette` constructor.
- Changing the `Color::blend` derivation factors (6 % / 16 %). The design-system narrates these but the derivation only fires once #402 lands.
- Regenerating any snapshot PNGs. No paint-time value changes here → no snapshot drift.
- Editing any file under `design-system/`. The design-system folder is the source of truth; this spec syncs code to it, never the reverse.
- Refactoring `DefaultStyle::paint::<Button>`'s 25 %-blend hover (`default_style.rs:92`) or its direct `ColorRole::Highlight` focus-ring usage. Both belong to #402.
- Updating the `text-edit-read-only-overlay` proposal's alphas. Already implemented and matches the proposal exactly.

## Deferred

- Removing or further rewriting the `palette-state-groups.proposal.md` cross-references that other Rust files might gain once #402 lands. None exist in the current tree — re-evaluate when #402 ships.

## Key decisions

| Question | Decision |
|---|---|
| What counts as "drift" for this spec | Only `(role, RGBA-value)` pairs in either shipped palette whose Rust value disagrees with the corresponding hex in any extant `design-system/` file. State-derivation cells, new roles, and signature changes are #402's job. |
| Replacement source for the removed `colors_and_type.css` reference | `design-system/README.md § Dark theme` — the surviving canonical narration of the dark palette. The README's dark-theme section lists every dark hex with the same "common name" mapping the deleted CSS file held. |
| Scope of the doc-comment fix | One line edit in `dark_palette.rs:6`. Do not also touch the surrounding documentation prose unless a value cited there has drifted (none have). |
| Whether to land the doc-comment fix in this spec or a separate PR | Land here. It is the only artefact the audit surfaced; bundling avoids a near-empty PR. Reviewer may split if desired. |
| Whether to add a regression test guarding the colour-value match | No. The match is structural: each `with_role(Role, Color::FOO)` cites a named constant whose hex is documented in its `#[doc]`. A test asserting `DARK_PALETTE.color(Highlight) == Color::DODGER_BLUE` adds no information over the source itself, and any future drift between `Color::DODGER_BLUE`'s numeric value and its `#1e90ffff` doc comment is caught by the existing doctest convention (constructors carry `# Examples` blocks). |

## Technical constraints

- Workspace lint policy (`-D warnings`, `missing_docs = deny`, `rustdoc::broken_intra_doc_links = deny`) applies. The doc-comment change must not introduce a broken intra-doc link.
- The new reference target — `design-system/README.md § Dark theme` — is a Markdown anchor outside the Rust workspace, so it stays a free-text mention (no rustdoc intra-link). The original phrasing was also free-text, so no rustdoc link rewriting is needed.
- `cargo doc --no-deps --workspace --all-features` must still pass with `-D warnings -D missing-docs` (AGENTS.md § *Build & Test*).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | **Audit recorded.** The per-slot comparison table in this spec (both palettes, 11 roles each) reflects the actual tree state at `master` HEAD; every "Match? ✓" row is verifiable by reading the cited file:line in the Rust workspace and the design-system narration side-by-side. (Documentation-only AC; satisfied by this spec on disk.) |
| AC2 | **`dark_palette.rs:6` doc-comment** no longer references the removed `design-system/colors_and_type.css`. The replacement reference points to an extant artefact in `design-system/` (recommended: `design-system/README.md § Dark theme`). |
| AC3 | **Build + lint + docs gates pass.** `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` all succeed. |
| AC4 | **No snapshot diffs.** `cargo test -p quartzite-style` regenerates zero PNGs; every committed golden under `quartzite-style/tests/snapshots/shared/` is byte-identical pre- and post-change. |
| AC5 | **No public API change.** `cargo public-api` (or equivalent diff of `pub` items) is empty across the workspace. The change is a doc-comment text edit only. |

## Open questions

- **Whether to also strip the stale `palette-state-groups.proposal.md` cross-link from any future Rust doc comments.** No such cross-link exists today, so nothing to do — but if #402's implementation work re-introduces one and the proposal is later removed from `design-system/` (it currently survives), a similar drift will recur. Left for the #402 task to handle in its own spec.
- **Whether `design-system/preview/dark-palette-compare.html` should be the doc-comment reference instead of `README.md § Dark theme`.** Both are extant; the README is canonical narration, the HTML is the rendered swatch. README is the better citation for "the values come from here" (it spells them out); the HTML is the better citation for "look here to see them". README chosen for the spec's `Key decisions` row; reviewer may flip during design.
