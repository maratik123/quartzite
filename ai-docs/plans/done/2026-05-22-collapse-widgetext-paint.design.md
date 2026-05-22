# Design: Collapse WidgetExt::paint into Style::draw_widget

**Issue:** #409
**Date:** 2026-05-22

## Approach

Delete the `paint` default method from the `WidgetExt` trait and the now-unused
`quartzite_paint_api::Painter` import in `widget_ext.rs`. Migrate the sole
in-tree caller — `snapshot_widget` in
`quartzite-widgets/tests/support/mod.rs` — to a **direct
`DefaultStyle.draw_widget(widget, painter, &Palette::default())` call** inside
the existing `harness.render_widget(...)` closure. Sweep every documentation
reference to `WidgetExt::paint` across live source / docs and rewrite to name
the replacement path (`Style::draw_widget`).

A precursor subtask updates `quartzite-widgets/tests/no_style_dep.rs` to filter
`cargo tree --edges=normal` **before** the `quartzite-style` dev-dependency is
added (see § *Risks → no_style_dep.rs broken by dev-dep addition*).

### Why direct `DefaultStyle::draw_widget` over `dispatch_paint`

Both routes terminate in `Style::draw_widget` and produce identical pixels
for the three single-widget snapshots (`label`, `button`, `line_edit`). The
direct route wins on three independent axes:

1. **Sync-group consistency.** The `quartzite-style` snapshot suite already
   uses exactly this pattern
   (`DefaultStyle.draw_widget(&w as &dyn AsWidget, painter, &Palette::default())`
   at `quartzite-style/tests/snapshots.rs:50`). The two `tests/support/mod.rs`
   files are a declared **Sync group** (AGENTS.md § *Propagation Rule*); the
   widgets-side helper staying close in shape to the style-side suite is the
   path of least drift.
2. **Smaller test surface, fewer transitive deps.** `dispatch_paint` requires
   `StyleRegistry::set_style(...)` setup (with a global lock and reentrancy
   considerations across parallel `#[test]` functions), an `ObjectId`, and a
   one-node `WidgetResolver` impl. The direct route needs none of these — the
   widget is already in scope as `&dyn WidgetExt`, and an upcast to
   `&dyn AsWidget` is one trait-object coercion.
3. **No regression risk for the `quartzite-style-dispatch` suite.** The
   dispatch crate keeps its full integration coverage (inline `tests` module
   in `quartzite-style-dispatch/src/dispatch.rs`). The widgets snapshot suite
   does not need to re-prove dispatch traversal — it tests widget rendering,
   not tree walking.

The dispatch route is rejected for this task; it remains the right choice
for any future multi-widget tree-rendering snapshot test. No AC distinguishes
the two — § *Open question — Harness wiring choice* in the spec explicitly
defers the decision to design.

### Palette source

`&Palette::default()` per call, matching the `quartzite-style` snapshot
suite verbatim. No existing widget snapshot test exercises a colour-role
flip that would need a non-default palette; if one is added later, it can
thread an explicit palette argument through `snapshot_widget` then.

### Dependencies

`quartzite-widgets/Cargo.toml` gains exactly **one** `dev-dependency`:
`quartzite-style = { path = "../quartzite-style" }`. This brings in
`DefaultStyle`, `Style`, and `Palette` (re-exported from
`quartzite-style-types`) for the test helper. No new production dependency.
`quartzite-style-dispatch` is **not** added (the direct route does not need
it).

Because dev-dependencies enter the `cargo tree` graph by default, the
existing AC13 guard test (`quartzite-widgets/tests/no_style_dep.rs`) must be
updated to scope its assertion to production edges only via `cargo tree
--edges=normal`. That update is sequenced as subtask 2 (before the dev-dep
is added in subtask 3) so that no intermediate commit leaves the build red.

### Snapshot-golden risk

