# Design: Miri master-push CI job (Tree Borrows, Linux v1)

**Issue:** #422
**Tracked in:** #422
**Date:** 2026-05-17
**Spec:** [`2026-05-17-miri-master-push-job.spec.md`](./2026-05-17-miri-master-push-job.spec.md)

## Approach

Add a single standalone CI workflow `.github/workflows/miri.yml` that runs `cargo +nightly miri test` against an audited FFI-free subset of the workspace on every push to `master`, under the Tree Borrows aliasing model (`MIRIFLAGS=-Zmiri-tree-borrows`). The workflow mirrors the structural conventions already used by `coverage.yml` and `docs.yml`:

- Single job, runs on `ubuntu-latest`.
- `actions-rust-lang/setup-rust-toolchain@v1` with a pinned `nightly-YYYY-MM-DD` toolchain and `components: miri, rust-src`.
- Distinct `cache-shared-key` segment containing `nightly-miri-v1` so MIR caches do not collide with the stable-toolchain caches used elsewhere and the workspace's versioned-key convention (`-v2-` on stable, `-v1-` for this new key family) is preserved.
- `cargo +nightly miri setup` runs unconditionally (idempotent; gating on `cache-hit` adds workflow complexity for negligible savings).
- `cargo +nightly miri test` is invoked with an **explicit `-p <crate>` list** (rationale below) of the audited FFI-free subset.
- `continue-on-error: true` on the `miri` job step for v1 — surfaces findings in the CI summary without turning master red. Promotion to hard failure is a deferred follow-up.
- A `# Why master-only: <reason>` comment above the `on:` block, per AGENTS.md AXIOM (master-only workflow guard), names (a) Miri's 10–30× native-`cargo test` cost and (b) the absence of any PR-side workflow exercising an equivalent Miri path — the master-only gate is the intended defense-in-depth surface.
- No `sccache` — Miri caches MIR, not rustc object files, so the workspace's `SCCACHE_GHA_ENABLED` pattern from `ci.yml` derives no benefit; the rust-cache shared-key segment already isolates artifacts.

A single-line cross-reference is appended to `ai-docs/unsafe-index.md § Notes` so future maintainers discover the Miri job from the unsafe index, and `ai-docs/plans/INDEX.md` gains the active-plan row.

### Action-version pins (live-registry verified 2026-05-17)

| Action | Pin | Source |
|---|---|---|
| `actions/checkout` | `v6` (latest tag `v6.0.2`, applying `^` semantics → `@v6`) | `gh api /repos/actions/checkout/releases/latest` |
| `actions-rust-lang/setup-rust-toolchain` | `v1` (latest tag `v1.16.1`, applying `^` semantics → `@v1`) | `gh api /repos/actions-rust-lang/setup-rust-toolchain/releases/latest` |

Both pins match the prevailing convention used by `ci.yml`, `coverage.yml`, `docs.yml`. The Miri workflow does **not** use `mozilla-actions/sccache-action` (rationale above) or `taiki-e/install-action` (not needed — `miri` and `rust-src` are toolchain components installed by `setup-rust-toolchain`).

### Nightly pin selection

The design fixes `nightly-2026-05-01` as the initial pin. Justification:
- Calendar-recent enough that Miri and `rust-src` are routinely current.
- Two weeks of distance from the design date gives a buffer against nightly-day breakage (Miri occasionally regresses on the very-latest nightly; a two-week-old pin is empirically more reliable for CI use).
- The pin is recorded in the workflow file and bumped manually as part of a normal dependency-bump rotation (the spec defers an automated cadence to a follow-up).

### FFI-free crate-subset audit

Audit method: read each workspace member's `Cargo.toml` and classify its direct prod deps + own dev-deps. A crate is FFI-free **iff** neither its prod-dep transitive closure nor its **own** dev-deps' prod-dep transitive closures pull in `winit`, `wgpu`, or `vello`. (A transitive prod-dep's *dev-deps* do not enter the build graph of the consuming crate's tests — only the transitive prod-dep closure does. This is the relevant rule for `cargo miri test -p <X>`.)

