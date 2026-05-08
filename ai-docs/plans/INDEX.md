# Plan Index

Legend: ✅ done · 🟢 ready (spec+design, no blockers) · 🟡 spec-only (no design yet) · 🔴 blocked

## Active plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [ci-sccache](done/2026-05-08-ci-sccache.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only — sccache layer added to 5 merge-gate compile jobs in ci.yml) | — |
| [widgets](done/2026-05-01-widgets.spec.md) | `quartzite-widgets` | ✅ implemented (64 unit + 82 doc tests) | — |
| [project-docs](done/2026-05-08-project-docs.spec.md) | `quartzite` (facade) + repo-level docs + CI | ✅ implemented (0 new tests; README description block + comprehensive `lib.rs` rustdoc + `CONTRIBUTING.md` + auto-generated `ROADMAP.md` + CI sync-gate) | — |
| [generic-simple-tags](done/2026-05-07-generic-simple-tags.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests; annotation-only) | — |
| [coverage-ci](done/2026-05-07-coverage-ci.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only) | — |
| [criterion-benchmarks](done/2026-05-07-criterion-benchmarks.spec.md) | `quartzite-core` `quartzite-runtime` CI | ✅ implemented (0 new tests; 10 benches, 3 CI workflows) | — |
| [cargo-doc-pages](done/2026-05-07-cargo-doc-pages.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only) | — |
| [macro-object-bench](done/2026-05-07-macro-object-bench.spec.md) | `quartzite` (facade) | ✅ implemented (0 new tests; 6 benches via criterion + macro-derived fixture) | — |
| [graphics-stack](done/2026-05-03-graphics-stack.spec.md) | `quartzite-paint-api` `quartzite-paint` `quartzite-renderer` | ✅ implemented (39 new tests) | — |
| [code-style-extraction](done/2026-05-07-code-style-extraction.spec.md) | (docs only) | ✅ implemented (0 new tests; docs only) | — |
| [generic-fn-split](done/2026-05-07-generic-fn-split.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests; refactoring) | — |
| [per-thread-event-loops](done/2026-05-06-per-thread-event-loops.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (7 new tests) | — |
| [tracing-spans](done/2026-05-06-tracing-spans.spec.md) | `quartzite-core` `quartzite-runtime` `quartzite` | ✅ implemented (0 new tests) | — |
| [object-tree-query](done/2026-05-06-object-tree-query.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (28 new tests) | — |
| [signal-to-signal](done/2026-05-06-signal-to-signal.spec.md) | `quartzite-core` `quartzite-macros` `quartzite` | ✅ implemented (23 new tests) | — |
| [thiserror-migration](done/2026-05-05-thiserror-migration.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests) | — |
| [tracing-itertools](done/2026-05-05-tracing-itertools.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests) | — |
| [log-facade](done/2026-05-05-log-facade.spec.md) | `quartzite-core` `quartzite-runtime` `quartzite` | ✅ implemented (0 new tests) | — |
## Completed plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [core-types](done/2026-05-01-core-types.spec.md) | `quartzite-core` | ✅ implemented (45 tests) | — |
| [github-workflow](done/2026-05-01-github-workflow.spec.md) | CI / repo config | ✅ live | — |
| [multi-platform-ci](done/2026-05-07-multi-platform-ci.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only) | — |
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
| [timer-object](done/2026-05-05-timer-object.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (36 new tests) | — |
| [signal-emit-rename](done/2026-05-05-signal-emit-rename.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (3 renamed tests; 0 new) | — |
| [signal-emit-macro](done/2026-05-06-emit-macro.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` `quartzite` | ✅ implemented (3 new tests) | — |
| [event-types-crate](done/2026-05-06-event-types-crate.spec.md) | `quartzite-event-types` `quartzite-events` `quartzite-runtime` | ✅ implemented (4 new tests) | — |
| [recursive-inline-annotations](done/2026-05-07-recursive-inline-annotations.spec.md) | `quartzite-core` `quartzite-geometry` `quartzite-runtime` | ✅ implemented (0 new tests; annotation-only) | — |

## Deferred plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [paint-style](deferred/2026-05-01-paint-style.spec.md) | `quartzite-paint` `quartzite-style` | 🟡 spec-only | style portion blocked on widgets #46 — tracked in #47; paint-api blocker (#73) ✅ resolved |

> Tracking issues for further deferred items not represented as plans here:
> #35 (dynamic_properties), #39 (signals_blocked serde — blocked on #107), #48 (BlockingQueued — blocked on per-thread loops ✅ done), #52 (object mobility), #53 (multi-window — blocked on #46, #73), #56 (property extensions), #58 (Python interop), #59 (CI extras), #60 (docs extras), #107 (serialization layer).

## Dependency order

```
core-types ✅
├── geometry-events ✅
│   └── graphics-stack             ✅ implemented (quartzite-paint-api + quartzite-paint stub + quartzite-renderer scaffold)
│       ├── quartzite-paint-api    ✅ (thin no_std crate; Painter trait + Color/Pen/Brush/PaintError)
│       ├── quartzite-paint        ✅ stub (re-exports paint-api + Path stub; full impl #47)
│       └── quartzite-renderer     ✅ scaffold (WindowedApplication + VelloPainter skeleton; vello+wgpu+winit)
├── macros ✅
├── runtime ✅
│   ├── auto-connection ✅
│   ├── widgets (#46)              ✅ implemented
│   │   └── paint-style/style      🔴 blocked on paint #47
│   └── paint-style/paint (#47)    🟡 spec-only (graphics-stack #73 ✅ resolved — now unblocked)
└── github-workflow ✅
    └── multi-platform-ci ✅        (Windows/macOS runners — build/test/clippy on all 3 OSes)
```

Serialization-layer track (#107) is independent of the dependency chain above and itself blocks #39.

Maintenance plans (cross-cutting, all ✅): see [`../context.md` § Maintenance plans](../context.md#maintenance-plans-cross-cutting) for the canonical list. These touched multiple crates and are not part of the dependency tree.

## Suggested next steps

1. **Start paint (#47)** — `quartzite-paint` full implementation (graphics-stack #73 ✅ resolved, widgets #46 ✅ done). This is the **single blocker** remaining for paint-style/style and multi-window (#53).
2. **After paint #47 lands**, paint-style/style spec can activate; multi-window (#53) follows.
3. **Expand** `quartzite` facade prelude as new crates are implemented
4. Any future PR adding public items must satisfy the workspace doc convention at [`ai-docs/doc-convention.md`](../doc-convention.md): `#![deny(missing_docs)]` + `# Examples` + `# Parameters` (when ≥1 non-receiver arg) + `# Errors`/`# Panics`/`# Safety` when applicable; section ordering enforced by reviewer checklist; clippy `missing_errors_doc`/`missing_panics_doc`/`missing_safety_doc`/`doc_markdown` enabled across all crates
5. Match-based lookups are in place for properties/signals/methods/enums; enum lookup (`#[object_impl]` generates noop) could be wired up to `#[meta_enum]`-annotated enums when widgets land
6. `#[inline]` rule (recursive — see [`ai-docs/code-style.md` → `#[inline]` and the `_Simple._` doc tag](../code-style.md#inline-and-the-_simple_-doc-tag)) is enforced by AGENTS.md and review agents; new simple fns must carry the marker matching their shape — `#[inline]` on concrete fns, `_Simple._` doc tag on generic fns and on trait method declarations whose every conforming impl is required to be simple
7. Single-dep ergonomics are **already in place**: `quartzite-macros` uses `proc-macro-crate` to emit `::quartzite::core` paths when the user depends only on `quartzite`. Verified by `quartzite-macros/tests/via_facade.rs` and `quartzite/tests/single_dep.rs`.
