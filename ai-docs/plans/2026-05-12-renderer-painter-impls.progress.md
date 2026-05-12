# Progress: Renderer Painter Method Implementations

**Issue:** #289 (also closes #277)
**Spec:** ai-docs/plans/2026-05-12-renderer-painter-impls.spec.md
**Design:** ai-docs/plans/2026-05-12-renderer-painter-impls.design.md
**Branch:** feat/2026-05-12-renderer-painter-impls
**base_commit:** 22df144bb5e0026b89ffe54256836b6e2d0ed1be
**Last build:** —

## Tasks

| # | Task | Status |
|---|------|--------|
| 1 | `RenderHarnessBuilder` replaces `RenderHarness::new`; `scale_factor` field; update snapshot callers | ✅ |
| 2 | `font.rs` (`FontCache`); add `skrifa` + `parley` deps; wire into `RenderHarness` + `WrappedHandler` | ✅ |
| 3 | Rewrite `VelloPainter` (lifetime borrow, non-text methods, both call sites updated) | ✅ |
| 4 | Implement `draw_text` / `draw_text_in` via parley + skrifa + vello | ✅ |
| 5 | Unit tests: stack/clip probe, builder, `all_painter_methods_are_invocable` refresh | ⬜ |
| 6 | Snapshot tests (AC1–AC12); regen goldens | ⬜ |
| 7 | Final hygiene: doc-gate, `# Examples`, `lib.rs` re-exports, `cargo doc` clean | ⬜ |

## Next action

Start Task 1: Replace `RenderHarness::new(w, h)` with `RenderHarnessBuilder`.
Files: `quartzite-renderer/src/render_harness.rs`, `quartzite-widgets/tests/snapshots.rs`
