# Plan Index

Legend: ✅ done · 🟢 ready (spec+design, no blockers) · 🟡 spec-only (no design yet) · 🔴 blocked

## Active plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [graphics-stack](2026-05-03-graphics-stack.spec.md) | `quartzite-paint-api` `quartzite-renderer` | 🟢 ready | — |
| [signal-emit-rename](2026-05-05-signal-emit-rename.spec.md) | `quartzite-core` `quartzite-macros` | 🔴 blocked | #36 (Timer as first-class object) |
| [thiserror-migration](done/2026-05-05-thiserror-migration.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests) | — |
| [tracing-itertools](done/2026-05-05-tracing-itertools.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests) | — |
| [log-facade](done/2026-05-05-log-facade.spec.md) | `quartzite-core` `quartzite-runtime` `quartzite` | ✅ implemented (0 new tests) | — |
## Completed plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [core-types](done/2026-05-01-core-types.spec.md) | `quartzite-core` | ✅ implemented (45 tests) | — |
| [github-workflow](done/2026-05-01-github-workflow.spec.md) | CI / repo config | ✅ live | — |
| [macros](done/2026-05-01-macros.spec.md) | `quartzite-macros` | ✅ implemented (47 tests) | — |
| [runtime](done/2026-05-01-runtime.spec.md) | `quartzite-runtime` | ✅ implemented (176 tests) | — |
| [auto-connection](done/2026-05-01-auto-connection.spec.md) | `quartzite-core` (extension) | ✅ implemented (6 tests) | — |
| [geometry-events](done/2026-05-01-geometry-events.spec.md) | `quartzite-geometry` `quartzite-events` | ✅ implemented (26 unit + 91 doc tests) | — |
| [code-quality-cleanup](done/2026-05-02-code-quality-cleanup.spec.md) | `quartzite-macros` `quartzite-runtime` `quartzite-core` | ✅ implemented (0 new tests) | — |
| [docs-and-facade](done/2026-05-02-docs-and-facade.spec.md) | all crates + `quartzite` | ✅ implemented (1 new test) | — |
| [public-api-docs](done/2026-05-02-public-api-docs.spec.md) | all crates | ✅ implemented (47 new doctests) | — |
| [lookup-perf](done/2026-05-02-lookup-perf.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (21 new tests) | — |
| [inline-simple-fns](done/2026-05-02-inline-simple-fns.spec.md) | all crates | ✅ implemented (8 new tests) | — |
| [examples-crate](done/2026-05-02-examples-crate.spec.md) | `quartzite-examples` `quartzite` | ✅ implemented (0 new tests; 4 runnable examples) | — |
| [signals-blocked](done/2026-05-02-signals-blocked.spec.md) | `quartzite-core` `quartzite-macros` | ✅ implemented (13 new tests) | — |
| [receiver-guard-auto](done/2026-05-03-receiver-guard-auto.spec.md) | `quartzite-core` `quartzite-macros` | ✅ implemented (4 new tests) | — |
| [connect-queued-codegen](done/2026-05-03-connect-queued-codegen.spec.md) | `quartzite-macros` | ✅ implemented (3 new tests) | — |
| [enumflags2-property-flags](done/2026-05-03-enumflags2-property-flags.spec.md) | `quartzite-core` `quartzite-macros` | ✅ implemented (6 new tests) | — |
| [signal-emit-checked](done/2026-05-03-signal-emit-checked.spec.md) | `quartzite-core` `quartzite-macros` | ✅ implemented (3 new tests) | — |
| [objectbase-debug-rename-factory](done/2026-05-03-objectbase-debug-rename-factory.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (16 new tests) | — |
| [macro-codegen-improvements](done/2026-05-03-macro-codegen-improvements.spec.md) | `quartzite-macros` | ✅ implemented (30 new tests) | — |
| [object-part-redesign](done/2026-05-03-object-part-redesign.spec.md) | `quartzite-macros` `quartzite` | ✅ implemented (27 new tests) | — |
| [doc-convention](done/2026-05-05-doc-convention.spec.md) | all crates | ✅ implemented (workspace-wide doc convention; 23+ new doctests; 645 tests total) | — |
| [parent-children-accessors](done/2026-05-05-parent-children-accessors.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (1 integration test covering AC1–AC9, 1 unit test) | — |

## Deferred plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [paint-style](deferred/2026-05-01-paint-style.spec.md) | `quartzite-paint` `quartzite-style` | 🟡 spec-only | `quartzite-paint` needs geometry-events · `quartzite-style` additionally needs widgets |
| [widgets](deferred/2026-05-01-widgets.spec.md) | `quartzite-widgets` | 🟡 spec-only | runtime · macros · geometry-events |

## Dependency order

```
core-types ✅
├── geometry-events ✅
│   ├── graphics-stack     (ready to start — paint-api needs Point/Rect)
│   └── paint-style/paint  (ready; depends on quartzite-paint-api)
├── macros ✅
├── runtime ✅
│   ├── auto-connection ✅
│   ├── widgets            (ready — geometry-events done)
│   │   └── paint-style/style  (ready after widgets)
│   └── paint-style/style  (same)
└── github-workflow ✅     (independent)

graphics-stack (ready — no blocker)
├── quartzite-paint-api    (new thin crate; no_std)
├── quartzite-paint        (depends on quartzite-paint-api; see paint-style plan)
└── quartzite-renderer     (depends on quartzite-paint-api; vello+wgpu+winit)
```

Maintenance plans (cross-cutting, all ✅): code-quality-cleanup, docs-and-facade, public-api-docs, lookup-perf, inline-simple-fns, signals-blocked, receiver-guard-auto, connect-queued-codegen, signal-emit-checked, macro-codegen-improvements, object-part-redesign, doc-convention. These touched multiple crates and are not part of the dependency tree.

## Suggested next steps

1. **Start** graphics-stack (unblocked — geometry-events done; needs `quartzite-paint-api` thin crate then `quartzite-renderer`)
2. **Start** widgets (unblocked — geometry-events done; needs `WidgetBase`, layouts, basic widgets)
3. **Expand** `quartzite` facade prelude as new crates are implemented
4. Any future PR adding public items must satisfy the workspace doc convention at [`ai-docs/doc-convention.md`](../doc-convention.md): `#![deny(missing_docs)]` + `# Examples` + `# Parameters` (when ≥1 non-receiver arg) + `# Errors`/`# Panics`/`# Safety` when applicable; section ordering enforced by reviewer checklist; clippy `missing_errors_doc`/`missing_panics_doc`/`missing_safety_doc`/`doc_markdown` enabled across all crates
5. Match-based lookups are in place for properties/signals/methods/enums; enum lookup (`#[object_impl]` generates noop) could be wired up to `#[meta_enum]`-annotated enums when widgets land
6. `#[inline]` rule is enforced by AGENTS.md and review agents; new simple non-generic functions must carry the attribute
7. `proc_macro_crate` ergonomics: macro users currently need both `quartzite` and `quartzite-core` as direct deps; using `proc_macro_crate` in `quartzite-macros` would enable true single-dep usage
