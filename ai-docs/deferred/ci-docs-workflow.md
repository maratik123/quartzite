# CI, Docs & Workflow

Items extracted from completed plans. See [index](../deferred-items.md).

## Deferred

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Badge links in README \| will be added later by user | [github-workflow spec](../plans/done/2026-05-01-github-workflow.spec.md) | | #60 (closed) |
| Contributing guide, roadmap \| user will add later | [github-workflow spec](../plans/done/2026-05-01-github-workflow.spec.md) | | #60 (closed) |
| Additional facade features \| will be added based on needs as the project grows | [docs-and-facade spec](../plans/done/2026-05-02-docs-and-facade.spec.md) | | #60 (closed) |
| Multi-version docs (post first `cargo publish`) — After docs.rs takes over for releases | [cargo-doc-pages spec](../plans/done/2026-05-07-cargo-doc-pages.spec.md) |  | — |
| Threshold tightening (below 1%) — after project matures | [coverage-ci spec](../plans/done/2026-05-07-coverage-ci.spec.md) |  | — |
| Per-crate coverage targets — not needed until more crates land | [coverage-ci spec](../plans/done/2026-05-07-coverage-ci.spec.md) |  | — |
| `.github/workflows/fork_pr_benchmarks_closed.yml` (PR branch archival in Bencher) — not needed at first land | [criterion-benchmarks spec](../plans/done/2026-05-07-criterion-benchmarks.spec.md) |  | — |
| `--no-default-features` cross-OS coverage — not in this issue | [multi-platform-ci spec](../plans/done/2026-05-07-multi-platform-ci.spec.md) |  | — |
| `Swatinem/rust-cache@v2` migration — smarter cargo-side caching with better eviction | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| Coverage workflow sccache integration — needs separate validation that sccache doesn't perturb coverage instrumentation | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| Self-hosted sccache backend — only worthwhile at much higher CI volume than this project has | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| **crates.io / docs.rs badges** — both URLs require first publish | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| **`lib.rs` versioning-policy section** — text changes post-publish | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| **Facade conveniences (re-exports / helpers)** — on-demand as new crates land | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| Force-rerun escape hatch — deferred to Open questions; separate issue needed only if the team wants explicit override semantics. | [ci-skip-rust-matrix spec](../plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) |  | — |