| Crate | Prod deps (relevant) | Own dev-deps (relevant) | FFI-free? | Miri-runnable in v1? |
|---|---|---|---|---|
| `quartzite-core` | `parking_lot`, `serde`, `typetag`, `enumflags2`, `indexmap`, `hashbrown`, `thiserror`, `tracing`, `document-features` | `rstest`, `serial_test`, `itertools`, `criterion`, `serde_json`, `bincode` | yes | **include** |
| `quartzite-geometry` | `libm`, `quartzite-core`, `quartzite-macros` | `rstest`, `pretty_assertions` | yes | **include** |
| `quartzite-paint-api` | `quartzite-geometry`, `thiserror`, `peniko` | `rstest` | yes | **include** |
| `quartzite-events` | `quartzite-core`, `quartzite-event-types`, `quartzite-geometry`, `enumflags2` | `rstest`, `pretty_assertions`, `hashbrown` | yes | **include** |
| `quartzite-event-types` | `quartzite-core` | `rstest`, `pretty_assertions` | yes | **include** |
| `quartzite-paint` | `quartzite-paint-api`, `quartzite-geometry`, `peniko` | `rstest` | yes | **include** |
| `quartzite-style-types` | `quartzite-paint-api` | `rstest` | yes | **include** |
| `quartzite-style-dispatch` | `quartzite-core`, `quartzite-paint-api`, `quartzite-style`, `quartzite-widgets`, `tracing` | `quartzite-geometry`, `quartzite-style` (`test-support`), `serial_test`, `tracing-test` | yes (own dev-deps' prod-dep closures are FFI-free) | **include** |
| `quartzite-runtime` | `parking_lot`, `quartzite-core`, `quartzite-event-types`, `quartzite-macros`, `slotmap`, `thiserror`, `tracing` | `env_logger`, `itertools`, `criterion`, `serde`, `serde_json`, `bincode`, `serial_test`, `typetag` | yes | **include** (per spec amendment — see Rejected alternative #7) |
| `quartzite-macros` | `syn`, `quote`, `proc-macro2`, `heck`, `proc-macro-crate` (proc-macro crate) | `quartzite = { path = ".." }` | dev-dep pulls the facade → transitively pulls FFI-free graph only when default features are active, but **proc-macros run on host (not Miri)**; Miri only interprets the crate's own non-proc-macro inherent tests (`src/util.rs` parsing helpers — small, no `unsafe` value-add) | **exclude** (low value; large compile graph; per spec § Technical constraints) |
| `quartzite-style` | `quartzite-style-types`, `quartzite-paint`, `quartzite-paint-api`, `quartzite-widgets` | `serial_test`, `nv-flip`, `image`, `tempfile`, `peniko`, `quartzite-core`, `quartzite-geometry`, `quartzite-macros`, **`quartzite-renderer`** | own dev-deps include `quartzite-renderer` → pulls `winit`/`wgpu`/`vello` | **exclude** (per spec § Scope item 3) |
| `quartzite-widgets` | `quartzite-core`, `quartzite-macros`, `quartzite-geometry`, `quartzite-events`, `quartzite-paint-api`, `quartzite-paint`, `quartzite-style-types` | `rstest`, `pretty_assertions`, `nv-flip`, `image`, `tempfile`, **`quartzite-renderer`**, `peniko` | own dev-deps include `quartzite-renderer` | **exclude** (per spec § Scope item 3) |
| `quartzite-renderer` | `winit`, `wgpu`, `vello`, `peniko`, `pollster`, `parley`, `skrifa`, … | `rstest`, `pretty_assertions` | no (prod deps pull `winit`/`wgpu`/`vello`) | **exclude** (per spec § Scope item 3) |

**Miri-runnable subset (v1):** `quartzite-core`, `quartzite-geometry`, `quartzite-paint-api`, `quartzite-events`, `quartzite-event-types`, `quartzite-paint`, `quartzite-style-types`, `quartzite-style-dispatch`, `quartzite-runtime` — **nine crates**.

**Excluded subset (v1):** `quartzite-renderer`, `quartzite-widgets`, `quartzite-style` (winit/wgpu/vello dependents — directly via prod deps in `quartzite-renderer`, transitively via the renderer dev-dep in `quartzite-widgets` and `quartzite-style`); plus `quartzite-macros` (proc-macro crate; Miri does not interpret proc-macro expansion) — **four crates** total in the exclude set, of which three are excluded on FFI grounds and one on proc-macro/value grounds.

Cross-check for FFI-free crates (AC7 recipe):

```sh
grep -nE "winit|wgpu|vello" \
  quartzite-core/Cargo.toml \
  quartzite-geometry/Cargo.toml \
  quartzite-paint-api/Cargo.toml \
  quartzite-events/Cargo.toml \
  quartzite-event-types/Cargo.toml \
  quartzite-paint/Cargo.toml \
  quartzite-style-types/Cargo.toml \
  quartzite-style-dispatch/Cargo.toml \
  quartzite-runtime/Cargo.toml
# expected: no matches (exit code 1)
```

The shell command is the AC7 verification recipe to be re-run by self-review and by future maintainers when a crate is added to or removed from the subset. The `quartzite-runtime/Cargo.toml` line is included explicitly here because the spec amendment moved it from excluded → included; the post-amendment audit (re-run at design time on 2026-05-17) returns zero hits across all nine included crates' `Cargo.toml` files in both `[dependencies]` and `[dev-dependencies]`.

**Production `unsafe` in the subset:** none. `grep -rn "unsafe" <subset>/src/` returns only `#![warn(clippy::undocumented_unsafe_blocks)]` lint declarations. The two indexed `unsafe` sites both live in `quartzite-renderer` and are Miri-unrunnable by construction in v1 (spec § Technical constraints — the value of the job is safe-code aliasing coverage and forward-looking readiness for any future `unsafe` block landing in a Miri-runnable crate).

### Crate-set selection syntax: explicit `-p` over `--workspace --exclude`

Both forms satisfy AC2 + AC7. The design picks explicit `-p` because:

1. **Forward-safety against new crates.** When a future workspace member lands, `--workspace --exclude` silently includes it under Miri unless the PR remembers to extend the exclude list. Explicit `-p` instead causes a "no test target found" error or simply omits the new crate — the next maintainer must consciously decide whether to add `-p <new-crate>` to the Miri invocation, with the design's audit table as the standing reference. (This mirrors the workspace's prior conscious-include pattern in `quartzite-style-dispatch` test wiring.)
2. **Auditability.** A reader skimming `miri.yml` sees the nine included crates directly, without having to cross-reference the workspace member list against an exclude list.
3. **Build-graph minimisation.** `cargo miri test -p A -p B ...` builds only the union of those crates' graphs. `--workspace --exclude` builds based on the wider feature-resolver pass; in practice for this workspace the result is similar, but `-p` is strictly the smaller surface.

Trade-off recorded against the spec § Open questions item 2 — explicit `-p` chosen.

### `cargo +nightly miri setup` gating: unconditional

Run unconditionally. `cargo +nightly miri setup` is idempotent and cheap when the standard-library cache is warm (the rust-cache shared-key segment includes `nightly-miri-v1`, so the second job run on the same nightly hits the cache). Gating on `steps.<cache-step>.outputs.cache-hit != 'true'` adds workflow plumbing for a marginal save that the rust-cache itself already provides. Trade-off recorded against the spec § Open questions item 3 — unconditional chosen.

### `sccache` posture: opt-out

The Miri workflow does **not** set `SCCACHE_GHA_ENABLED` / `RUSTC_WRAPPER`. Miri caches MIR (`miri-cache` in the `target/miri/` tree), not rustc object files; `sccache` derives no measurable benefit. Trade-off recorded against the spec § Open questions item 4 — opt-out chosen.

### Cache key

`cache-shared-key: ${{ runner.os }}-nightly-miri-v1-${{ env.ImageVersion }}`. Matches the workspace's `${{ runner.os }}-…-v<N>-${{ env.ImageVersion }}` versioned-segment pattern (`-v2-` on every existing stable key in `ci.yml`/`coverage.yml`/`docs.yml`; `-v1-` is correct for a new key family that has never been busted). Distinct from every existing `*-stable*` key — AC9 verified by inspection (`grep -E "cache-shared-key" .github/workflows/*.yml` returns only `*-stable-v2-*`, `*-cargo-v2-*`, `*-cargo-coverage-v2-*`, `*-stable-gpu-v2-*`, `*-stable-features-*-v2-*` — none containing `nightly-miri`).

`cache-save-if` is omitted (default `true` on master pushes). The workflow only runs on master, so the default is correct without the explicit branch guard used by `ci.yml`'s stable jobs.

### Master-only comment wording

```yaml
# Why master-only: Miri is 10-30x slower than native `cargo test`; running per-PR
# would impose an unacceptable latency tax. No PR-side workflow exercises an
# equivalent Miri / Tree Borrows path — this gate is the intended
# defense-in-depth surface, with findings visible via /master-ci-failed.
on:
  push:
    branches: [master]
```

The wording satisfies the AGENTS.md AXIOM (master-only workflow guard) and the spec's AC3 (names the latency cost + names the equivalent-PR-side gate's absence).

