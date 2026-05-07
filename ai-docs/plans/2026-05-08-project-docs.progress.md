# Progress: Project docs (#60) — ACTIVE
_Updated: 2026-05-08 02:15_

> Read THIS FIRST → ready to continue. No need to re-read the codebase.

**Branch:** `feat/2026-05-08-project-docs`
**base_commit:** `48324555a9aaa083ca8ee2b705c31ef60a861977`
**Last build:** PASS (subtask 6; `cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt -- --check` / `cargo build -p quartzite --no-default-features` all clean; `Cargo.lock` unchanged — no Rust source touched)
**Issue:** #60
**Spec:** [`ai-docs/plans/2026-05-08-project-docs.spec.md`](2026-05-08-project-docs.spec.md)
**Design:** [`ai-docs/plans/2026-05-08-project-docs.design.md`](2026-05-08-project-docs.design.md)
**Design-review:** GO (Round 2 of max 3) — one informational Note: quickstart auto-emit comment isn't runtime-verified under `no_run`; implementer to spot-check.

## Next action

**All subtasks complete — handing back to /task Step 9 (Verify).** The orchestrator should run the full standard-gates loop one more time, then proceed to Step 10 (Push + open PR). Pre-merge user follow-up still required for AC6 closure: run the BSD-awk container cross-check (see *Subtask 5 cross-check log* for the exact command) — this is implementer-discipline per design *Risks*; the CI sync-gate added in subtask 6 is `ubuntu-latest` only, so it does not catch GNU-awk-only constructs slipping into the generator. Post-merge user follow-up: add `roadmap-sync-pass` to the `master` branch-protection required-checks list per design *Post-merge owner actions* (the `*-pass` aggregator is the stable required-check name; `roadmap-sync` is the worker job).

## Subtasks

- [x] 1. README description block + CI/license badges (commit `136e29b`)
- [x] 2. `src/lib.rs` rewrite — overview + `no_run` quickstart + 5 per-concept sections + ecosystem map + design notes (commit `4afa02f`)
- [x] 3. `CONTRIBUTING.md` at standard depth — 10-section AGENTS.md excerpt (commit `7465fa9`)
- [x] 4. `scripts/gen-roadmap.sh` — POSIX bash + awk/sed generator with comment-banner of banned constructs (commit pending below)
- [x] 5. Run the generator and commit `ROADMAP.md` at repo root; macOS / BSD-awk-container portability cross-check (cross-check **skipped** — no docker/podman on host; see Subtask 5 cross-check log below)
- [x] 6. CI sync-gate: `roadmap-sync` job + `roadmap-sync-pass` aggregator in `.github/workflows/ci.yml` (ubuntu-latest only) (commit pending below)

## Key discoveries (don't re-investigate)

