# cargo doc publishing to GitHub Pages

**Source:** issue #137
**Date:** 2026-05-07
**Tracked in:** #137

## Scope

1. New `.github/workflows/docs.yml` — triggers on `push` to `master`
2. Runs `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`; fails the build on any doc warning
3. Injects a top-level `index.html` redirect pointing at `quartzite/index.html` (facade crate)
4. Deploys via the official Pages trio: `actions/configure-pages@v6` + `actions/upload-pages-artifact@v5` + `actions/deploy-pages@v5`; no `gh-pages` branch (artifact-based source)
5. Workflow-level permissions: `contents: read`, `pages: write`, `id-token: write`
6. Concurrency group: `group: "pages"`, `cancel-in-progress: false`
7. README docs badge inserted under the existing `codecov` badge: `[![docs](https://img.shields.io/badge/docs-master-blue)](https://maratik123.github.io/quartzite/)`

## Out of scope

- Enabling GitHub Pages in Settings → Pages (user action — must be done by the repo owner after the first workflow run)
- Per-PR docs preview
- `--document-private-items`
- Versioned/multi-version docs

## Deferred

- Multi-version docs (post first `cargo publish`) | After docs.rs takes over for releases | No separate issue needed yet

## Key decisions

| Question | Decision |
|---|---|
| Trigger | `push` to `master` only — no PR preview |
| Doc flags | `RUSTDOCFLAGS="-D warnings -D missing-docs"` + `--no-deps --workspace` — matches AGENTS.md doc-gate |
| Redirect target | Facade crate (`quartzite/index.html`) |
| Pages source | GitHub Actions artifact (no `gh-pages` branch) |
| Action versions | `configure-pages@v6`, `upload-pages-artifact@v5`, `deploy-pages@v5` — verified 2026-05-07 |
| Concurrency | `group: "pages"`, `cancel-in-progress: false` — prevents overlapping deploys on rapid pushes |

## Technical constraints

- Action versions must match project convention (major-only pins, e.g. `@v5`).
- `actions/checkout` must match existing workflows (`@v6`).
- `upload-pages-artifact` is composite (no Node runtime); `deploy-pages` requires Node 24.
- The redirect `index.html` must be injected into `target/doc/` before the artifact upload step.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC1 | `.github/workflows/docs.yml` triggers on `push` to `master` (and only on push to master) |
| AC2 | Workflow runs `RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --no-deps --workspace`; a doc warning causes the workflow to fail |
| AC3 | A `target/doc/index.html` is created that redirects the browser to `quartzite/index.html` |
| AC4 | Deploys via `actions/configure-pages@v6` + `actions/upload-pages-artifact@v5` + `actions/deploy-pages@v5`; no `gh-pages` branch used |
| AC5 | Workflow permissions are scoped to `contents: read`, `pages: write`, `id-token: write` |
| AC6 | Workflow has `concurrency: group: "pages"`, `cancel-in-progress: false` |
| AC7 | README contains a docs badge linking to `https://maratik123.github.io/quartzite/`, inserted under the codecov badge |

## Open questions

_None._