Expected to be zero. Every in-tree widget's `WidgetExt::paint` is the inherited
no-op, so today's call `harness.render_widget(|p| widget.paint(p))` writes
nothing — the goldens encode the wgpu clear-colour baseline. The migrated
call routes through `DefaultStyle::draw_widget`. For the three affected
snapshots (`label`, `button`, `line_edit`) the test widgets are constructed
with the default (0×0) geometry — neither `set_geometry(...)` nor `show()` is
called. `DefaultStyle::draw_widget` paints into the widget's `geometry()`
rect; a 0×0 rect produces no pixels regardless of the `WidgetView` arm. The
output therefore stays byte-identical to the clear-colour goldens already on
disk. If a flip nevertheless occurs at implementation time, the contingency
in § *Open questions* of the spec applies (likely a one-line commit-message
note plus a regenerated golden).

### Rejected alternatives

- **Add `WidgetExt::paint` deprecation shim.** Forbidden by AGENTS.md § *API
  Stability* — pre-publish, clean breaks only, no `#[deprecated]` wrappers.
- **`dispatch_paint` over a one-node `WidgetResolver`.** See above — heavier
  test surface, more global state, no AC-visible win.
- **Move `snapshot_widget` to take an explicit `Palette` parameter.** YAGNI —
  no caller needs it; can be added in a future PR if a palette-flip snapshot
  test is added.
- **Keep `WidgetExt::paint` as a `Sealed`-marked deprecated default.** Same
  reason as the shim — clean break is permitted.
- **Add the `quartzite-style` dev-dep first and accept a transient red
  `no_style_dep.rs`.** Rejected — every commit must keep `cargo test`
  green (AGENTS.md § *Workflow*). The `--edges=normal` filter must land
  *before* the dev-dep, not after.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Update `quartzite-widgets/tests/no_style_dep.rs` to invoke `cargo tree -p quartzite-widgets --edges=normal --prefix none --no-dedupe` (add the `--edges=normal` flag to the existing `args([...])` slice). The assertion body — "no line starts with `quartzite-style ` (trailing space)" — is unchanged; only the `cargo tree` invocation is narrowed to production edges. Update the file-header `//! AC13 mechanical contract: ...` doc to note the production-edge scope. Run `cargo test -p quartzite-widgets --test no_style_dep` to confirm the test still passes against the current (no-dev-dep) graph, then verify the test would catch a hypothetical production-edge regression by sanity-eyeballing the `cargo tree --edges=normal` output locally. (AC11.) | `quartzite-widgets/tests/no_style_dep.rs` | — |
