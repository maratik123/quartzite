# Design: DefaultStyle snapshot tests

**Issue:** #297
**Date:** 2026-05-13

## Approach

Add a GPU snapshot-test suite for `DefaultStyle` in `quartzite-style/tests/snapshots.rs`.
The new suite mirrors the structure already established by `quartzite-widgets/tests/snapshots.rs` —
same `tests/support/mod.rs` helper contract, same `RenderHarness` + closure pattern,
same per-backend-dir / `shared/` fallback scheme.

The key difference from the widget snapshot suite is the paint closure: instead of
calling `widget.paint(painter)`, each test calls
`DefaultStyle::default().draw_widget(&widget as &dyn AsWidget, painter, &Palette::default())`
directly. This exercises `DefaultStyle`'s routing logic and drawing code through the real
`VelloPainter` pipeline.

**Rejected alternative — third top-level test crate:** overkill for a single suite; would
also require a new Cargo workspace member and a dependency path from that crate to both
`quartzite-style` and `quartzite-renderer`, adding unnecessary build-graph complexity.

**Rejected alternative — tests in `quartzite-widgets`:** would invert the dep direction
(`quartzite-style` currently depends on `quartzite-widgets`, not the reverse). Confirmed
from `quartzite-style/Cargo.toml` (lists `quartzite-widgets` as a `[dependencies]` entry).

**Chosen:** duplicate the support helper into `quartzite-style/tests/support/mod.rs`
(acknowledged in spec as a deliberate v1 decision; extraction to a shared dev crate
deferred until a third consumer appears). The two copies become a sync group enforced by
the Propagation Rule in AGENTS.md.

The `update-snapshots.sh` script currently runs only `cargo test -p quartzite-widgets --test snapshots`.
It must be extended to also regenerate `quartzite-style` snapshots so the workflow stays
consistent. A new `--crate all|widgets|style` flag is added; the default remains
`all` (both crates) to avoid "forgot to regen the other crate" errors.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Add dev-dependencies to `quartzite-style/Cargo.toml` | `quartzite-style/Cargo.toml` | — |
| 2 | Create `quartzite-style/tests/support/mod.rs` (copy + adapt from `quartzite-widgets`) | `quartzite-style/tests/support/mod.rs` | 1 |
| 3 | Create `quartzite-style/tests/snapshots.rs` with all seven test functions | `quartzite-style/tests/snapshots.rs` | 2 |
| 4 | Extend `scripts/update-snapshots.sh` to cover `quartzite-style` | `scripts/update-snapshots.sh` | 3 |
| 5 | Generate and commit the seven golden PNGs under `quartzite-style/tests/snapshots/shared/` | `quartzite-style/tests/snapshots/shared/*.png` | 3, 4 |
| 6 | Add snapshot-helper sync group row to AGENTS.md Propagation Rule table | `AGENTS.md` | 2 |

## Risks

- **No GPU adapter in CI:** mitigated by `SKIP_RENDER_SNAPSHOT=1` early-return in every
  test; same mechanism used by the existing widget suite. CI already runs behind this flag.
- **Goldens look "wrong":** spec explicitly acknowledges `DefaultStyle`'s visual choices
  (1 px outline, flat fill, palette-direct text) may be revisited in a follow-up. Golden
  commit is v1-as-is; a follow-up issue tracks a styling pass.
- **Font unavailability:** `draw_text_in` / `draw_text` calls are font-dependent. The
  `draw_text_in_center` pattern from the widget suite (check non-background pixel count and
  skip with a notice if zero) should be applied in tests that rely on text rendering. However,
  since these are snapshot tests (not metric assertions), the non-background check is not
  needed — the FLIP comparison against the committed golden will show a mismatch if the
  rendered text changes. The golden itself is generated on a box with fonts available;
  the test is skipped in font-free containers via `SKIP_RENDER_SNAPSHOT=1`.
- **`update-snapshots.sh` change scope:** extending the script is a minor behavioural
  change; the existing `--backend` flag is unchanged. The addition of a `--crate` flag
  (defaulting to `all`) is backward-compatible for callers that pass no flag.
- **Propagation Rule compliance:** the two `tests/support/mod.rs` copies must stay in
  lock-step. AGENTS.md gains a new row; both files gain cross-link module-doc comments.

## Test Design

### Task 3 — `quartzite-style/tests/snapshots.rs`

**Location:** `quartzite-style/tests/snapshots.rs`

**Pattern (identical for all seven tests):**

```
fn <name>_renders() {
    let Some(mut harness) = harness_or_skip("<name>_renders") else { return; };
    let mut widget = <Widget>::new(...);
    widget.set_geometry(Rect::new(Point::ZERO, Size::new(CANVAS as i32, CANVAS as i32)));
    // state mutations (checked, enabled, read_only) applied here
    let image = harness.render_widget(|painter| {
        DefaultStyle::default().draw_widget(&widget as &dyn AsWidget, painter, &Palette::default());
    });
    snapshot_assert("<name>", &image);
}
```

**Test functions and their widget setup:**

| Test fn | Widget | State mutations |
|---|---|---|
| `button_idle_renders` | `Button::new("OK".into())` | — |
| `button_checked_renders` | `Button::new("OK".into())` | `w.checked = true` |
| `button_disabled_renders` | `Button::new("OK".into())` | `w.set_enabled(false)` |
| `label_renders` | `Label::new("hi".into())` | — (default `Alignment::Left`) |
| `text_edit_plain_renders` | `TextEdit::new()` | `w.plain_text = "abc".into()` |
| `text_edit_read_only_renders` | `TextEdit::new()` | `w.plain_text = "abc".into(); w.read_only = true` |
| `scroll_area_chrome_renders` | `ScrollArea::new()` | — |

