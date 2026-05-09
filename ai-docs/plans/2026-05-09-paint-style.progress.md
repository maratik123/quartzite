# Paint & Style — Progress

**Branch:** feat/2026-05-09-paint-style
**base_commit:** 837128843ea3e91581aa2f409af3da3c57249b2b
**Last build:** subtask 1 (geometry) — ✅ green (3 new tests, both std and no_std)
**Issue:** #47
**Spec:** ai-docs/plans/2026-05-09-paint-style.spec.md
**Design:** ai-docs/plans/2026-05-09-paint-style.design.md (round 2 — GO verdict)

## Files touched

- `quartzite-core/src/args_to_values.rs` — added `use alloc::vec;` to fix pre-existing no_std build failure (transitively required by AC14)
- `quartzite-geometry/Cargo.toml` — added `quartzite-core` (default-features = false) + `quartzite-macros` deps
- `quartzite-geometry/src/alignment.rs` — new file; Alignment enum with MetaEnum derive (verbatim from widgets) + 3 unit tests
- `quartzite-geometry/src/lib.rs` — declared `mod alignment;` and re-exported `Alignment`

## Subtask status

Per design Decomposition (16 tasks, geometry → paint-api → paint → style-types → widgets → style):

| # | Task | Status |
|---|---|---|
| 1 | quartzite-geometry: add Alignment enum + macros/core deps | ✅ done (3 new tests) |
| 2 | quartzite-paint-api: ungate `extern crate alloc;` | ⬜ pending |
| 3 | quartzite-paint-api: add `Color::with_alpha` | ⬜ pending |
| 4 | quartzite-paint-api: add Font + FontWeight | ⬜ pending |
| 5 | quartzite-paint-api: add Image (try_new validation) | ⬜ pending |
| 6 | quartzite-paint-api: add Path + Segment | ⬜ pending |
| 7 | quartzite-paint-api + quartzite-renderer: extend Painter trait, sync VelloPainter (atomic) | ⬜ pending |
| 8 | quartzite-paint: replace Path stub with full builder, re-exports | ⬜ pending |
| 9 | quartzite-style-types: new leaf crate scaffold | ⬜ pending |
| 10 | quartzite-style-types: ColorRole + ALL constant | ⬜ pending |
| 11 | quartzite-style-types: Palette (color/with_role, default) | ⬜ pending |
| 12 | quartzite-widgets: remove local Alignment/Font/Palette, re-export from upstream | ⬜ pending |
| 13 | quartzite-style: new downstream crate scaffold | ⬜ pending |
| 14 | quartzite-style: Style trait (Send + Sync, generic-only draw_widget) | ⬜ pending |
| 15 | quartzite-style: StyleRegistry (Box::leak, Mutex+OnceLock, poison-recovery, test helpers) | ⬜ pending |
| 16 | facade re-exports + cargo tree assertion + workspace doc/clippy gate | ⬜ pending |

## Next action

Subtask 2: quartzite-paint-api — move `extern crate alloc;` out of `#[cfg(test)]` gate (line 16 of `quartzite-paint-api/src/lib.rs`); confirm `cargo build -p quartzite-paint-api` and `--no-default-features` both still pass.