| 2 | Add `quartzite-style` as a `dev-dependency` of `quartzite-widgets`; run `cargo tree -p quartzite-widgets --edges=normal` to verify the production-edge set is unchanged (AC10) and `cargo test -p quartzite-widgets --test no_style_dep` to confirm AC11 stays green now that the dev-dep is present. | `quartzite-widgets/Cargo.toml`, `Cargo.lock` | 1 |
| 3 | Migrate `snapshot_widget` to direct `DefaultStyle::draw_widget` dispatch. Replace the closure body `widget.paint(p)` with `DefaultStyle.draw_widget(widget, p, &Palette::default())` (no cast needed — `widget` is already `&dyn AsWidget` after the parameter-type change). Update the surrounding rustdoc that names the removed `\|p\| widget.paint(p)` idiom and the module-header line that names `WidgetExt::paint`. Drop the `quartzite_widgets::WidgetExt` import (no longer required) and add the `quartzite_widgets::AsWidget` import for the upcast plus `quartzite_style::{DefaultStyle, Style}` and `quartzite_style_types::Palette`. (**Rationale for the split import shape:** matches the sync-group sibling `quartzite-style/tests/snapshots.rs:23–27`, which imports `quartzite_style::{DefaultStyle, Style}` and `quartzite_style_types::{DARK_PALETTE, Palette}` separately rather than going through the `quartzite_style::Palette` re-export. Keeping the two `tests/support/mod.rs` files structurally close minimises drift under the declared Sync group.) `widget: &dyn AsWidget` is the trait the helper now needs to accept — keeping `&dyn WidgetExt` would require an extra upcast at every call site. Change the helper's parameter type to `&dyn AsWidget`; the three call sites in `tests/snapshots.rs` (`&label`, `&button`, `&edit`) coerce to `&dyn AsWidget` exactly as they coerced to `&dyn WidgetExt` because `WidgetExt: AsWidget`. | `quartzite-widgets/tests/support/mod.rs` | 2 |
| 4 | Remove `WidgetExt::paint` method, its `#[inline]`, and its rustdoc from the trait. Remove the `use quartzite_paint_api::Painter;` import at the top of the file (now unused — no other trait method takes a `Painter`). Verify the file compiles standalone via `cargo build -p quartzite-widgets`. | `quartzite-widgets/src/widget_ext.rs` | 3 |
| 5 | Sweep documentation references across the workspace. Update three file-header docstrings to drop / rewrite mentions of `WidgetExt::paint`: `quartzite-widgets/tests/snapshots.rs` (line 9 file-header comment), `quartzite-style/tests/snapshots.rs` (line 7 file-header comment, "Tests deliberately do not call `WidgetExt::paint`"), `quartzite-style/tests/support/mod.rs` (line 13 module-doc, "rather than going through `WidgetExt::paint`"). Update `quartzite-renderer/src/render_harness.rs:308` doc-comment example (`harness.render_widget(\|p\| label.paint(p));`) to a `Style::draw_widget`-flavoured idiom or a generic `\|p\| {}` placeholder, keeping the surrounding rationale about why the bound is a closure rather than `WidgetExt`. Update `quartzite-renderer/src/window_root.rs` `WidgetRoot::paint` rustdoc to drop the `WidgetExt::paint` cross-reference while preserving the substantive `&self` + interior-mutability guidance (AC7). | `quartzite-widgets/tests/snapshots.rs`, `quartzite-style/tests/snapshots.rs`, `quartzite-style/tests/support/mod.rs`, `quartzite-renderer/src/render_harness.rs`, `quartzite-renderer/src/window_root.rs` | 4 |
| 6 | Final mechanical sweep + full gate. Run `cargo build`, `cargo test --workspace --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`, then the doc gate first (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) — the doc gate catches broken intra-doc links (e.g. `[WidgetExt::paint]` split across `///` lines) that a prose grep would miss. After the doc gate is clean, run `grep -rn 'WidgetExt::paint' --include='*.rs' --include='*.md' .` and confirm zero hits outside `ai-docs/plans/done/*.md`, `ai-docs/learnings.md`, and `ai-docs/deferred/*.md` (historical surfaces, explicitly exempt per AC2). Run `cargo build -p quartzite --no-default-features --features libm`. Confirm the three widget goldens (`label.png`, `button.png`, `line_edit.png`) under `quartzite-widgets/tests/snapshots/shared/` remain byte-identical via `git status` showing no changes there; if any flip, refresh with `scripts/update-snapshots.sh` and add a one-line commit-message note explaining the diff (AC4 contingency clause). | (verification only; no edits) | 5 |

## Handoff plan

`M = 6` (two groups, 3 + 3):

- **Group A — spawn `/context-reset`** per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry): subtasks 1–3 — `no_style_dep.rs` filter update + `quartzite-style` dev-dep addition + test-helper migration. Up to the point where `cargo test -p quartzite-widgets --tests` is expected to be green and the `WidgetExt::paint` trait method is the only remaining reference site in production source.
- **Handoff after Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry). Parent `/task` resumes in Group B with fresh context.
- **Group B — spawn `/context-reset`** per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry): subtasks 4–6 — trait method removal + documentation sweep + full gate verification. Terminal group (3 subtasks; within the 1..=3 range).

## Risks