- **Intra-doc-link path qualifiers required from the facade scope.** The implicit `[`Foo`]` form does not resolve from `quartzite/src/lib.rs` through `pub use quartzite_core::*`. Use explicit `[`Foo`](core::Foo)` form. Caught by `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc`. The doctest in `src/lib.rs` already uses this pattern.
- **Doctest needs explicit `fn main()`.** The quickstart in `src/lib.rs` uses an explicit `fn main()` because rustdoc's auto-`fn main` wrapping puts the `Counter` struct inside `main()`, breaking the derive macro's generated `super::Counter` references. Same gotcha applies to any future doctest with `#[derive(Extend, DeriveObject)]`.
- **`Object::read_property` / `write_property`, NOT `ObjectExt`.** The reflection methods are on the `Object` trait (in `quartzite-core/src/traits.rs`); `ObjectExt` is for blanket helpers (`id`, `name`, `is_on_current_thread`, downcast_*, is). The CONTRIBUTING and lib.rs prose was corrected during subtask 2.
- **Bench-file `#[cfg(test)]` exemption escalated in PR #153.** Files under `benches/` declared with `[[bench]] harness = false` are exempt — already in AGENTS.md `## Rust Test Conventions` and reflected in CONTRIBUTING.md Tests section.
- **INDEX.md heading triggers verbatim.** `## Active plans`, `## Completed plans`, `## Deferred plans`, `## Dependency order` (input). Terminator: `## Suggested next steps`. ROADMAP.md output renames `## Dependency order` → `## Dependency tree`. Verified via `grep -n '^## ' ai-docs/plans/INDEX.md` at design time.
- **Sed link-rewrite scope.** Only `](done/` / `](deferred/` / `](2026-` prefixes appear in the three plan tables and the dependency-tree code-block. Parent-up `](../code-style.md…)` and `](../doc-convention.md)` references live ONLY in the `## Suggested next steps` section, which is dropped — so no parent-up rewrite class needed.
- **Action version pinning.** Per AGENTS.md `## Dependency Versions`, query `gh api /repos/actions/checkout/releases --jq '.[0].tag_name'` before pinning the new `roadmap-sync` job's `actions/checkout` major. Existing CI jobs use `@v6`; if unchanged, match.
- **CI sync-gate ubuntu-only rationale.** Content equality is platform-independent in spirit; Windows bash-via-Git-Bash awk/sed flavour drift produces false negatives. Linux-only avoids that, at the cost of not catching GNU-only constructs slipping into the generator. Compensate via macOS / BSD-awk-container cross-check at implementation time per design AC6.
- **Subtask 4 outputs (load-bearing for subtask 5).** `scripts/gen-roadmap.sh` is 249 lines (POSIX bash + awk/sed; banned-constructs comment banner present; `LC_ALL=C`; `set -eu`; mktemp tmpdir cleaned via trap). Generated `ROADMAP.md` is 87 lines. Linux-side determinism: `sha256sum ROADMAP.md` = **`53303112f1a7703fda5891247d9c989b34113e82e79bfee4eb5e280bb0d20d6b`** (verified across two consecutive runs on this Gentoo box; this is the value the BSD-awk container run in subtask 5 must match byte-for-byte). `shellcheck scripts/gen-roadmap.sh` is clean — three SC2016 false positives on the markdown-backtick header literals are silenced via a single `# shellcheck disable=SC2016` directive on the printf block, with a short comment explaining the silencing rationale.
- **Awk state-machine refinement found in subtask 4.** Naïve "extract everything between `## Deferred plans` and the next `## ` heading" captured the trailing `> Tracking issues for further deferred items not represented as plans here:` blockquote (INDEX.md lines 64–65) — which the design *What the generator drops* list (lines 242–243) explicitly drops. Fix: each plan-table state captures **only** lines beginning with `|`, treats blank lines as no-ops, and closes the state on the first non-blank, non-pipe line. This implicitly drops the tracking-issues blockquote, the maintenance-plans paragraph, and any future prose injected between the deferred table and the next H2 — robust against further INDEX.md prose drift in the same gap.
- **Cargo.lock not refreshed.** `cargo build` after subtask 4 was a no-op rebuild (no Rust source touched); `Cargo.lock` is unchanged, nothing to stage from `Cargo.lock`. Subtasks 5 and 6 (ROADMAP.md content + workflow YAML) likewise won't touch Rust sources, so the same applies — but still run `cargo build` per AGENTS.md `## Workflow`.
- **Subtask 6 took the design literally — duplicate display name `ROADMAP sync` retained.** Both `roadmap-sync` (worker) and `roadmap-sync-pass` (aggregator) emit GitHub Actions display name `ROADMAP sync`. The design called this an informational note (Actions allows the duplication; branch-protection UI is the only place it surfaces). Following the precedent set by the existing `build`/`build-pass`, `test`/`test-pass`, `clippy`/`clippy-pass`, `features`/`features-pass` pairs in `ci.yml` — all of which already share display names with their workers — preserves a consistent in-file pattern. No deviation from design.
- **`actions/checkout` registry-query result.** `gh api /repos/actions/checkout/releases --jq '.[0].tag_name'` returned `v6.0.2` at implementation time → major `v6` matches the existing `@v6` pin used by all other CI jobs in this file, so the new `roadmap-sync` job uses `@v6`. No bump warranted (per AGENTS.md `## Dependency Versions`, training-stale data is the failure mode the registry-query rule guards against — the result confirms `@v6` is current, not stale).
- **Drift-test PASS.** Hand-edited `ROADMAP.md` (appended a synthetic comment) → `git diff --exit-code ROADMAP.md` returned exit code 1 (= the CI gate would FAIL the PR). After `cp` revert → exit code 0. Note: the in-CI sequence (`./scripts/gen-roadmap.sh && git diff --exit-code ROADMAP.md`) will catch BOTH failure modes — (a) hand-edit + forgot to regen (covered by drift test above), and (b) edit `INDEX.md` + forgot to regen (regen step rewrites ROADMAP.md, then diff fails). Linux-side `sha256sum ROADMAP.md` after standard gates is unchanged: `53303112f1a7703fda5891247d9c989b34113e82e79bfee4eb5e280bb0d20d6b` (matches the recorded subtask-4/5 hash).

