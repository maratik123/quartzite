# DefaultStyle snapshot tests

**Source:** issue #297
**Date:** 2026-05-13
**Tracked in:** #297

> Issue #296 shipped `DefaultStyle` with call-level unit tests (10 tests using a recording-painter mock; AC1–AC10 in `2026-05-13-default-style-content.spec.md`). The mocks verify *which* painter methods get called and with which colour / alpha / alignment arguments, but **no pixel-level GPU snapshot exists** that exercises `DefaultStyle` through the real `VelloPainter` and compares against a committed golden PNG. This spec adds that missing layer.

## Scope

Add a GPU snapshot-test suite for `DefaultStyle` that renders each supported widget shape through `DefaultStyle::draw_widget` via the existing `RenderHarness` and compares the rendered image against a committed golden PNG, on the same per-backend-dir + `shared/` fallback scheme already used by `quartzite-widgets/tests/snapshots.rs`.

Concretely:

- New test file `quartzite-style/tests/snapshots.rs` and a test-side helper at `quartzite-style/tests/support/mod.rs` that provides the snapshot-assert / harness-or-skip / FLIP-diff plumbing.
- New committed goldens under `quartzite-style/tests/snapshots/shared/` (one PNG per scenario; per-backend overrides under `vulkan/` / `dx12/` / `metal/` / `auto/` may follow if real rasterization drifts).
- `quartzite-style/Cargo.toml` gains the dev-dependencies the harness needs (`quartzite-renderer`, `image`, `nv-flip`, `tempfile` if support-internal unit tests come along) — none of these enter the runtime dep graph.
- One snapshot scenario per visually-distinct shape `DefaultStyle` renders:
  - `button_idle` — `Button::new("OK")`, default palette, enabled, not checked.
  - `button_checked` — same as `button_idle` but `checked == true` (must produce Highlight-coloured fill, distinct PNG).
  - `button_disabled` — same as `button_idle` but `widget_base().enabled == false` (must produce half-alpha fill + text; distinct PNG).
  - `label` — `Label::new("hi")`, default alignment (`Left`).
  - `text_edit_plain` — `TextEdit` with `plain_text == "abc"`, `read_only == false`.
  - `text_edit_read_only` — same as `text_edit_plain` but `read_only == true` (must show the Window-coloured half-alpha overlay; distinct PNG).
  - `scroll_area_chrome` — `ScrollArea::new()`, chrome only (no children traversed).
- Each scenario follows the same pattern as `quartzite-widgets/tests/snapshots.rs`:
  - `harness_or_skip(name)` builds a 64×64 `RenderHarness` and returns `None` when no GPU adapter is available locally or `SKIP_RENDER_SNAPSHOT=1` is set.
  - The closure form of `RenderHarness::render_widget` wraps a single call to `DefaultStyle::default().draw_widget(&widget, painter, &Palette::default())`.
  - The captured image is fed through `snapshot_assert(name, &image)` which dispatches per-backend → shared fallback.