## Out of scope

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| Windows / macOS CI runners | [github-workflow spec](../plans/done/2026-05-01-github-workflow.spec.md) | | #133 (closed) |
| Auto-merge | [github-workflow spec](../plans/done/2026-05-01-github-workflow.spec.md) | | #214 |
| Code coverage (`cargo-llvm-cov` + Codecov) | [github-workflow spec](../plans/done/2026-05-01-github-workflow.spec.md) | | #134 (closed) |
| Benchmarks (`criterion` + PR regression comments) | [github-workflow spec](../plans/done/2026-05-01-github-workflow.spec.md) | | #135 (closed) |
| Release workflow (cargo publish on tagged versions) | [github-workflow spec](../plans/done/2026-05-01-github-workflow.spec.md) | | #136 |
| Removing `rstest` from `quartzite-core` — it is genuinely used in `quartzite-core/src/value.rs` | [code-quality-cleanup spec](../plans/done/2026-05-02-code-quality-cleanup.spec.md) | | #215 |
| Any other code-quality changes not listed above | [code-quality-cleanup spec](../plans/done/2026-05-02-code-quality-cleanup.spec.md) | | untracked |
| `std` feature on `quartzite-runtime` (would gate nothing today) | [docs-and-facade spec](../plans/done/2026-05-02-docs-and-facade.spec.md) | | #216 |
| Full wildcard re-exports beyond `prelude` | [docs-and-facade spec](../plans/done/2026-05-02-docs-and-facade.spec.md) | | #217 |
| Future features (`extension`, `8k_pages`, etc.) | [docs-and-facade spec](../plans/done/2026-05-02-docs-and-facade.spec.md) | | untracked |
| `cargo doc` publishing / CI integration | [docs-and-facade spec](../plans/done/2026-05-02-docs-and-facade.spec.md) | | #137 (closed) |
| Unit tests inside example files | [examples-crate spec](../plans/done/2026-05-02-examples-crate.spec.md) | | #218 |
| Private/internal items | [public-api-docs spec](../plans/done/2026-05-02-public-api-docs.spec.md) | | #219 |
| Separate EXAMPLES.md or README additions beyond existing content | [public-api-docs spec](../plans/done/2026-05-02-public-api-docs.spec.md) | | #220 |
| Enabling GitHub Pages in Settings → Pages (user action — must be done by the repo owner after the first workflow run) | [cargo-doc-pages spec](../plans/done/2026-05-07-cargo-doc-pages.spec.md) |  | — |
| Per-PR docs preview | [cargo-doc-pages spec](../plans/done/2026-05-07-cargo-doc-pages.spec.md) |  | — |
| `--document-private-items` | [cargo-doc-pages spec](../plans/done/2026-05-07-cargo-doc-pages.spec.md) |  | — |
| Versioned/multi-version docs | [cargo-doc-pages spec](../plans/done/2026-05-07-cargo-doc-pages.spec.md) |  | — |
| Multi-OS coverage (Windows/macOS) — duplicate work, ubuntu-only is sufficient | [coverage-ci spec](../plans/done/2026-05-07-coverage-ci.spec.md) |  | — |
| Auto-fail on coverage drop — comment-only, informational only | [coverage-ci spec](../plans/done/2026-05-07-coverage-ci.spec.md) |  | — |
| Special proc-macro crate handling — `cargo-llvm-cov` v0.5+ handles proc-macros automatically | [coverage-ci spec](../plans/done/2026-05-07-coverage-ci.spec.md) |  | — |
| `quartzite-runtime/benches/` — event-loop timing dominated by OS scheduler noise on shared CI runners | [criterion-benchmarks spec](../plans/done/2026-05-07-criterion-benchmarks.spec.md) |  | — |
| Multi-platform benchmarks — statistically incoherent across OS/allocator/scheduler differences | [criterion-benchmarks spec](../plans/done/2026-05-07-criterion-benchmarks.spec.md) |  | — |
| Self-hosted runners | [criterion-benchmarks spec](../plans/done/2026-05-07-criterion-benchmarks.spec.md) |  | — |
| Object-tree lookup benchmarks (already in `quartzite-runtime/benches/object_tree.rs`) | [macro-object-bench spec](../plans/done/2026-05-07-macro-object-bench.spec.md) |  | — |
| Benchmarking the proc-macro compilation step itself | [macro-object-bench spec](../plans/done/2026-05-07-macro-object-bench.spec.md) |  | — |
| Workflow file changes (Bencher CI picks up `cargo bench --workspace` automatically) | [macro-object-bench spec](../plans/done/2026-05-07-macro-object-bench.spec.md) |  | — |
| `docs` and `features` jobs — stay on `ubuntu-latest` only (not in issue scope) | [multi-platform-ci spec](../plans/done/2026-05-07-multi-platform-ci.spec.md) |  | — |
| Rust toolchain matrix (stable only; toolchain matrix is a separate concern) | [multi-platform-ci spec](../plans/done/2026-05-07-multi-platform-ci.spec.md) |  | — |
| Windows path-length / line-ending issues (file follow-up if they surface at runtime) | [multi-platform-ci spec](../plans/done/2026-05-07-multi-platform-ci.spec.md) |  | — |
| Changing PR #182's `RUSTC_WRAPPER` / `SCCACHE_GHA_ENABLED` env vars. | [ci-rust-cache-migration spec](../plans/done/2026-05-08-ci-rust-cache-migration.spec.md) |  | — |
| Workflow-level `env:` block migration (samply variant). | [ci-rust-cache-migration spec](../plans/done/2026-05-08-ci-rust-cache-migration.spec.md) |  | — |
| Authoring an internal composite action (Mozilla pattern). | [ci-rust-cache-migration spec](../plans/done/2026-05-08-ci-rust-cache-migration.spec.md) |  | — |
| Removing sccache (Mozilla "rust-cache only" pattern). | [ci-rust-cache-migration spec](../plans/done/2026-05-08-ci-rust-cache-migration.spec.md) |  | — |
| Touching non-compile jobs: `format`, `roadmap-sync`, and the `*-pass` aggregator jobs (`build-pass`, `test-pass`, `clippy-pass`, `features-pass`, `roadmap-sync-pass`). | [ci-rust-cache-migration spec](../plans/done/2026-05-08-ci-rust-cache-migration.spec.md) |  | — |
| `format` job in ci.yml — runs `cargo fmt --check` only; no compilation, sccache wouldn't help | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| `*-pass` aggregator jobs (`build-pass`, `test-pass`, `clippy-pass`, `features-pass`, `roadmap-sync-pass`) — pure `needs:`-wait jobs, no compile | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| `roadmap-sync` — runs the gen-roadmap script, no Rust compile | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| `coverage.yml` — does not gate merge; coverage instrumentation may interact with sccache in ways that need separate evaluation | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| `docs.yml` (Pages-deploy workflow) — does not gate merge; rebuilds docs only on master push | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| `base_benchmarks.yml`, `fork_pr_benchmarks_run.yml`, `fork_pr_benchmarks_track.yml` — don't gate merge; benchmark wall-time should be measured on a stable build, not a sccache-affected one | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| Self-hosted sccache backend (S3 / Azure) — overkill at this CI volume | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| Replacing `actions/cache@v5` with `Swatinem/rust-cache@v2` — separate concern, file as follow-up if desired | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| **Additional facade conveniences (re-exports, helper functions)** — added on demand as new crates land; no spec required. | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| **API stability discussion** — covered by AGENTS.md `## API Stability` (no shims pre-crates.io). | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| Path filtering for the separate workflows: `coverage.yml`, `docs.yml`, `base_benchmarks.yml`, `fork_pr_benchmarks_run.yml`, `fork_pr_benchmarks_track.yml`. Each is its own file with its own trigger; address separately if/when they become noisy. | [ci-skip-rust-matrix spec](../plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) |  | — |
| Removing `format` or `roadmap-sync` from instruction-only PRs. | [ci-skip-rust-matrix spec](../plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) |  | — |
| Adding a `Cargo.toml`-only fast-path. `Cargo.toml` changes can affect feature flags, dependency resolution, and workspace topology — safer to run the full matrix. | [ci-skip-rust-matrix spec](../plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) |  | — |
| Changing branch-protection settings on `origin`. The existing required contexts (`Format`, `Build`, `Test`, `Clippy`, `Docs`, `Feature matrix`) are preserved by aggregator naming. | [ci-skip-rust-matrix spec](../plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) |  | — |
| A `[force-ci]` / `workflow_dispatch` escape hatch for forcing the full matrix on a doc-only PR (see § Open questions). | [ci-skip-rust-matrix spec](../plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) |  | — |

