# Miri master-push CI job (Tree Borrows, Linux v1)

**Source:** issue #422
**Date:** 2026-05-17
**Tracked in:** #422

## Scope

1. Add a new CI workflow that runs **Miri** under **Tree Borrows** on every push to `master`, providing mechanical defense-in-depth against memory-model UB in safe and unsafe code paths.
2. The workflow triggers on `push: branches: [master]` only (no PR trigger in v1) and uses a pinned `nightly` toolchain with `components: miri, rust-src`.
3. The job runs `cargo +nightly miri test` against the **Miri-runnable subset** of the workspace: crates that do **not** depend on GPU drivers (`wgpu` / `vello`) or the platform event loop (`winit`). The starting subset is `quartzite-core`, `quartzite-geometry`, `quartzite-paint-api`, `quartzite-events`, `quartzite-paint`, `quartzite-runtime`, plus any other crate whose transitive `Cargo.toml` dep graph (audited at design time) is FFI-free. Excluded by construction: `quartzite-renderer`, `quartzite-widgets`, `quartzite-style` (winit/wgpu/vello dependents — verified by `grep -nE 'winit|wgpu|vello' <crate>/Cargo.toml` returning at least one hit per excluded crate). (An earlier draft of this spec also listed `quartzite-runtime` under the exclusion — that was a factual error: `quartzite-runtime` is FFI-free, so it belongs in the included set per the spec's own binding rule.)
4. `MIRIFLAGS=-Zmiri-tree-borrows` is set in the job environment to select the Tree Borrows aliasing model.
5. Failure policy v1: the Miri job carries `continue-on-error: true` so a Miri-flagged regression surfaces in the CI summary but does not turn master red. Promotion to hard failure (`continue-on-error: false`) is a separate follow-up after a stabilisation window of Miri-clean runs.
6. The workflow file passes `actionlint` per AGENTS.md AXIOM, and (since it is master-only) carries a `# Why master-only: <reason>` comment above the `on:` block explaining the speed trade-off and naming the equivalent PR-side gate (or its absence).
7. `ai-docs/unsafe-index.md § Notes` gains a single-line cross-reference pointing readers from the index to the Miri job, so future maintainers discover the defense-in-depth surface from either entry point.

## Out of scope

- **Cross-target Miri.** Windows, macOS, big-endian, 32-bit targets — Linux baseline only in v1.
- **Stacked Borrows alongside Tree Borrows.** v1 picks one model.
- **Miri-instrumented benchmarks** (criterion harness compatibility + runtime cost).
- **Miri against doctests** (`cargo miri test --doc` harness behaviour varies).
- **Auto-bridging Miri findings into `ai-docs/unsafe-index.md`** (Miri findings remain a CI artefact; index updates stay manual).
- **PR-side Miri trigger.** Master-only in v1; per-PR latency (10–30× native cargo test) is prohibitive.
- **Flipping `continue-on-error` to `false`.** Tracked as a deferred follow-up keyed to N consecutive Miri-clean master runs.
- **Re-routing master Miri failures into `/master-ci-failed`.** Existing flow already covers any red master CI signal; no special-case wiring required.
- **Editing AGENTS.md § Build & Test for a Miri local-reproduction recipe.** Not required by ACs; deferred follow-up if developers ask.

## Deferred

- Hard-failure flip (`continue-on-error: false`) after a stabilisation window | requires observed Miri-clean runs that cannot be locked at v1-merge time | **yes — separate issue** after first run completes.
- Stacked Borrows secondary job (cross-check against TB) | only justified if Tree Borrows lets a known-UB site slip through; speculative in v1 | **yes — follow-up issue** if/when needed.
- Cross-target Miri (Windows / macOS / big-endian / 32-bit) | Linux baseline must be green first | **yes — separate issue per target**, opened after v1 is hard-failure on Linux.
- Miri-against-doctests | harness compatibility audit needed | **yes — separate issue** if doctest coverage gap becomes load-bearing.
- Auto-bridging Miri findings into `ai-docs/unsafe-index.md` (e.g. populate the `Preferred fix` field with a Miri citation when the job flags an indexed site) | depends on the unsafe-index growing to a size where manual edits become friction | **yes — follow-up issue**, deferrable until that pressure exists.
- Local-developer Miri recipe in AGENTS.md § Build & Test (`cargo +nightly miri test -p <crate>` boilerplate) | not required to land the CI gate | **no — deferred row in `ai-docs/deferred/*.md`** suffices.

## Key decisions

| Question | Decision |
|---|---|
| Trigger | `on: push: branches: [master]` only. No `pull_request:` sibling — see *Out of scope* and the master-only comment requirement (AC3). |
| Toolchain | Pinned `nightly-YYYY-MM-DD` (specific date selected at design time from a recent known-good nightly; the date is recorded in the workflow file and updated as part of a normal dependency-bump rotation). `components: miri, rust-src` required. |
| Aliasing model | **Tree Borrows** — `MIRIFLAGS=-Zmiri-tree-borrows`. Rationale: leading candidate for Rust's official aliasing model; far fewer false positives in `Cell`/`RefCell`/`Rc`/`Arc` patterns than Stacked Borrows; if it proves too permissive, the symmetric flip cost is a flag swap. |
| Crate-set selection syntax | Either `cargo +nightly miri test --workspace --exclude <gpu/event-loop crates>` **or** explicit `cargo +nightly miri test -p A -p B …`. Design phase picks based on the audited crate list and the prevailing workspace convention (ci.yml uses `--workspace` throughout). The chosen form must be FFI-free by construction — exclude the crates enumerated in § Scope item 3's "Excluded by construction" list (`quartzite-renderer`, `quartzite-widgets`, `quartzite-style` — winit/wgpu/vello dependents) plus any crate whose `Cargo.toml` transitively pulls in `winit`, `wgpu`, or `vello`. |
| Workflow file structure | New standalone `.github/workflows/miri.yml` (matching the existing convention used by `coverage.yml`, `docs.yml`). Separate failure budget, separate cache key, separate scheduling. **Not** a new job inside `ci.yml`. |
| Cache strategy | Reuse the project's existing pattern: `actions-rust-lang/setup-rust-toolchain@v1` with a fresh `cache-shared-key` segment (e.g. `${{ runner.os }}-nightly-miri-${{ env.ImageVersion }}`) so Miri MIR caches do not collide with the stable-toolchain caches used by `ci.yml`. The `cargo +nightly miri setup` step runs once per cache-miss. (Whether to gate that step on `cache-hit` is design-detail.) |
| Failure policy v1 | `continue-on-error: true` on the Miri job. Surfaces results in the CI summary, does not block master push. Flip to `false` is a deferred follow-up. |
| Master-only comment | Per AGENTS.md AXIOM (master-only workflows): the file carries a `# Why master-only: <reason>` comment above the `on:` block. Reason text names the cost (Miri is 10–30× slower than native cargo test) and the policy (results visible in `/master-ci-failed` surface, no per-PR latency tax). |
| `ai-docs/unsafe-index.md` cross-reference | A one-line addition to `§ Notes` (≲ 100 chars) such as: "A Miri job runs on every master push under Tree Borrows; see `.github/workflows/miri.yml`." |
| Symmetry follow-up note (from issue context) | Do **not** add an Agent Docs row for `.github/workflows/miri.yml` in AGENTS.md. Agent Docs lists `ai-docs/**` reference pages, not CI workflows. The workflow file is discovered via its `.github/workflows/` location, not the Agent Docs table. The deferred symmetry question (#420 / `_inbox.md`) is unaffected by this spec. |
| `learnings.md` posture | Miri CI output is external content; per existing CI-skill convention (`/pr-ci-failed`, `/master-ci-failed` "never edit `learnings.md`" rule), CI fixes flowing from Miri findings do not append learnings unless the fix is an in-`/task` flow per the boundary-rule-2 in-flow exception. No special-case rule needed in this spec. |

## Technical constraints

- **AGENTS.md AXIOM — `actionlint` MUST pass before `git add`** on `.github/workflows/miri.yml`. (Build & Test § AXIOM.)
- **AGENTS.md AXIOM — master-only workflow guard.** The file must carry the `# Why master-only: <reason>` comment AND the spec / commit must record that no PR-side workflow exercises an equivalent Miri path (because there isn't one — that's the whole point of the new gate). The `learnings.md` 2026-05-13 master-only-trigger entry is the precedent for why this is enforced.
- **AGENTS.md § Dependency Versions** — when the design agent writes the specific nightly date and any action versions (`actions-rust-lang/setup-rust-toolchain@v1`, `actions/checkout@v6`, …), it queries the live registry per the AXIOM and applies the `^` semantics rule. The Miri job is **not** a Cargo dep — it pins a *toolchain* nightly date directly; the live-registry rule still applies in spirit (use a recent known-good nightly, not a remembered one).
- **The two indexed `unsafe` sites are Miri-unrunnable.** Both live in `quartzite-renderer` which depends on `winit`/`wgpu`/`vello` and is excluded from the Miri subset by construction (issue context). The value of this job is therefore (a) safe-code aliasing coverage in the included crates, and (b) readiness for any future production `unsafe` block that lands in a Miri-runnable crate. The spec records this explicitly so the design agent and self-review do not chase the false expectation that Miri must test the indexed sites in v1.
- **`quartzite-macros` is a proc-macro crate.** If it is in the subset, Miri runs against the crate's own non-proc-macro tests; Miri does not interpret proc-macro expansion. Design agent audits this and either includes or excludes the crate based on whether it has Miri-runnable test coverage.
- **`SCCACHE_GHA_ENABLED` is workspace-wide pattern in `ci.yml`** — the Miri workflow may opt out of `sccache` since Miri's MIR caches are managed independently and the rust-cache shared-key segment already isolates artifacts. Design-detail; either approach satisfies the spec.
- **Workflow file size budget** is unconstrained — the Miri workflow is a single-job file similar in size to `docs.yml` (~50–80 lines).
- **`ai-docs/unsafe-index.md`** = 5,589 chars (issue context). The one-line addition keeps it well under any extraction threshold.
- **AGENTS.md** = 33,328 chars (issue context). No mandatory edits from this spec; if a § Build & Test sentence is added describing the Miri job, the early-warning band (35,000) is the cap to watch.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.github/workflows/miri.yml` exists. Triggers on `push: branches: [master]` (and only that — no `pull_request:` sibling, by design). Runs on `ubuntu-latest`. |
| AC2 | The job installs a pinned `nightly-YYYY-MM-DD` with `components: miri, rust-src` via the same toolchain action used elsewhere in the workspace (`actions-rust-lang/setup-rust-toolchain@v1`), runs `cargo +nightly miri setup` (gated or unconditional), and then runs `cargo +nightly miri test` against the FFI-free crate subset chosen at design time. `MIRIFLAGS=-Zmiri-tree-borrows` is present in the job or step env. |
| AC3 | The workflow file carries a `# Why master-only: <reason>` comment above the `on:` block per the AGENTS.md AXIOM. The reason text names (a) the latency cost (10–30× native `cargo test`) and (b) that no PR-side workflow exercises an equivalent Miri path, so the master-only gate is the intended defense-in-depth surface. |
| AC4 | The Miri job carries `continue-on-error: true` for v1. (The flip to `false` is a separate, deferred issue.) |
| AC5 | `actionlint .github/workflows/miri.yml` passes locally and in CI. |
| AC6 | `ai-docs/unsafe-index.md § Notes` gains a one-line cross-reference to the Miri job (≲ 100 chars). |
| AC7 | The crate subset under test is FFI-free: a grep of the chosen invocation against `**/Cargo.toml` confirms that none of the included crates transitively depend on `winit`, `wgpu`, or `vello`. If `--workspace --exclude` form is used, the exclude list is exhaustive against the GPU/event-loop dependents. The design agent records the audit method in the design doc; self-review walks it. |
| AC8 | The first push of `miri.yml` to master produces a runnable CI artifact: either a green Miri pass, OR a list of findings (one issue opened per finding, OR a documented `# Miri: <reason>` exclusion / `MIRIFLAGS` adjustment per finding with that decision recorded in the PR description). The artifact is grep-able by a future `/ai-audit` run. |
| AC9 | The cache-shared-key for the Miri job is **distinct** from the `*-stable*` keys used by `ci.yml`'s build / test / clippy / docs jobs (e.g. contains `nightly-miri`). Verified by inspection of the workflow file. |

## Open questions

- **Initial nightly pin date.** Design agent selects from the most recent known-good nightly available at design time; the spec does not lock a date because the calendar at merge time depends on when design lands. Update cadence (manual bump when Miri or rust-src breaks against the pinned nightly) is recorded in the design doc but not in v1 ACs.
- **`--workspace --exclude` vs explicit `-p` list.** Design agent picks based on the audited crate set. Trade-off recorded in *Key decisions*; either form satisfies AC2 and AC7.
- **`cargo +nightly miri setup` cache gating.** Whether to gate the setup step on `steps.<cache-step>.outputs.cache-hit != 'true'` (Swatinem-style) vs running unconditionally (idempotent). Design-detail; either is acceptable.
- **`sccache` opt-in for the Miri job.** Existing workspace pattern enables `sccache` for all native build/test jobs. The Miri job's MIR pipeline derives little benefit from `sccache` (Miri caches MIR, not rustc object files); design agent picks. Either decision satisfies the ACs.
- **Threshold for the deferred hard-failure flip.** Issue suggests "~2 weeks of Miri-clean runs" — the concrete number of consecutive green runs / calendar window is left to the deferred follow-up issue, decided based on observed run cadence.
