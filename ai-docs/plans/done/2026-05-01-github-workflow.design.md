# Design: GitHub Workflow & Project Setup

**Issue:** user description
**Date:** 2026-05-01

## Approach

The task is purely infrastructure: add CI, branch protection, README, and LICENSE. No Rust source changes are involved.

**CI workflow** — a single `.github/workflows/ci.yml` file using the official `dtolnay/rust-toolchain` action (stable channel) on `ubuntu-latest`. Three jobs are merged into one job with sequential steps (`cargo build`, `cargo test`, `cargo clippy -- -D warnings`) to avoid redundant dependency fetch overhead. A dedicated `Cargo.lock` cache step via `actions/cache` keeps build times short.

**Branch protection** — applied via `gh api` (GitHub REST v3) rather than the UI or Terraform, as required by the spec. The rule enforces: `required_status_checks` for the CI job, `required_pull_request_reviews` with 1 required approver, and disables direct pushes (`allow_force_pushes: false`, `allow_deletions: false`).

**README** — minimal Markdown (project name, one-paragraph description, prerequisites, build & test commands). No badges for now (deferred by the spec).

**LICENSE** — the verbatim LGPL-3.0-only text fetched from the SPDX canonical source and placed at the repo root as `LICENSE`.

**Rejected alternatives:**

- Separate CI jobs per check: more parallelism, but for a small crate the overhead of multiple job setups outweighs the benefit.
- GitHub rulesets API (newer): requires `gh api /repos/{owner}/{repo}/rulesets`; the classic branch protection API is simpler and sufficient here.
- `actions-rs` action suite: deprecated; `dtolnay/rust-toolchain` is the community standard.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `.github/workflows/ci.yml` with build/test/clippy steps on ubuntu-latest | `.github/workflows/ci.yml` | — |
| 2 | Create `README.md` with project name, description, and build instructions | `README.md` | — |
| 3 | Create `LICENSE` with verbatim LGPL-3.0 text | `LICENSE` | — |
| 4 | Apply branch protection on `master` via `gh api` | — (remote GitHub state) | 1 (CI job name must be known before configuring required checks) |

## Risks

- **Required check name mismatch:** The job/step name in `ci.yml` must exactly match the string registered in branch protection `required_status_checks.contexts`. Mitigation: use the top-level `jobs.<id>` key name (e.g., `ci`) as the required context; document this in the design so the implementer uses the same string in the `gh api` call.
- **Branch protection applied before CI runs once:** GitHub may refuse to add a required check that has never reported a status. Mitigation: push the workflow file first (or use a PR), let CI run once, then apply protection — or use `strict: false` with `contexts` set even if the check has not run yet (GitHub accepts this).
- **`gh` CLI authentication scope:** Branch protection requires `repo` scope on the token. Mitigation: confirm `gh auth status` shows `repo` before running the API call; instruct the implementer to re-authenticate if needed.
- **LGPL-3.0 text accuracy:** A hand-typed license introduces errors. Mitigation: copy verbatim from the SPDX text at `https://spdx.org/licenses/LGPL-3.0-only.html` or use `gh repo edit --license LGPL-3.0-only` (creates the file automatically).
- **`cargo clippy` toolchain:** The workspace uses edition 2024 (`edition = "2024"` in `Cargo.toml`). Stable Rust must be new enough to support edition 2024 (≥ 1.85). The `dtolnay/rust-toolchain@stable` action always tracks the latest stable, so this is safe; document it as a note.

## Test Design

No Rust logic is introduced, so there are no unit or integration tests to write. Verification is behavioral:

- **AC1 — direct push rejected:** Attempt `git push origin master` on a branch after protection is applied; expect rejection.
- **AC2 & AC3 — CI triggered on PR:** Open a test PR; confirm the `ci` workflow appears in the Checks tab and runs the three steps.
- **AC4 — merge blocked by failing CI:** Introduce a deliberate clippy warning in a branch, open a PR; confirm the merge button is disabled.
- **AC5 — merge blocked without review:** Open a PR without a review; confirm the merge button shows "Review required".
- **AC6 — README exists:** `gh repo view maratik123/quartzite` shows the README rendered.
- **AC7 — LICENSE exists:** `gh api repos/maratik123/quartzite/license` returns `LGPL-3.0-only`.

## Open questions

- None.