## Open questions

| Item | Source | Status | Tracked |
|------|--------|--------|---------|
| **Bench split**: accepted — tree-lookup benches live in `quartzite-runtime/benches/` to avoid circular dep. | [criterion-benchmarks design](../plans/done/2026-05-07-criterion-benchmarks.design.md) |  | — |
| **`find_by_path`**: bench `find_by_name` + `find_by_name_in` as-is; no new API needed. | [criterion-benchmarks design](../plans/done/2026-05-07-criterion-benchmarks.design.md) |  | — |
| Are there any branch-protection required-status-check rules referencing the bare job names `build`, `test`, or `clippy`? If so, they must be updated to the matrix-suffixed names (e.g. `build (ubuntu-latest)`) after this PR merges. Check the repository settings before merging. | [multi-platform-ci design](../plans/done/2026-05-07-multi-platform-ci.design.md) |  | — |
| **Cache contention with existing 10 GB GHA repo cache** — current cargo cache + sccache cache + benchmarks cache + coverage cache may collectively exceed the GHA repo limit, causing eviction churn that erodes the win. Worth observing post-merge: if cache hit rates show high churn, tune `SCCACHE_CACHE_SIZE` per OS. | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| **Does sccache benefit `cargo clippy`?** — clippy runs the full type-check / borrow-check pipeline; sccache caches up to and including codegen. Clippy may not actually drive codegen, in which case sccache hit rate on the clippy job will be lower than on build/test. Worth observing in stats. | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| **`cargo doc` interaction** — `cargo doc` invokes `rustdoc`, not `rustc` directly. sccache's wrapping is on `rustc` via `RUSTC_WRAPPER`. doc may bypass sccache entirely. Acceptable if so — `docs` job is shorter than build/test anyway. | [ci-sccache spec](../plans/done/2026-05-08-ci-sccache.spec.md) |  | — |
| Exact prose wording inside the `lib.rs` overview / per-concept-section bodies, provided each section has the structure specified (short prose + small example + sub-crate deep-link). | [project-docs design](../plans/done/2026-05-08-project-docs.design.md) |  | — |
| The exact 5 per-concept section heading names — reasonable variants (`# Signals` vs `# Signals and slots`, `# Object tree` vs `# Object hierarchy`) are interchangeable; pick the form that reads cleanest. | [project-docs design](../plans/done/2026-05-08-project-docs.design.md) |  | — |
| Per-section deep-link targets — sub-crate landing pages (`[`quartzite_core::signal`]`) vs. specific item pages (`[`quartzite_core::Signal`]`) — implementer picks based on what produces the most useful rustdoc landing for a reader who clicks through. | [project-docs design](../plans/done/2026-05-08-project-docs.design.md) |  | — |
| The exact `actions/checkout` major version pin in the new `roadmap-sync` job — query the registry per AGENTS.md `## Dependency Versions` rule. | [project-docs design](../plans/done/2026-05-08-project-docs.design.md) |  | — |
| Exact wording / structure of the project-description block (3–5 paragraphs vs. tight tagline + bullet list). | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| Exact `lib.rs` quickstart content shape — likely a counter-style example with one signal/slot pair, or a config-style example with one property. Either fits `no_run` and the conceptual focus. | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| Whether the CI sync-gate runs on all OS matrix entries or `ubuntu-latest` only (depends on bash-on-Windows behavior of the generator's `awk`/`sed` invocations). | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| Exact set of AGENTS.md bullets to excerpt into `CONTRIBUTING.md` (a "high-leverage" judgment call per the standard-depth choice). | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| Generator format choices for `ROADMAP.md`: how literally to mirror `INDEX.md` vs. how much to distill / re-narrate. | [project-docs spec](../plans/done/2026-05-08-project-docs.spec.md) |  | — |
| **Force-rerun escape hatch** (carried from spec § Open questions). Default position is "no" — contributors who want to validate the matrix against an instruction change can include a trivial whitespace edit to a `.rs` file. If this proves friction-creating in practice, open a follow-up issue to add a `[force-ci]` PR-title token or a `workflow_dispatch` trigger; not part of this design. | [ci-skip-rust-matrix design](../plans/done/2026-05-09-ci-skip-rust-matrix.design.md) |  | — |
| **Filter contract documentation.** Should the path-filter list carry an inline YAML comment naming the contract ("any future Rust artefact must be added here in the same PR that introduces it")? Recommendation: yes — one-line comment above the filter is cheap insurance against silent drift. Defer to implementer / reviewer if they prefer to keep the YAML uncommented. | [ci-skip-rust-matrix design](../plans/done/2026-05-09-ci-skip-rust-matrix.design.md) |  | — |
| **Force-rerun escape hatch**: should there be an explicit way to force the full matrix on a doc-only PR (e.g. `[force-ci]` PR title token, or `workflow_dispatch`)? Default position: no — if a contributor wants to validate the matrix against an instruction change, they can include a trivial whitespace edit to a `.rs` file (which would also serve as a clear signal in the diff). Revisit if this proves friction-creating in practice. | [ci-skip-rust-matrix spec](../plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) |  | — |
| **`actionlint` validation of the `if:` expression syntax**: confirmed expressible as `if: needs.changes.outputs.rust == 'true'` per GHA docs; design phase will verify against the live `actionlint` rule set. | [ci-skip-rust-matrix spec](../plans/done/2026-05-09-ci-skip-rust-matrix.spec.md) |  | — |
