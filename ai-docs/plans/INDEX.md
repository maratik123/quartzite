# Plan Index

Legend: ✅ done · 🟢 ready (spec+design, no blockers) · 🟡 spec-only (no design yet) · 🔴 blocked

## Active plans

| Plan | Crate(s) | Status | Blocked by |
|------|----------|--------|------------|
| [next-deferred-discoverability](done/2026-05-10-next-deferred-discoverability.spec.md) | Claude Code tooling (`.claude/skills/next/SKILL.md` + `AGENTS.md`) | ✅ implemented (0 new tests; instruction-file-only — A1 of process-improvements: extends `/next` to read 8 thematic deferred files + `widget-backlog.md` via 9 `!`-blocks; surfaces untracked rows in new *Candidates needing /triage* output section; adds AGENTS.md AXIOM restricting `_inbox.md` writes to `/task` Step 12 + `/triage`; adds *Agent Docs* row for `_inbox.md`; AC1/AC2 deferred to manual `/next` reviewer-run per spec) | — |
| [docs-cleanup-197](done/2026-05-10-docs-cleanup-197.spec.md) | all crates (Cargo.toml metadata + `src/**/*.rs` doc comments + `quartzite-macros/src/{object,extend}/codegen.rs`) + `ai-docs/doc-convention.md` | ✅ implemented (0 new tests; metadata + doc-comment + convention text + macro-codegen-doc only — Part A: `all-features = true` propagated to 11 leaf-crate `[package.metadata.docs.rs]` blocks; Part B: cross-crate intra-doc-link audit re-verified all targets are direct deps; Part C: bare-backtick → intra-doc-link audit + conversion across `src/**/*.rs`; Part D: 3 new bullets + 3 before/after pairs in `ai-docs/doc-convention.md` § *Linking and code references*; Part E: `quartzite-macros` codegen audit — converted bare `` `Auto` `` / `` `Queued` `` to umbrella-pathed intra-doc links in macro-emitted `#[doc=…]` attributes, verified via `cargo expand` on `Counter` test type; 11 forced-bare qualified-path sites in `quartzite-core` / `quartzite-macros` left bare per dep-graph direction) | — |
| [object-property-serialization-layer](done/2026-05-10-object-property-serialization-layer.spec.md) | `quartzite-core` `quartzite-runtime` `quartzite` | ✅ implemented (34 new tests — 8 core error/type unit tests, 5 object-layer unit tests, 8 tree-layer unit tests incl. WeakObjectRef remap, 13 integration tests covering JSON + bincode + CustomValue; `serde` Cargo feature on quartzite-core/runtime/facade; `Value`/`WeakObjectRef` Serialize/Deserialize; typetag supertrait on CustomValue; `ObjectSnapshot`, `TreeSnapshot`, `ObjectNode`, `SerializeError`, `DeserializeError`; `capture_object`/`restore_object`/`capture_tree`/`restore_tree`; `quartzite::snapshot` facade module) | — |
| [gpu-snapshot-tests-ci](done/2026-05-10-gpu-snapshot-tests-ci.spec.md) | `quartzite-renderer` `quartzite-widgets` CI / repo config | ✅ implemented (15 new tests — 4 harness incl. GPU smoke, 8 helper-internals, 5 widget snapshots; 5 vulkan goldens committed; new `gpu-tests` matrix CI job with Win/Mac `continue-on-error: true` until per-backend goldens are bootstrapped; `xvfb_smoke` Linux integration test; `actions/upload-artifact@v7` for `*.actual.png`/`*.diff.png` on failure; `scripts/update-snapshots.sh`; CONTRIBUTING.md `## GPU snapshot tests` section; wgpu pinned 29 → 28 to match vello 0.8) | — |
| [ci-skip-rust-matrix](done/2026-05-09-ci-skip-rust-matrix.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only — `dorny/paths-filter@v4` `changes` job gates Rust matrix on Rust-affecting paths; aggregators reshaped to treat `skipped` as `success`; new `docs-pass` aggregator) | — |
| [interview-spec-writer-subagent](done/2026-05-09-interview-spec-writer-subagent.spec.md) | Claude Code tooling (`.claude/agents/`, `.claude/skills/interview/`, `AGENTS.md`) | ✅ implemented (0 new tests; instruction-file-only — extracts `/interview` spec-drafting into `spec-writer` opus subagent with structured YAML output; AC4–AC7 live tests deferred to post-merge per Verification protocol) | — |
| [ci-rust-cache-migration](done/2026-05-08-ci-rust-cache-migration.spec.md) | CI / repo config | ✅ implemented (0 new tests; CI config only — `actions/cache@v5` → `Swatinem/rust-cache@v2` in 5 compile jobs; `SCCACHE_CACHE_SIZE: "2G"` added per-job; `shared-key` + `save-if: master only` tuning) | — |
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
| [paint-style](done/2026-05-09-paint-style.spec.md) | `quartzite-paint-api` `quartzite-paint` `quartzite-geometry` `quartzite-widgets` `quartzite-style-types` (new) `quartzite-style` (new) | ✅ implemented (38 new tests; full Painter trait + paint-side Font/Image/Path; new `quartzite-style-types` leaf + `quartzite-style` downstream crates with `Box::leak`-backed `StyleRegistry`; `Alignment` moved to `quartzite-geometry`; `style ↔ widgets` cycle broken by leaf-crate split, enforced by `cargo tree` integration test) | — |

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

