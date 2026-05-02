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
│   ├── auto-connection    (ready to start)
│   ├── widgets            (ready after geometry-events)
│   │   └── paint-style/style  (ready after widgets)
│   └── paint-style/style  (same)
└── github-workflow ✅     (independent)
```

## Suggested next steps

1. **Start** geometry-events (no blockers, unblocks widgets and paint-style)