## AC Status

| AC | Status |
|----|--------|
| AC1 (README description block) | PASS — covered by subtask 1, commit `136e29b` |
| AC2 (README badges) | PASS — covered by subtask 1, commit `136e29b` |
| AC3 (lib.rs comprehensive doc) | PASS — covered by subtask 2, commit `4afa02f` |
| AC4 (CONTRIBUTING.md) | PASS — covered by subtask 3, commit `7465fa9` |
| AC5 (ROADMAP.md) | PASS — generated, visual-reviewed (all expected sections present; dropped sections absent), committed in subtask 5 |
| AC6 (gen-roadmap.sh portable) | PARTIAL — script written with comment-banner of banned constructs; Linux-side determinism verified (three runs, identical sha256); shellcheck clean; **BSD-awk container cross-check SKIPPED** — no docker/podman on the implementer host (Gentoo). Implementer-discipline check per design *Risks*; user must run the alpine-container check manually before merge to fully close AC6 |
| AC7 (CI sync-gate) | PASS — subtask 6: `roadmap-sync` worker + `roadmap-sync-pass` aggregator added to `.github/workflows/ci.yml` (ubuntu-latest only per design); local synthetic-drift test confirmed `git diff --exit-code ROADMAP.md` returns exit code 1 on hand-edited drift and 0 after revert; runtime CI verification will land once branch is pushed |
| AC8 (doc-gate) | PASS — verified after subtask 2 |
| AC9 (quickstart doctest) | PASS — `cargo test --doc -p quartzite` green after subtask 2 |
| AC10 (standard gates) | PASS — verified after subtask 2; re-run after each remaining subtask |
| AC11 (actionlint) | PASS — subtask 6: `actionlint .github/workflows/ci.yml` clean (no output) after adding the new `roadmap-sync` + `roadmap-sync-pass` jobs |
| AC12 (`document_features` placement) | PASS — verified after subtask 2; new content lands BEFORE the `# Feature flags` heading |

## Files touched

- `README.md` — subtask 1: project-description block + CI/license badges (commit `136e29b`)
- `src/lib.rs` — subtask 2: comprehensive crate-level rustdoc (commit `4afa02f`)
- `CONTRIBUTING.md` — subtask 3: NEW, 120 lines (commit `7465fa9`)
- `ai-docs/plans/2026-05-08-project-docs.spec.md` — added in subtask 1 commit
- `ai-docs/plans/2026-05-08-project-docs.design.md` — added in subtask 1 commit
- `ai-docs/plans/2026-05-08-project-docs.progress.md` — this file
- `scripts/gen-roadmap.sh` — subtask 4: NEW, 249 lines, +x bit set (commit pending below)
- `ROADMAP.md` — subtask 5: produced on disk by subtask 4's run-once; NOT staged yet (commit is subtask 5's deliverable)
- `.github/workflows/ci.yml` — subtask 6: added `roadmap-sync` worker + `roadmap-sync-pass` aggregator, slotted between `clippy-pass` and `docs` (commit pending below)

## Subtask 5 cross-check log