All widgets have geometry set to `Rect::new(Point::ZERO, Size::new(64, 64))` before drawing
(CANVAS = 64).

**Scenarios:** happy path (golden matches), skip path (SKIP_RENDER_SNAPSHOT=1 → early return),
regen path (QUARTZITE_REGENERATE_SNAPSHOTS=1 → write without assert), missing golden
(panics with helpful message).

**Fixtures / helpers:** `harness_or_skip` from `mod support` (handles both SKIP env var and
GPU unavailability in a single call); `snapshot_assert` from `mod support` (dispatches
backend → shared fallback).

### Task 2 — `quartzite-style/tests/support/mod.rs`

**Location:** `quartzite-style/tests/support/mod.rs`

Identical to `quartzite-widgets/tests/support/mod.rs` with one change:
`default_snapshot_root()` returns `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("snapshots")`
(resolves to `quartzite-style/tests/snapshots` instead of `quartzite-widgets/tests/snapshots`).

The `snapshot_widget` helper (which calls `widget.paint(p)`) is **not** copied — it is not
needed here; the style tests drive `DefaultStyle::draw_widget` directly from the test closure.
All other public surface is preserved: `FLIP_TOLERANCE`, `SKIP_ENV`, `REGEN_ENV`,
`BACKEND_ENV`, `DEFAULT_BACKEND_DIR`, `SHARED_DIR_NAME`, `snapshot_assert_at`,
`snapshot_assert`, `default_snapshot_root`, `backend_dir_name`, `harness_or_skip`.

`harness_or_skip` is lifted from `quartzite-widgets/tests/snapshots.rs` into the support
module (it currently lives in the test file, not the support module, in that crate). Both
copies are put in their respective `support/mod.rs` so the API contract is symmetric.

Module-doc comment header includes a cross-link to the sibling:

```
//! **Sync group:** kept in lock-step with
//! `quartzite-widgets/tests/support/mod.rs` (snapshot-helper group).
//! A change to one MUST be mirrored to the other in the same PR.
```

The same cross-link is added to `quartzite-widgets/tests/support/mod.rs`.

**Note on `snapshot_widget` in `quartzite-widgets/tests/support/mod.rs`:** The existing
`snapshot_widget` fn in `quartzite-widgets/tests/support/mod.rs` is retained as-is; Task 2
adds only `harness_or_skip` to both modules. Moving `harness_or_skip` from `snapshots.rs`
into the support module in `quartzite-widgets` requires updating the callers in
`quartzite-widgets/tests/snapshots.rs` to reference `support::harness_or_skip` instead of
the local function. This is a mechanical refactor with zero behaviour change.

**Note on `harness_or_skip_with`:** `harness_or_skip_with` (the two-arg HiDPI variant in
`quartzite-widgets/tests/snapshots.rs`) is **not** lifted — it is not needed by the style
suite and its HiDPI builder variant is `quartzite-widgets`-specific.

### Task 4 — `scripts/update-snapshots.sh` extension

Add a `--crate` flag that accepts `all` (default), `widgets`, or `style`. When `all` or
`style`, the script runs `cargo test -p quartzite-style --test snapshots` in addition to
(or instead of) the `quartzite-widgets` invocation. The `target_dir` summary message is
updated accordingly.

**Note:** The script writes goldens to the per-backend dir (e.g. `tests/snapshots/vulkan/`),
not to `shared/`. As with `quartzite-widgets`, bootstrapping `shared/` is a manual `mv`
after regen — this step is part of Task 5, not Task 4.

### Task 5 — Golden generation

Procedure:
1. Run `QUARTZITE_REGENERATE_SNAPSHOTS=1 cargo test -p quartzite-style --test snapshots`
   (or `scripts/update-snapshots.sh --crate style`) to write PNGs into
   `quartzite-style/tests/snapshots/<backend>/`.
2. Manually move `quartzite-style/tests/snapshots/<backend>/*.png` to
   `quartzite-style/tests/snapshots/shared/`.
3. Verify the seven files have correct names and non-zero size.
4. Inspect visually: `button_checked.png` should differ from `button_idle.png` (fill colour);
   `button_disabled.png` should be visibly dimmer; `text_edit_read_only.png` should show the
   overlay wash.
5. Commit PNGs together with the code changes.

### Task 6 — AGENTS.md update

Add to the Propagation Rule table in AGENTS.md (in the `| If you edit... | You MUST also check / update... |` table):

```
| `quartzite-widgets/tests/support/mod.rs` | `quartzite-style/tests/support/mod.rs` (snapshot-helper group) |
| `quartzite-style/tests/support/mod.rs` | `quartzite-widgets/tests/support/mod.rs` (snapshot-helper group) |
```

And to the Sync groups section:

```
- **Snapshot-helper group:** `quartzite-widgets/tests/support/mod.rs` ↔ `quartzite-style/tests/support/mod.rs` — same public surface; only `default_snapshot_root()` differs (crate-local `CARGO_MANIFEST_DIR`). A change to shared logic (FLIP tolerance, env var names, lookup algorithm) MUST be mirrored in the same PR.
```

## Open questions

- None blocking. The spec's open questions are resolved as per the spec's stated defaults:
  goldens are committed v1-as-is; `harness_or_skip` is lifted into `support/mod.rs` from
  the start; per-backend overrides ship only when drift is observed.
