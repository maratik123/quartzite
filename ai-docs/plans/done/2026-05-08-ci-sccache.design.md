# Design: CI sccache layer for compiler-artefact caching

**Issue:** #178
**Date:** 2026-05-08

## Approach

Add a single `Run sccache-cache` step (using `mozilla-actions/sccache-action`)
to every merge-gate job in `.github/workflows/ci.yml` that compiles Rust:
`build`, `test`, `clippy`, `docs`, and `features`. The action wraps the
toolchain by exporting `RUSTC_WRAPPER=sccache` into the job environment and
configures the GitHub Actions cache as the sccache backend (the default
`SCCACHE_GHA_ENABLED=true` mode of the action). All subsequent `cargo`
invocations in the job route compiler calls through sccache transparently.

The existing `actions/cache@v5` block (cargo registry + `target/`) stays
untouched in every job. sccache layers under it: the cargo cache covers
`target/` and the registry, keyed on `Cargo.lock`; sccache covers per-source
compiler outputs (object files / `rlib`s) keyed on source-content hash.
The two layers are complementary, not redundant — when `Cargo.lock` changes
and the cargo cache is invalidated, sccache still hits when source bytes
have not changed.

### Per-job step placement

Per the spec's "Per-job placement" constraint, the sccache-action step is
inserted **after** `dtolnay/rust-toolchain@stable` and **before**
`actions/cache@v5`. This ordering ensures `RUSTC_WRAPPER=sccache` is
exported into the job environment before any cargo-aware step runs,
including the cache restore (which does not itself invoke `cargo`, but
keeping the action ordering uniform across jobs reduces cognitive load
during review).

Concretely, every affected job's `steps:` list becomes:

```
- uses: actions/checkout@v6
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@stable
  with: { ... }            # only on jobs that already had a `with:` block
- name: Run sccache-cache
  uses: mozilla-actions/sccache-action@<pinned major>
- name: Cache dependencies
  uses: actions/cache@v5
  with: { ... }            # unchanged
- name: <existing compile step>   # unchanged
```

No `env:` block, no `with:` block on the sccache-action step is required —
the action's defaults (`SCCACHE_GHA_ENABLED=true`, `RUSTC_WRAPPER=sccache`)
match the spec's "GHA-backed (default)" decision and the "Leave default"
cache-size decision. Stats emission at end-of-job is also default behaviour
of the action and satisfies AC4 without any extra step.

### Rejected alternatives

