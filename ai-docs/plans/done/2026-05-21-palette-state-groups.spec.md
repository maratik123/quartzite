# Palette state groups — `ColorGroup` axis + `FocusRing` role

**Source:** issue #402
**Date:** 2026-05-21
**Tracked in:** #402

> Promotes deferred row from [`ai-docs/plans/done/2026-05-15-button-hover-pressed-focused-states.spec.md`](done/2026-05-15-button-hover-pressed-focused-states.spec.md) (the source spec deferred new palette roles for hover / pressed / focus ring as a follow-up "when a designer-driven theming overhaul lands"). The detailed designer-driven proposal — [`design-system/palette-state-groups.proposal.md`](../../design-system/palette-state-groups.proposal.md) — is the canonical reference for values, derivation, and migration; this spec captures the implementation contract.

## Scope

Two additions to `quartzite-style-types`, plus the call-site refactor in `quartzite-style`, plus a design-system documentation refresh:

- **New `ColorGroup` enum** in `quartzite-style-types` (orthogonal to `ColorRole`):
  - Variants: `Normal` (`#[default]`), `Hover`, `Pressed`.
  - `pub const ALL: &'static [Self]` listing every variant in declaration order, matching the `ColorRole::ALL` pattern.
  - `Derive`s: `Copy, Clone, Debug, Default, PartialEq, Eq, Hash`.
  - Exhaustive-match unit test (`all_constant_lists_every_variant`-style) enforcing `ALL.len() == match arm count`, parallel to `color_role.rs`'s test.
  - Re-exported from `quartzite_style_types::ColorGroup`.

- **New `ColorRole::FocusRing`** variant — focus-outline stroke colour, single value per theme (no meaningful `Hover` / `Pressed` cells; derivation makes them mirror `Normal`). `ColorRole::ALL` grows from 11 to 12; the existing exhaustive-match test gains a `FocusRing` arm.

- **`Palette` storage and signature changes** in `quartzite-style-types/src/palette.rs` (breaking, pre-publish):
  - Storage grows from `[Color; ROLE_COUNT]` to `[Color; ROLE_COUNT * GROUP_COUNT]` (or equivalent two-axis indexing — design picks the exact layout). `GROUP_COUNT == ColorGroup::ALL.len()`.
  - `Palette::color(role: ColorRole) -> Color` → `Palette::color(role: ColorRole, group: ColorGroup) -> Color`.
  - `Palette::with_role(role: ColorRole, color: Color) -> Self` → `Palette::with_role(role: ColorRole, group: ColorGroup, color: Color) -> Self`.
  - New convenience: `Palette::with_role_all_groups(role: ColorRole, color: Color) -> Self` — seeds the role's `Normal`, `Hover`, and `Pressed` cells in one builder call.
  - `Palette::new()` (the `const fn` constructor) seeds every `(role, group)` cell to a non-transparent value via the derivation rule below; the existing `default != Color::TRANSPARENT` invariant becomes per-cell.
  - **No** standalone `Palette::role(role)` shorthand is added — call sites use `palette.color(role, ColorGroup::Normal)` directly when they need the resting colour.

- **Default derivation** applied at palette construction for every `(role, group)` cell not explicitly overridden:
  - `Hover(c) = c.blend(palette.color(WindowText, Normal), 0.06)`
  - `Pressed(c) = c.blend(palette.color(WindowText, Normal), 0.16)`
  - Derivation runs after the `Normal` cells are seeded, using the palette's own `WindowText × Normal` value as the contrast target — light theme blends toward `#000000`, dark blends toward `#E8E8E8` (Mercury).

