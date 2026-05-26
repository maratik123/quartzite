# Design: Dependabot configuration

**Issue:** #570
**Date:** 2026-05-26

## Approach

Author a single new file `.github/dependabot.yml` containing a Dependabot config-schema-v2 document with exactly two `updates:` entries — one for `cargo` and one for `github-actions` — both targeting `master`, both weekly, both grouping minor + patch into a single combined PR, and both ignoring `version-update:semver-major` for every dependency. No source code, no workflow files, no `Cargo.toml`, no instruction-file edits — config-only.

The shape is fully constrained by the spec's *Key decisions* table + AC1–AC11. The design's job is to pin (a) the exact YAML structure, (b) the validation tooling (since `actionlint` is **not** applicable here — `.github/dependabot.yml` is not a workflow file), and (c) the decomposition + handoff plan.

### Exact YAML shape (target file content)

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    target-branch: "master"
    commit-message:
      prefix: "chore(deps)"
    groups:
      cargo-deps:
        patterns:
          - "*"
        update-types:
          - "minor"
          - "patch"
    ignore:
      - dependency-name: "*"
        update-types:
          - "version-update:semver-major"

  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
    target-branch: "master"
    commit-message:
      prefix: "chore(ci)"
    groups:
      github-actions:
        patterns:
          - "*"
        update-types:
          - "minor"
          - "patch"
    ignore:
      - dependency-name: "*"
        update-types:
          - "version-update:semver-major"
