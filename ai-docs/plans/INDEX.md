# Plan Index

Legend: ✅ done · 🟢 ready (spec+design, no blockers) · 🟡 spec-only (no design yet) · 🔴 blocked

## Active plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [core-types](done/2026-05-01-core-types.spec.md) | `quartzite-core` | ✅ implemented (45 tests) | — |
| [github-workflow](done/2026-05-01-github-workflow.spec.md) | CI / repo config | ✅ live | — |
| [macros](done/2026-05-01-macros.spec.md) | `quartzite-macros` | ✅ implemented (47 tests) | — |
| [runtime](done/2026-05-01-runtime.spec.md) | `quartzite-runtime` | ✅ implemented (176 tests) | — |
| [auto-connection](done/2026-05-01-auto-connection.spec.md) | `quartzite-core` (extension) | ✅ implemented (6 tests) | — |
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

## Deferred plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [geometry-events](deferred/2026-05-01-geometry-events.spec.md) | `quartzite-geometry` `quartzite-events` | 🟡 spec-only | — |
| [paint-style](deferred/2026-05-01-paint-style.spec.md) | `quartzite-paint` `quartzite-style` | 🟡 spec-only | `quartzite-paint` needs geometry-events · `quartzite-style` additionally needs widgets |
| [widgets](deferred/2026-05-01-widgets.spec.md) | `quartzite-widgets` | 🟡 spec-only | runtime · macros · geometry-events |

## Dependency order

```
core-types ✅
├── geometry-events        (ready to start)
│   └── paint-style/paint  (ready after geometry-events)
├── macros ✅
├── runtime ✅
│   ├── auto-connection ✅
│   ├── widgets            (ready after geometry-events)
│   │   └── paint-style/style  (ready after widgets)
│   └── paint-style/style  (same)
└── github-workflow ✅     (independent)
```

Maintenance plans (cross-cutting, all ✅): code-quality-cleanup, docs-and-facade, public-api-docs, lookup-perf, inline-simple-fns, signals-blocked, receiver-guard-auto, connect-queued-codegen. These touched multiple crates and are not part of the dependency tree.

## Suggested next steps

1. **Start** geometry-events (no blockers, unblocks widgets and paint-style)
2. **Expand** `quartzite` facade prelude as new crates are implemented
3. Any future PR adding public items must satisfy `#![deny(missing_docs)]` + `# Examples` (enforced by CI and self-review checklist)
4. Match-based lookups are in place for properties/signals/methods/enums; enum lookup (`#[object_impl]` generates noop) could be wired up to `#[meta_enum]`-annotated enums when widgets land
5. `#[inline]` rule is enforced by AGENTS.md and review agents; new simple non-generic functions must carry the attribute
6. `proc_macro_crate` ergonomics: macro users currently need both `quartzite` and `quartzite-core` as direct deps; using `proc_macro_crate` in `quartzite-macros` would enable true single-dep usage