Tracked future work without dedicated specs (cross-cutting items only — not plans). INDEX.md-only footnote; not surfaced in `ROADMAP.md`.

- **#35** dynamic_properties — runtime read/write of non-schema properties
- **#39** signals_blocked serde (persist across serialization) — unblocked by #107 ✅
- **#48** BlockingQueued connection type — ready (per-thread loops ✅ implemented)
- **#52** object mobility / thread migration with stale `thread_id` invalidation
- **#53** multi-window support — ready (#46, #47, #73 all ✅ implemented)
- **#56** property system extensions — computed properties / bindings / custom getter/setter closures
- **#58** Python interop crate (`quartzite-python` via PyO3)

## Dependency order

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

Serialization-layer track (#107) ✅ implemented — unblocks #39.

Maintenance plans (cross-cutting, all ✅): see [`../context.md` § Maintenance plans](../context.md#maintenance-plans-cross-cutting) for the canonical list. These touched multiple crates and are not part of the dependency tree.

## Suggested next steps

1. **`/task` Step 12 propagation + `_inbox.md` backfill (#203, A2 of process-improvements)** — A1 (#202 ✅) just landed governance for `_inbox.md`. A2 creates the file, wires `/task` Step 12 to append spec follow-ups into it, and runs a one-shot deduped backfill over the 55 done specs. Blocked-label clears on A1 merge; pick this up next via `/task 203`.
2. **`signals_blocked` persistence (#39)** — now unblocked by the serialization layer (#107 ✅); small targeted change to carry `signals_blocked` through `ObjectSnapshot` / restore.
2. **Multi-window support (#53)** — both paint-style (#47) ✅ and widgets (#46) ✅ are now landed; the multi-window track is unblocked. Likely the next biggest milestone.
2. **Concrete `Style` implementation in `quartzite-style`** — the `Style` trait shipped in #47 with no built-in concrete impl. A "Quartzite Default" struct whose `draw_widget` covers Button/Label/TextEdit/ScrollArea is the natural follow-up.
3. **Real `Painter` impls in `quartzite-renderer`** — `VelloPainter`'s new `draw_text`/`draw_text_in`/`draw_image`/`draw_path` methods landed as no-op stubs in #47; flesh them out against vello once usage pressure justifies it. The offscreen `RenderHarness` (#192) is now in place to snapshot real-pixel output against goldens as soon as draw methods produce content.
4. **Bootstrap Windows/macOS snapshot goldens (#192 follow-up)** — `gpu-snapshot-tests-ci` shipped Linux/vulkan goldens only; Win/Mac matrix lanes are `continue-on-error: true` until contributors with those platforms run `scripts/update-snapshots.sh` and commit the per-backend PNGs. Drop `continue-on-error` once both lanes are green.
4. **Expand** `quartzite` facade prelude as new crates are implemented
4. Any future PR adding public items must satisfy the workspace doc convention at [`ai-docs/doc-convention.md`](../doc-convention.md): `#![deny(missing_docs)]` + `# Examples` + `# Parameters` (when ≥1 non-receiver arg) + `# Errors`/`# Panics`/`# Safety` when applicable; section ordering enforced by reviewer checklist; clippy `missing_errors_doc`/`missing_panics_doc`/`missing_safety_doc`/`doc_markdown` enabled across all crates
5. Match-based lookups are in place for properties/signals/methods/enums; enum lookup (`#[object_impl]` generates noop) could be wired up to `#[meta_enum]`-annotated enums when widgets land
6. `#[inline]` rule (recursive — see [`ai-docs/code-style.md` → `#[inline]` and the `_Simple._` doc tag](../code-style.md#inline-and-the-_simple_-doc-tag)) is enforced by AGENTS.md and review agents; new simple fns must carry the marker matching their shape — `#[inline]` on concrete fns, `_Simple._` doc tag on generic fns and on trait method declarations whose every conforming impl is required to be simple
7. Single-dep ergonomics are **already in place**: `quartzite-macros` uses `proc-macro-crate` to emit `::quartzite::core` paths when the user depends only on `quartzite`. Verified by `quartzite-macros/tests/via_facade.rs` and `quartzite/tests/single_dep.rs`.
