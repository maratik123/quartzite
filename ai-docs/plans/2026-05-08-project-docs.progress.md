# Progress: Project docs (#60)

**Branch:** `feat/2026-05-08-project-docs`
**base_commit:** `48324555a9aaa083ca8ee2b705c31ef60a861977`
**Issue:** #60
**Spec:** [`2026-05-08-project-docs.spec.md`](2026-05-08-project-docs.spec.md)
**Design:** [`2026-05-08-project-docs.design.md`](2026-05-08-project-docs.design.md)
**Design-review:** GO (Round 2 of max 3) — one informational Note: quickstart auto-emit comment isn't runtime-verified under `no_run`; implementer to spot-check.
**Last build:** clean (subtask 3; CONTRIBUTING.md added, no Rust changes)

## Files touched

| File | Subtask | Status |
|---|---|---|
| `README.md` | 1 | ✅ done |
| `src/lib.rs` | 2 | ✅ done |
| `CONTRIBUTING.md` | 3 | ✅ done |
| `scripts/gen-roadmap.sh` | 4 | ⬜ pending |
| `ROADMAP.md` | 5 | ⬜ pending |
| `.github/workflows/ci.yml` | 6 | ⬜ pending |

## Subtasks

| # | Task | Status |
|---|---|---|
| 1 | README description block + CI/license badges (badge order: CI, docs, codecov, license) | ✅ done |
| 2 | `src/lib.rs` rewrite: overview + `no_run` quickstart + 5 per-concept sections + ecosystem map + design notes; preserves `# Feature flags` + `document_features!()` | ✅ done |
| 3 | `CONTRIBUTING.md` at repo root, standard depth (10-section AGENTS.md excerpt) | ✅ done |
| 4 | `scripts/gen-roadmap.sh` (POSIX bash + awk/sed; comment banner of banned constructs) | ⬜ pending |
| 5 | Generated `ROADMAP.md` at repo root | ⬜ pending |
| 6 | CI sync-gate: `roadmap-sync` job + `roadmap-sync-pass` aggregator (ubuntu-latest only) | ⬜ pending |

> M = 6 subtasks (≥ 5). Per `/task` Step 8 rule, after subtask 3 hand off via `/context-reset`.

## Next action

**Hand off via `/context-reset`** per `/task` Step 8 rule (N=3 of M=6 ≥ 5). The next session resumes from **Subtask 4**: create `scripts/gen-roadmap.sh` (POSIX bash + awk/sed) per the design's *Open questions resolved → 5*.

### Notes from subtask 2

- Three intra-doc links needed explicit path qualifiers (`[Foo](core::Foo)`) because the implicit `[`Foo`]` form doesn't resolve from the facade scope. Fixed: `Object::read_property`/`write_property` (initially mis-attributed to `ObjectExt`), `Value`, `Object`. Dead-link gate caught all of them.
- Quickstart doctest required an explicit `fn main()` because rustdoc's auto-`fn main` wrapping puts the `Counter` struct inside `main()`, breaking the derive macro's generated `super::Counter` references. Comment in the doctest documents why.
- Auto-emit comment ("Writing the property emits count_changed automatically") preserved per design Round-2 note. Runtime semantics not verified under `no_run`; left as informational claim per AC9 contract (compile-only).

### Notes from subtask 3

- 120 lines — at the upper bound of the design's 80–120 target.
- Excluded from the 10-section list: /task workflow steps, Propagation Rule, Corrections Log format, Permissions section, PR review-comment-resolution mechanics, Communication semantics — these are agent-internal.
- Bench-file exemption from `#[cfg(test)] mod tests` (escalated to AGENTS.md in PR #153) added to the Tests section as a small bonus clarification — external contributors will hit this gap if they add benches.

### Subtasks remaining (4–6) for the resume session

- **Subtask 4** — `scripts/gen-roadmap.sh`: POSIX bash + awk/sed parsing `ai-docs/plans/INDEX.md`. Comment-banner of banned constructs at top. Awk state machine matches `## Active plans` / `## Completed plans` / `## Deferred plans` / `## Dependency order` (input headings — verify by `grep -n '^## ' ai-docs/plans/INDEX.md`). Terminator: `## Suggested next steps`. Sed pass rewrites `](done/` / `](deferred/` / `](2026-` link prefixes only. Output skeleton at design lines 196–222. `LC_ALL=C` at script top.
- **Subtask 5** — Run `./scripts/gen-roadmap.sh > ROADMAP.md` (or whatever invocation the script uses) and commit the produced `ROADMAP.md` at repo root. Includes the macOS / BSD-awk-container portability cross-check (alpine container example at design line 293) — verify `sha256sum ROADMAP.md` matches between Linux and macOS runs. Document the verification in this progress file.
- **Subtask 6** — `.github/workflows/ci.yml`: add a `roadmap-sync` job (ubuntu-latest only) that runs `./scripts/gen-roadmap.sh` and `git diff --exit-code ROADMAP.md`; add a `roadmap-sync-pass` aggregator alongside the existing `*-pass` aggregators. Workflow snippet at design lines 142–161. Query `gh api /repos/actions/checkout/releases --jq '.[0].tag_name'` for the current major before pinning. Run `actionlint .github/workflows/ci.yml` before staging.

### Step 9 / 9.5 / 10 / 11 / 12 reminders

After subtask 6:
- **Step 9 verify**: build / test / clippy / fmt / no-default-features / doc-gate / actionlint — all clean. Per-AC summary table.
- **Step 9.5**: update `ai-docs/context.md` (docs-and-facade plan progress) and verify README is updated (already done in subtask 1).
- **Step 10**: spawn self-review agent. On APPROVE delete the progress file FIRST, then Step 12.
- **Step 12**: move spec/design to `ai-docs/plans/done/`, update `INDEX.md`, commit + push + open PR with `Closes #60`.
