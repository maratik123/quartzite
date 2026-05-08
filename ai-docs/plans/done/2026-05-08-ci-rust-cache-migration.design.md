# Design: CI migrate to Swatinem/rust-cache@v2; tune sccache size and cache-key strategy

**Issue:** #183
**Date:** 2026-05-08

## Approach

Replace the `actions/cache@v5` block in each of the five merge-gate compile
jobs in `.github/workflows/ci.yml` (`build`, `test`, `clippy`, `docs`,
`features`) with a `Swatinem/rust-cache@v2` step, and add
`SCCACHE_CACHE_SIZE: "2G"` to each of those same jobs' existing per-job
`env:` block alongside the `SCCACHE_GHA_ENABLED` / `RUSTC_WRAPPER` lines
landed by PR #182. The sccache install step
(`mozilla-actions/sccache-action@v0.0.10`) and its placement before the
cargo cache step are preserved exactly as they exist today; only the
caching action itself changes.

The rust-cache step is configured per the spec's Key Decisions:

- `shared-key`:
  - `build`, `test`, `clippy`, `docs`: `${{ runner.os }}-stable`
  - `features`: `${{ runner.os }}-stable-features-${{ matrix.features }}`
- `save-if: ${{ github.ref == 'refs/heads/master' }}`
- All other inputs left at their upstream defaults
  ([action.yml @ v2.9.1](https://github.com/Swatinem/rust-cache/blob/v2.9.1/action.yml)):
  `cache-targets: true`, `cache-bin: true`, `cache-all-crates: false`,
  `cache-workspace-crates: false`, `add-rust-environment-hash-key: true`,
  `cache-on-failure: false`, `prefix-key: v0-rust`,
  `env-vars: CARGO CC CFLAGS CXX CMAKE RUST`.

This produces the layered topology the spec describes: rust-cache
manages the registry, git checkout, `target/` dependency artefacts, and
installed binaries with a Cargo.lock-aware key; sccache catches the
"Cargo.lock changed but the source bytes for crate X are byte-identical"
case underneath. Workspace crates themselves are not cached and rebuild
every run — accepted per the spec's technical-constraints note for our
small workspace.

### Per-job step placement

Step ordering in every affected job remains:

```
- uses: actions/checkout@v6
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@stable
  with: { ... }                 # only on jobs that already had a `with:` block
- name: Run sccache-cache
  uses: mozilla-actions/sccache-action@v0.0.10
- name: Rust cache
  uses: Swatinem/rust-cache@<pinned major from registry query>
  with:
    shared-key: ...             # per-job value, see Decomposition
    save-if: ${{ github.ref == 'refs/heads/master' }}
- name: <existing compile step>   # unchanged
```

Two ordering points the spec mandates and this design preserves:

1. **rust-cache runs AFTER `dtolnay/rust-toolchain`** — rust-cache reads
   the active rustc version into the cache key; if it ran first, the key
   would not include the toolchain identity and a toolchain bump would
   not invalidate the cache.
2. **rust-cache runs AFTER the sccache-action install step** — choice is
   functionally equivalent either way (sccache install only drops a
   binary; it does not read or write cargo state), but matching the
   ordering already in the file (PR #182's pattern) and Mozilla's
   composite action keeps reviewer cognitive load low.

### Per-job env additions

Each of the five compile jobs already has an `env:` block from PR #182:

```yaml
env:
  SCCACHE_GHA_ENABLED: "true"
  RUSTC_WRAPPER: "sccache"
```

This becomes:

```yaml
env:
  SCCACHE_GHA_ENABLED: "true"
  RUSTC_WRAPPER: "sccache"
  SCCACHE_CACHE_SIZE: "2G"
```

The `"2G"` value is the spec's decision: it caps sccache so rust-cache
has roughly 8G of headroom inside GHA's 10G per-repo cache limit
(3 OS × ~2G dep `target/` plus four `features` matrix entries on Ubuntu).
No new env keys, no removals.

### What is NOT changed

- Step names, step order, action versions for `actions/checkout`,
  `dtolnay/rust-toolchain`, `mozilla-actions/sccache-action`.
- Workflow-level `env:` block (`CARGO_TERM_COLOR`).
- Non-compile jobs: `format`, `roadmap-sync`, and every `*-pass`
  aggregator (`build-pass`, `test-pass`, `clippy-pass`, `features-pass`,
  `roadmap-sync-pass`) — none cache cargo state today and the spec lists
  them as out of scope.
- The `RUSTC_WRAPPER` / `SCCACHE_GHA_ENABLED` env values from PR #182.
- `CARGO_INCREMENTAL` — rust-cache sets this to `0` automatically per
  its action.yml; the spec's Key Decisions accept that behaviour.

### Action source verification (per learnings.md 2026-05-08)

The implementer must read
[`Swatinem/rust-cache/blob/v2.9.1/action.yml`](https://github.com/Swatinem/rust-cache/blob/v2.9.1/action.yml)
at task start to confirm the input semantics this design relies on:

- `shared-key` participates in the cache key (it does — `inputs.shared-key`).
- `save-if` controls only the save phase, not restore (it does — the
  action restores unconditionally, gates the post-step save on this
  expression).
- The default `env-vars` value is `CARGO CC CFLAGS CXX CMAKE RUST` and
  `RUSTC_WRAPPER` therefore participates in the key by virtue of its
  `RUSTC_*` prefix not matching, but `CARGO`/`RUSTC` env values whose
  names start with those prefixes do — the spec's "toggling sccache
  produces a separate cache entry" claim depends on this. (Verify the
  prefix list at the pinned tag matches; if upstream changed the
  default between `v2.9.1` and the major resolved at task time, flag
  it before implementation.)

The spec already cites the v2.9.1 schema URL and lists the default
values; the design reuses those URLs rather than re-deriving from
training memory. **Do not** treat the version stated in this design as
authoritative — query at task time and pin to the registry-current
major (Step 1 below).

### Rejected alternatives

- **Workflow-level `env:` block migration.** Out of scope per spec
  ("samply variant"). Per-job placement keeps non-compile jobs free of
  unused sccache env vars.
- **Internal composite action (Mozilla pattern).** Out of scope per
  spec. Five jobs is below the threshold where the indirection pays off,
  and a composite would obscure the diff for reviewers.
- **Removing sccache (Mozilla "rust-cache only" pattern).** Out of
  scope per spec; sccache stays because Quartzite churns `Cargo.lock`
  often during active development and rust-cache invalidates wholesale
  on lockfile change while sccache catches "lock changed but source
  bytes identical".
- **Per-OS `shared-key` divergence beyond what the spec lists.** No
  data yet to justify; uniform per-OS keying is what Mozilla's
  composite uses.
- **`cache-on-failure: true`.** Default is `false`. Failed-build caches
  may capture half-compiled state; not worth the risk on a five-job
  fan-out.
- **`cache-all-crates: true`.** Default is `false`. Caching every crate
  in the registry would blow the 10G budget; the spec's per-job ~2G
  rust-cache target depends on the dependency-only default.
- **Removing the existing `restore-keys:` fallback semantics.** Not
  applicable — rust-cache manages restore-keys internally; switching
  caching actions implicitly drops the manual `restore-keys` array,
  which is the intended behaviour, not a regression.

## Decomposition

All edits target a single file (`.github/workflows/ci.yml`). The five
job edits are mechanical and parallel; one commit is appropriate. The
table breaks the work into reviewable units rather than separate
commits — review and bisection can isolate any per-job regression.

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Run registry-query commands. Pin `Swatinem/rust-cache` to the live-current major (expected `@v2`); confirm `mozilla-actions/sccache-action@v0.0.10` is still current; read [`action.yml`](https://github.com/Swatinem/rust-cache/blob/v2.9.1/action.yml) at the resolved tag to confirm `shared-key` / `save-if` / `env-vars` semantics still match the spec. Record observed tags + verification date in the implementation log and PR body. | — (research only) | — |
| 2 | In `build` job: replace `actions/cache@v5` step with `Swatinem/rust-cache@<pinned>` configured `shared-key: ${{ runner.os }}-stable`, `save-if: ${{ github.ref == 'refs/heads/master' }}`. Add `SCCACHE_CACHE_SIZE: "2G"` to existing `env:` block. | `.github/workflows/ci.yml` | 1 |
| 3 | In `test` job: same edits as Task 2 (`shared-key: ${{ runner.os }}-stable`). | `.github/workflows/ci.yml` | 1 |
| 4 | In `clippy` job: same edits as Task 2 (`shared-key: ${{ runner.os }}-stable`). | `.github/workflows/ci.yml` | 1 |
| 5 | In `docs` job: same edits as Task 2 (`shared-key: ${{ runner.os }}-stable`). | `.github/workflows/ci.yml` | 1 |
| 6 | In `features` job: replace `actions/cache@v5` with `Swatinem/rust-cache@<pinned>` configured `shared-key: ${{ runner.os }}-stable-features-${{ matrix.features }}`, `save-if: ${{ github.ref == 'refs/heads/master' }}`. Add `SCCACHE_CACHE_SIZE: "2G"` to existing `env:` block. | `.github/workflows/ci.yml` | 1 |
| 7 | (per-subtask check) Run `actionlint .github/workflows/ci.yml` after each of Tasks 2–6 — fast feedback on indentation / expression-syntax errors before the next edit. | — | depends on the immediately-preceding task |
| 8 | (final gate) Run `actionlint .github/workflows/ci.yml` once more after all edits, with all five jobs migrated. Resolve any remaining diagnostics. **Required gate per AGENTS.md § Build & Test before `git add`.** | `.github/workflows/ci.yml` | 2, 3, 4, 5, 6 |
| 9 | Commit + push. After push, on the resulting PR-CI run: confirm AC1 (no `actions/cache@v5` left), AC2 (rust-cache configured per spec), AC3 (`SCCACHE_CACHE_SIZE: "2G"` in each compile job's env), AC4 (rust-cache "Cache miss" line + sccache stats visible in at least one job's log on the cold PR run), AC5 (actionlint passes — automatic from Step 8). | `.github/workflows/ci.yml` | 8 |

Tasks 2–6 each touch a different region of the same file; they can be
applied as one editing pass and reviewed as one diff. Tasks 7 and 8 are
the per-subtask and final actionlint gates the project mandates.

## Implementation steps

### Step 1 — registry queries (run at task time, not pre-resolved here)

Per AGENTS.md § Dependency Versions ("Query the registry before
pinning"), the implementer runs these at task-execution time and uses
the **observed** version, not anything cached in conversation memory or
training data:

```bash
# rust-cache: latest release tag; pin to the resolved major (expected v2)
gh api /repos/Swatinem/rust-cache/releases --jq '.[0].tag_name'

# sccache-action: confirm v0.0.10 is still current (re-pin if not)
gh api /repos/Mozilla-Actions/sccache-action/releases --jq '.[0].tag_name'

# Confirm Node-runtime currency on each action's manifest
gh api /repos/Swatinem/rust-cache/contents/action.yml --jq '.content' \
    | base64 -d | grep -E 'using:|node'
gh api /repos/Mozilla-Actions/sccache-action/contents/action.yml --jq '.content' \
    | base64 -d | grep -E 'using:|node'
```

Per the spec ("pin to the major"), apply the `0.x` / `x` rule from
AGENTS.md to the rust-cache result. If `0.x.y` (unlikely; rust-cache
publishes `v2.x.y`), pin `@v2`. If `x.y.z` (current), pin `@v<major>`.

Per learnings.md 2026-05-08 ("verify a GitHub Action's actual behaviour
against its source"), additionally read the rust-cache `action.yml` at
the resolved tag (or v2.9.1 as the spec cites) and confirm:

- `inputs.shared-key.description` exists and participates in keying.
- `inputs.save-if.description` gates only the save phase.
- `inputs.env-vars.default` is `"CARGO CC CFLAGS CXX CMAKE RUST"` (the
  default the spec relies on for `RUSTC_WRAPPER` participation).
- `inputs.cache-targets.default` is `"true"`,
  `inputs.cache-all-crates.default` is `"false"`,
  `inputs.cache-workspace-crates.default` is `"false"`,
  `inputs.cache-bin.default` is `"true"`,
  `inputs.cache-on-failure.default` is `"false"`,
  `inputs.prefix-key.default` is `"v0-rust"`.

If any of those defaults have changed between `v2.9.1` and the
resolved major, stop and reconcile with the spec before editing.

Record the observed rust-cache tag + sccache-action tag + the
verification date in the PR body for the AGENTS.md long-lived-doc
annotation rule.

### Step 2 — edit ci.yml (Tasks 2–6)

In each of the five jobs (`build`, `test`, `clippy`, `docs`,
`features`), perform the same two edits.

**Edit A — replace the `actions/cache@v5` block** with:

```yaml
- name: Rust cache
  uses: Swatinem/rust-cache@<tag from Step 1>
  with:
    shared-key: <per-job value>
    save-if: ${{ github.ref == 'refs/heads/master' }}
```

`<per-job value>`:

| Job | shared-key |
|---|---|
| `build` | `${{ runner.os }}-stable` |
| `test` | `${{ runner.os }}-stable` |
| `clippy` | `${{ runner.os }}-stable` |
| `docs` | `${{ runner.os }}-stable` |
| `features` | `${{ runner.os }}-stable-features-${{ matrix.features }}` |

**Edit B — extend the per-job `env:` block** with one new line:

```yaml
env:
  SCCACHE_GHA_ENABLED: "true"
  RUSTC_WRAPPER: "sccache"
  SCCACHE_CACHE_SIZE: "2G"          # added
```

No other key reorderings. No new top-level keys.

### Step 3 — actionlint gate

```bash
actionlint .github/workflows/ci.yml
```

Required per AGENTS.md § Build & Test before `git add`. Catches:
indentation regressions from the YAML edits, deprecated action
versions, expression-syntax errors in `${{ ... }}` blocks, runner-version
mismatches. `cargo` cannot see any of these.

If the per-subtask runs in Task 7 caught issues during edits, the final
gate run (Task 8) is mostly a confirmation pass; non-zero exit blocks
the commit until fixed.

### Step 4 — verify on PR-CI

After `git push`, on the resulting PR-CI run:

- Open at least one of `Build (ubuntu-latest)`, `Test (ubuntu-latest)`,
  `Clippy (ubuntu-latest)`, `Docs`, `Feature matrix (--no-default-features)`.
- Confirm: a step named `Rust cache` ran after `Run sccache-cache` and
  before the compile step.
- Confirm: a `rust-cache` log line near the start of the cache step
  (either `Cache restored …` if the cache survived from a previous
  master push, or `Cache miss …` for the cold PR run — the spec accepts
  either as AC4-passing).
- Confirm: a sccache stats block ("Compile requests / Cache hits /
  Cache misses") at the end of the build step, only required when at
  least one rustc invocation actually ran (cargo may decide nothing
  needs rebuilding if rust-cache restored a hot `target/`; in that
  case AC4 is satisfied by the rust-cache line alone).
- Confirm: `Format`, `Build`, `Test`, `Clippy`, `Docs`, `Feature matrix`
  all pass green (AC5).

For AC4 specifically, the spec's verification protocol allows the cold
PR run to show a "Cache miss" — that is structurally correct. The
post-merge master run is what populates the cache; the next PR after
that is the first to show "Cache restored". Step 3 of the spec's
verification protocol is observed manually post-merge and does not
block this PR.

## Risks

- **Cache-budget overrun against GHA's 10G per-repo limit.**
  Pre-migration, `actions/cache@v5` consumed roughly 3 OS × ~2G = ~6G
  for `build/test/clippy` and ~2G for `docs`/`features`. Post-migration,
  rust-cache budgets ~2G per (OS, shared-key) — the same order, but
  re-keyed. The new failure mode is the four-entry `features` matrix
  on Ubuntu adding ~2G × 4 entries on top. **Mitigation:**
  `save-if: master only` guarantees feature-branch runs are read-only
  (no cache writes from PR runs). `SCCACHE_CACHE_SIZE: "2G"` caps the
  sccache layer. Observe via the rust-cache stats output on master
  pushes; if eviction churn shows up, consider tightening `shared-key`
  on the `features` job or dropping `cache-bin`.
- **`shared-key` collision between feature-branch reads and master
  saves.** Two jobs with the same `shared-key` race for the same cache
  entry on master save. With `build`, `test`, `clippy`, `docs` all
  using `${{ runner.os }}-stable`, the four save phases on each master
  push will overwrite each other's entry — last-writer-wins. Spec Key
  Decisions explicitly accept this trade-off ("jobs share `target/` on
  master save (last-writer-wins), accepted for low cache pressure").
  **Mitigation:** none required for this PR; if cross-job hit rates
  degrade, split `shared-key` per (OS, job) in a follow-up.
- **rust-cache implicit `CARGO_INCREMENTAL=0`.** rust-cache sets this
  unconditionally per its action.yml; some local mental models of
  cargo behaviour assume incremental on. Spec Key Decisions explicitly
  accept this. **Mitigation:** none required; documented behaviour.
- **YAML indentation regression.** Inserting a `with:` block under
  `Swatinem/rust-cache` and a new line in `env:` are both
  whitespace-sensitive. **Mitigation:** the per-subtask actionlint
  runs (Task 7) plus the final gate (Task 8) catch indent / expression
  errors locally; the GHA validator on push is the second net.
- **Action source-behaviour drift between `v2.9.1` (cited in spec) and
  the live-current major.** rust-cache could in principle change a
  default between minor versions in a way that invalidates the spec's
  "all other inputs at upstream defaults" claim. **Mitigation:** Step 1
  re-reads `action.yml` at the resolved tag and reconciles with the
  spec before editing — the explicit application of learnings.md
  2026-05-08 to this task.
- **`actions/cache@v5` accidentally surviving in one job.** A copy-paste
  miss across five jobs could leave a stale block. **Mitigation:** AC1
  is verified by `! grep -q 'actions/cache@v5' .github/workflows/ci.yml`
  in the post-edit verification (mechanical, fast).
- **First PR-CI run shows "Cache miss" because no master save has
  happened yet.** Reviewers unfamiliar with rust-cache's
  master-only-write design might read this as a regression.
  **Mitigation:** the spec's Verification protocol Step 3 is explicit
  about this — the design references it; the PR body should restate it
  briefly so the reviewer doesn't have to chase the spec.
- **`SCCACHE_CACHE_SIZE` underbudget for first cold runs on macOS or
  Windows.** sccache evicts when its on-disk store crosses the cap; a
  cold cold-run that compiles the entire dep tree on Windows could
  exceed 2G. **Mitigation:** spec Key Decisions document `"2G"`
  explicitly; observe sccache stats post-merge for eviction-rate
  signal; tune up if needed in a follow-up.

## Test Design

This task produces only a CI workflow YAML edit — there is no Rust
source change. There are no `#[cfg(test)]` modules to write, no
benchmarks to author, and no `panic-index.md` implications (no
production panic sites added). Verification is structural and lives in
the spec's "Verification protocol" section.

**Local lint (mandatory pre-commit, per AGENTS.md):**

- `actionlint .github/workflows/ci.yml` exits 0. Run after each
  per-job edit (Task 7) and once more after all edits (Task 8 — the
  final gate before `git add`).

**Post-push verification on PR-CI:**

| AC | Verification |
|---|---|
| AC1 | `! grep -q 'actions/cache@v5' .github/workflows/ci.yml` (or read the diff: every removed-cache block is gone, no stale references in any of the five jobs). |
| AC2 | Read the diff: each of `build`, `test`, `clippy`, `docs`, `features` has a `Swatinem/rust-cache@<pinned>` step with `shared-key` and `save-if` matching the spec's Key Decisions table. |
| AC3 | Read the diff: each of the same five jobs has `SCCACHE_CACHE_SIZE: "2G"` in its per-job `env:` block alongside the existing `SCCACHE_GHA_ENABLED` / `RUSTC_WRAPPER` lines. |
| AC4 | Open one compile job's CI log on the PR run. Expect: a `Rust cache` step with either `Cache restored from key: …` or `Cache miss …` near its start; a sccache stats block ("Compile requests / Cache hits / Cache misses") near the end of the compile step (only when at least one rustc invocation occurred — if `target/` was hot-restored and cargo determined no rebuild was needed, the rust-cache line alone satisfies AC4). |
| AC5 | `actionlint` passed locally before push (Task 8); the GHA workflow validator on push also passes (a syntax error blocks the run from starting). |

**Non-goals for this PR's verification (out of scope per spec):**

- No formal wall-clock comparison between `actions/cache@v5` and
  `rust-cache@v2`. Spec drops this as informational only — CI timing
  too noisy for hard assertion.
- No A/B run with `Swatinem/rust-cache` removed. The PR is one
  direction; any rollback would be a separate PR.
- No "Cache restored" assertion on the PR run itself. The first PR
  run is structurally guaranteed to be a cold miss (no master save
  yet); the spec's Verification protocol Step 3 is observed manually
  on the first post-merge PR.

## Open questions

_(none — spec lists no open questions; both Key Decisions and
Technical constraints are fully resolved.)_
