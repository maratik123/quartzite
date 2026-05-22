# Collapse WidgetExt::paint into Style::draw_widget

**Source:** issue #409
**Date:** 2026-05-22
**Tracked in:** #409

> Surfaced by `/triage` from [`ai-docs/deferred/widget-backlog.md`](../deferred/widget-backlog.md). Source spec: [`2026-05-13-renderer-style-dispatch.spec.md`](done/2026-05-13-renderer-style-dispatch.spec.md). With `Style::draw_widget` (and the `quartzite-style-dispatch` bridge crate) now driving every paint, `WidgetExt::paint` is a no-op default with zero overrides in-tree. This task removes it and migrates the one remaining caller (the widget snapshot harness) to dispatch through `Style::draw_widget`, leaving a single paint path.

## Scope

Remove `fn paint(&self, _painter: &mut dyn Painter)` from the `WidgetExt` trait in `quartzite-widgets/src/widget_ext.rs`, along with the trait's `quartzite_paint_api::Painter` import (it is used only by this method). Concretely:

- **Remove** the `paint` default method (lines 436–444 of `widget_ext.rs`), its `#[inline]`, and its rustdoc.
- **Remove** the unused `Painter` import in `widget_ext.rs` once `paint` is gone (no other trait method takes a `Painter`).
- **Migrate** the one in-tree consumer — `snapshot_widget` in `quartzite-widgets/tests/support/mod.rs:229–232` — from `harness.render_widget(|p| widget.paint(p))` to a `Style::draw_widget`-based dispatch path using the `quartzite-style-dispatch` crate (preferred) or, if a single-widget harness path proves cleaner, a direct `DefaultStyle::default().draw_widget(widget, p, &palette)` call inside the closure. Design phase picks the exact wiring (single-widget direct call vs. `dispatch_paint` over a one-node `WidgetResolver`); both end at `Style::draw_widget` and remove the `WidgetExt::paint` dependency.
- **Update** rustdoc and module-level comments that mention `WidgetExt::paint` so the file-headers in the snapshot suites no longer claim "widget `WidgetExt::paint` overrides are still no-ops" or "tests deliberately do not call `WidgetExt::paint`". Touch only the comments that name the removed method; do not rewrite unrelated text.
- **Update** `quartzite-renderer/src/window_root.rs` rustdoc — the `WidgetRoot::paint` doc currently says *"The `paint` receiver is `&self` to match `WidgetExt::paint` in `quartzite-widgets`"* — drop the cross-reference to the removed method while keeping the substantive guidance about `&self` + interior mutability.
- **Refresh** `quartzite-widgets/tests/support/mod.rs` doc-comment on `snapshot_widget` that names the removed idiom (`|p| widget.paint(p)`); replace with the new wiring's idiom.

The result: a single paint dispatch path. Concrete widget rendering goes `caller → dispatch_paint → Style::draw_widget → Paint<W>` (production) or `test harness → Style::draw_widget` (snapshot tests). `WidgetExt::paint` no longer exists as a surface.

## Out of scope

- Renaming or restructuring `Style::draw_widget`, `Paint<W>`, `WidgetView`, or `quartzite-style-dispatch::dispatch_paint`. This task only deletes the `WidgetExt::paint` surface and rewires the one caller.
- Removing or restructuring `WidgetRoot::paint` in `quartzite-renderer`. `WidgetRoot::paint` is a window-level entry point (called by the wgpu redraw loop) — it is distinct from `WidgetExt::paint` and stays. Only its cross-reference rustdoc is updated.
- Removing or restructuring the `quartzite-paint-api::Painter` trait. The `Painter` trait is consumed by `Style::draw_widget` and `dispatch_paint`; only `widget_ext.rs`'s no-longer-needed import is dropped.
- Adding a new `WidgetExt::children()` default method, or otherwise altering the `WidgetExt` / `AsWidget` trait surface beyond removing `paint`. `AsWidget::children()` already exists via the `extend_widget!` macro and is unchanged.
- Removing or renaming the `quartzite-style-dispatch` crate or its public API. The crate stays; this task is only about deleting `WidgetExt::paint`.
- Touching golden PNGs under `quartzite-widgets/tests/snapshots/` or `quartzite-style/tests/snapshots/` unless the migrated harness produces different pixels. The expected outcome is that goldens stay byte-for-byte identical because (a) every in-tree widget currently has a no-op `WidgetExt::paint`, so `widget.paint(p)` writes nothing; and (b) `DefaultStyle::draw_widget` is the same code that already drives the `quartzite-style` snapshots. If a snapshot does flip, design phase decides whether to refresh the golden or revisit the migration. (See § *Open questions* for the trade-off.)
- Per-widget repaint / damage tracking, hit-testing dispatch, multi-window dispatch — these stay deferred under the dispatch spec's § *Out of scope*.