- **`Swatinem/rust-cache@v2`** — out of scope per spec ("file as follow-up
  if desired"). It would replace the existing `actions/cache@v5` block, not
  layer with sccache. Different concern, different PR.
- **Per-OS sccache version overrides in the matrix** — rejected per spec
  ("Matrix consistency — same sccache version across all matrix entries").
  No data yet to justify divergence.
- **Custom `SCCACHE_CACHE_SIZE`** — rejected per spec ("Leave default").
  Tunable later if 10 GB GHA repo cache shows pressure.
- **`continue-on-error: true` on the sccache step** — rejected per spec
  ("Fail loud — surfaces regressions immediately"). A misconfigured
  sccache wrapper that breaks `cargo build` should block merge by design.
- **Adding sccache to `coverage.yml` / benchmark workflows** — out of scope
  per spec. Coverage instrumentation interaction is unevaluated; benchmarks
  intentionally measure unwrapped wall time.
- **Adding sccache to the `format` job** — out of scope per spec
  (`cargo fmt --check` does not compile).

## Decomposition

All edits target the same file (`.github/workflows/ci.yml`). They are
mechanical and parallel; one commit is appropriate. The task table breaks
the work into reviewable units rather than separate commits.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Run registry-query commands to obtain live-current `mozilla-actions/sccache-action` major tag and confirm the action's Node-runtime currency. Record the tag + verification date in the implementation log (and PR body, per AC3). | — (research only) | — |
| 2 | Insert `Run sccache-cache` step into the `build` job between `Install Rust toolchain` and `Cache dependencies`. | `.github/workflows/ci.yml` | 1 |
| 3 | Insert `Run sccache-cache` step into the `test` job in the same position. | `.github/workflows/ci.yml` | 1 |
| 4 | Insert `Run sccache-cache` step into the `clippy` job in the same position. | `.github/workflows/ci.yml` | 1 |
| 5 | Insert `Run sccache-cache` step into the `docs` job in the same position. | `.github/workflows/ci.yml` | 1 |
| 6 | Insert `Run sccache-cache` step into the `features` job in the same position. | `.github/workflows/ci.yml` | 1 |
| 7 | Run `actionlint .github/workflows/ci.yml`; resolve any diagnostics; commit. After push, verify on the PR-CI run that sccache stats are emitted in at least one job log (AC4) and all required checks pass (AC5). | `.github/workflows/ci.yml` | 2, 3, 4, 5, 6 |

Tasks 2–6 are independent edits in different regions of the same file and
can be applied in one editing pass. They are listed separately so that
review and (if needed) bisection can isolate any per-job regression.

## Implementation steps

### Step 1 — registry query (run at task time, not pre-resolved here)

Per AGENTS.md § Dependency Versions ("Query the registry before pinning"),
the implementer runs these commands at task execution time and uses the
**observed** version, not anything cached in conversation memory or
training data:

```bash
# Latest release tag (apply x / 0.x pinning rule per AGENTS.md to the result)
gh api /repos/Mozilla-Actions/sccache-action/releases --jq '.[0].tag_name'

# Confirm Node-runtime currency on the action's manifest
gh api /repos/Mozilla-Actions/sccache-action/contents/action.yml --jq '.content' \
    | base64 -d \
    | grep -E 'using:|node'
```

Pin to the **major** the registry returns (e.g. if the tag is `v0.0.9`,
the design treats this as a `0.x`-style pinning rule and the spec's intent
is "live-current major"; if Mozilla starts publishing `v1.x.y`, pin `@v1`).
Record the tag plus the verification date in the PR body for AC3.

If the Node-runtime grep shows a deprecated runtime (Node 16 or earlier),
flag it before pinning — per AGENTS.md, that is the failure mode the
runtime check exists to catch.

### Step 2 — edit ci.yml

Add the step block below into each of the five affected jobs, between
`Install Rust toolchain` and `Cache dependencies`:

```yaml
- name: Run sccache-cache
  uses: mozilla-actions/sccache-action@<tag from Step 1>
```

No `env:` and no `with:` block. The action sets `RUSTC_WRAPPER=sccache`
and `SCCACHE_GHA_ENABLED=true` itself; both are required and both are the
defaults.

### Step 3 — verify locally

```bash
actionlint .github/workflows/ci.yml
```

Fix any diagnostics before staging. `actionlint` catches indentation
errors, deprecated action versions, expression-syntax errors, and shell
quoting issues that `cargo` checks cannot see. Per AGENTS.md, this is a
required gate.

### Step 4 — verify on PR-CI

After `git push`, on the resulting PR-CI run:

- Open at least one of `Build (ubuntu-latest)`, `Test (ubuntu-latest)`,
  `Clippy (ubuntu-latest)`, `Docs`, `Feature matrix (*)`. Confirm a step
  named `Run sccache-cache` ran near the start of the job and a sccache
  stats block ("Compile requests", "Cache hits", etc.) appears near the
  end of the log. (sccache-action emits stats automatically by default —
  no extra step required.) **AC4 satisfied.**
- Confirm Format, Build, Test, Clippy, Docs, Feature matrix all pass green.
  **AC5 satisfied.**

## Risks

- **Cache contention with the existing 10 GB GHA repo cache.** The repo
  already runs cargo cache (multi-OS, several keys), benchmarks, and
  coverage caches. Adding sccache claims more cache budget. **Mitigation:**
  ship at default `SCCACHE_CACHE_SIZE` per spec; observe hit rate and
  eviction churn in the post-merge sccache stats blocks. If churn is
  visibly high (low hit rate after a warm-up period), tune
  `SCCACHE_CACHE_SIZE` per OS or evict less-valuable caches first
  (benchmarks have separate retention policies). Tracked as the spec's
  first open question.
- **clippy may not benefit from sccache.** sccache caches `rustc` outputs
  via `RUSTC_WRAPPER`; `cargo clippy` runs `clippy-driver` (a `rustc`
  fork) and may or may not drive codegen. Hit rate on the `clippy` job
  may be substantially lower than `build` / `test`. **Mitigation:**
  observe in stats; this is acceptance, not a blocker — the spec's second
  open question already calls this out. If clippy gains nothing and the
  cache cost is non-trivial, removing sccache from `clippy` only is a
  cheap follow-up.
- **`cargo doc` may bypass sccache entirely.** `cargo doc` invokes
  `rustdoc`, not `rustc`; `RUSTC_WRAPPER` does not wrap `rustdoc`.
  **Mitigation:** acceptable per spec ("doc may bypass sccache entirely
  — acceptable if so"). The `docs` job is short anyway.
- **Action version drift between authoring and merge.** If the spec /
  design / PR pin lag behind the action's `main` branch by weeks, a
  reviewer running the registry query later sees a different tag.
  **Mitigation:** spec mandates live-current tag at task-execution time
  and explicit citation in the PR body with the verification date.
- **YAML indentation regressions.** Inserting a step at the wrong indent
  would silently break a job. **Mitigation:** `actionlint` gate (Step 3).
- **`RUSTC_WRAPPER` collision.** If a future change adds another
  `RUSTC_WRAPPER` (e.g. for a custom linker driver), the latter would
  override sccache and silently disable the layer. **Mitigation:** none
  pre-emptive; sccache stats showing zero compile requests in a future
  CI log is the early-warning signal.
- **Fail-loud consequence.** A transient sccache-action outage (GHA
  cache backend unavailable) would fail the merge gate. **Mitigation:**
  spec accepts this trade-off explicitly ("Fail loud — surfaces
  regressions immediately"). If outages prove repeated, revisit with
  `continue-on-error: true` as a follow-up.

## Test Design

This task produces only a CI workflow YAML edit — no Rust source changes.
There is no `#[cfg(test)]` module to write. Validation is CI-level:

**Local lint (mandatory pre-commit):**

- `actionlint .github/workflows/ci.yml` exits 0. Required gate per AGENTS.md.

**Post-push verification on PR-CI (one PR-CI run is sufficient):**

| AC | Verification |
|---|---|
| AC1 | Read the diff in the PR: every job in `{build, test, clippy, docs, features}` has a `mozilla-actions/sccache-action@<pinned>` step before its compile step. |
| AC2 | `actionlint` passed locally before push (Step 3); the GHA workflow validator on push also passes (a syntax error blocks the run from starting). |
| AC3 | PR body cites the pinned tag and the date the registry query was run. |
| AC4 | Open any compile job log; locate the `Run sccache-cache` step; locate the end-of-job sccache stats block (the action prints "Compile requests / Cache hits / Cache misses" automatically). One job log suffices. |
| AC5 | All required checks pass on the PR (Format, Build, Test, Clippy, Docs, Feature matrix). |

**Non-goals for this PR's verification (out of scope per spec):**

- No formal Windows-runtime measurement (≥30% reduction). Spec dropped
  this; observe in GitHub Actions UI post-merge instead.
- No comparison run with sccache disabled to A/B the cache-hit rate.
  First-run stats (cold cache) are not informative; second-run-onwards
  observation is informal and post-merge.

## Open questions

_(none — spec's two open questions are explicit observe-and-tune
items for post-merge, not blockers for this design.)_