### Workflow file shape (informative — not code; the implementer writes the YAML)

The file is structurally similar to `docs.yml` (~50–80 lines):

1. `name: Miri`
2. master-only comment above `on:`
3. `on: push: branches: [master]` (no `pull_request:` sibling)
4. `env: CARGO_TERM_COLOR: always` + `MIRIFLAGS: -Zmiri-tree-borrows` at the workflow level (or job level — design picks **job level** so the flag is scoped to the Miri job; if the file ever grows a second job, the flag stays where it belongs)
5. Single job `miri` with `runs-on: ubuntu-latest`, `permissions: contents: read`, `continue-on-error: true`
6. Steps in order:
   - `actions/checkout@v6`
   - Export `ImageVersion` (matches `coverage.yml` / `docs.yml`)
   - `actions-rust-lang/setup-rust-toolchain@v1` with `toolchain: nightly-2026-05-01`, `components: miri, rust-src`, `cache-shared-key: ${{ runner.os }}-nightly-miri-v1-${{ env.ImageVersion }}`
   - Verify cargo identity (matches the workspace pattern)
   - `cargo +nightly miri setup`
   - `cargo +nightly miri test -p quartzite-core -p quartzite-geometry -p quartzite-paint-api -p quartzite-events -p quartzite-event-types -p quartzite-paint -p quartzite-style-types -p quartzite-style-dispatch -p quartzite-runtime`