- **`no_style_dep.rs` broken by dev-dep addition.** The existing AC13 guard test runs `cargo tree -p quartzite-widgets --prefix none --no-dedupe` **without** `--edges=normal`, so dev-dependencies appear in the output and the trailing-space substring check `"quartzite-style "` would trigger as soon as the `quartzite-style` dev-dep lands. Mitigation: subtask 1 (precursor) adds `--edges=normal` to the `cargo tree` invocation **before** subtask 2 introduces the dev-dep. The production-edge cycle-break invariant is preserved — `--edges=normal` includes normal (non-dev, non-build) edges only, which is exactly the surface AC13 must guard.
- **Hidden intra-doc link to `[WidgetExt::paint]`.** Spec § *Technical constraints* names the doc gate as the mechanical check. If the sweep in subtask 5 misses a `[WidgetExt::paint]` link anywhere in rustdoc, `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` flags it in subtask 6. Mitigation: the `grep -rn 'WidgetExt::paint' --include='*.rs' --include='*.md' .` in subtask 6 catches both bracketed and unbracketed forms before the doc gate runs.
- **Snapshot golden flip.** Discussed above (§ *Approach — Snapshot-golden risk*); expected probability is near zero given the default 0×0 widget geometry, but the contingency is named and recoverable inside the same PR via `scripts/update-snapshots.sh`.
- **`harness.render_widget` doc-example becomes a stale `ignore` block.** Line 308 of `render_harness.rs` uses ```rust ignore` so the example is not compiled — replacing the example body needs care to keep the surrounding rationale ("`WidgetExt` lives in `quartzite-widgets` and that crate is the renderer's *dev-dependency*") intact. The reason for the closure shape (avoiding a cycle) survives the rewrite; only the body of the example is updated. Mitigation: subtask 5 names this file explicitly.
- **Helper parameter type change (`&dyn WidgetExt` → `&dyn AsWidget`).** Call sites in `tests/snapshots.rs` pass `&label`, `&button`, `&edit` — these coerce to either bound because `WidgetExt: AsWidget`. No call-site edit required. The only downstream impact is the import list in `tests/support/mod.rs` (now `AsWidget` instead of `WidgetExt`). Mitigation: subtask 3 explicitly enumerates the imports.
- **`cargo build -p quartzite --no-default-features --features libm` still green.** The facade crate (`quartzite`) does not re-export `WidgetExt::paint`; the libm / no_std build only depends on whether `quartzite-widgets` still compiles without `paint`. Since `paint` had no body work beyond `_painter: &mut dyn Painter`, its removal cannot break a `no_std` configuration. Subtask 6 names this gate.

## Test Design

This is a **pure-deletion + one-call-site migration** task. No new test logic is required; the existing snapshot suites are themselves the test plan.

- **`quartzite-widgets/tests/snapshots.rs`** — `label_renders`, `button_renders`, `line_edit_renders`. These already invoke `snapshot_widget(&mut harness, "<name>", &<widget>)`. After subtask 3, they still pass (now driving `DefaultStyle::draw_widget` instead of the no-op `WidgetExt::paint`). They are the **primary acceptance probe for AC3 + AC4**.
- **`quartzite-widgets/tests/no_style_dep.rs`** — the AC13 mechanical contract test. After subtask 1's `--edges=normal` filter update it remains green; after subtask 2's `quartzite-style` dev-dep addition it stays green because dev-edges are no longer in scope. Primary acceptance probe for **AC11**.
- **`quartzite-style/tests/snapshots.rs`** — `button_idle_renders`, `button_checked_renders`, `button_disabled_renders`, `label_renders`, `text_edit_plain_renders`, etc. Already drive `DefaultStyle::draw_widget` directly; this task does not change them but verifies the broader paint pipeline still works (AC5).
- **`quartzite-style-dispatch/src/dispatch.rs::tests` (inline `#[cfg(test)] mod tests`)** — `dispatch_paint`-suite tests starting at `quartzite-style-dispatch/src/dispatch.rs:168`. These do not call `WidgetExt::paint`; they call `Style::draw_widget` through the dispatcher. AC6 probe.
- **`quartzite-widgets/src/widget_ext.rs::tests`** — module-level unit tests (`show_sets_visible`, `set_visible_true_and_false`, etc.; ~30 tests starting at line 585). None of these reference `paint`; they exercise geometry / state-flag mutation only. After the deletion they continue to pass without edit.
- **No new test file or `#[test]` fn is added.** The behaviour under test (single-widget rendering) is already covered by the existing snapshot suites on both sides of the sync group; this task only changes the call path, not the observable behaviour. Subtask 1 updates an existing test invocation (the `cargo tree` flag set), not its assertion logic.

### Fixtures / helpers

None new. The migrated `snapshot_widget` uses `&Palette::default()` inline.

## Open questions

None. The spec's three open questions are resolved here:

- **Harness wiring choice** — direct `DefaultStyle::draw_widget` (§ *Approach* above).
- **`Palette` source** — `&Palette::default()` per call (§ *Approach — Palette source*).
- **Snapshot-golden flip handling** — expected zero flip; contingency in § *Risks* and AC4's commit-message-note clause.
