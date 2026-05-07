# Design: cargo doc publishing to GitHub Pages

**Issue:** #137
**Date:** 2026-05-07

## Approach

Add a single GitHub Actions workflow file `.github/workflows/docs.yml` that, on every push to
`master`, builds `cargo doc` with the same flags used in the AGENTS.md doc gate, injects a
top-level redirect page into `target/doc/`, and deploys the artifact to GitHub Pages via the
official Pages action trio (configure → upload → deploy). A docs badge is added to `README.md`
under the existing codecov badge.

**Why this approach:**
- Artifact-based Pages deployment (no `gh-pages` branch) is the current GitHub-recommended
  approach; it avoids branch-management noise and works cleanly with the existing workflow
  structure.
- Reusing the exact `RUSTDOCFLAGS` and `cargo doc` flags from `ci.yml`'s `docs` job keeps the
  doc gate consistent between CI and deployment.
- A single `run:` step to write the redirect HTML keeps the workflow minimal — no extra actions
  or scripts needed.

**Rejected alternatives:**
- `peaceiris/actions-gh-pages`: third-party action, requires a `gh-pages` branch, more moving
  parts than the official trio.
- Running `cargo doc` in an existing workflow job and uploading from there: couples doc build to
  unrelated CI concerns; the Pages deployment must be its own job to hold the `pages: write` and
  `id-token: write` permissions without granting them to all CI jobs.

## Decomposition

| # | Task | Files | Depends on |
|---|------|-------|------------|
| 1 | Create `.github/workflows/docs.yml` with trigger, permissions, concurrency, build job (checkout + toolchain + cache + `cargo doc` + inject redirect + upload artifact), and deploy job | `.github/workflows/docs.yml` | — |
| 2 | Insert docs badge in `README.md` under the codecov badge line | `README.md` | — |

## Risks

- **`gh-pages` source vs. artifact source misconfiguration:** GitHub Pages must be configured in
  Settings → Pages to use "GitHub Actions" as the source, not a branch. This is a one-time manual
  step by the repo owner after the first run. The workflow will succeed but the site will 404
  until this is done. Mitigation: document in the spec's Out of scope section (already done).
- **Stale action major versions:** The spec pins `configure-pages@v6`, `upload-pages-artifact@v5`,
  `deploy-pages@v5` as verified on 2026-05-07. If GitHub releases a new major, the workflow
  continues to work on the pinned major. Mitigation: major-only pins per project convention.
- **`target/doc/` path assumption:** The redirect step writes to `target/doc/index.html`. If
  `cargo doc` ever changes its output path this step silently produces a bad artifact. Mitigation:
  the step runs immediately after `cargo doc` and the workflow would fail at the upload step if
  the path were wrong.
- **Concurrency with rapid pushes:** `cancel-in-progress: false` means a second push queues
  rather than kills the in-flight deploy. This is the correct choice for Pages (avoids deploying
  a partial build), but a burst of pushes could queue several deploys. Mitigation: this matches
  the spec decision and is acceptable given the low push frequency on `master`.

## Test Design

This task involves no Rust code — there are no `#[cfg(test)]` modules or integration tests to
write. Correctness is verified by the workflow running successfully on GitHub Actions after the
PR is merged to `master`. The following manual checks serve as acceptance verification:

- AC1: Workflow file has `on: push: branches: [master]` and no `pull_request` trigger.
- AC2: `cargo doc` step sets `RUSTDOCFLAGS: "-D warnings -D missing-docs"` and uses
  `--no-deps --workspace`; a deliberate missing-doc warning in a test branch would fail the job.
- AC3: A `run:` step writes `<meta http-equiv="refresh" ...>` HTML to `target/doc/index.html`
  pointing at `quartzite/index.html` before the upload step.
- AC4: The deploy job uses exactly `actions/configure-pages@v6`, `actions/upload-pages-artifact@v5`,
  `actions/deploy-pages@v5` and no `gh-pages` branch reference appears anywhere.
- AC5: Top-level `permissions:` block lists only `contents: read`, `pages: write`,
  `id-token: write`.
- AC6: Top-level `concurrency:` block sets `group: "pages"` and `cancel-in-progress: false`.
- AC7: `README.md` line 3 (after codecov badge) contains the shield badge linking to
  `https://maratik123.github.io/quartzite/`.

## Open questions

_None._
