# quartzite — Roadmap

> **Auto-generated** from [`ai-docs/plans/INDEX.md`](ai-docs/plans/INDEX.md) by
> [`scripts/gen-roadmap.sh`](scripts/gen-roadmap.sh). Do not edit by hand —
> changes here will be reverted by the CI sync-gate. Edit `INDEX.md` instead
> and re-run the generator.

Legend: ✅ done · 🟢 ready (spec+design, no blockers) · 🟡 spec-only (no design yet) · 🔴 blocked

## Dependency tree

```
core-types ✅
├── geometry-events ✅
│   └── graphics-stack             ✅ implemented (quartzite-paint-api + quartzite-paint stub + quartzite-renderer scaffold)
│       ├── quartzite-paint-api    ✅ (thin no_std crate; 11-method Painter trait + Color/Pen/Brush/Font/Image/Path/PaintError)
│       ├── quartzite-paint        ✅ (re-export shell over paint-api + Alignment from geometry; full vocabulary completed in #47)
│       └── quartzite-renderer     ✅ scaffold (WindowedApplication + VelloPainter skeleton; vello+wgpu+winit; new Painter methods land as no-op stubs)
├── macros ✅
├── runtime ✅
│   ├── auto-connection ✅
│   ├── widgets (#46)              ✅ implemented (refactored in #47 to re-export Alignment / Font / Palette from upstream)
│   └── paint-style (#47)          ✅ implemented (full Painter trait + paint-side Font/Image/Path; quartzite-style-types leaf + quartzite-style downstream)
└── github-workflow ✅
    └── multi-platform-ci ✅        (Windows/macOS runners — build/test/clippy on all 3 OSes)
```

