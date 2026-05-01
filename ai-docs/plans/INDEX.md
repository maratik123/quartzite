# Plan Index

Legend: ✅ done · 🟢 ready (spec+design, no blockers) · 🟡 spec-only (no design yet) · 🔴 blocked

## Active plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [core-types](done/2026-05-01-core-types.spec.md) | `quartzite-core` | ✅ implemented (45 tests) | — |
| [github-workflow](done/2026-05-01-github-workflow.spec.md) | CI / repo config | ✅ live | — |

## Deferred plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [geometry-events](deferred/2026-05-01-geometry-events.spec.md) | `quartzite-geometry` `quartzite-events` | 🟡 spec-only | — |
| [macros](deferred/2026-05-01-macros.spec.md) | `quartzite-macros` | 🟢 spec+design | — |
| [runtime](deferred/2026-05-01-runtime.spec.md) | `quartzite-runtime` | 🟢 spec+design | — (ownership model decided: Arena/SlotMap — see design §Approach) |
| [paint-style](deferred/2026-05-01-paint-style.spec.md) | `quartzite-paint` `quartzite-style` | 🟡 spec-only | `quartzite-paint` needs geometry-events · `quartzite-style` additionally needs widgets |
| [widgets](deferred/2026-05-01-widgets.spec.md) | `quartzite-widgets` | 🟡 spec-only | runtime · macros · geometry-events |
| [auto-connection](deferred/2026-05-01-auto-connection.spec.md) | `quartzite-core` (extension) | 🟢 spec+design | runtime Task 0 (`QueuedDispatcher` + `ConnectionType::Queued`) · runtime design `post()` signature must use `+ 'static` |

## Dependency order

```
core-types ✅
├── geometry-events        (ready to start)
│   └── paint-style/paint  (ready after geometry-events)
├── macros                 (ready to start)
├── runtime                (ready after ownership model decision)
│   ├── auto-connection    (ready after runtime Task 0)
│   ├── widgets            (ready after runtime + macros + geometry-events)
│   │   └── paint-style/style  (ready after widgets)
│   └── paint-style/style  (same)
└── github-workflow ✅     (independent)
```

## Suggested next steps

1. **Start** geometry-events (no blockers, no design needed if straightforward) or macros (design ready)
2. **runtime** — ready to implement (ownership model: Arena/SlotMap; unblocks widgets, auto-connection, and the full stack)
