# Design: Miri subset widening v2 (`--lib` + `--test` filtering)

**Issue:** #431
**Date:** 2026-05-17
**Tracked in:** #431

## Approach

Extend `.github/workflows/miri.yml` with the v2 widening as **additive `steps:` under the existing `miri` job**, preserving the v1 9-crate invocation verbatim. The job-level `MIRIFLAGS`, `continue-on-error: true`, nightly pin, and `cache-shared-key` are all unchanged — v2 is purely a `steps:` extension.

**Final v2 invocation shape — six separate `- name:` workflow steps** (one cargo invocation per step). Rationale:

1. **Separate exit codes per invocation in the CI summary.** A `--lib` failure under `quartzite-widgets` must not mask a `--test no_style_dep` failure on the same crate; without separate steps the first failing cargo invocation aborts the rest of the bash session. This is the spec's stated default (Open question #1).
2. **Matches the existing convention in `ci.yml`'s `gpu-tests` job**, which uses separate `- name:` steps for logically distinct cargo invocations (`Snapshot tests` vs `Windowed smoke test`). The miri job's existing v1 step is a single `cargo miri test` over many `-p` flags because they share a *target-selector strategy*; v2's six invocations differ in target selector (`--lib` vs `--test <allow-list>` vs `--test <single>`) and so warrant per-step granularity.
3. **`actionlint` impact: neutral.** Each new step is a plain `run:` block; no new actions, no expression syntax, no shell quoting subtleties beyond what v1 already does.

Step placement: the six new steps are appended immediately after the existing `cargo miri test (FFI-free crate subset)` step. Sequential dependency on toolchain setup is satisfied transitively (steps share the same job). No parallel jobs needed — the job already runs serially on a single `ubuntu-latest` runner, and per-step caching is irrelevant under `cargo miri test` (Miri rebuilds with its own sysroot regardless).

### Design-time `--test`-FFI-free re-audit (binding gate, AC2)

Per Scope §3 / AC2 of the spec, before binding invocations §2.4 / §2.5 / §2.6 into the workflow, the design phase MUST verify each named test file is `--test`-FFI-free. Re-audit executed at design time:

```bash
$ for f in quartzite-widgets/tests/no_style_dep.rs \
           quartzite-widgets/tests/re_exports.rs \
           quartzite-style/tests/third_party_paint.rs; do
    echo "=== $f ==="
    grep -nE 'use quartzite_renderer|extern crate quartzite_renderer|mod support' "$f"
  done
=== quartzite-widgets/tests/no_style_dep.rs ===
(no matches)
=== quartzite-widgets/tests/re_exports.rs ===
(no matches)
=== quartzite-style/tests/third_party_paint.rs ===
(no matches)
```

**All three files PASS the re-audit** (zero hits across the three patterns). Specifically:

- Neither `use quartzite_renderer` nor `extern crate quartzite_renderer` appears in any of the three files.
- No `mod support;` declaration in any of the three files — they are stand-alone integration test binaries with no shared support module.
- Top-level `use` statements verified by `grep -nE '^use ' <file>`:
  - `quartzite-widgets/tests/no_style_dep.rs`: `use std::process::Command;` only — the test shells out to `cargo metadata`/`cargo tree` to verify a *negative* dep-graph property; no renderer code is exercised at runtime.
  - `quartzite-widgets/tests/re_exports.rs`: `use core::any::TypeId;` only — `TypeId`-based re-export identity check; renderer-free.
  - `quartzite-style/tests/third_party_paint.rs`: `use quartzite_macros::Extend;`, `use quartzite_paint_api::Painter;`, `use quartzite_style::{...};`, `use quartzite_widgets::{...};` — all four crates are renderer-free at the `--lib` / `--test`-target level per the spec's audit (`quartzite-paint-api` and `quartzite-macros` carry no renderer prod-dep at all; `quartzite-widgets` and `quartzite-style` carry the renderer only as a dev-dep, which the chosen `--test` selector does not pull in for *this* test binary because this file does not import the renderer).