- **`Palette::default()` (light theme)** — seeds per proposal § *Light theme* table. The meaningful `Hover` / `Pressed` cells are listed below; all other `(role, group)` cells are set by the derivation formula (roles whose `Normal` happens to equal `WindowText × Normal = #000000` derive to an identical value; other roles, such as `Window`, derive to a distinct `Hover` / `Pressed` value — see derivation rule above):
  - `Button × Normal = #FFFFFF` (existing `WHITE`); `Button × Hover = #F0F0F0` (`HOVER_WHITE`); `Button × Pressed = #D6D6D6` (`PRESSED_WHITE`).
  - `Highlight × Normal = #0080FF` (existing `SKY_BLUE`); `Highlight × Hover = #0078F0` (`HOVER_SKY_BLUE`); `Highlight × Pressed = #006CD6` (`PRESSED_SKY_BLUE`).
  - `FocusRing × Normal = #0080FF` (mirrors `Highlight × Normal` in light theme).

- **`DARK_PALETTE`** (`quartzite-style-types/src/dark_palette.rs`) — updated per proposal § *Dark theme* table:
  - `Button × Normal = #3C3C3C` (`ECLIPSE`); `Button × Hover = #464646` (`HOVER_ECLIPSE`); `Button × Pressed = #585858` (`PRESSED_ECLIPSE`).
  - `Highlight × Normal = #1E90FF` (`DODGER_BLUE`); `Highlight × Hover = #2A95FE` (`HOVER_DODGER_BLUE`); `Highlight × Pressed = #3E9EFB` (`PRESSED_DODGER_BLUE`).
  - `FocusRing × Normal = #1E90FF` (mirrors `Highlight × Normal` in dark theme).
  - All other `(role, group)` cells are set by the derivation formula using `WindowText × Normal = #E8E8E8 (MERCURY)` as the blend target; no additional explicit `with_role` overrides are applied for non-stateful roles.