- **Linux (this Gentoo host, GNU awk/sed):** `./scripts/gen-roadmap.sh` → `sha256sum ROADMAP.md` = `53303112f1a7703fda5891247d9c989b34113e82e79bfee4eb5e280bb0d20d6b` (87 lines). Matches the subtask-4 reference byte-for-byte across two more consecutive runs (third total) — generator is deterministic on Linux/GNU awk.
- **BSD-awk container cross-check (alpine:latest, busybox awk):** **SKIPPED.** Reason: neither `docker` nor `podman` is available on this Gentoo implementer host (`command -v docker` → exit 1; `command -v podman` → exit 1). The check is implementer-discipline per design *Risks → CI gate is Linux-only* (the CI sync-gate added in subtask 6 is `ubuntu-latest` only, by deliberate design). User action recommended before merge: run `docker run --rm -v "$PWD":/w -w /w alpine:latest sh -c 'apk add --no-cache bash && ./scripts/gen-roadmap.sh && sha256sum ROADMAP.md'` (or equivalent on macOS) and confirm the hash equals `53303112f1a7703fda5891247d9c989b34113e82e79bfee4eb5e280bb0d20d6b`. If they differ, the generator has a GNU-awk-only construct that slipped past the banned-constructs comment banner in `scripts/gen-roadmap.sh` and must be fixed before merge.
- **Verdict:** Linux-side determinism PASS; BSD-portability verdict UNCONFIRMED on this host — explicit user follow-up flagged for AC6 closure.
- **Visual review of `ROADMAP.md` (87 lines):**
  - Header (lines 1–8): auto-generated note + legend match design lines 196–204 verbatim.
  - `## Dependency tree` heading present (line 10) — correctly renamed from INDEX.md's `## Dependency order`.
  - `## Active plans` (line 29), `## Completed plans` (line 49), `## Deferred plans` (line 82) tables present, verbatim from INDEX.md.
  - All plan-table links rewritten to `](ai-docs/plans/done/...)`, `](ai-docs/plans/deferred/...)`, `](ai-docs/plans/2026-...)` — no bare `](done/`, `](deferred/`, or `](2026-` prefixes remain (verified by full file read).
  - `## Suggested next steps` **absent** ✓.
  - Trailing maintenance-plans paragraph **absent** ✓.
  - Tracking-issues blockquote (INDEX.md lines 64–65) **absent** ✓ — confirms the awk state-machine refinement noted in *Key discoveries → Subtask 4* worked.

## Subtask 5 implementation brief

The agent picking up subtask 5 should:

1. Re-run `./scripts/gen-roadmap.sh` on Linux. Confirm `sha256sum ROADMAP.md` matches the Linux-side hash recorded in *Key discoveries → Subtask 4 outputs* (`53303112f1a7703fda5891247d9c989b34113e82e79bfee4eb5e280bb0d20d6b`). If different, the input INDEX.md changed — investigate before proceeding.
2. **Portability cross-check** (per design *Test Design → AC5/AC6 implementer responsibility*, line 293): run the generator inside a BSD-awk-flavoured container, e.g.:
   ```
   docker run --rm -v "$PWD":/w -w /w alpine:latest sh -c 'apk add --no-cache bash && ./scripts/gen-roadmap.sh'
   ```
   Then `sha256sum ROADMAP.md` from the host. Confirm the hash matches the Linux/GNU-awk hash byte-for-byte. Document the cross-check command, container image (with digest if practical), and the matching hashes in this progress file under a new "Subtask 5 cross-check log" section. If the hash differs, the script has a portability bug (likely a GNU-awk-only construct slipped past the comment banner) — fix the script, repeat, then commit `scripts/gen-roadmap.sh` again as a fix-up commit before proceeding.
3. Visual-review the generated `ROADMAP.md`:
   - `## Dependency tree` heading present (renamed from INDEX.md's `## Dependency order`).
   - Three plan tables present: `## Active plans`, `## Completed plans`, `## Deferred plans` (verbatim from INDEX.md).
   - Dropped sections **absent**: no `## Suggested next steps`, no maintenance-plans paragraph, no tracking-issues blockquote, no "Serialization-layer track" footnote.
   - Plan-table links rewritten: `](ai-docs/plans/done/...)`, `](ai-docs/plans/deferred/...)`, `](ai-docs/plans/2026-...)`. No bare `](done/` or `](deferred/` or `](2026-` prefixes remain.
4. Determinism re-check: run the generator again; `sha256sum ROADMAP.md` unchanged.
5. `cargo build` (no Rust changes expected; `Cargo.lock` should not change).
6. Stage `ROADMAP.md` and this progress file (NOT `.github/workflows/ci.yml` — that is subtask 6). Stage explicitly by name; never `git add -A`.
7. Update this progress file: mark subtask 5 ✅, set Next action to subtask 6, update `_Updated:` timestamp and Last build, AC5 and AC6 → PASS, append "Subtask 5 cross-check log" with hashes.
8. Commit (suggested message):
   ```
   docs(roadmap): add generated ROADMAP.md + macOS/BSD-awk portability cross-check

   Subtask 5 of #60. Re-runs scripts/gen-roadmap.sh and commits the byte-stable output.
   Cross-check: alpine:latest (busybox awk) produces byte-identical ROADMAP.md
   (sha256: <recorded-hash>) — confirms no GNU-awk-only constructs leaked past
   the banned-constructs banner in scripts/gen-roadmap.sh.
   ```
9. Stop. Do NOT proceed to subtask 6 — that is a separate handoff.