**Result:** all three widening invocations (`§2.4`, `§2.5`, `§2.6`) are bound into the v2 set. **Final v2 invocation count: 6** (the maximum permitted by AC1).

### Six v2 invocations (final list)

| # | Invocation | Target selector | Why FFI-free |
|---|---|---|---|
| (a) | `cargo miri test --lib -p quartzite-widgets` | `--lib` (lib-only tests) | `src/` is renderer-free; `--lib` sidesteps `tests/snapshots.rs` and `tests/support/mod.rs` which import the renderer |
| (b) | `cargo miri test --lib -p quartzite-style` | `--lib` (lib-only tests) | Same shape as (a); `--lib` sidesteps `tests/snapshots.rs` + `tests/support/mod.rs` |
| (c) | `cargo miri test -p quartzite-macros --test extend --test meta_enum --test object --test object_impl` | `--test` allow-list (4 tests) | The 4 named tests use only the proc-macro's own re-exports; `via_facade.rs` excluded because it pulls the facade crate which depends on the renderer |
| (d) | `cargo miri test -p quartzite-widgets --test no_style_dep` | `--test` single | Pure `Command`-based dep-graph negative test; no renderer imports anywhere in the file or transitively |
| (e) | `cargo miri test -p quartzite-widgets --test re_exports` | `--test` single | Pure `core::any::TypeId`-based re-export check; no renderer imports |
| (f) | `cargo miri test -p quartzite-style --test third_party_paint` | `--test` single | Uses `quartzite-{macros, paint-api, style, widgets}` only; none of these pull the renderer for the *test-binary* target |

### `ai-docs/unsafe-index.md § Notes` cross-reference (AC6)

**unsafe-index.md § Notes — no edit needed.** The current 106-char bullet ("Miri runs Tree Borrows on every master push over the FFI-free subset (see `.github/workflows/miri.yml`).") is already accurate after v2: the widening adds further FFI-free targets within the same "FFI-free subset" framing. AC6's edit-or-add-bullet flexibility is exercised as "no edit" with the audit recipe preserved verbatim by the unchanged bullet.

### Rejected alternatives