- **`DefaultStyle::draw_button`** (`quartzite-style/src/default_style.rs`) refactor to consume the new axis, per proposal § *`DefaultStyle::draw_button` refactor*:
  - The 25 % `Color::blend` hover heuristic from #316 is removed.
  - State group selection: `pressed → ColorGroup::Pressed`; else `hovered → ColorGroup::Hover`; else `ColorGroup::Normal`.
  - Role selection (existing `disabled > pressed > checked > hovered` precedence preserved): `pressed || checked → (Highlight, HighlightedText)`; else `(Button, ButtonText)`.
  - `disabled` continues to alpha-halve the resolved colour post-lookup (unchanged behaviour).
  - Focus ring: `palette.color(ColorRole::FocusRing, ColorGroup::Normal)` replaces the inlined `Highlight` lookup.
  - `Color::blend` stays in `quartzite-paint-api` as a general primitive (used by `Palette`'s derivation now; no widget calls `blend` directly).

- **Migration of every in-tree `Palette::color` / `Palette::with_role` call site** to the new two-argument signature:
  - `quartzite-style/src/default_style.rs` — ≈ 8 call sites of `palette.color`.
  - `quartzite-style-types/src/palette.rs` doctests.
  - `quartzite-style-types/src/dark_palette.rs` — 12 chained `with_role` calls (mostly migrate to `with_role_all_groups` for the non-stateful roles; `Button`, `Highlight` use per-group setters for the meaningful state cells).
  - Any other in-tree caller surfaced by `rg 'palette.color\|with_role'` in `quartzite-*/src/`.

- **Snapshot regeneration** (per proposal § *Snapshot tests*):
  - `button_hovered.png` — regenerate (now `Button × Hover = #F0F0F0` instead of the 25 % `Button × Highlight` blend `#BFDFFF`).
  - `button_pressed.png` — regenerate (now `Highlight × Pressed = #006CD6` instead of `Highlight × Normal = #0080FF`).
  - `button_idle.png` / `button_checked.png` / `button_focused.png` / `button_disabled.png` — unchanged (verify byte-identical in the gate run; `button_focused.png` reads `FocusRing × Normal` which mirrors the pre-change `Highlight × Normal` value).
  - All `text_edit_*`, `scroll_area_chrome`, `label`, `text_edit_read_only` goldens — unchanged.

- **Design-system documentation refresh** in `design-system/`:
  - `design-system/README.md` § *Dark theme* and § *Color* — refreshed with the new `(role, group)` matrix.
  - `design-system/palette-state-groups-swatches.html` — already exists as the visual companion to the proposal; no change needed unless the table values drift during implementation.

## Out of scope

- **Per-group cells for `FocusRing`.** v1 ships `FocusRing` as a single-value role; the `Hover` / `Pressed` cells exist in the matrix but mirror `Normal`. Proposal § *Open questions* documents the default; revisit if focus-on-hover becomes a real design need.
- **Meaningful `Base.Hover` / `Base.Pressed` seeds.** Editable surfaces (`LineEdit`, `TextEdit`) don't render hover/pressed visuals yet — the cells derive to `Normal` mirrors until issue #406 lands. Matrix is ready; values are not yet seeded.
- **Dark-theme "darker on press" Qt convention.** v1 ships pure derivation in both themes (the dark theme lightens on hover/press, matching the formula). Themes that want the Qt convention override two cells via `with_role`. Proposal § *Default derivation* + § *Overrides* document the trade-off.
- **Hover / pressed / focused rendering on widgets other than `Button`.** Tracked by issue #403 (Label / TextEdit / ScrollArea) and issue #406 (LineEdit). This spec unblocks them by providing the palette vocabulary; the per-widget paint paths are separate tasks.
- **Materialising the 8 derived-colour names (`HOVER_WHITE`, `PRESSED_WHITE`, `HOVER_SKY_BLUE`, `PRESSED_SKY_BLUE`, `HOVER_ECLIPSE`, `PRESSED_ECLIPSE`, `HOVER_DODGER_BLUE`, `PRESSED_DODGER_BLUE`) as `pub const` items on `Color`.** They are documentary names for the values the palette computes via derivation; the design phase decides whether to add any `Color::*_*` constants (test fixtures may use literal `Color::new(...)` or `Color::from_rgb_u8(...)`). Proposal § *Naming convention* paragraph 4 calls them "documentary".
- **CSS custom-property additions in `design-system/colors_and_type.css`.** That file was removed in PR #514 (design-system-code-sync); the source of truth for palette values is now `quartzite-style-types::Palette::default` and the README tables. Design-system documentation references update in `README.md` only.
- **Animation / transition between states.** The framework remains stateless-render-per-frame; transitions are not modelled.
- **A `Color::HOVER_*` / `Color::PRESSED_*` shorthand convenience layer.** Builders use literal hex via `Color::from_rgb_u8` or already-existing named colours.

## Deferred

- **Per-widget hover / pressed / focused rendering for `Label` / `TextEdit` / `ScrollArea` / `LineEdit`** | needs per-widget visual idioms once the palette vocabulary lands | tracked by issues #403, #406 (no new issue from this spec)
- **`FocusRing` per-group cells (focus-on-hover ring variant)** | deferred; framework already supports it because `ColorGroup` is orthogonal — the cells exist, they just mirror `Normal` in v1 | no new issue (proposal § *Open questions* item 1; revisit on real demand)
- **`Base.Hover` / `Base.Pressed` meaningful seeds** | wired into the matrix but seeded to `Normal` mirrors until the editable-surface hover/press visuals arrive | tracked by issue #406 (LineEdit visual states)
- **Derivation-factor tuning (6 % / 16 %)** | the chosen defaults match Qt Fusion idle-vs-hover and produce visibly distinct dark-theme cells; tunable in one line of `Palette` derivation if downstream feedback flags them | no new issue (proposal § *Open questions* item 3 documents the call)

## Key decisions

| Question | Decision |
|---|---|
| Per-state colours: store in `Palette` or compute in `Style`? | In `Palette`. The proposal's § *Problem* paragraph 3 calls out the duplication risk (#403, #406 would each copy the heuristic from `draw_button`). `Palette` is the shared vocabulary. |
| Axis representation | New `ColorGroup` enum orthogonal to `ColorRole`; lookups become `palette.color(role, group)`. Single `ColorRole::Hover{X}` expansion (one variant per state per role) was rejected because it scales as `roles × states` and forces every `match` to enumerate the cross-product. |
| `Disabled` as a `ColorGroup` variant | No. Stays a post-resolution alpha-modifier (`color.with_alpha(0.5)`). Promoting it would force every theme to seed redundant per-role disabled cells whose values are mechanically derivable. Proposal § *1. New `ColorGroup` axis on `Palette`* paragraph 2. |
| `Focused` as a `ColorGroup` variant | No. Focus is an additive stroke modifier (a 2 px outline drawn on top of the fill), not a fill variant. Modelled by the new `ColorRole::FocusRing` role with `Normal`-only data. |
| `FocusRing` location | New `ColorRole` variant inside the existing `color_role.rs` file (not a new module). Proposal § *Open questions* item 5. |
| Default derivation formula | `Hover(c) = c.blend(WindowText.Normal, 0.06)`; `Pressed(c) = c.blend(WindowText.Normal, 0.16)`. Single blend target (`WindowText.Normal`) means the formula self-inverts between light and dark themes — both blend "toward maximum contrast". Proposal § *Default derivation*. |
| Derivation factors (6 % / 16 %) | Defaults match Qt Fusion's idle-vs-hover separation in light theme and produce visibly distinct dark-theme cells. Tunable in one line of `Palette` derivation; out-of-scope to bikeshed in v1. |
| Where derivation runs | At `Palette::new` / `Palette::default` / `DARK_PALETTE` construction (`const fn`-compatible because `Color::blend` is already `const fn`). Lookup remains a constant-time array read. |
| Dark theme: derivation vs. Qt "darker on press" | Derivation (lightens on press, matching the single-formula contract). Themes that want Qt convention override via `with_role`. Proposal § *Default derivation* + § *Overrides*. |
| `Palette::with_role` signature | Grows to `with_role(role, group, color)`. The convenience method `with_role_all_groups(role, color)` keeps `DARK_PALETTE`'s non-stateful roles terse (one call per role). |
| `Palette::role(role)` shorthand | Not added. Call sites that need the resting colour use `palette.color(role, ColorGroup::Normal)` directly — keeps the API surface minimal and explicit. |
| Derived-colour names (`HOVER_WHITE`, `PRESSED_SKY_BLUE`, …) | **Documentary**, not materialised on `Color`. The proposal's § *Naming convention* names them for tests and docs to refer to; `Palette` computes the values via derivation. The design phase decides whether to add any `pub const` items for test-fixture readability. |
| Snapshot regeneration scope | Two goldens regenerate (`button_hovered.png`, `button_pressed.png`). `button_focused.png` is byte-identical because `FocusRing × Normal` is seeded to the same value the old code read from `Highlight × Normal`. Verify byte-identity in the gate run. |
| Migration strategy | Single PR; every in-tree caller updates in lockstep with the signature changes. AGENTS.md § *API Stability* governs the clean-break posture. |
| `colors_and_type.css` references in design-system docs | The file was removed in PR #514 (design-system-code-sync). Replace the cross-references in `design-system/README.md` § *Color* / § *Dark theme* with the new `(role, group)` matrix; the CSS file is **not** re-introduced. |

## Technical constraints

- **API stability posture:** breaking signature changes on `Palette::color` / `Palette::with_role` follow AGENTS.md § *API Stability* AXIOM (clean break); every in-tree caller updates in the same PR.
- **`const fn` propagation:** `Palette::color` is currently `const fn`. The new two-argument signature must stay `const fn`. `Color::blend` is already `const fn` so the derivation formula in `Palette::new` / `Palette::default` can run in `const` context.
- **`ColorRole::ALL` and `ColorGroup::ALL` matrix:** `Palette` storage grows from `ROLE_COUNT == 11` to `ROLE_COUNT * GROUP_COUNT == 12 * 3 == 36` cells (12 roles after `FocusRing` lands × 3 groups). Indexing scheme is a design choice (`role as usize * GROUP_COUNT + group as usize`, or a 2D array, or per-group sub-arrays); whichever the design picks, the existing exhaustive-match test for `ColorRole::ALL` extends to cover `ColorGroup::ALL` as well.
- **Doc gate:** every new public item (the `ColorGroup` enum, its `ALL`, the new `ColorRole::FocusRing` variant, the new `Palette::with_role_all_groups` method) carries `///` first-line docs plus a `# Examples` block where applicable. `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean.
- **Lint gate:** `cargo clippy --workspace --all-targets -- -D warnings` clean.
- **Test gate:** existing `cargo test --workspace` suite (incl. doctests) green after migration; new derivation unit tests (per design) added.
- **Snapshot gate:** the snapshot harness in `quartzite-style/tests/snapshots.rs` regenerates `button_hovered.png` and `button_pressed.png`; the remaining goldens stay byte-identical. Goldens are committed.
- **`no_std` / `libm`:** `cargo build -p quartzite --no-default-features --features libm` must stay green. `Palette` derivation runs in `const fn` context and uses no floating-point intrinsics that the existing `Color::blend` doesn't already use.
- **`Send + Sync`:** unchanged. `Palette`'s array storage stays `Copy`-free but `Clone + Send + Sync`. `ColorGroup` is `Copy + Clone + Send + Sync` like `ColorRole`.
- **Code style:** new `const`s for the derivation factors (`HOVER_BLEND_FACTOR = 0.06_f32`, `PRESSED_BLEND_FACTOR = 0.16_f32`) live as module-level `SCREAMING_SNAKE_CASE` per AGENTS.md § *Code Style* — *Magic numbers*. The `8 derived-colour-name` documentary identifiers (`HOVER_WHITE`, …) may or may not become `pub const` items on `Color` — design decides.
- **Single PR scope:** one commit minimum, but a logical 2–3-commit decomposition is acceptable (commit 1: `ColorGroup` + `FocusRing` + `Palette` API change with all in-tree migrations green; commit 2: `DefaultStyle::draw_button` refactor + snapshot regen; commit 3: docs).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-style-types` exports a new `pub enum ColorGroup { Normal, Hover, Pressed }` with `#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]`, `#[default]` on `Normal`, and a `pub const ALL: &'static [Self]` listing every variant in declaration order. |
| AC2 | `quartzite-style-types::ColorRole` gains a `FocusRing` variant, and `ColorRole::ALL.len() == 12` with `FocusRing` appearing in the slice. The existing `all_constant_lists_every_variant` exhaustive-match test passes against the 12-variant enum. |
| AC3 | A new exhaustive-match unit test in `quartzite-style-types` (parallel to `color_role.rs`'s `all_constant_lists_every_variant`) enforces `ColorGroup::ALL.len() == <number of `ColorGroup` arms>`. |
| AC4 | `Palette::color` signature is `pub const fn color(&self, role: ColorRole, group: ColorGroup) -> Color`. The function returns a constant-time array lookup; no derivation logic runs at call time. |
| AC5 | `Palette::with_role` signature is `pub fn with_role(self, role: ColorRole, group: ColorGroup, color: Color) -> Self`. A new `pub fn with_role_all_groups(self, role: ColorRole, color: Color) -> Self` seeds all three group cells of `role` to `color`. |
| AC6 | `Palette::new()` (the `const fn` constructor) seeds every `(role, group)` cell so that no cell equals `Color::TRANSPARENT`. The seeding applies the derivation rule (`Hover = blend(c, WindowText.Normal, 0.06)`, `Pressed = blend(c, WindowText.Normal, 0.16)`) using the palette's own `WindowText × Normal` value as the contrast target. A unit test loops over `(role, group)` pairs and asserts `palette.color(role, group) != Color::TRANSPARENT` for every pair. |
| AC7 | `Palette::default()` seeds the meaningful cells per the light-theme table in this spec (`Button × Hover = #F0F0F0`, `Button × Pressed = #D6D6D6`, `Highlight × Hover = #0078F0`, `Highlight × Pressed = #006CD6`, `FocusRing × Normal = #0080FF`). All other `(role, group)` cells are set by the derivation formula — no additional explicit `with_role` overrides are applied for non-stateful roles (their `Hover` and `Pressed` cells carry the derivation result, which may differ from `Normal` for roles whose `Normal` is not already `WindowText × Normal`). Unit tests assert each of the five meaningful cells. |
| AC8 | `DARK_PALETTE` seeds the meaningful cells per the dark-theme table in this spec (`Button × Hover = #464646`, `Button × Pressed = #585858`, `Highlight × Hover = #2A95FE`, `Highlight × Pressed = #3E9EFB`, `FocusRing × Normal = #1E90FF`). All other `(role, group)` cells are set by the derivation formula — no additional explicit `with_role` overrides are applied for non-stateful roles. Unit tests assert each of the five meaningful cells. |
| AC9 | `DefaultStyle::draw_button` selects `ColorGroup` per the precedence `pressed → Pressed; else hovered → Hover; else Normal`, looks up `palette.color(role, group)` for fill and text, and reads the focus-ring stroke from `palette.color(ColorRole::FocusRing, ColorGroup::Normal)`. The `Color::blend(_, _, 0.25)` call inside `draw_button` from #316 is removed; no widget paint code calls `Color::blend` directly. |
| AC10 | The existing precedence (`disabled > pressed > checked > hovered` for fill/text; `focused` additive outline) is preserved end-to-end. A `pressed && checked` button picks `Highlight × Pressed`; a `disabled && focused` button paints the half-alpha fill plus the 2 px focus outline. Unit tests cover both combinations. |
| AC11 | Four snapshot goldens regenerate: light-theme `button_hovered.png` and `button_pressed.png`, dark-theme `dark_button_hovered.png` and `dark_button_pressed.png`. All other goldens (`button_idle.png`, `button_checked.png`, `button_focused.png`, `button_disabled.png`, and every non-button golden in both themes) remain byte-identical. The PR's snapshot diff lists exactly these four paths. |
| AC12 | Every in-tree call site of `Palette::color(role)` and `Palette::with_role(role, color)` is migrated to the new signature in the same PR. `rg 'palette\.color\([^,]+\)\|with_role\([^,]+,[^,)]+\)' -t rust` returns zero hits over `quartzite-*/src/` and the workspace test suites compile clean. |
| AC13 | `design-system/README.md` § *Dark theme* and § *Color* reflect the new `(role, group)` matrix (the dark-theme table grows the `Hover` and `Pressed` columns; § *Color* gains the `FocusRing` row). `design-system/SKILL.md` § *State derivations* line already references the new formula (verified — no change needed). The `colors_and_type.css` file is **not** resurrected. |
| AC14 | All workspace gates green on the final commit: `cargo build`, `cargo test --workspace` (incl. doctests), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt -- --check`, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`, `cargo build -p quartzite --no-default-features --features libm`. |

## Open questions

- **Materialise the 8 derived-colour documentary names as `pub const` items on `Color`?** Defensible default: do not — the names are doc-only identifiers; tests compute expected values via `Color::blend(...)` or use `Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)` literals. Adding 8 new public `Color::*` constants ties the names to the API surface, which is a reversal trade-off (more discoverable vs. more API surface). Design phase picks; either choice satisfies the spec.
- **Indexing scheme for the `Palette` storage array** — `[Color; ROLE_COUNT * GROUP_COUNT]` with computed indices, `[[Color; GROUP_COUNT]; ROLE_COUNT]`, or `[[Color; ROLE_COUNT]; GROUP_COUNT]`. All three are `const fn`-compatible; design picks based on access pattern and clippy noise. Spec is indifferent.
- **`ColorGroup` location** — `quartzite-style-types/src/color_role.rs` (alongside `ColorRole`) vs. a new `color_group.rs` module. Defensible default: new file `color_group.rs` for symmetry with `color_role.rs` and to keep both enums' exhaustive-match tests in their own files. Design picks; either is fine.
- **Whether the snapshot diff in AC11 includes the full byte-identical verification command** (e.g. a `git diff --stat quartzite-style/tests/snapshots/` snippet in the PR body) — pure presentation; the AC itself is verifiable mechanically.
