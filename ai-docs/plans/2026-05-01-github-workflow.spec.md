# GitHub Workflow & Project Setup

**Source:** user description
**Date:** 2026-05-01

## Scope

1. Branch protection on `master`: no direct pushes; all changes via PR
2. GitHub Actions CI workflow: `cargo build`, `cargo test`, `cargo clippy -- -D warnings` on Linux
3. PR merge requires: CI passing + at least 1 approving review from the repository owner
4. `README.md`: project name, brief description, build instructions
5. `LICENSE`: LGPL-3.0

## Out of scope

- Windows / macOS CI runners
- Auto-merge
- Code coverage, benchmarks, release workflow

## Deferred

- Badge links in README | will be added later by user
- Contributing guide, roadmap | user will add later

## Key decisions

| Question | Decision |
|---|---|
| Branch protection method | `gh` CLI / GitHub API (not manual UI) |
| CI checks | `cargo build` + `cargo test` + `cargo clippy -- -D warnings` |
| CI platform | Ubuntu Linux only |
| README style | Minimal: name + description + build instructions |
| License | LGPL-3.0 (full `LICENSE` file) |
| Required reviews | 1 approving review (repo owner) |

## Technical constraints

- Branch protection must be applied to `master` via `gh api` calls
- CI workflow file: `.github/workflows/ci.yml`
- `LICENSE` must be the exact LGPL-3.0 text
- README must be valid GitHub-rendered Markdown

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | A direct push to `master` is rejected by GitHub |
| AC2 | Opening a PR triggers the CI workflow automatically |
| AC3 | CI workflow runs `cargo build`, `cargo test`, and `cargo clippy -- -D warnings` on ubuntu-latest |
| AC4 | A PR cannot be merged unless all CI checks pass |
| AC5 | A PR cannot be merged without at least 1 approving review |
| AC6 | `README.md` exists at the repo root with project name, description, and build instructions |
| AC7 | `LICENSE` file exists at the repo root containing the LGPL-3.0 license text |

## Open questions

- None