```

Notes on the field choices:

- `directory: "/"` for `cargo`: Cargo resolver v2 + workspace at repo root means Dependabot walks every workspace member from the single root `Cargo.toml`. Per-crate entries are explicitly out of scope (Spec § Out of scope item 1).
- `directory: "/"` for `github-actions`: Dependabot scans the entire `.github/workflows/` tree from a single `/` entry (verified by GitHub docs; current workflow inventory confirmed: `base_benchmarks.yml`, `ci.yml`, `coverage.yml`, `docs.yml`, `fork_pr_benchmarks_run.yml`, `fork_pr_benchmarks_track.yml`, `miri.yml`).
- `target-branch: "master"` is explicit, not implicit. The repo's default branch *is* `master`, but stating it makes intent unambiguous and is robust if the default branch ever changes.
- `commit-message.prefix`: `chore(deps)` for cargo, `chore(ci)` for github-actions. Matches the recent commit log convention (`chore: ...`, `chore(quartzite-core): ...`). Dependabot appends the dependency name(s) and version range, producing messages like `chore(deps): bump serde from 1.0.x to 1.0.y`.
- `groups:` block uses one group per ecosystem named `cargo-deps` / `github-actions`. The group name appears in the PR title and branch name (e.g. `dependabot/cargo/cargo-deps-...`). `patterns: ["*"]` matches every dependency; `update-types: ["minor", "patch"]` confines the group to non-major bumps.
- `ignore:` filters `version-update:semver-major` for `dependency-name: "*"`. This sits at the ecosystem-entry top level (a sibling of `groups:`, NOT nested inside it) — Dependabot's `ignore:` is a per-entry filter applied before grouping.
- No `open-pull-requests-limit` override — the default of 5 is sufficient (Spec § Key decisions).
- No `labels:` / `reviewers:` / `assignees:` — Spec § Key decisions explicitly opts out.
- No `versioning-strategy:` override — Cargo's default (`auto`, respecting `^` semantics) aligns with AGENTS.md § *Dependency Versions* (`0.x`, `x`). Setting it would risk drifting from the project pinning rule.

### Validation tooling decision

Three candidate validators were considered:

1. **`actionlint`** — REJECTED. `actionlint` validates `.github/workflows/*.yml` only. `.github/dependabot.yml` is not in its scope (verified by reading actionlint's documented file matchers). AGENTS.md § *Build & Test* axiom "*`actionlint` MUST pass before `git add` on any modified workflow file*" does NOT fire for this file.
2. **`python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml'))"`** — ACCEPTED as AC6 validator. Verifies well-formed YAML. Python 3.13 + PyYAML are available locally (`/usr/bin/python3` confirmed; `PyYAML` is a standard system package on the dev box). Catches the most common authoring mistakes (indentation, missing colons, unbalanced quotes).
3. **GitHub's Dependabot parser (post-merge UI check)** — REQUIRED for AC11 only, runs after merge. Cannot be invoked locally.

Validation contract:
- Pre-commit: AC6 via `python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml'))"` (zero output, exit 0).
- Post-merge: AC11 via visual inspection of GitHub's `https://github.com/maratik123/quartzite/network/updates` page — both ecosystems present, zero parser errors. *This AC fires asynchronously after the PR merges; the design treats it as a post-merge follow-up step, not a pre-merge gate.*

### Rejected alternatives

- **Per-crate `cargo` entries (one per workspace member).** REJECTED — explicitly out of scope (Spec § Out of scope item 1). Workspace-aware Dependabot from `/` is sufficient.
- **Daily schedule instead of weekly.** REJECTED — Spec § Key decisions pins `weekly` for both ecosystems. Daily would generate noise disproportionate to a single-maintainer project.
- **Auto-merge automation via a workflow.** REJECTED — Spec § Out of scope item 3. Adding it would conflict with the project's `gh pr merge --merge` AGENTS.md convention and require separate review.
- **`labels: ["dependencies"]`.** REJECTED — Spec § Key decisions: project does not currently use a `dependencies` label.
- **`versioning-strategy: "increase"` to bump `Cargo.toml` to the new lower-bound.** REJECTED — would conflict with AGENTS.md § *Dependency Versions* (`0.x` for `0.x.y`, `x` for `x.y.z`). Default `auto` is correct.
- **Splitting major-version bumps into a separate non-grouped PR stream (instead of fully ignoring).** REJECTED — Spec scope items 7 + 8 are unambiguous: ignore, do not surface.
- **Adding `dev-dependencies` to a separate group.** REJECTED — out of scope; not requested. The single group per ecosystem covers both `dependencies` and `dev-dependencies` (Dependabot defaults to bundling them when grouped by `patterns: ["*"]`).

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `.github/dependabot.yml` with the exact two-entry YAML shape specified in § Approach. Two `updates:` entries (`cargo` + `github-actions`), both targeting `master`, both weekly, both with single-group minor+patch bundling, both ignoring `semver-major` for `*`. Commit-message prefixes: `chore(deps)` for cargo, `chore(ci)` for github-actions. | `.github/dependabot.yml` | — |
| 2 | Run AC6 validation: `python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml'))"` (must exit 0, no output). Also re-read the file and walk AC1–AC10 line-by-line against the spec to confirm every key is present. AC11 is deferred to post-merge (UI check after the PR is merged). | (verification only) | 1 |

`M = 2`. Single-file change, no source-code touch, no test code to author. The 7-task ceiling is not approached.

## Handoff plan

`M = 2` → one group, terminal:

- **Handoff entry into Group A:** spawn `/context-reset` per `.claude/skills/context-reset/SKILL.md` § Compaction recovery (re-entry).
- **Group A:** subtasks 1–2 — terminal group (2 subtasks; within the 1..=3 range). Creates `.github/dependabot.yml`, then runs AC6 validation + AC1–AC10 line-walk. No further group; AC11 is a post-merge UI check tracked separately.

## Risks

- **YAML indentation typo silently parses but Dependabot rejects on upload.** `python3 -c "import yaml; ..."` will accept any well-formed YAML, including syntactically-valid YAML that violates Dependabot's schema (e.g. `update-types` nested under `patterns:` instead of as a sibling). *Mitigation:* The exact YAML in § Approach has been written to match GitHub's documented schema; subtask 1 copies it verbatim. AC11 (post-merge UI check) is the final authoritative gate.

- **`commit-message.prefix` interacts with `groups:` in surprising ways.** Dependabot grouped-PR commits use the prefix you set on the *entry*, then append the group name. The result for the cargo entry will look like `chore(deps): bump the cargo-deps group with N updates` — verify the prefix shape matches `chore(deps):` (note the colon is auto-added by Dependabot, so we set `prefix: "chore(deps)"` without a trailing colon). *Mitigation:* The YAML in § Approach uses the no-trailing-colon form, which is the documented norm.

- **Workflow-trigger axiom misfires.** AGENTS.md § *Build & Test* requires `actionlint` for any modified workflow file. `.github/dependabot.yml` is **not** a workflow file (it lives under `.github/` but NOT `.github/workflows/`), so the axiom does not fire. *Mitigation:* Subtask 2 still runs the YAML well-formedness check as a substitute gate; design § Validation tooling decision documents why `actionlint` is deliberately skipped.

- **Spec amendment from Dependabot UI rejection.** If AC11 fails after merge (Dependabot's GitHub-side parser surfaces an error not caught locally), this triggers AGENTS.md § *Spec-Amendment recipe* — `/pr-commented` or a fresh `/bugfix` round. *Mitigation:* low likelihood given the schema is copied from GitHub's documented example structure; tolerable risk for a config-only change.

- **`.gitignore` blocks the new file.** `.gitignore` does not currently match anything under `.github/`. *Mitigation:* confirmed via inspection of the existing `.github/workflows/*.yml` files (all tracked); no action needed.

- **Pre-existing `.github/dependabot.yml` collision.** `ls .github/` confirms only `workflows/` is present; no existing `dependabot.yml` to overwrite. *Mitigation:* none needed.

## Test Design

This task adds **no Rust source code** — no `#[cfg(test)]` module, no integration test, no doctest. The validation surface is config-file well-formedness + spec-AC enumeration. AGENTS.md § *Rust Test Conventions* and AGENTS.md § *Workflow* `#[cfg(test)]`-requirement do not fire (the threshold is "~50+ lines of substantial logic" and the artefact is YAML, not Rust).

**Subtask 2 — validation walkthrough:**

- *Tool:* `python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml'))"`. Must exit 0 with no output.
- *Manual AC walk:* re-read the written file and assert each of AC1 through AC10 against the spec:
  - AC1: file exists at `.github/dependabot.yml`.
  - AC2: `version: 2` present at the top.
  - AC3: cargo entry has `package-ecosystem`, `directory`, `schedule.interval`, `target-branch` with the four required values.
  - AC4: same shape for github-actions entry.
  - AC5: `commit-message.prefix` is `chore(deps)` for cargo, `chore(ci)` for github-actions.
  - AC6: YAML parser exit 0 (the python3 command above).
  - AC7: cargo entry has `groups:` with a single group, `patterns: ["*"]`, `update-types: ["minor", "patch"]`.
  - AC8: cargo entry has `ignore:` with `dependency-name: "*"` + `update-types: ["version-update:semver-major"]`.
  - AC9: github-actions entry has the same `groups:` shape as AC7.
  - AC10: github-actions entry has the same `ignore:` shape as AC8.
- *AC11:* deferred to post-merge — visit `https://github.com/maratik123/quartzite/network/updates` after PR merge, confirm both ecosystems are registered with no parser errors. If parsing fails, treat as Spec-Amendment trigger.

## Open questions

- _(none — spec resolved every key decision during interview Round 1 + Round 2)_