## Deferred

- A `WidgetExt::children()` default method that would let the dispatch loop walk children without per-type knowledge | needs a coherent default + audit of every widget type | tracked by issue surfaced from `2026-05-13-renderer-style-dispatch.spec.md` § *Deferred*; not opened by this task.
- Removing `WidgetRoot::paint` in favour of a higher-level dispatch surface | needs a window-level dispatch design (multi-window awareness, surface acquisition) | tracked under `2026-05-13-renderer-style-dispatch.spec.md` § *Out of scope*; not opened by this task.

## Key decisions

| Question | Decision |
|---|---|
| What does "collapse" mean concretely? | **Remove** `WidgetExt::paint` entirely — the project is pre-`crates.io` and the trait method may be deleted outright. The single in-tree caller (`snapshot_widget` test helper) is rewired to drive `Style::draw_widget`. |
| Where does the test harness route paint now? | Through `Style::draw_widget` (production-equivalent path). Design phase picks between `quartzite_style_dispatch::dispatch_paint` over a one-node `WidgetResolver` and a direct `DefaultStyle::default().draw_widget(widget, p, &palette)` invocation. The free-fn route exercises the production dispatcher (more realistic); the direct route keeps the test surface narrow (less plumbing, no `WidgetResolver` impl). Both arrive at the same `Style::draw_widget` body. |
| Is `WidgetRoot::paint` affected? | **No.** `WidgetRoot::paint` is the window-level dispatch hook called by the wgpu redraw loop in `quartzite-renderer`. It is structurally separate from `WidgetExt::paint`. Only its rustdoc cross-reference to `WidgetExt::paint` is updated. |
| Are widget overrides at risk? | **No.** A `grep -rn "fn paint" quartzite-widgets/src/widgets/` returns nothing — there are zero in-tree overrides of `WidgetExt::paint`. The default `_no-op_` body is the only implementation. |
| Snapshot-golden flip risk | Expected: zero. Every in-tree widget's `paint` is the inherited no-op; `widget.paint(p)` writes nothing today. The migrated harness routes through `DefaultStyle::draw_widget`, the same code that already drives the `quartzite-style` snapshot suite. If a golden does flip during implementation, design phase decides (likely refresh the widget golden to match the style golden, since the style suite is the authoritative paint reference). |

## Technical constraints

