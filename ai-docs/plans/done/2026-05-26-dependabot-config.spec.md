# Dependabot configuration

**Source:** user description (free-text)
**Date:** 2026-05-26
**Tracked in:** #570

## Scope

1. Add `.github/dependabot.yml` configuring automated dependency updates for the Quartzite workspace.
2. Enable the `cargo` ecosystem (single root manifest at `/` — Cargo resolver v2 walks the whole workspace from `Cargo.toml`; no need for a per-crate entry).
3. Enable the `github-actions` ecosystem at `/` (Dependabot scans all `.github/workflows/*.yml` from this single directory entry — covers `base_benchmarks.yml`, `ci.yml`, `coverage.yml`, `docs.yml`, `fork_pr_benchmarks_run.yml`, `fork_pr_benchmarks_track.yml`, `miri.yml`).
4. Target the `master` branch (project default; only branch with CI gates).
5. Weekly schedule for both ecosystems.
6. **`cargo` ecosystem** — group all minor + patch updates into a single PR per week (one combined PR per cycle rather than one PR per crate).
7. **`cargo` ecosystem** — filter out major-version bumps via `ignore: - dependency-name: "*" update-types: ["version-update:semver-major"]` (major bumps to be handled manually when intentional).
8. **`github-actions` ecosystem** — mirror the cargo policy: group minor + patch updates into a single combined PR per week, and filter out major-version bumps via the same `ignore:` rule (Round 2 answer: "Mirror cargo").

## Out of scope

- Per-crate `cargo` entries (workspace-aware Dependabot handles this from `/`).
- Custom registries or private feed configuration.
- Auto-merge automation (separate concern; would live in a workflow, not `dependabot.yml`).
- `docker`, `npm`, `pip`, or other ecosystems — none are present in the project.
- Security-update-only mode (security updates are always enabled by default once `dependabot.yml` exists; no extra config needed).
- Automatic major-version PRs for `cargo` (explicitly filtered — see Scope #7).
- Automatic major-version PRs for `github-actions` (explicitly filtered — see Scope #8).

## Deferred

- (none)

## Key decisions

| Question | Decision |
|---|---|
| Which ecosystems? | `cargo` + `github-actions` only (from task description) |
| Directory entries | `/` for both — workspace-root Cargo + single `.github/workflows/` tree |
| Target branch | `master` (default; only PR-gated branch) |
| Schedule | `weekly` for both ecosystems — standard Dependabot default; aligns with low-activity dependency churn typical of a single-maintainer Rust project |
| Cargo grouping | **Group minor + patch into one combined PR per week** (Round 1 answer). Implementation: `groups:` block with a single group (e.g. `cargo-deps`) using `update-types: ["minor", "patch"]` and `patterns: ["*"]`. |
| Cargo major-bumps | **Filtered out** (Round 1 answer). Implementation: top-level `ignore:` with `dependency-name: "*"` + `update-types: ["version-update:semver-major"]`. |
| GitHub Actions grouping | **Mirror cargo — group minor + patch into one combined PR per week** (Round 2 answer). Implementation: `groups:` block with a single group (e.g. `github-actions`) using `update-types: ["minor", "patch"]` and `patterns: ["*"]`. |
| GitHub Actions major-bumps | **Mirror cargo — filtered out** (Round 2 answer). Implementation: top-level `ignore:` with `dependency-name: "*"` + `update-types: ["version-update:semver-major"]`. |
| Commit-message prefix | `chore(deps):` for `cargo`, `chore(ci):` for `github-actions` — matches existing commit convention (`chore: ...`, `chore(quartzite-core): ...` in recent log) |
| Open-PR limit | Default (5) per ecosystem — Dependabot's built-in cap; no reason to override |
| Labels | None — project does not currently use a `dependencies` label; can be added later if desired |
| Reviewers / assignees | None — single-maintainer project; PRs surface via standard notifications |
| Version pin compatibility | Aligns with AGENTS.md § *Dependency Versions* (`0.x` for `0.x.y`, `x` for `x.y.z`) — Dependabot's default behaviour respects Cargo's `^` semantics, so no `versioning-strategy` override required |

## Technical constraints

- AGENTS.md § *Dependency Versions* AXIOM: live state must be queried before asserting any external dep claim. Dependabot's auto-generated PRs already do this (it queries crates.io / the GHA registry directly), so no manual reconciliation step is needed in the config file itself.
- AGENTS.md § *Workflow*: PRs are merged via merge commit (`gh pr merge --merge`); Dependabot's default commit message style is compatible.
- AGENTS.md § *Build & Test*: any modified workflow file requires `actionlint` to pass. `.github/dependabot.yml` is **not** a workflow file (it lives under `.github/` but not `.github/workflows/`); `actionlint` does not validate it. Validation falls back to YAML well-formedness + Dependabot's own parser on GitHub.
- File size: `.github/dependabot.yml` is a short config file (well under any size cap).
- Grouped Dependabot PRs still trigger every PR-gated CI job (`Format`, `Clippy`, `Test`, `Docs`, `Miri`, `Coverage`) — failure of any required check on a grouped PR will need manual ungrouping (close the PR, let Dependabot reopen per-dep) or per-dep follow-up commits.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.github/dependabot.yml` exists at the repository root. |
| AC2 | File contains `version: 2` (Dependabot config schema v2). |
| AC3 | `updates:` section includes an entry with `package-ecosystem: "cargo"`, `directory: "/"`, `schedule.interval: "weekly"`, `target-branch: "master"`. |
| AC4 | `updates:` section includes an entry with `package-ecosystem: "github-actions"`, `directory: "/"`, `schedule.interval: "weekly"`, `target-branch: "master"`. |
| AC5 | Commit-message prefixes follow project convention (`chore(deps):` for cargo, `chore(ci):` for github-actions). |
| AC6 | YAML is well-formed (`python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml'))"` or equivalent passes). |
| AC7 | The `cargo` entry defines a `groups:` block that bundles minor + patch updates into a single combined PR (single group with `update-types: ["minor", "patch"]` and `patterns: ["*"]`). |
| AC8 | The `cargo` entry defines an `ignore:` rule that drops `version-update:semver-major` for `dependency-name: "*"`. |
| AC9 | The `github-actions` entry defines a `groups:` block that bundles minor + patch updates into a single combined PR (single group with `update-types: ["minor", "patch"]` and `patterns: ["*"]`). |
| AC10 | The `github-actions` entry defines an `ignore:` rule that drops `version-update:semver-major` for `dependency-name: "*"`. |
| AC11 | After merge to `master`, GitHub's Dependabot UI (`/network/updates` page) shows both ecosystems as registered with no parser errors. |

## Open questions

- (none)