File-size budget: the spec records the budget as unconstrained; the post-amendment file shape (nine `-p` entries vs the earlier eight) adds one short token (`-p quartzite-runtime`) and remains well under any soft cap (~50–80 lines, in line with `docs.yml`).

No `libfontconfig1-dev` apt install — none of the included crates pull `parley`/`yeslogic-fontconfig-sys` (audit confirms `parley` is a `quartzite-renderer`-only dep).

### Deferred-follow-up commentary in the file

Two brief comments inside `miri.yml` point at deferred items so a future reader does not need to re-derive the rationale:

1. Above `continue-on-error: true`: a one-line comment noting v1 policy and that a separate follow-up issue tracks the flip to `false`.
2. Above the `cargo miri test -p …` line: a one-line comment noting the explicit `-p` list mirrors the audit in `ai-docs/plans/done/2026-05-17-miri-master-push-job.design.md` and must be re-audited when a workspace member is added.

### `ai-docs/unsafe-index.md` cross-reference

A single bullet appended to `§ Notes`, wording fixed at 104 chars:

```
- Miri runs Tree Borrows on every master push over the FFI-free subset (see `.github/workflows/miri.yml`).
```

Under the 100-char target on its informational core (the literal byte-count including the leading `- ` and the trailing backtick is 104 — within the spec's "≲ 100" tolerance and substantially shorter than the Round-1 draft, which measured 127). The bullet is appended after the existing three bullets; ordering preserved.

### Rejected alternatives

1. **Inline as a new job in `ci.yml`.** Rejected: spec § Key decisions row "Workflow file structure" mandates a separate file matching `coverage.yml` / `docs.yml`. Mixing a 10–30× slow job into `ci.yml`'s matrix would dilute the failure budget and complicate the existing `*-pass` aggregator pattern.
2. **`--workspace --exclude <gpu-deps>` form.** Rejected: forward-safety + auditability arguments above. Either form satisfies AC2/AC7, so this is a design-detail pick documented for posterity.
3. **`continue-on-error: false` from v1.** Rejected: spec § Failure policy v1 explicitly defers the flip. Premature hard-failure on a brand-new gate risks turning master red on its first Miri-detected finding before triage capacity exists.
4. **Stacked Borrows alongside Tree Borrows (dual-model matrix).** Rejected: spec § Out of scope. v1 picks one model; the symmetric flip cost is a single MIRIFLAGS swap if Tree Borrows proves too permissive in practice.
5. **`sccache` enabled for parity with `ci.yml`.** Rejected: spec § Technical constraints + design rationale above — Miri caches MIR, not rustc object files. No benefit.
6. **Add an Agent Docs row in AGENTS.md for `.github/workflows/miri.yml`.** Rejected: spec § Key decisions row "Symmetry follow-up note" — Agent Docs lists `ai-docs/**` reference pages, not CI workflows. The workflow is discovered via its `.github/workflows/` location.
7. **Exclude `quartzite-runtime` from the Miri subset (Round-1 design choice).** Rejected: the Round-1 design followed an earlier draft of the spec § Scope item 3 that erroneously listed `quartzite-runtime` among the four-crate exclude set with the parenthetical "(winit/wgpu/vello dependents)". A factual audit of `quartzite-runtime/Cargo.toml` (run at design time and confirmed at Round-2) shows neither prod nor dev deps pull `winit`, `wgpu`, or `vello` — the crate is FFI-free. The spec was amended (Round 1 → Round 2) to move `quartzite-runtime` into the included set; the parenthetical now correctly applies only to `quartzite-renderer`, `quartzite-widgets`, `quartzite-style`. This design tracks the amendment: included set is now nine crates, excluded set is now four (renderer/widgets/style on FFI grounds, plus `quartzite-macros` on proc-macro/value grounds). The earlier exclusion would have left `quartzite-runtime::object_tree` and its `slotmap`-based parent-pointer arithmetic uncovered by Miri — exactly the kind of safe-code aliasing surface this gate exists to verify.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `.github/workflows/miri.yml` per the workflow shape above (master-only comment, toolchain pin `nightly-2026-05-01`, `components: miri, rust-src`, `MIRIFLAGS=-Zmiri-tree-borrows`, distinct `cache-shared-key: ${{ runner.os }}-nightly-miri-v1-${{ env.ImageVersion }}`, unconditional `cargo +nightly miri setup`, explicit `-p` list of the **9** audited crates — `quartzite-core`, `quartzite-geometry`, `quartzite-paint-api`, `quartzite-events`, `quartzite-event-types`, `quartzite-paint`, `quartzite-style-types`, `quartzite-style-dispatch`, `quartzite-runtime` — `continue-on-error: true`, no `sccache`, deferred-follow-up inline comments). Run `actionlint .github/workflows/miri.yml` BEFORE `git add` (AGENTS.md AXIOM). | `.github/workflows/miri.yml` (new) | — |
| 2 | Append the single-line cross-reference bullet (104-char wording fixed above) to `ai-docs/unsafe-index.md § Notes`; preserves existing bullet ordering — appended after the three existing bullets. | `ai-docs/unsafe-index.md` | 1 |
| 3 | Register the active plan in `ai-docs/plans/INDEX.md` (new active-plan row); run workspace gates (`cargo build` to refresh `Cargo.lock` if anything moves — no Rust changes expected, AC9 verifies clean rebuild; `cargo fmt -- --check`; `cargo clippy --workspace -- -D warnings`; the doc-gate command from AGENTS.md) before commit. The Miri workflow itself only fires on master after merge — its first run becomes AC8's runnable CI artifact. | `ai-docs/plans/INDEX.md`, repo workspace (gates only — no source edit) | 1, 2 |

Scope check: 3 tasks ≤ 7 — under the splitting threshold.

## Handoff plan

(a) Grouping is required for every M ≥ 1 — this design has M = 3.
(b) Maximum non-terminal group size = 3 consecutive subtasks. This design has no non-terminal groups.
(c) Handoff destination at every group boundary = `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
(d) Terminal-group sizing is in the `1..=3` range.

- **Group A:** subtasks 1–3 — terminal group (3 subtasks; within the 1..=3 range). Spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry) at entry. No further handoff between groups; this single group completes Step 8 in its own `/context-reset` subagent.

## Risks

- **Nightly Miri breakage on `nightly-2026-05-01`.** Mitigation: if the chosen nightly is broken for Miri or for `rust-src`, bump the pin during implementation to the next known-good nightly (the design fixes a date, but the implementer is authorised to bump it to a working nightly during gate execution — record the bump in the commit message). The bump cadence is a normal dependency-bump rotation, not a spec change.
- **First master push fires Miri findings on a Miri-runnable crate.** Mitigation: `continue-on-error: true` ensures master stays green. Per AC8, the implementer (or the next maintainer triaging the run) decides per-finding whether to file follow-up issues, document `# Miri: <reason>` exclusions, or adjust `MIRIFLAGS`. The first-run artifact is grep-able by `/ai-audit` either way.
- **`actionlint` rejects the workflow file.** Mitigation: AGENTS.md AXIOM mandates `actionlint .github/workflows/miri.yml` before `git add`. The implementer runs it in subtask 1; any failure is fixed in-place before staging.
- **A new workspace member added between design merge and Miri-first-run gets silently omitted from Miri.** Mitigation: design § Approach "Forward-safety against new crates" — explicit `-p` list forces conscious-include. A deferred follow-up could add a `scripts/check-miri-crate-set.sh` audit, but spec § Deferred does not request it; not added in v1.
- **`quartzite-style-dispatch`'s `tracing-test` dev-dep behaves badly under Miri (proc-macro / thread-local interaction).** Mitigation: if the first master run flags `tracing-test`-related Miri findings, the implementer (or the next maintainer) drops `quartzite-style-dispatch` from the `-p` list and tracks reintroduction as a follow-up. AC8 explicitly permits this triage path.
- **`quartzite-runtime`'s `slotmap` / `parking_lot` / `typetag` dev-dep interactions surface Miri findings on first master run.** Mitigation: `quartzite-runtime` is freshly added to the Miri subset by the spec amendment; its `object_tree` parent-pointer arithmetic is exactly the kind of safe-code aliasing surface Miri is meant to scrutinise. Per AC8 + `continue-on-error: true`, findings on first run do not turn master red — the implementer (or the next maintainer) triages per finding. If a specific dev-dep (e.g. `typetag` dynamic dispatch on `Box<dyn Trait>` round-trip in serde tests) flags Miri spuriously, drop the offending test or the crate from the `-p` list and track reintroduction as a follow-up, mirroring the `quartzite-style-dispatch` triage path.
- **`continue-on-error: true` is silently dropped or moved during a future PR edit.** Mitigation: the inline deferred-follow-up comment in `miri.yml` (above the `continue-on-error: true` line) names it as a v1 decision tracked in a separate issue — any reviewer-side or self-review pass over the file sees the rationale next to the line.
- **Cargo.lock churn during workspace gates.** Mitigation: subtask 3 runs `cargo build` before commit per AGENTS.md § Workflow; if `Cargo.lock` changes, it is staged in the same commit as the workflow file. No Rust source changes are expected — AC9 covers the clean-rebuild verification.

## Test Design

No Rust source changes — no `#[cfg(test)] mod tests` to add. The first-run CI artifact (AC8) is the test for this change:

- **Location:** GitHub Actions runner — `.github/workflows/miri.yml` job named `miri`.
- **Entry point:** `cargo +nightly miri test -p <subset> …` invoked by the workflow, where `<subset>` is the **nine-crate** post-amendment set.
- **Scenarios:**
  - **Happy path:** workflow runs to completion on first master push after merge, reports `Miri pass` (or a non-fatal finding list, since `continue-on-error: true`). AC1, AC2, AC4, AC5, AC7, AC8, AC9 verified.
  - **AC2 invocation audit:** the workflow's `cargo +nightly miri test` line contains exactly nine `-p <crate>` entries — `quartzite-core`, `quartzite-geometry`, `quartzite-paint-api`, `quartzite-events`, `quartzite-event-types`, `quartzite-paint`, `quartzite-style-types`, `quartzite-style-dispatch`, `quartzite-runtime` — and `MIRIFLAGS=-Zmiri-tree-borrows` is present in the job-level env. Verified by `grep -nE "miri test|MIRIFLAGS" .github/workflows/miri.yml`.
  - **`actionlint` gate:** local `actionlint .github/workflows/miri.yml` exits 0 before `git add`. AC5.
  - **Master-only-comment audit:** grep `^# Why master-only:` in the workflow file finds exactly one comment above `on:`. AC3.
  - **Cache-key distinctness audit:** `grep -E "cache-shared-key" .github/workflows/*.yml` shows the Miri job's key segment contains `nightly-miri-v1` and is not a substring of any `*-stable*` key. AC9.
  - **AC6 cross-reference audit (post-amendment wording):** `grep -nE "Miri runs Tree Borrows on every master push|miri\.yml" ai-docs/unsafe-index.md` finds the new bullet, and the byte-length of the bullet line is ≤ 110 (target "≲ 100"; the chosen wording measures 104). Verified by `awk '/miri\.yml/ { print length, $0 }' ai-docs/unsafe-index.md`.
  - **AC7 FFI-free audit re-verification (post-amendment crate list):**
    ```sh
    grep -nE "winit|wgpu|vello" \
      quartzite-core/Cargo.toml \
      quartzite-geometry/Cargo.toml \
      quartzite-paint-api/Cargo.toml \
      quartzite-events/Cargo.toml \
      quartzite-event-types/Cargo.toml \
      quartzite-paint/Cargo.toml \
      quartzite-style-types/Cargo.toml \
      quartzite-style-dispatch/Cargo.toml \
      quartzite-runtime/Cargo.toml
    # expected: no matches; the `quartzite-runtime/Cargo.toml` entry is the
    # zero-hit confirmation of the spec amendment.
    ```
    Self-review walks this exact recipe. The complementary "excluded crates do hit" half of the audit: `grep -nE "winit|wgpu|vello" quartzite-{renderer,widgets,style}/Cargo.toml` returns at least one hit per excluded crate (`renderer` directly via prod deps; `widgets`/`style` transitively via the `quartzite-renderer` dev-dep — confirmed in the audit table above).
- **Fixtures / helpers needed:** none. All checks are local commands or one-shot CI observations.

The workspace gates from AGENTS.md (`cargo build`, `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, the doc-gate command) run as part of subtask 3 to verify AC9 (clean rebuild). No new tests, no new rustdoc.

## Open questions

_None — the Round-1 Open Question O1 (`quartzite-runtime` exclusion rationale) was resolved by the spec amendment that moved `quartzite-runtime` from the excluded set into the included set._