## Active plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [object-property-serialization-layer](ai-docs/plans/done/2026-05-10-object-property-serialization-layer.spec.md) | `quartzite-core` `quartzite-runtime` `quartzite` | ✅ implemented (34 new tests — 8 core error/type unit tests, 5 object-layer unit tests, 8 tree-layer unit tests incl. WeakObjectRef remap, 13 integration tests covering JSON + bincode + CustomValue; `serde` Cargo feature on quartzite-core/runtime/facade; `Value`/`WeakObjectRef` Serialize/Deserialize; typetag supertrait on CustomValue; `ObjectSnapshot`, `TreeSnapshot`, `ObjectNode`, `SerializeError`, `DeserializeError`; `capture_object`/`restore_object`/`capture_tree`/`restore_tree`; `quartzite::snapshot` facade module) | — |
| [gpu-snapshot-tests-ci](ai-docs/plans/done/2026-05-10-gpu-snapshot-tests-ci.spec.md) | `quartzite-renderer` `quartzite-widgets` CI / repo config | ✅ implemented (15 new tests — 4 harness incl. GPU smoke, 8 helper-internals, 5 widget snapshots; 5 vulkan goldens committed; new `gpu-tests` matrix CI job with Win/Mac `continue-on-error: true` until per-backend goldens are bootstrapped; `xvfb_smoke` Linux integration test; `actions/upload-artifact@v7` for `*.actual.png`/`*.diff.png` on failure; `scripts/update-snapshots.sh`; CONTRIBUTING.md `## GPU snapshot tests` section; wgpu pinned 29 → 28 to match vello 0.8) | — |
| [ci-skip-rust-matrix](ai-docs/plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only — `dorny/paths-filter@v4` `changes` job gates Rust matrix on Rust-affecting paths; aggregators reshaped to treat `skipped` as `success`; new `docs-pass` aggregator) | — |
| [interview-spec-writer-subagent](ai-docs/plans/done/2026-05-09-interview-spec-writer-subagent.spec.md) | Claude Code tooling (`.claude/agents/`, `.claude/skills/interview/`, `AGENTS.md`) | ✅ implemented (0 new tests; instruction-file-only — extracts `/interview` spec-drafting into `spec-writer` opus subagent with structured YAML output; AC4–AC7 live tests deferred to post-merge per Verification protocol) | — |
| [ci-rust-cache-migration](ai-docs/plans/done/2026-05-08-ci-rust-cache-migration.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only — `actions/cache@v5` → `Swatinem/rust-cache@v2` in 5 compile jobs; `SCCACHE_CACHE_SIZE: "2G"` added per-job; `shared-key` + `save-if: master only` tuning) | — |
| [ci-sccache](ai-docs/plans/done/2026-05-08-ci-sccache.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only — sccache layer added to 5 merge-gate compile jobs in ci.yml) | — |
| [widgets](ai-docs/plans/done/2026-05-01-widgets.spec.md) | `quartzite-widgets` | ✅ implemented (64 unit + 82 doc tests) | — |
| [project-docs](ai-docs/plans/done/2026-05-08-project-docs.spec.md) | `quartzite` (facade) + repo-level docs + CI | ✅ implemented (0 new tests; README description block + comprehensive `lib.rs` rustdoc + `CONTRIBUTING.md` + auto-generated `ROADMAP.md` + CI sync-gate) | — |
| [generic-simple-tags](ai-docs/plans/done/2026-05-07-generic-simple-tags.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests; annotation-only) | — |
| [coverage-ci](ai-docs/plans/done/2026-05-07-coverage-ci.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only) | — |
| [criterion-benchmarks](ai-docs/plans/done/2026-05-07-criterion-benchmarks.spec.md) | `quartzite-core` `quartzite-runtime` CI | ✅ implemented (0 new tests; 10 benches, 3 CI workflows) | — |
| [cargo-doc-pages](ai-docs/plans/done/2026-05-07-cargo-doc-pages.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only) | — |
| [macro-object-bench](ai-docs/plans/done/2026-05-07-macro-object-bench.spec.md) | `quartzite` (facade) | ✅ implemented (0 new tests; 6 benches via criterion + macro-derived fixture) | — |
| [graphics-stack](ai-docs/plans/done/2026-05-03-graphics-stack.spec.md) | `quartzite-paint-api` `quartzite-paint` `quartzite-renderer` | ✅ implemented (39 new tests) | — |
| [code-style-extraction](ai-docs/plans/done/2026-05-07-code-style-extraction.spec.md) | (docs only) | ✅ implemented (0 new tests; docs only) | — |
| [generic-fn-split](ai-docs/plans/done/2026-05-07-generic-fn-split.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests; refactoring) | — |
| [per-thread-event-loops](ai-docs/plans/done/2026-05-06-per-thread-event-loops.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (7 new tests) | — |
| [tracing-spans](ai-docs/plans/done/2026-05-06-tracing-spans.spec.md) | `quartzite-core` `quartzite-runtime` `quartzite` | ✅ implemented (0 new tests) | — |
| [object-tree-query](ai-docs/plans/done/2026-05-06-object-tree-query.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (28 new tests) | — |
| [signal-to-signal](ai-docs/plans/done/2026-05-06-signal-to-signal.spec.md) | `quartzite-core` `quartzite-macros` `quartzite` | ✅ implemented (23 new tests) | — |
| [thiserror-migration](ai-docs/plans/done/2026-05-05-thiserror-migration.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests) | — |
| [tracing-itertools](ai-docs/plans/done/2026-05-05-tracing-itertools.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (0 new tests) | — |
| [log-facade](ai-docs/plans/done/2026-05-05-log-facade.spec.md) | `quartzite-core` `quartzite-runtime` `quartzite` | ✅ implemented (0 new tests) | — |
| [paint-style](ai-docs/plans/done/2026-05-09-paint-style.spec.md) | `quartzite-paint-api` `quartzite-paint` `quartzite-geometry` `quartzite-widgets` `quartzite-style-types` (new) `quartzite-style` (new) | ✅ implemented (38 new tests; full Painter trait + paint-side Font/Image/Path; new `quartzite-style-types` leaf + `quartzite-style` downstream crates with `Box::leak`-backed `StyleRegistry`; `Alignment` moved to `quartzite-geometry`; `style ↔ widgets` cycle broken by leaf-crate split, enforced by `cargo tree` integration test) | — |

## Completed plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [core-types](ai-docs/plans/done/2026-05-01-core-types.spec.md) | `quartzite-core` | ✅ implemented (45 tests) | — |
| [github-workflow](ai-docs/plans/done/2026-05-01-github-workflow.spec.md) | CI / repo config | ✅ live | — |
| [multi-platform-ci](ai-docs/plans/done/2026-05-07-multi-platform-ci.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only) | — |
| [macros](ai-docs/plans/done/2026-05-01-macros.spec.md) | `quartzite-macros` | ✅ implemented (47 tests) | — |
| [runtime](ai-docs/plans/done/2026-05-01-runtime.spec.md) | `quartzite-runtime` | ✅ implemented (176 tests) | — |
| [auto-connection](ai-docs/plans/done/2026-05-01-auto-connection.spec.md) | `quartzite-core` (extension) | ✅ implemented (6 tests) | — |
| [geometry-events](ai-docs/plans/done/2026-05-01-geometry-events.spec.md) | `quartzite-geometry` `quartzite-events` | ✅ implemented (26 unit + 91 doc tests) | — |
| [code-quality-cleanup](ai-docs/plans/done/2026-05-02-code-quality-cleanup.spec.md) | `quartzite-macros` `quartzite-runtime` `quartzite-core` | ✅ implemented (0 new tests) | — |
| [docs-and-facade](ai-docs/plans/done/2026-05-02-docs-and-facade.spec.md) | all crates + `quartzite` | ✅ implemented (1 new test) | — |
| [public-api-docs](ai-docs/plans/done/2026-05-02-public-api-docs.spec.md) | all crates | ✅ implemented (47 new doctests) | — |
| [lookup-perf](ai-docs/plans/done/2026-05-02-lookup-perf.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (21 new tests) | — |
| [inline-simple-fns](ai-docs/plans/done/2026-05-02-inline-simple-fns.spec.md) | all crates | ✅ implemented (8 new tests) | — |
| [examples-crate](ai-docs/plans/done/2026-05-02-examples-crate.spec.md) | `quartzite-examples` `quartzite` | ✅ implemented (0 new tests; 4 runnable examples) | — |
| [signals-blocked](ai-docs/plans/done/2026-05-02-signals-blocked.spec.md) | `quartzite-core` `quartzite-macros` | ✅ implemented (13 new tests) | — |
| [receiver-guard-auto](ai-docs/plans/done/2026-05-03-receiver-guard-auto.spec.md) | `quartzite-core` `quartzite-macros` | ✅ implemented (4 new tests) | — |
| [connect-queued-codegen](ai-docs/plans/done/2026-05-03-connect-queued-codegen.spec.md) | `quartzite-macros` | ✅ implemented (3 new tests) | — |
| [enumflags2-property-flags](ai-docs/plans/done/2026-05-03-enumflags2-property-flags.spec.md) | `quartzite-core` `quartzite-macros` | ✅ implemented (6 new tests) | — |
| [signal-emit-checked](ai-docs/plans/done/2026-05-03-signal-emit-checked.spec.md) | `quartzite-core` `quartzite-macros` | ✅ implemented (3 new tests) | — |
| [objectbase-debug-rename-factory](ai-docs/plans/done/2026-05-03-objectbase-debug-rename-factory.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (16 new tests) | — |
| [macro-codegen-improvements](ai-docs/plans/done/2026-05-03-macro-codegen-improvements.spec.md) | `quartzite-macros` | ✅ implemented (30 new tests) | — |
| [object-part-redesign](ai-docs/plans/done/2026-05-03-object-part-redesign.spec.md) | `quartzite-macros` `quartzite` | ✅ implemented (27 new tests) | — |
| [doc-convention](ai-docs/plans/done/2026-05-05-doc-convention.spec.md) | all crates | ✅ implemented (workspace-wide doc convention; 23+ new doctests; 645 tests total) | — |
| [parent-children-accessors](ai-docs/plans/done/2026-05-05-parent-children-accessors.spec.md) | `quartzite-core` `quartzite-runtime` | ✅ implemented (1 integration test covering AC1–AC9, 1 unit test) | — |
| [timer-object](ai-docs/plans/done/2026-05-05-timer-object.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (36 new tests) | — |
| [signal-emit-rename](ai-docs/plans/done/2026-05-05-signal-emit-rename.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` | ✅ implemented (3 renamed tests; 0 new) | — |
| [signal-emit-macro](ai-docs/plans/done/2026-05-06-emit-macro.spec.md) | `quartzite-core` `quartzite-macros` `quartzite-runtime` `quartzite` | ✅ implemented (3 new tests) | — |
| [event-types-crate](ai-docs/plans/done/2026-05-06-event-types-crate.spec.md) | `quartzite-event-types` `quartzite-events` `quartzite-runtime` | ✅ implemented (4 new tests) | — |
| [recursive-inline-annotations](ai-docs/plans/done/2026-05-07-recursive-inline-annotations.spec.md) | `quartzite-core` `quartzite-geometry` `quartzite-runtime` | ✅ implemented (0 new tests; annotation-only) | — |

## Deferred plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