1. **Single-step variant** (extend the existing v1 step's `run:` block with the six new invocations chained by `&&` or newlines). **Rejected** because:
   - One failing invocation aborts the rest of the bash session (`set -e` is bash's default for `&&` chains, and a non-zero exit of one cargo invocation prevents the rest from running). Diagnosing which invocation failed requires scrolling the bash log instead of glancing at the per-step CI summary panel.
   - Mixes two distinct target-selector strategies (`--lib` and `--test <allow-list>`) into one opaque step; the existing v1 step at least has a uniform `-p <crate>` shape that maps to "the 9-crate FFI-free subset" mental model.
   - The spec's Open question #1 default explicitly favours separate steps "so failures don't mask each other".

2. **Per-crate combined `--lib` + `--test` invocation** (e.g., `cargo miri test --lib --test no_style_dep --test re_exports -p quartzite-widgets`). **Rejected** because: same exit-code masking concern; Open question #2's default is **NO** for the same reason.

3. **Demote any of the three widened `--test` candidates ((d), (e), (f))**. **Rejected**: re-audit returned zero hits for all three (recorded verbatim above); none qualify for demotion. If a future workspace edit adds `use quartzite_renderer` or `mod support;` to any of these files, the re-audit recipe (Spec § Technical constraints) catches the regression at audit time.

4. **Add `-Zmiri-permissive-provenance` or any other new flag speculatively.** **Rejected**: Spec § Out of scope and AC3 forbid new `MIRIFLAGS` in v2; the AC8 envelope handles any new finding via the same disposition path as v1 (`cfg_attr(miri, ignore)` / file-level `#![cfg(not(miri))]` / follow-up issue / `MIRIFLAGS` adjustment in a follow-up PR).

5. **Flip `continue-on-error` to `false`.** **Rejected**: explicitly out of scope per Spec § Out of scope; tracked as the separately-deferred follow-up to #422.

6. **Edit the `ai-docs/unsafe-index.md § Notes` Miri bullet in place to enumerate widgets / style / macros (or `--lib` / `--test` targets) explicitly.** **Rejected** because: (i) all reasonable edit-in-place candidates explored in Round 1 overflowed AC6's 110-char budget (138, 128, and 126 chars vs the 110 cap; sibling-bullet fallback at 104 chars would have fit but is also unnecessary); (ii) the **current 106-char bullet is already accurate after v2** — the widening adds further FFI-free targets within the same "FFI-free subset" framing the bullet already names. AC6's edit-or-add-bullet flexibility is exercised as **no edit**, and subtask 2 verifies line 42 hasn't drifted at implementation time.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Extend `.github/workflows/miri.yml` with the six v2 `cargo miri test` invocation steps. Append immediately after the existing `cargo miri test (FFI-free crate subset)` step. Each step gets a descriptive `- name:` matching the table in § Approach (e.g., `cargo miri test (widgets --lib)`, `cargo miri test (style --lib)`, `cargo miri test (macros --test allow-list)`, `cargo miri test (widgets --test no_style_dep)`, `cargo miri test (widgets --test re_exports)`, `cargo miri test (style --test third_party_paint)`). Run `actionlint .github/workflows/miri.yml` and confirm exit 0 BEFORE `git add` (AGENTS.md AXIOM). | `.github/workflows/miri.yml` | — |
| 2 | **Verify only — no edit needed.** Verify the existing 106-char bullet at line 42 of `ai-docs/unsafe-index.md` still accurately describes the post-v2 surface; no edit needed unless the implementer's `wc -c` shows the bullet has drifted from 106 chars or wording. Effort drops from "edit + verify" to "verify only" — no file mutation expected. | _(read-only verification of `ai-docs/unsafe-index.md`)_ | 1 |
| 3 | Workspace gates: `cargo build` (refreshes `Cargo.lock` — no change expected since no Rust source touched, but the AGENTS.md "before-commit `cargo build`" rule still applies) and `cargo fmt -- --check`. No `cargo test` / `cargo clippy` run is needed because v2 touches no `.rs` file — but record this skip in the commit message rationale so reviewers see it was intentional. | `Cargo.lock` (may or may not refresh) | 2 |

(M = 3 — well within the every-group handoff contract and far below the 7-subtask split-this-issue threshold.)

## Handoff plan

- **Group A:** subtasks 1–3 — terminal group (3 subtasks; within the 1..=3 range). No handoff between groups; the single group completes Step 8 in its own `/context-reset` subagent per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).

## Risks

- **`actionlint` regression on the modified workflow file** — mitigation: run `actionlint .github/workflows/miri.yml` locally before `git add` per AGENTS.md AXIOM. The six new steps reuse the existing v1 step's `run: |` shape verbatim (newline-folded multi-line cargo command), so the AXIOM gate should pass unchanged.
- **One of the six v2 invocations turns red on the first post-merge master run** — mitigation: `continue-on-error: true` (preserved) keeps master green regardless; the AC8 envelope (Spec § Acceptance Criteria AC8) governs per-finding disposition (`cfg_attr(miri, ignore)` / file-level `#![cfg(not(miri))]` / follow-up issue / `MIRIFLAGS` adjustment in a follow-up PR). No in-task fix is required for AC8 to be satisfied — the first-master-push log captures whichever outcome.
- **`ai-docs/unsafe-index.md` bullet drift between Round 1 (design) and Round 2 (implementation)** — mitigation: subtask 2 is a verify-only step that re-checks line 42's length and wording at implementation time; if `wc -c`/`awk 'NR==42 { print length }'` shows the bullet has drifted (e.g., a parallel PR edited the same file), the implementer reverts to AC6's edit-or-add-bullet flexibility under a follow-up. The current 106-char "FFI-free subset" framing is accurate after v2 because the widening adds further FFI-free targets within the same envelope.
- **Cumulative Miri job runtime drift** — informational only per Spec § Out of scope ("Cumulative-runtime ceiling"). Estimated 5 min → ~15 min per the issue body; no binding ceiling is added.
- **Future regression in audited test files** (someone adds `use quartzite_renderer` to one of the three widened test files without updating the workflow) — mitigation: the per-file grep recipe is recorded verbatim in Spec § Technical constraints and AC7 so future maintainers know to re-audit at workspace-member edit time. Out of scope for this task to add a CI gate enforcing this; existing convention (audit on workspace member add/remove) carries over.
- **Workspace gates AC** (clean rebuild verification, per the task prompt) — `cargo build` from a clean working tree must succeed; mitigated by subtask 3 running `cargo build` before commit. Since v2 touches no `.rs` file, the build is effectively a no-op refresh of `Cargo.lock`, but the AGENTS.md "refresh `Cargo.lock` before commit" rule still binds.

## Test Design

**No new Rust tests.** This task touches `.github/workflows/miri.yml` (CI workflow) only; no `.rs` file is modified, and `ai-docs/unsafe-index.md` is verified-only (no mutation expected per the AC6 "no edit" decision in § Approach). The Rust-side test conventions in AGENTS.md (`#[cfg(test)]` block, behavior tests, etc.) do not apply.

**Verification gates** (replace the per-task test design for this CI-only task):

- **AC1 invocation count:** the v2 widening binds six cargo invocations into the workflow; AC1 is satisfied at PR-open time by listing the six step names verbatim in the PR body. The first-master-push CI log is expected to report **~160 new Miri-executed tests** (Tests-count projection below) — a count materially below ~150 indicates an unexpected dispatch silently dropped a test binary.
- **AC2 audit log:** the re-audit recipe output in § Approach is the binding audit artefact; self-review walks it before APPROVE.
- **AC5 `actionlint`:** subtask 1 records `actionlint .github/workflows/miri.yml` exit 0 in the commit message body.
- **AC6 verification (no edit):** `awk 'NR==42 { print length }' ai-docs/unsafe-index.md` returns ≤ 110 AND `grep -nE 'FFI-free' ai-docs/unsafe-index.md` matches line 42 (wording-invariant check — guards against a parallel PR drifting the bullet to a different ≤ 110-char wording that would still satisfy the length check). Subtask 2 records the observed line-42 length AND the grep hit in the commit message body (or omits the AC6 line if no commit is produced for `ai-docs/unsafe-index.md`; the verification log lives in the PR body instead).
- **AC8 first-master-push:** verified post-merge by the first scheduled-master run of `miri.yml`. The PR body's "Verification" section lists the six new step names verbatim so `/ai-audit` / the user can grep the CI log for the invocation set.
- **Workspace gates AC (clean rebuild):** subtask 3 runs `cargo build` and confirms `Cargo.lock` refreshes (or no-ops) cleanly.

**Tests-count projection.** Approximately 160 new tests will run under Miri post-v2: widgets lib (83 `#[test]` items across 11 src/ files), style lib (41 across 4 src/ files), macros allow-list (25 across 4 integration tests: extend 3, meta_enum 5, object 10, object_impl 7), widened integration (11 across 3 files: no_style_dep 1, re_exports 5, third_party_paint 5). Spec/issue said 80–150 — actual ~160 is slightly above the upper estimate, accepted. The first-master-push log should report this exact count (modulo any Miri-specific test ordering / filtering); a count materially below ~150 indicates an unexpected dispatch silently dropped a test binary.

## Open questions

None. Spec § Open questions are addressed by the design as follows:

- **Combined-step vs separate-steps shape** — resolved to **separate steps** (six `- name:` blocks) per § Approach rationale.
- **Combine widget `--lib` + `--test`** — resolved to **NO** per the same rationale (Open question #2 default).
- **Workflow-step placement** — resolved to **immediately after the existing v1 step**, sequentially under the same job (no new job, no parallelism).
- **`quartzite-macros --test via_facade` under `--no-default-features`** — explicitly out of scope per Spec § Out of scope; recorded in the v2 deferred row to v3 if `/next` ever prioritises it.
- **Stabilisation-window threshold for `continue-on-error: false` flip** — separately tracked deferred follow-up to #422; v2 does not advance.