- The `WidgetExt` trait lives in `quartzite-widgets/src/widget_ext.rs`. Removing `paint` requires no upstream coordination — no widget overrides it.
- The migrated snapshot harness path needs `quartzite-style` (for `DefaultStyle` + `Palette`) and optionally `quartzite-style-dispatch` (for `dispatch_paint`) as `dev-dependencies` in `quartzite-widgets/Cargo.toml`. Verify with `cargo tree` after the edit. `quartzite-widgets` does **not** gain a non-`dev` dependency on either.
- `quartzite-paint-api` import in `widget_ext.rs` is removed only after the `paint` method is gone — verify no other line in the file uses `Painter` before deleting.
- The doc gate (`RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features`) covers every workspace crate. Removing `WidgetExt::paint` removes a rustdoc anchor — any intra-doc link to `[WidgetExt::paint]` elsewhere in the workspace becomes a broken intra-doc link and must be updated in the same PR. Mechanical check: `grep -rn 'WidgetExt::paint' --include='*.rs' --include='*.md' .` before commit; every hit is either a comment to update or a stale intra-doc link to rewrite.
- `cargo build`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and the doc gate above must all be clean after the edit.
- The removal is a clean break — no wrappers, no aliases. Pre-`crates.io`, no downstream clients (AGENTS.md § *API Stability*).

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `WidgetExt::paint` no longer exists. `grep -n 'fn paint' quartzite-widgets/src/widget_ext.rs` returns no hits. |
| AC2 | `grep -rn 'WidgetExt::paint' --include='*.rs' --include='*.md' .` returns no hits in source — every documentation mention of the removed method is either deleted or rewritten to name the replacement path (`Style::draw_widget` or `dispatch_paint`). The only allowed surviving mentions are inside `ai-docs/plans/done/*.md` (historical specs), `ai-docs/learnings.md` (historical corrections), and `ai-docs/deferred/*.md` (historical decision / forward-link entries that reference the removed method by name; these are not live API references). |
| AC3 | `quartzite-widgets/tests/support/mod.rs::snapshot_widget` no longer references `WidgetExt::paint`; its body routes paint through `Style::draw_widget` (via `dispatch_paint` over a one-node resolver, or via a direct `DefaultStyle::default().draw_widget(...)` call — design decides). The function's signature and call sites in `quartzite-widgets/tests/snapshots.rs` (the three `snapshot_widget(&mut harness, "label", ...)` style invocations) are unchanged. |
| AC4 | The `quartzite-widgets` snapshot suite (`cargo test -p quartzite-widgets --tests`) is green. Goldens for `label`, `button`, `line_edit` either remain byte-for-byte identical (expected) or are refreshed in the same PR with a one-line commit-message note explaining the pixel diff (if any). |
| AC5 | The `quartzite-style` snapshot suite (`cargo test -p quartzite-style --tests`) is green — this task does not touch its harness, but verifies the broader paint pipeline still works after the trait edit. |
| AC6 | The `quartzite-style-dispatch` test suite (`cargo test -p quartzite-style-dispatch`) is green — no regression in `dispatch_paint` (it does not call `WidgetExt::paint` today, but the test confirms the dispatch path stays intact). |
| AC7 | `quartzite-renderer/src/window_root.rs`'s `WidgetRoot::paint` rustdoc no longer cross-references `WidgetExt::paint`. The substantive guidance about `&self` + interior mutability is preserved. |
| AC8 | `cargo build`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` are all clean. |
| AC9 | `cargo build -p quartzite --no-default-features --features libm` still compiles (the derive-free / `no_std` path — `WidgetExt::paint` removal must not break it). |
| AC10 | `quartzite-widgets/Cargo.toml` gains at most one new `dev-dependency` (`quartzite-style` and/or `quartzite-style-dispatch`); production deps are unchanged. `cargo tree -p quartzite-widgets --edges=normal` shows the same production-edge set as before. |
| AC11 | `quartzite-widgets/tests/no_style_dep.rs` passes (after being updated to check `cargo tree -p quartzite-widgets --edges=normal`, so only production edges are asserted cycle-free). The production-edge cycle-break invariant — `quartzite-widgets` must not pull `quartzite-style` in its production dep graph — is preserved. |

## Open questions

- **Harness wiring choice.** Single-widget snapshot tests can route paint via (a) `quartzite_style_dispatch::dispatch_paint` over a one-node `WidgetResolver` impl (production-equivalent path, more plumbing), or (b) a direct `DefaultStyle::default().draw_widget(widget, p, &palette)` invocation (less plumbing, narrower test surface). Both arrive at the same `Style::draw_widget` body and produce identical pixels for the in-tree widgets. Design phase picks one; no AC-affecting trade-off — pick whichever shortens `tests/support/mod.rs`.
- **`Palette` source for the snapshot harness.** Pass `&Palette::default()` per call (matches `dispatch_paint` v1 convention) or thread an explicit fixture palette through `snapshot_widget`? Sensible default = `&Palette::default()`; design phase confirms unless a snapshot test needs a non-default palette to exercise a colour-role flip.
- **Snapshot-golden flip handling.** If migrating from the no-op `widget.paint(p)` to `Style::draw_widget` changes any committed widget golden by more than the snapshot-suite's tolerance (`MEAN_THRESHOLD` etc.), the PR must either refresh the golden or revisit the harness wiring. Expected outcome: no flip (the `quartzite-style` snapshots already cover this exact paint code). If a flip happens, treat it as a finding — likely refresh the golden to match the authoritative `quartzite-style` reference.