- Goldens are written with `QUARTZITE_REGENERATE_SNAPSHOTS=1` via `scripts/update-snapshots.sh` (already exists; works regardless of which crate's tests trigger it because the script invokes `cargo test` workspace-wide).
- The unknown-widget fall-through and the `StyleRegistry::set_style(Box::new(DefaultStyle))` round-trip (AC6 / AC10 from the prior spec) are **not** worth committing as pixel goldens — the first produces a clear-colour image identical to the harness background, the second is byte-identical to `button_idle`. They stay covered by the existing recording-painter unit tests; this spec does not duplicate them at the pixel level.

The existing call-level unit tests in `quartzite-style/src/default_style.rs` (`mod tests`) stay as-is. Pixel snapshots **complement** them — call-level tests assert *which method got which Colour/Brush*; pixel snapshots assert *the rendered image looks the same as it did last release*. Both are needed because the call-level tests cannot catch regressions in the renderer pipeline (palette resolution → brush flattening → Vello scene → wgpu draw), and the pixel snapshots cannot easily assert "the brush alpha is exactly half".

## Out of scope

- Refactoring or removing the existing call-level `mod tests` block in `quartzite-style/src/default_style.rs`. Those tests stay; they cover invariants the pixel goldens can't.
- Adding snapshot tests for widgets `DefaultStyle` does not yet support (`Container`, `LineEdit`) — those fall through the unknown-widget arm and are already covered by the call-level no-op test.
- Per-platform `DefaultStyle` variants (macOS / Windows flavours). Tracked separately under #284.
- Scrollbar track / thumb rendering for `ScrollArea`. Deferred in the prior spec; chrome only here too.
- Hover / pressed / focused button states. Deferred in the prior spec; only `checked` and `enabled` are wired.
- TextEdit caret / selection / scroll offset rendering. Deferred in the prior spec.
- Extracting the snapshot-assert helper into a shared dev-only crate. The helper is duplicated between `quartzite-widgets/tests/support/mod.rs` and `quartzite-style/tests/support/mod.rs` in v1 — this keeps the change surface small. Extraction is recorded as a deferred follow-up.
- Snapshot goldens for the unknown-widget fall-through (empty PNG = harness clear colour; adds no signal).
- Snapshot golden for the registry round-trip. Already covered at the call level (AC10 of the prior spec); a pixel golden would be byte-identical to `button_idle`.

## Deferred

- Shared `quartzite-test-support` dev-only crate that hosts `snapshot_assert` + `harness_or_skip` for every crate that wants pixel goldens | duplicating the helper between `quartzite-widgets/tests/support/mod.rs` and `quartzite-style/tests/support/mod.rs` is fine for two consumers but starts to drift at three+ | new issue once a third crate needs it
- Per-backend (vulkan / dx12 / metal) override goldens for `DefaultStyle` | the `shared/` fallback handles every backend today because no real rasterization drift has surfaced | new override added per-backend when a drift is observed (same workflow as `quartzite-widgets`)
- Snapshot tests for `Container` / `LineEdit` under `DefaultStyle` | both fall through the unknown-widget arm today; testing requires extending `DefaultStyle` itself first | follow-up spec when `DefaultStyle` supports those widgets

## Key decisions

| Question | Decision |
|---|---|
| Crate that hosts the new tests | `quartzite-style` (the crate that defines `DefaultStyle`). Adding tests in `quartzite-widgets` would force `quartzite-widgets` to depend on `quartzite-style`, inverting the current dep direction (widgets is *below* style today; style depends on widgets, not the reverse). Adding a third top-level test crate is overkill for one new suite. |
| Support-helper location | Duplicated at `quartzite-style/tests/support/mod.rs`, mirroring the structure used in `quartzite-widgets/tests/support/mod.rs`. The duplication is acknowledged in *Deferred* — extraction to a shared crate happens when a third consumer appears. Both copies stay in lock-step via the Propagation Rule (a change to one is mirrored in the other in the same PR). |
| Canvas size | `64 × 64`, same as the existing widget snapshot suite. Keeps committed PNGs small (≤ a few KB each) and matches the existing baseline. |
| Background colour | Harness clear-colour `[0, 0, 0, 255]` (black), same as `quartzite-widgets/tests/snapshots.rs`. Goldens that show only the harness clear colour (e.g. unknown widget) are not committed — they add no signal. |
| Tolerance | `FLIP_TOLERANCE = 0.05`, same workspace default. The existing `quartzite-widgets` helper documents this as the v1 cross-backend tolerance; same justification applies. |
| Skip / regen env vars | Same as widgets: `SKIP_RENDER_SNAPSHOT=1` → skip with notice; `QUARTZITE_REGENERATE_SNAPSHOTS=1` → write golden instead of compare; `WGPU_BACKEND` selects the per-backend dir. No new env-var surface introduced. |
| Per-backend lookup order | `<crate>/tests/snapshots/<backend>/<name>.png` (override) → `<crate>/tests/snapshots/shared/<name>.png` (fallback) → fail with actual-png written + reviewer message. Identical to the widget helper. |
| Number of committed goldens (v1) | Seven, one per scenario in *Scope* above (`button_idle`, `button_checked`, `button_disabled`, `label`, `text_edit_plain`, `text_edit_read_only`, `scroll_area_chrome`). All in `shared/` initially; per-backend overrides only added when drift is observed. |
| Font handling | `Font::new("sans-serif", <size>)` resolves through the system font registry (same as the existing widget tests). If the font cannot be resolved (rare on dev boxes, possible in minimal CI containers), the per-test fall-through prints a `font may be unavailable — skipping` notice and exits without asserting — same pattern as `draw_text_in_center` in `quartzite-widgets/tests/snapshots.rs`. |
| Test names | snake_case describing the scenario: `button_idle_renders`, `button_checked_renders`, `button_disabled_renders`, `label_renders`, `text_edit_plain_renders`, `text_edit_read_only_renders`, `scroll_area_chrome_renders`. (The `quartzite-widgets` suite uses the `<thing>_renders` form; matching that here.) |
| Driving the paint pass | Inside the `RenderHarness::render_widget(|painter| ...)` closure, the test calls `DefaultStyle::default().draw_widget(&widget as &dyn AsWidget, painter, &Palette::default())` directly. No `WidgetExt::paint` involvement — the widget's own `paint` is still a no-op. The closure is the bridge between `&mut dyn Painter` (what `draw_widget` wants) and the harness's owned painter. |
| Widget geometry | Each widget's geometry is set to fill `(0, 0) -> (CANVAS, CANVAS)` before drawing, so the snapshot exercises the full canvas. (Matches the call-level tests, which use `Rect::new(Point::ZERO, Size::new(64, 64))` for the same reason.) |
| Palette | `Palette::default()` for every scenario. The prior spec's call-level tests use the same default; differentiation between scenarios comes from widget state (`checked`, `enabled`, `read_only`), not palette swaps. |
| New runtime deps | None. `quartzite-style/Cargo.toml` gains only dev-dependencies — `quartzite-renderer` (already a dev-dep of `quartzite-widgets`, no cycle since `quartzite-style` does not appear in `quartzite-renderer`'s tree), `image`, `nv-flip`, and `tempfile` if the support-internals tests are mirrored. |
| Mirroring `support_internals.rs` | Out of scope. The widget-side internals tests exercise the helper's skip / regen / fallback behaviour against a `tempfile::TempDir`; that is a property of the helper, not of `DefaultStyle`. Duplicating the internals tests in `quartzite-style/tests/` would add ~200 lines of redundant coverage. If extraction-to-shared-crate happens, the internals tests live with the helper. |

## Technical constraints

- The existing `quartzite-widgets/tests/support/mod.rs` is the reference implementation. The `quartzite-style/tests/support/mod.rs` copy must keep the same public surface (`FLIP_TOLERANCE`, `SKIP_ENV`, `REGEN_ENV`, `BACKEND_ENV`, `DEFAULT_BACKEND_DIR`, `SHARED_DIR_NAME`, `snapshot_assert_at`, `snapshot_assert`, `default_snapshot_root`, `backend_dir_name`), differing only in `default_snapshot_root()` (which uses the crate's own `CARGO_MANIFEST_DIR`).
- No new public API on `DefaultStyle` or `quartzite-style`. The new tests use the public API that the prior spec already shipped.
- `cargo clippy --workspace -- -D warnings` and `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` must stay clean.
- `cargo test -p quartzite-style` must pass locally with a GPU adapter and on `SKIP_RENDER_SNAPSHOT=1` (skipping the suite cleanly). Same as `quartzite-widgets`.
- The CI integration path is unchanged. `quartzite-widgets`'s snapshot tests already run in CI behind the same skip mechanism; adding a sibling suite in `quartzite-style` adds zero new CI configuration — the same `cargo test` invocation picks them up via workspace membership.
- AGENTS.md *Dependency Versions* applies to the new dev-deps. Versions in the spec match the live registry when this spec was written: `image 0.25`, `nv-flip 0.1`, `tempfile 3` (mirrored from `quartzite-widgets`'s already-pinned values; design must re-verify before pinning the patch versions).
- The Propagation Rule applies to the two `tests/support/mod.rs` copies as soon as this lands (they become a new sync group, "snapshot-helper group"). Design adds a row to AGENTS.md's Propagation Rule table for the group.
- The widget paint surface (`WidgetExt::paint` is no-op for v1) is unchanged. The snapshot tests bypass `WidgetExt::paint` entirely and drive `DefaultStyle::draw_widget` directly inside the harness closure.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `quartzite-style/tests/snapshots.rs` exists, declares `mod support;`, and contains seven `#[test]` fns: `button_idle_renders`, `button_checked_renders`, `button_disabled_renders`, `label_renders`, `text_edit_plain_renders`, `text_edit_read_only_renders`, `scroll_area_chrome_renders`. Each invokes `DefaultStyle::default().draw_widget(...)` inside the harness closure and asserts via the local `snapshot_assert(name, &image)`. |
| AC2 | `quartzite-style/tests/support/mod.rs` exists and exports `snapshot_assert`, `snapshot_assert_at`, `harness_or_skip` (a sibling helper to the one in `quartzite-widgets/tests/snapshots.rs`, lifted into the support module here for reuse across the test file), `FLIP_TOLERANCE`, `SKIP_ENV`, `REGEN_ENV`, `BACKEND_ENV`, `DEFAULT_BACKEND_DIR`, `SHARED_DIR_NAME`, `default_snapshot_root`, `backend_dir_name`. Public surface matches `quartzite-widgets/tests/support/mod.rs` except `default_snapshot_root()` resolves to `quartzite-style/tests/snapshots`. |
| AC3 | `quartzite-style/tests/snapshots/shared/` contains seven committed PNGs — `button_idle.png`, `button_checked.png`, `button_disabled.png`, `label.png`, `text_edit_plain.png`, `text_edit_read_only.png`, `scroll_area_chrome.png` — each 64 × 64, generated by running the suite with `QUARTZITE_REGENERATE_SNAPSHOTS=1` then `mv quartzite-style/tests/snapshots/auto/* quartzite-style/tests/snapshots/shared/`. |
| AC4 | Running `cargo test -p quartzite-style` on a box with a GPU adapter passes all seven snapshot tests (FLIP mean ≤ `0.05` against each committed golden). |
| AC5 | Running `SKIP_RENDER_SNAPSHOT=1 cargo test -p quartzite-style` skips every snapshot test with a notice on stderr and exits 0. (Confirms the skip path is wired through the same env var as `quartzite-widgets`.) |
| AC6 | Running `QUARTZITE_REGENERATE_SNAPSHOTS=1 cargo test -p quartzite-style` writes new goldens under `quartzite-style/tests/snapshots/<backend_dir>/` and exits 0 without asserting against the prior goldens. (Confirms regen mode is wired.) |
| AC7 | `button_checked.png` and `button_idle.png` differ at the pixel level — the button fill colour changes from `Button` (idle) to `Highlight` (checked). The check is implicit in the test suite: regenerating the goldens with the same palette but `checked == true` vs `false` produces visibly different PNGs (asserted at review time by inspecting the committed files). |
| AC8 | `button_disabled.png` and `button_idle.png` differ at the pixel level — fill / text brushes use half-alpha when `enabled == false`. Same review-time check as AC7. |
| AC9 | `text_edit_read_only.png` and `text_edit_plain.png` differ at the pixel level — the `read_only` overlay (Window-coloured, half-alpha) lands on top of the Base fill. Same review-time check as AC7. |
| AC10 | `cargo clippy --workspace -- -D warnings` clean; `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace --all-features` clean; `cargo fmt -- --check` clean. |
| AC11 | `quartzite-style/Cargo.toml` lists `quartzite-renderer`, `image`, `nv-flip`, and (if the helper-internals tests are mirrored) `tempfile` under `[dev-dependencies]` only. The `[dependencies]` table is unchanged. |
| AC12 | AGENTS.md *Propagation Rule* gains a row for the "snapshot-helper group": `quartzite-widgets/tests/support/mod.rs` ↔ `quartzite-style/tests/support/mod.rs`. Both files reference each other in their module doc-comment header so the next editor sees the cross-link without reading AGENTS.md first. |
| AC13 | Inside the harness closure, the snapshot tests do not call `WidgetExt::paint` — they only call `DefaultStyle::default().draw_widget(&widget as &dyn AsWidget, painter, &Palette::default())`. Verifies the test exercises `DefaultStyle`, not the widget's own (no-op) paint method. |

## Open questions

- Whether the seven goldens look "right" enough for v1, or whether `DefaultStyle`'s visual choices (1 px outline, flat fill, palette-direct text colours) want a follow-up styling pass *before* goldens are committed. Default: commit the v1 goldens as-is; revisit via a follow-up issue if review surfaces "the chrome looks too flat" feedback. Not blocking.
- Whether `harness_or_skip` should live alongside `snapshot_assert` in `tests/support/mod.rs` from the outset (so the `quartzite-widgets` copy can adopt it too via the snapshot-helper sync group). Default: yes — lift it into support during this PR so both copies stay symmetric. Design may revisit if the lift creates churn the reviewer flags.
- Whether per-backend goldens land in this PR or in a follow-up once drift is observed. Default: ship `shared/` only; per-backend overrides happen reactively when CI on a new backend flags a FLIP-mean breach. Not blocking.
